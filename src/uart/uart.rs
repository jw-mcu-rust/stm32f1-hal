#![allow(dead_code)]
#![allow(private_bounds)]

use crate::pac::uart4::{self, cr1};

// Do NOT manually modify the code between begin and end!
// It's synced by scripts/sync_code.py.
// sync 1 begin

use crate::{
    Mcu, Steal,
    afio::{RemapMode, uart_remap::*},
    common::{uart::*, wrap_trait::*},
    pac,
    rcc::{BusClock, Enable, Reset},
};

// Initialization interface ------------------------------------------------------

macro_rules! impl_uart_init {
    ($($reg:ty),+) => {$(
        impl UartInit<$reg> for $reg {
            fn constrain(self) -> Uart<$reg> {
                Uart { reg: self }
            }
        }
    )+};
}
pub(crate) use impl_uart_init;

pub trait UartInit<REG: RegisterBlock> {
    fn constrain(self) -> Uart<REG>;
}

// Initialization -------------------------------------------------------------

// Use a wrapper to avoid conflicting implementations of trait
pub struct Uart<REG: RegisterBlock> {
    reg: REG,
}

#[allow(unused_variables)]
impl<REG: RegisterBlock + Steal> Uart<REG> {
    fn steal(&self) -> Self {
        Self {
            reg: unsafe { self.reg.steal() },
        }
    }

    pub fn into_tx_rx<REMAP: RemapMode<REG>>(
        mut self,
        pins: (Option<impl UartTxPin<REMAP>>, Option<impl UartRxPin<REMAP>>),
        config: Config,
        mcu: &mut Mcu,
    ) -> (Option<Tx<Self>>, Option<Rx<Self>>) {
        REMAP::remap(&mut mcu.afio);
        self.config(config, mcu);
        self.enable_comm(pins.0.is_some(), pins.1.is_some());
        (
            pins.0.map(|_| Tx::new([self.steal(), self.steal()])),
            pins.1.map(|_| Rx::new([self.steal(), self.steal()])),
        )
    }

    pub fn get_idle_interrupt_handler(&self) -> UartIdleInterrupt<Self> {
        UartIdleInterrupt::new(self.steal())
    }

    fn config(&mut self, config: Config, mcu: &mut Mcu) {
        REG::enable(&mut mcu.rcc);
        REG::reset(&mut mcu.rcc);

        // Configure baud rate
        let brr = REG::clock(&mcu.rcc.clocks).raw() / config.baudrate;
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

    // sync 1 end

    fn set_stop_bits(&mut self, bits: StopBits) {
        use pac::uart4::cr2::STOP;

        // StopBits::STOP0P5 and StopBits::STOP1P5 aren't supported when using UART
        // STOP_A::STOP1 and STOP_A::STOP2 will be used, respectively
        self.reg.cr2().write(|w| {
            w.stop().variant(match bits {
                StopBits::STOP0P5 | StopBits::STOP1 => STOP::Stop1,
                StopBits::STOP1P5 | StopBits::STOP2 => STOP::Stop2,
            })
        });
    }
}

// sync 2 begin

// Implement Peripheral -------------------------------------------------------

impl<REG: RegisterBlock> UartPeriph for Uart<REG> {
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

    #[inline]
    fn is_tx_complete(&self) -> bool {
        self.reg.sr().read().tc().bit_is_set()
    }

    fn write(&mut self, word: u16) -> nb::Result<(), Error> {
        if self.is_tx_empty() {
            self.reg.dr().write(|w| unsafe { w.dr().bits(word.into()) });
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    fn read(&mut self) -> nb::Result<u16, Error> {
        let sr = self.reg.sr().read();

        // Check if a byte is available
        if sr.rxne().bit_is_set() {
            // Read the received byte
            return Ok(self.reg.dr().read().dr().bits());
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
            UartEvent::RxNotEmpty => {
                self.reg.cr1().modify(|_, w| w.rxneie().bit(enable));
            }
            UartEvent::TxEmpty => {
                self.reg.cr1().modify(|_, w| w.txeie().bit(enable));
            }
        }
    }

    #[inline]
    fn is_interrupt_enable(&mut self, event: UartEvent) -> bool {
        let cr1 = self.reg.cr1().read();
        match event {
            UartEvent::Idle => cr1.idleie().bit_is_set(),
            UartEvent::RxNotEmpty => cr1.rxneie().bit_is_set(),
            UartEvent::TxEmpty => cr1.txeie().bit_is_set(),
        }
    }

    #[inline]
    fn is_interrupted(&mut self, event: UartEvent) -> bool {
        let sr = self.reg.sr().read();
        match event {
            UartEvent::Idle => {
                if sr.idle().bit_is_set() && self.reg.cr1().read().idleie().bit_is_set() {
                    self.clear_err_flag();
                    return true;
                }
            }
            UartEvent::RxNotEmpty => {
                if (sr.rxne().bit_is_set() || sr.ore().bit_is_set())
                    && self.reg.cr1().read().rxneie().bit_is_set()
                {
                    return true;
                }
            }
            UartEvent::TxEmpty => {
                if sr.txe().bit_is_set() && self.reg.cr1().read().txeie().bit_is_set() {
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
        let _ = self.reg.sr().read();
        let _ = self.reg.dr().read();
    }

    #[inline]
    fn is_rx_not_empty(&self) -> bool {
        self.reg.sr().read().rxne().bit_is_set()
    }
}

// sync 2 end

impl_uart_init!(pac::UART4, pac::UART5);
wrap_trait_deref!(
    (pac::UART4, pac::UART5,),
    pub(super) trait RegisterBlock: BusClock + Enable + Reset {
        fn cr1(&self) -> &uart4::CR1;
        fn dr(&self) -> &uart4::DR;
        fn brr(&self) -> &uart4::BRR;
        fn sr(&self) -> &uart4::SR;
        fn cr2(&self) -> &uart4::CR2;
        fn cr3(&self) -> &uart4::CR3;
        fn gtpr(&self) -> &uart4::GTPR;
    }
);
