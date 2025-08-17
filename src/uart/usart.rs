#![allow(dead_code)]

use crate::pac::usart1::{self, cr1};

// sync begin

use crate::{
    Mcu, Steal, afio,
    afio::uart_remap::*,
    common::{uart::*, wrap_trait::*},
    pac,
    rcc::{BusClock, Enable, Reset},
};
use core::ops::Deref;
use core::sync::atomic::{Ordering, compiler_fence};

// Register Block -------------------------------------------------------------

pub trait Instance: RegisterBlock + BusClock + Enable + Reset + Steal + afio::SerialAsync {}

impl<T: Instance> UartReg for Uart<T> {
    #[inline]
    fn set_dma_tx(&mut self, enable: bool) {
        self.reg.cr3().modify(|_, w| w.dmat().bit(enable));
    }

    #[inline]
    fn set_dma_rx(&mut self, enable: bool) {
        self.reg.cr3().modify(|_, w| w.dmar().bit(enable));
    }

    #[inline]
    fn is_tx_empty(&self) -> bool {
        self.reg.sr().read().txe().bit_is_set()
    }

    fn write(&mut self, word: u16) -> nb::Result<(), Infallible> {
        if self.is_tx_empty() {
            self.reg.dr().write(|w| unsafe { w.dr().bits(word.into()) });
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    fn read(&mut self) -> nb::Result<u16, Error> {
        let sr = self.reg.sr().read();

        // Check for any errors
        let err = if sr.pe().bit_is_set() {
            Some(Error::Parity)
        } else if sr.fe().bit_is_set() {
            Some(Error::FrameFormat)
        } else if sr.ne().bit_is_set() {
            Some(Error::Noise)
        } else if sr.ore().bit_is_set() {
            Some(Error::Overrun)
        } else {
            None
        };

        if let Some(err) = err {
            self.clear_pe_flag();
            Err(nb::Error::Other(err))
        } else {
            // Check if a byte is available
            if sr.rxne().bit_is_set() {
                // Read the received byte
                Ok(self.reg.dr().read().dr().bits())
            } else {
                Err(nb::Error::WouldBlock)
            }
        }
    }

    #[inline]
    fn get_tx_data_reg_addr(&self) -> u32 {
        &self.reg.dr() as *const _ as u32
    }

    #[inline]
    fn get_rx_data_reg_addr(&self) -> u32 {
        &self.reg.dr() as *const _ as u32
    }

    #[inline]
    fn set_interrupt(&mut self, event: UartEvent, enable: bool) {
        match event {
            UartEvent::Idle => {
                self.reg.cr1().modify(|_, w| w.idleie().bit(enable));
            }
            _ => (),
        }
    }

    fn is_interrupted(&mut self, event: UartEvent) -> bool {
        match event {
            UartEvent::Idle => {
                if self.reg.sr().read().idle().bit_is_set() {
                    self.clear_pe_flag();
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// In order to clear that error flag, you have to do a read from the sr register
    /// followed by a read from the dr register.
    #[inline]
    fn clear_pe_flag(&self) {
        let _ = self.reg.sr().read();
        compiler_fence(Ordering::Acquire);
        let _ = self.reg.dr().read();
        compiler_fence(Ordering::Acquire);
    }

    #[inline]
    fn is_rx_not_empty(&self) -> bool {
        self.reg.sr().read().rxne().bit_is_set()
    }
}

// Initialization interface ------------------------------------------------------

macro_rules! impl_uart_init {
    ($($uart:ty),+) => {$(
        impl Instance for $uart {}
        impl UartInit<$uart> for $uart {
            fn constrain(self) -> Uart<$uart> {
                Uart { reg: self }
            }
        }
    )+};
}

pub trait UartInit<T: Instance> {
    fn constrain(self) -> Uart<T>;
}

// UART Initialization -------------------------------------------------------------

// Use a wrap to avoid conflicting implementations of trait
pub struct Uart<T: Instance> {
    reg: T,
}

impl<T: Instance> Steal for Uart<T> {
    unsafe fn steal(&self) -> Self {
        Self {
            reg: unsafe { self.reg.steal() },
        }
    }
}

impl<T: Instance> Uart<T> {
    pub fn into_tx_rx(
        mut self,
        pins: (Option<impl UartTxPin<Self>>, Option<impl UartRxPin<Self>>),
        config: Config,
        mcu: &mut Mcu,
    ) -> (Option<Tx<Self>>, Option<Rx<Self>>) {
        match (
            pins.0.as_ref().map(|p| p.set_remap_reg(&mut mcu.afio)),
            pins.1.as_ref().map(|p| p.set_remap_reg(&mut mcu.afio)),
        ) {
            (Some(v1), Some(v2)) => {
                // Two Pins must correspond to the same remap.
                assert_eq!(v1, v2)
            }
            (None, None) => {
                panic!("Missing Pins!");
            }
            _ => (),
        }
        self.config(config, mcu);
        self.enable_comm(pins.0.is_some(), pins.1.is_some());
        (
            pins.0.map(|_| Tx::<Self>::new(unsafe { self.steal() })),
            pins.1.map(|_| Rx::<Self>::new(unsafe { self.steal() })),
        )
    }

    fn config(&mut self, config: Config, mcu: &mut Mcu) {
        T::enable(&mut mcu.rcc);
        T::reset(&mut mcu.rcc);

        // Configure baud rate
        let brr = T::clock(&mcu.rcc.clocks).raw() / config.baudrate;
        assert!(brr >= 16, "impossible baud rate");
        self.reg.brr().write(|w| unsafe { w.bits(brr as u16) });

        // Configure word
        self.reg.cr1().modify(|_, w| {
            w.m().bit(match config.word_length {
                WordLength::Bits8 => false,
                WordLength::Bits9 => true,
            });
            w.ps().variant(match config.parity {
                Parity::ParityOdd => cr1::PS::Odd,
                _ => cr1::PS::Even,
            });
            w.pce().bit(!matches!(config.parity, Parity::ParityNone));
            w
        });

        // Configure stop bits
        self.set_stop_bits(config.stop_bits);
    }

    fn enable_comm(&mut self, tx: bool, rx: bool) {
        // UE: enable USART
        // TE: enable transceiver
        // RE: enable receiver
        self.reg.cr1().modify(|_, w| {
            w.ue().set_bit();
            w.te().bit(tx);
            w.re().bit(rx);
            w
        });
    }

    // sync end

    fn set_stop_bits(&mut self, bits: StopBits) {
        use pac::usart1::cr2::STOP;

        self.reg.cr2().write(|w| {
            w.stop().variant(match bits {
                StopBits::STOP0P5 => STOP::Stop0p5,
                StopBits::STOP1 => STOP::Stop1,
                StopBits::STOP1P5 => STOP::Stop1p5,
                StopBits::STOP2 => STOP::Stop2,
            })
        });
    }
}

pub type Uart1 = Uart<pac::USART1>;
pub type Uart2 = Uart<pac::USART1>;
pub type Uart3 = Uart<pac::USART1>;
impl_uart_init!(pac::USART1, pac::USART2, pac::USART3);
wrap_trait_deref!(
    (pac::USART1, pac::USART2, pac::USART3,),
    pub trait RegisterBlock {
        fn cr1(&self) -> &usart1::CR1;
        fn dr(&self) -> &usart1::DR;
        fn brr(&self) -> &usart1::BRR;
        fn sr(&self) -> &usart1::SR;
        fn cr2(&self) -> &usart1::CR2;
        fn cr3(&self) -> &usart1::CR3;
        fn gtpr(&self) -> &usart1::GTPR;
    }
);
