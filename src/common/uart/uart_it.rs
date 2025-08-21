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
            if let Some(n) = self.w.push_slice(buf) {
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
                    } else if !self.uart.is_interrupt_enable(UartEvent::TxEmpty) {
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
        if let Ok(data) = self.r.peek() {
            if self.uart.write(*data as u16).is_ok() {
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
            if let Some(n) = self.r.pop_slice(buf) {
                return Ok(n);
            } else if !self.uart.is_interrupt_enable(UartEvent::RxNotEmpty) {
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
    // count: [u32; 10],
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
            // count: [0; 10],
        }
    }

    pub fn handler(&mut self) {
        if let Ok(data) = self.uart.read() {
            self.w.push(data as u8).ok();
        }

        // match self.uart.read() {
        //     Ok(data) => match self.w.push(data as u8) {
        //         Ok(()) => self.count[0] = self.count[0].saturating_add(1),
        //         Err(_) => self.count[1] = self.count[1].saturating_add(1),
        //     },
        //     Err(nb::Error::WouldBlock) => self.count[2] = self.count[2].saturating_add(1),
        //     Err(nb::Error::Other(e)) => match e {
        //         Error::Overrun => self.count[3] = self.count[3].saturating_add(1),
        //         Error::Other => self.count[4] = self.count[4].saturating_add(1),
        //         Error::Noise => self.count[5] = self.count[5].saturating_add(1),
        //         Error::FrameFormat => self.count[6] = self.count[6].saturating_add(1),
        //         Error::Parity => self.count[7] = self.count[7].saturating_add(1),
        //         Error::Busy => self.count[8] = self.count[8].saturating_add(1),
        //     },
        // }
    }
}
