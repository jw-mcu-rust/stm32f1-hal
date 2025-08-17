use super::Mcu;
use crate::{
    Steal, pac,
    rcc::{BusClock, Clocks, Enable, Rcc, Reset},
    common::uart::*,
};
use core::sync::atomic::{Ordering, compiler_fence};

trait UartConfig: UartConfigStopBits + Steal {
    fn config(&mut self, config: Config, clocks: &Clocks);
    fn enable_clock(&mut self, rcc: &mut Rcc);
    fn enable_comm(&mut self, tx: bool, rx: bool);
}

trait UartConfigStopBits {
    fn set_stop_bits(&mut self, bits: StopBits);
}

macro_rules! st_uart {
    ($(
        $UARTx:ty,
    )+) => {$(
        impl UartReg for $UARTx {
            #[inline]
            fn set_dma_tx(&mut self, enable: bool) {
                self.cr3().modify(|_, w| w.dmat().bit(enable));
            }

            #[inline]
            fn set_dma_rx(&mut self, enable: bool) {
                self.cr3().modify(|_, w| w.dmar().bit(enable));
            }

            #[inline]
            fn is_tx_empty(&self) -> bool {
                self.sr().read().txe().bit_is_set()
            }

            fn write(&mut self, word: u8) -> nb::Result<(), Infallible> {
                if self.is_tx_empty() {
                    self.dr().write(|w| unsafe{
                        w.dr().bits(word.into())
                    });
                    Ok(())
                } else {
                    Err(nb::Error::WouldBlock)
                }
            }

            fn read(&mut self) -> nb::Result<u8, Error> {
                let sr = self.sr().read();

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
                        Ok(self.dr().read().dr().bits() as u8)
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }
            }

            #[inline]
            fn get_tx_data_reg_addr(&self) -> u32 {
                &self.dr() as *const _ as u32
            }

            #[inline]
            fn get_rx_data_reg_addr(&self) -> u32 {
                &self.dr() as *const _ as u32
            }

            #[inline]
            fn set_interrupt(&mut self, event: UartEvent, enable: bool) {
                match event {
                    UartEvent::Idle => {
                        self.cr1().modify(|_, w| w.idleie().bit(enable));
                    },
                    _ => (),
                }
            }

            fn is_interrupted(&mut self, event: UartEvent) -> bool {
                match event {
                    UartEvent::Idle => {
                        if self.sr().read().idle().bit_is_set() {
                            self.clear_pe_flag();
                            true
                        } else {
                            false
                        }
                    },
                    _ => false,
                }
            }

            /// In order to clear that error flag, you have to do a read from the sr register
            /// followed by a read from the dr register.
            #[inline]
            fn clear_pe_flag(&self) {
                let _ = self.sr().read();
                compiler_fence(Ordering::Acquire);
                let _ = self.dr().read();
                compiler_fence(Ordering::Acquire);
            }

            #[inline]
            fn is_rx_not_empty(&self) -> bool {
                self.sr().read().rxne().bit_is_set()
            }
        }

        impl UartConfig for $UARTx {
            fn config(&mut self, config: Config, clocks: &Clocks) {
                // Configure baud rate
                let brr = <$UARTx>::clock(clocks).raw() / config.baudrate;
                assert!(brr >= 16, "impossible baud rate");
                self.brr().write(|w| unsafe { w.bits(brr as u16) });

                // Configure word
                self.cr1().modify(|_r, w| {
                    w.m().bit(match config.word_length {
                        WordLength::Bits8 => false,
                        WordLength::Bits9 => true,
                    });
                    use pac::usart1::cr1::PS;
                    w.ps().variant(match config.parity {
                        Parity::ParityOdd => PS::Odd,
                        _ => PS::Even,
                    });
                    w.pce().bit(!matches!(config.parity, Parity::ParityNone));
                    w
                });

                // Configure stop bits
                self.set_stop_bits(config.stop_bits);
            }

            fn enable_clock(&mut self, rcc: &mut Rcc) {
                <$UARTx>::enable(rcc);
                <$UARTx>::reset(rcc);
            }

            fn enable_comm(&mut self, tx: bool, rx: bool) {
                // UE: enable USART
                // TE: enable transceiver
                // RE: enable receiver
                self.cr1().modify(|_r, w| {
                    w.ue().set_bit();
                    w.te().bit(tx);
                    w.re().bit(rx);
                    w
                });
            }
        }
    )+};
}

