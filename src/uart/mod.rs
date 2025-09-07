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
    dma::DmaBindRx,
    rcc::{BusClock, Enable, Reset},
};

use crate::Mcu;

pub trait UartInit<U> {
    fn constrain(self, mcu: &mut Mcu) -> Uart<U>;
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

// ------------------------------------------------------------------------------------------------

/// UART Transmitter
pub struct Tx<U> {
    uart: [U; 2],
}

impl<U: UartPeriph> Tx<U> {
    pub(crate) fn new(uart: [U; 2]) -> Self {
        Self { uart }
    }

    pub fn into_poll(self, retry_times: u32, flush_retry_times: u32) -> UartPollTx<U> {
        let [uart, _] = self.uart;
        UartPollTx::<U>::new(uart, retry_times, flush_retry_times)
    }

    pub fn into_interrupt(
        self,
        buf_size: usize,
        transmit_retry_times: u32,
        flush_retry_times: u32,
    ) -> (UartInterruptTx<U>, UartInterruptTxHandler<U>) {
        UartInterruptTx::new(self.uart, buf_size, transmit_retry_times, flush_retry_times)
    }

    // pub fn into_dma<CH>(self, dma_ch: CH) -> UartDmaTx<U, CH>
    // where
    //     CH: BindDmaTx<U>,
    // {
    //     UartDmaTx::<U, CH>::new(self.uart, dma_ch)
    // }

    // pub fn into_dma_ringbuf<CH>(self, dma_ch: CH, buf_size: usize) -> UartDmaBufTx<U, CH>
    // where
    //     CH: BindDmaTx<U>,
    // {
    //     UartDmaBufTx::<U, CH>::new(self.uart, dma_ch, buf_size)
    // }
}

// ------------------------------------------------------------------------------------------------

/// UART Receiver
pub struct Rx<U: UartPeriph> {
    uart: [U; 2],
}

impl<U: UartPeriph> Rx<U> {
    pub(crate) fn new(uart: [U; 2]) -> Self {
        Self { uart }
    }

    pub fn into_poll(self, retry_times: u32, continue_retry_times: u32) -> UartPollRx<U> {
        let [uart, _] = self.uart;
        UartPollRx::<U>::new(uart, retry_times, continue_retry_times)
    }

    pub fn into_interrupt(
        self,
        buf_size: usize,
        retry_times: u32,
    ) -> (UartInterruptRx<U>, UartInterruptRxHandler<U>) {
        UartInterruptRx::new(self.uart, buf_size, retry_times)
    }

    pub fn into_dma_circle<CH>(self, dma_ch: CH, buf_size: usize) -> UartDmaBufRx<U, CH>
    where
        CH: DmaBindRx<U>,
    {
        let [uart, _] = self.uart;
        UartDmaBufRx::<U, CH>::new(uart, dma_ch, buf_size)
    }
}
