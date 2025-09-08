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
    common::os::*,
    dma::{DmaBindRx, DmaBindTx, DmaRingbufTxLoader},
    rcc::{BusClock, Enable, Reset},
};

use crate::Mcu;

pub trait UartInit<U> {
    fn constrain(self, mcu: &mut Mcu) -> Uart<U>;
}

pub trait UartPeriphExt: UartPeriph + BusClock + Enable + Reset + Steal {
    fn config(&mut self, config: Config, mcu: &mut Mcu);
    fn enable_comm(&mut self, tx: bool, rx: bool);
    fn set_stop_bits(&mut self, bits: StopBits);
}

// wrapper
pub struct Uart<U> {
    uart: U,
}

impl<U: UartPeriphExt> Uart<U> {
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
                pins.0.map(|_| Tx::new(self.uart.steal())),
                pins.1.map(|_| Rx::new(self.uart.steal())),
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
    uart: U,
}

impl<U: UartPeriphExt> Tx<U> {
    pub(crate) fn new(uart: U) -> Self {
        Self { uart }
    }

    pub fn into_poll<T: Timeout>(self, timeout: T, flush_retry_times: u32) -> UartPollTx<U, T> {
        UartPollTx::<U, T>::new(self.uart, timeout, flush_retry_times)
    }

    pub fn into_interrupt(
        self,
        buf_size: usize,
        transmit_retry_times: u32,
        flush_retry_times: u32,
    ) -> (UartInterruptTx<U>, UartInterruptTxHandler<U>) {
        let u2 = unsafe { self.uart.steal() };
        UartInterruptTx::new(
            [self.uart, u2],
            buf_size,
            transmit_retry_times,
            flush_retry_times,
        )
    }

    // pub fn into_dma<CH>(self, dma_ch: CH) -> UartDmaTx<U, CH>
    // where
    //     CH: BindDmaTx<U>,
    // {
    //     UartDmaTx::<U, CH>::new(self.uart, dma_ch)
    // }

    pub fn into_dma_ringbuf<CH>(
        self,
        dma_ch: CH,
        buf_size: usize,
    ) -> (UartDmaBufTx<U, CH>, DmaRingbufTxLoader<u8, CH>)
    where
        CH: DmaBindTx<U>,
    {
        UartDmaBufTx::<U, CH>::new(self.uart, dma_ch, buf_size)
    }
}

// ------------------------------------------------------------------------------------------------

/// UART Receiver
pub struct Rx<U> {
    uart: U,
}

impl<U: UartPeriphExt> Rx<U> {
    pub(crate) fn new(uart: U) -> Self {
        Self { uart }
    }

    pub fn into_poll<T: Timeout>(self, timeout: T, continue_retry_times: u32) -> UartPollRx<U, T> {
        UartPollRx::<U, T>::new(self.uart, timeout, continue_retry_times)
    }

    pub fn into_interrupt(
        self,
        buf_size: usize,
        retry_times: u32,
    ) -> (UartInterruptRx<U>, UartInterruptRxHandler<U>) {
        let u2 = unsafe { self.uart.steal() };
        UartInterruptRx::new([self.uart, u2], buf_size, retry_times)
    }

    pub fn into_dma_circle<CH>(self, dma_ch: CH, buf_size: usize) -> UartDmaRx<U, CH>
    where
        CH: DmaBindRx<U>,
    {
        UartDmaRx::<U, CH>::new(self.uart, dma_ch, buf_size)
    }
}
