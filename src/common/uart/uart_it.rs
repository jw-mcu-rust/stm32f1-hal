//! UART interrupt implementation

use super::*;
use crate::os;
use crate::ringbuf::*;
use embedded_io::{ErrorType, Read, Write};

// TX -------------------------------------------------------------------------

pub struct UartInterruptTx<U> {
    uart: U,
    w: Producer<u8>,
    transmit_retry_times: u32,
    flush_retry_times: u32,
}

impl<U> UartInterruptTx<U>
where
    U: UartDev,
{
    pub(super) fn new(
        uart: U,
        w: Producer<u8>,
        transmit_retry_times: u32,
        flush_retry_times: u32,
    ) -> Self {
        Self {
            uart,
            w,
            transmit_retry_times,
            flush_retry_times,
        }
    }
}

impl<U: UartDev> ErrorType for UartInterruptTx<U> {
    type Error = Error;
}

impl<U: UartDev> Write for UartInterruptTx<U> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        if buf.len() == 0 {
            return Err(Error::Other);
        }

        for _ in 0..=self.transmit_retry_times {
            let free_len = self.w.slots();
            if free_len > 0 {
                let chunk = self.w.write_chunk_uninit(free_len).unwrap();
                let n = chunk.copy_from_slice(buf);
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
            if self.uart.is_tx_empty() && self.uart.is_tx_complete() {
                return Ok(());
            } else {
                let tmp = self.w.slots();
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

pub struct UartInterruptTxHandler<U> {
    uart: U,
    r: Consumer<u8>,
}

impl<U> UartInterruptTxHandler<U>
where
    U: UartDev,
{
    pub(super) fn new(uart: U, r: Consumer<u8>) -> Self {
        Self { uart, r }
    }
}

impl<U> UartInterruptTxHandler<U>
where
    U: UartDev,
{
    pub fn handler(&mut self) {
        if let Ok(d) = self.r.peek() {
            if self.uart.write(*d as u16).is_ok() {
                self.r.pop().ok();
            }
        } else if self.uart.is_interrupt_enable(UartEvent::TxEmpty) {
            self.uart.set_interrupt(UartEvent::TxEmpty, false);
        }
    }
}

// RX -------------------------------------------------------------------------

pub struct UartInterruptRx<U> {
    uart: U,
    r: Consumer<u8>,
    retry_times: u32,
}

impl<U> UartInterruptRx<U>
where
    U: UartDev,
{
    pub(super) fn new(uart: U, r: Consumer<u8>, retry_times: u32) -> Self {
        Self {
            uart,
            r,
            retry_times,
        }
    }
}

impl<U: UartDev> ErrorType for UartInterruptRx<U> {
    type Error = Error;
}

impl<U> Read for UartInterruptRx<U>
where
    U: UartDev,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if buf.len() == 0 {
            return Err(Error::Other);
        }

        for _ in 0..=self.retry_times {
            let n = self.r.slots();
            if n > 0 {
                let chunk = self.r.read_chunk(n).unwrap();
                return Ok(chunk.copy_to_slice(buf));
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

pub struct UartInterruptRxHandler<U> {
    uart: U,
    w: Producer<u8>,
    e_count: u32,
    rbe_count: u32,
}

impl<U> UartInterruptRxHandler<U>
where
    U: UartDev,
{
    pub(super) fn new(mut uart: U, w: Producer<u8>) -> Self {
        uart.set_interrupt(UartEvent::RxNotEmpty, true);
        Self {
            uart,
            w,
            e_count: 0,
            rbe_count: 0,
        }
    }
}

impl<U> UartInterruptRxHandler<U>
where
    U: UartDev,
{
    pub fn handler(&mut self) {
        if let Ok(data) = self.uart.read() {
            if self.w.push(data as u8).is_err() {
                self.rbe_count = self.rbe_count.wrapping_add(1);
            }
        } else {
            self.e_count = self.e_count.wrapping_add(1);
        }
    }
}