macro_rules! st_usart_config_stop_bits {
    ($(
        $USARTx:ty,
    )+) => {$(
        impl UartConfigStopBits for $USARTx {
            fn set_stop_bits(&mut self, bits: StopBits) {
                use pac::usart1::cr2::STOP;

                self.cr2().write(|w| {
                    w.stop().variant(match bits {
                        StopBits::STOP0P5 => STOP::Stop0p5,
                        StopBits::STOP1 => STOP::Stop1,
                        StopBits::STOP1P5 => STOP::Stop1p5,
                        StopBits::STOP2 => STOP::Stop2,
                    })
                });
            }
        }

        st_uart! {
            $USARTx,
        }
    )+};
}

#[cfg(any(all(feature = "stm32f103", feature = "high"), feature = "connectivity"))]
macro_rules! st_uart_config_stop_bits {
    ($(
        $UARTx:ty,
    )+) => {$(
        impl UartConfigStopBits for $UARTx {
            fn set_stop_bits(&mut self, bits: StopBits) {
                use pac::uart4::cr2::STOP;

                // StopBits::STOP0P5 and StopBits::STOP1P5 aren't supported when using UART
                // STOP_A::STOP1 and STOP_A::STOP2 will be used, respectively
                self.cr2().write(|w| {
                    w.stop().variant(match bits {
                        StopBits::STOP0P5 | StopBits::STOP1 => STOP::Stop1,
                        StopBits::STOP1P5 | StopBits::STOP2 => STOP::Stop2,
                    })
                });
            }
        }

        st_uart! {
            $UARTx,
        }
    )+};
}

st_usart_config_stop_bits! {
    pac::USART1,
    pac::USART2,
    pac::USART3,
}
#[cfg(any(all(feature = "stm32f103", feature = "high"), feature = "connectivity"))]
st_uart_config_stop_bits! {
    pac::UART4,
    pac::UART5,
}

pub trait UartInit<U: UartReg> {
    fn init_tx_rx(self, config: Config, mcu: &mut Mcu) -> (Tx<U>, Rx<U>);
    fn init_tx(self, config: Config, mcu: &mut Mcu) -> Tx<U>;
    fn init_rx(self, config: Config, mcu: &mut Mcu) -> Rx<U>;
}

impl<U> UartInit<U> for U
where
    U: UartReg + UartConfig,
{
    fn init_tx_rx(mut self, config: Config, mcu: &mut Mcu) -> (Tx<U>, Rx<U>) {
        self.enable_clock(&mut mcu.rcc);
        self.config(config, &mcu.rcc.clocks);
        // TODO let pins = (pins.0.map(RInto::rinto), pins.1.map(RInto::rinto));
        self.enable_comm(true, true);
        (
            Tx::<U>::new(unsafe { self.steal() }),
            Rx::<U>::new(unsafe { self.steal() }),
        )
    }

    fn init_tx(mut self, config: Config, mcu: &mut Mcu) -> Tx<U> {
        self.enable_clock(&mut mcu.rcc);
        self.config(config, &mcu.rcc.clocks);
        self.enable_comm(true, false);
        Tx::<U>::new(unsafe { self.steal() })
    }

    fn init_rx(mut self, config: Config, mcu: &mut Mcu) -> Rx<U> {
        self.enable_clock(&mut mcu.rcc);
        self.config(config, &mcu.rcc.clocks);
        self.enable_comm(false, true);
        Rx::<U>::new(unsafe { self.steal() })
    }
}
