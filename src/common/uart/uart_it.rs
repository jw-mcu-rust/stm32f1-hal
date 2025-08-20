//! UART interrupt implementation

use super::*;
use crate::os;
use embedded_io::{ErrorType, Read, Write};

// TX -------------------------------------------------------------------------

pub struct UartInterruptTx<U, W> {
    uart: U,
    w: W,
    transmit_retry_times: u32,
    flush_retry_times: u32,
}

impl<U, W> UartInterruptTx<U, W>
where
    U: UartDev,
    W: Producer,
{
    pub(super) fn new(uart: U, w: W, transmit_retry_times: u32, flush_retry_times: u32) -> Self {
        Self {
            uart,
            w,
            transmit_retry_times,
            flush_retry_times,
        }
    }
}

impl<U: UartDev, W> ErrorType for UartInterruptTx<U, W> {
    type Error = Error;
}

impl<U: UartDev, W: Producer<Item = u8>> Write for UartInterruptTx<U, W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        for _ in 0..=self.transmit_retry_times {
            let n = self.w.push_slice(buf);
            if n > 0 {
                self.uart.set_interrupt(UartEvent::TxEmpty, true);
                return Ok(n);
            } else if !self.uart.is_interrupt_enable(UartEvent::TxEmpty) {
                self.uart.set_interrupt(UartEvent::TxEmpty, true);
            }
            os::yield_cpu();
        }
        return Err(Error::Busy);
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        let mut retry = 0;
        let mut rest = 0;
        loop {
            if self.w.is_empty() && self.uart.is_tx_empty() && self.uart.is_tx_complete() {
                return Ok(());
            } else {
                let tmp = self.w.occupied_len();
                if rest != tmp {
                    rest = tmp;
                    retry = 0;
                } else {
                    // unchanged
                    retry += 1;
                    if retry > self.flush_retry_times {
                        return Err(Error::Other);
                    }
                    if !self.uart.is_interrupt_enable(UartEvent::TxEmpty) {
                        self.uart.set_interrupt(UartEvent::TxEmpty, true);
                    }
                }
                os::yield_cpu();
            }
        }
    }
}

// TX interrupt -----------------

pub struct UartInterruptTxHandler<U, R> {
    uart: U,
    r: R,
}

impl<U, R> UartInterruptTxHandler<U, R>
where
    U: UartDev,
    R: Consumer,
{
    pub(super) fn new(uart: U, r: R) -> Self {
        Self { uart, r }
    }
}

impl<U, R> UartInterruptTxHandler<U, R>
where
    U: UartDev,
    R: Consumer<Item = u8>,
{
    pub fn handler(&mut self) {
        if self.uart.is_interrupted(UartEvent::TxEmpty) {
            if let Some(d) = self.r.try_pop() {
                self.uart.write(d as u16).ok();
            } else {
                self.uart.set_interrupt(UartEvent::TxEmpty, false);
            }
        }
    }
}

// RX -------------------------------------------------------------------------

pub struct UartInterruptRx<U, R> {
    uart: U,
    r: R,
    retry_times: u32,
}

impl<U, R> UartInterruptRx<U, R>
where
    U: UartDev,
    R: Consumer,
{
    pub(super) fn new(uart: U, r: R, retry_times: u32) -> Self {
        Self {
            uart,
            r,
            retry_times,
        }
    }
}

impl<U: UartDev, R> ErrorType for UartInterruptRx<U, R> {
    type Error = Error;
}

impl<U, R> Read for UartInterruptRx<U, R>
where
    U: UartDev,
    R: Consumer<Item = u8>,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        for _ in 0..=self.retry_times {
            let n = self.r.pop_slice(buf);
            if n > 0 {
                return Ok(n);
            }
            if !self.uart.is_interrupt_enable(UartEvent::RxNotEmpty) {
                self.uart.set_interrupt(UartEvent::RxNotEmpty, true);
            }
            os::yield_cpu();
        }
        Err(Error::Other)
    }
}

// RX interrupt -----------------

pub struct UartInterruptRxHandler<U, W> {
    uart: U,
    w: W,
}

impl<U, W> UartInterruptRxHandler<U, W>
where
    U: UartDev,
    W: Producer,
{
    pub(super) fn new(mut uart: U, w: W) -> Self {
        uart.set_interrupt(UartEvent::RxNotEmpty, true);
        Self { uart, w }
    }
}

impl<U, W> UartInterruptRxHandler<U, W>
where
    U: UartDev,
    W: Producer<Item = u8>,
{
    pub fn handler(&mut self) {
        if self.uart.is_interrupted(UartEvent::RxNotEmpty) {
            if let Ok(data) = self.uart.read() {
                self.w.try_push(data as u8).ok();
            }
        }
    }
}
