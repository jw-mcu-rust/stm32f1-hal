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

    pub fn into_poll<T: Timeout>(self, timeout: T, flush_timeout: T) -> UartPollTx<U, T> {
        UartPollTx::new(self.uart, timeout, flush_timeout)
    }

    pub fn into_interrupt<T: Timeout>(
        self,
        buf_size: usize,
        timeout: T,
        flush_timeout: T,
    ) -> (UartInterruptTx<U, T>, UartInterruptTxHandler<U>) {
        let u2 = unsafe { self.uart.steal() };
        UartInterruptTx::new([self.uart, u2], buf_size, timeout, flush_timeout)
    }

    // pub fn into_dma<CH>(self, dma_ch: CH) -> UartDmaTx<U, CH>
    // where
    //     CH: BindDmaTx<U>,
    // {
    //     UartDmaTx::<U, CH>::new(self.uart, dma_ch)
    // }

    pub fn into_dma_ringbuf<CH, T>(
        self,
        dma_ch: CH,
        buf_size: usize,
        timeout: T,
        flush_timeout: T,
    ) -> (UartDmaBufTx<U, CH, T>, DmaRingbufTxLoader<u8, CH>)
    where
        CH: DmaBindTx<U>,
        T: Timeout,
    {
        UartDmaBufTx::new(self.uart, dma_ch, buf_size, timeout, flush_timeout)
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

    pub fn into_poll<T: Timeout>(self, timeout: T, continue_timeout: T) -> UartPollRx<U, T> {
        UartPollRx::new(self.uart, timeout, continue_timeout)
    }

    pub fn into_interrupt<T: Timeout>(
        self,
        buf_size: usize,
        timeout: T,
    ) -> (UartInterruptRx<U, T>, UartInterruptRxHandler<U>) {
        let u2 = unsafe { self.uart.steal() };
        UartInterruptRx::new([self.uart, u2], buf_size, timeout)
    }

    pub fn into_dma_circle<CH, T>(
        self,
        dma_ch: CH,
        buf_size: usize,
        timeout: T,
    ) -> UartDmaRx<U, CH, T>
    where
        CH: DmaBindRx<U>,
        T: Timeout,
    {
        UartDmaRx::new(self.uart, dma_ch, buf_size, timeout)
    }
}
