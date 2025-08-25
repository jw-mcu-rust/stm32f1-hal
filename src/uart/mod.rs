#[cfg(any(all(feature = "stm32f103", feature = "high"), feature = "connectivity"))]
pub mod uart4;
#[cfg(any(all(feature = "stm32f103", feature = "high"), feature = "connectivity"))]
pub mod uart5;
pub mod usart1;
pub mod usart2;
pub mod usart3;
pub use crate::common::uart::*;

use crate::{
    Steal,
    afio::{RemapMode, uart_remap::*},
    rcc::{BusClock, Enable, Reset},
};

use crate::Mcu;

pub trait UartInit<U> {
    fn constrain(self) -> Uart<U>;
}

pub trait UartPeriphExt: UartPeriph + BusClock + Enable + Reset {
    fn config(&mut self, config: Config, mcu: &mut Mcu);
    fn enable_comm(&mut self, tx: bool, rx: bool);
    fn set_stop_bits(&mut self, bits: StopBits);
}

// wrapper
pub struct Uart<U> {
    uart: U,
}

impl<U: UartPeriphExt + Steal> Uart<U> {
    pub fn into_tx_rx<REMAP: RemapMode<U>>(
        mut self,
        pins: (Option<impl UartTxPin<REMAP>>, Option<impl UartRxPin<REMAP>>),
        config: Config,
        mcu: &mut Mcu,
    ) -> (Option<Tx<U>>, Option<Rx<U>>) {
        REMAP::remap(&mut mcu.afio);
        self.uart.config(config, mcu);
        self.uart.enable_comm(pins.0.is_some(), pins.1.is_some());
        unsafe {
            (
                pins.0
                    .map(|_| Tx::new([self.uart.steal(), self.uart.steal()])),
                pins.1
                    .map(|_| Rx::new([self.uart.steal(), self.uart.steal()])),
            )
        }
    }

    pub fn get_idle_interrupt_handler(&self) -> UartIdleInterrupt<U> {
        UartIdleInterrupt::new(unsafe { self.uart.steal() })
    }
}
