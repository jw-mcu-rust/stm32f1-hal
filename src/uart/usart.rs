#![allow(dead_code)]

use crate::pac::usart1::{self, cr1};

// sync begin

use crate::{
    Mcu, Steal, afio,
    afio::{RemapMode, uart_remap::*},
    common::{uart::*, wrap_trait::*},
    pac,
    rcc::{BusClock, Enable, Reset},
};
use core::ops::Deref;
use core::sync::atomic::{Ordering, compiler_fence};

// Register Block -------------------------------------------------------------

pub trait Instance: RegisterBlock + BusClock + Enable + Reset + afio::SerialAsync {}

impl<T: Instance> UartDev for Uart<T> {
    #[inline]
    fn set_dma_tx(&mut self, enable: bool) {
        self.periph.cr3().modify(|_, w| w.dmat().bit(enable));
    }

    #[inline]
    fn set_dma_rx(&mut self, enable: bool) {
        self.periph.cr3().modify(|_, w| w.dmar().bit(enable));
    }

    #[inline]
    fn is_tx_empty(&self) -> bool {
        self.periph.sr().read().txe().bit_is_set()
    }

    #[inline]
    fn is_tx_complete(&self) -> bool {
        self.periph.sr().read().tc().bit_is_set()
    }

    fn write(&mut self, word: u16) -> nb::Result<(), Error> {
        if self.is_tx_empty() {
            self.periph
                .dr()
                .write(|w| unsafe { w.dr().bits(word.into()) });
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    fn read(&mut self) -> nb::Result<u16, Error> {
        let sr = self.periph.sr().read();

        // Check if a byte is available
        if sr.rxne().bit_is_set() {
            // Read the received byte
            return Ok(self.periph.dr().read().dr().bits());
        }

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
            self.clear_err_flag();
            Err(nb::Error::Other(err))
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    #[inline]
    fn get_tx_data_reg_addr(&self) -> u32 {
        &self.periph.dr() as *const _ as u32
    }

    #[inline]
    fn get_rx_data_reg_addr(&self) -> u32 {
        &self.periph.dr() as *const _ as u32
    }

    #[inline]
    fn set_interrupt(&mut self, event: UartEvent, enable: bool) {
        match event {
            UartEvent::Idle => {
                self.periph.cr1().modify(|_, w| w.idleie().bit(enable));
            }
            UartEvent::RxNotEmpty => {
                self.periph.cr1().modify(|_, w| w.rxneie().bit(enable));
            }
            UartEvent::TxEmpty => {
                self.periph.cr1().modify(|_, w| w.txeie().bit(enable));
            }
        }
    }

    #[inline]
    fn is_interrupt_enable(&mut self, event: UartEvent) -> bool {
        let cr1 = self.periph.cr1().read();
        match event {
            UartEvent::Idle => cr1.idleie().bit_is_set(),
            UartEvent::RxNotEmpty => cr1.rxneie().bit_is_set(),
            UartEvent::TxEmpty => cr1.txeie().bit_is_set(),
        }
    }

    #[inline]
    fn is_interrupted(&mut self, event: UartEvent) -> bool {
        let sr = self.periph.sr().read();
        match event {
            UartEvent::Idle => {
                if sr.idle().bit_is_set() && self.periph.cr1().read().idleie().bit_is_set() {
                    self.clear_err_flag();
                    return true;
                }
            }
            UartEvent::RxNotEmpty => {
                if (sr.rxne().bit_is_set() || sr.ore().bit_is_set())
                    && self.periph.cr1().read().rxneie().bit_is_set()
                {
                    return true;
                }
            }
            UartEvent::TxEmpty => {
                if sr.txe().bit_is_set() && self.periph.cr1().read().txeie().bit_is_set() {
                    return true;
                }
            }
        }
        false
    }

    /// In order to clear that error flag, you have to do a read from the sr register
    /// followed by a read from the dr register.
    #[inline]
    fn clear_err_flag(&self) {
        let _ = self.periph.sr().read();
        compiler_fence(Ordering::Acquire);
        let _ = self.periph.dr().read();
        compiler_fence(Ordering::Acquire);
    }

    #[inline]
    fn is_rx_not_empty(&self) -> bool {
        self.periph.sr().read().rxne().bit_is_set()
    }
}

// Initialization interface ------------------------------------------------------

macro_rules! impl_uart_init {
    ($($uart:ty),+) => {$(
        impl Instance for $uart {}
        impl UartInit<$uart> for $uart {
            fn constrain(self) -> Uart<$uart> {
                Uart { periph: self }
            }
        }
    )+};
}

pub trait UartInit<T: Instance> {
    fn constrain(self) -> Uart<T>;
}

// UART Initialization -------------------------------------------------------------

// Use a wrap to avoid conflicting implementations of trait
pub struct Uart<U: Instance> {
    periph: U,
}

#[allow(private_bounds)]
#[allow(unused_variables)]
impl<U: Instance + Steal> Uart<U> {
    fn steal(&self) -> Self {
        Self {
            periph: unsafe { self.periph.steal() },
        }
    }

    pub fn into_tx_rx<REMAP: RemapMode>(
        mut self,
        pins: (
            impl UartTxPin<U, RemapMode = REMAP>,
            impl UartRxPin<U, RemapMode = REMAP>,
        ),
        config: Config,
        mcu: &mut Mcu,
    ) -> (Tx<Self>, Rx<Self>) {
        REMAP::remap(&mut mcu.afio);
        self.config(config, mcu);
        self.enable_comm(true, true);
        (
            Tx::new([self.steal(), self.steal()]),
            Rx::new([self.steal(), self.steal()]),
        )
    }

    pub fn into_tx<REMAP: RemapMode>(
        mut self,
        tx_pin: impl UartTxPin<U, RemapMode = REMAP>,
        config: Config,
        mcu: &mut Mcu,
    ) -> Tx<Self> {
        REMAP::remap(&mut mcu.afio);
        self.config(config, mcu);
        self.enable_comm(true, false);
        Tx::new([self.steal(), self.steal()])
    }

    pub fn into_rx<REMAP: RemapMode>(
        mut self,
        rx_pin: impl UartRxPin<U, RemapMode = REMAP>,
        config: Config,
        mcu: &mut Mcu,
    ) -> Rx<Self> {
        REMAP::remap(&mut mcu.afio);
        self.config(config, mcu);
        self.enable_comm(true, false);
        Rx::<Self>::new([self.steal(), self.steal()])
    }

    pub fn get_idle_interrupt_handler(&self) -> UartIdleInterrupt<Self> {
        UartIdleInterrupt::new(self.steal())
    }

    fn config(&mut self, config: Config, mcu: &mut Mcu) {
        U::enable(&mut mcu.rcc);
        U::reset(&mut mcu.rcc);

        // Configure baud rate
        let brr = U::clock(&mcu.rcc.clocks).raw() / config.baudrate;
        assert!(brr >= 16, "impossible baud rate");
        self.periph.brr().write(|w| unsafe { w.bits(brr as u16) });

        // Configure word
        self.periph.cr1().modify(|_, w| {
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
        self.periph.cr1().modify(|_, w| {
            w.ue().set_bit();
            w.te().bit(tx);
            w.re().bit(rx);
            w
        });
    }

    // sync end

    fn set_stop_bits(&mut self, bits: StopBits) {
        use pac::usart1::cr2::STOP;

        self.periph.cr2().write(|w| {
            w.stop().variant(match bits {
                StopBits::STOP0P5 => STOP::Stop0p5,
                StopBits::STOP1 => STOP::Stop1,
                StopBits::STOP1P5 => STOP::Stop1p5,
                StopBits::STOP2 => STOP::Stop2,
            })
        });
    }
}

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
