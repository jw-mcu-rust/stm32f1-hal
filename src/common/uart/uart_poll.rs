//! It doesn't depend on DMA or interrupts, relying instead on continuous polling.

use super::*;
use crate::os;
use embedded_io::{ErrorType, Read, Write};

// TX -------------------------------------------------------------------------

pub struct UartPollTx<U> {
    uart: U,
    flush_retry_times: u32,
}

impl<U: UartPeripheral> UartPollTx<U> {
    pub(super) fn new(uart: U, flush_retry_times: u32) -> Self {
        Self {
            uart,
            flush_retry_times,
        }
    }
}

impl<U: UartPeripheral> ErrorType for UartPollTx<U> {
    type Error = Error;
}

impl<U: UartPeripheral> Write for UartPollTx<U> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        for (i, &data) in buf.iter().enumerate() {
            match self.uart.write(data as u16) {
                Ok(()) => {}
                Err(nb::Error::WouldBlock) => {
                    if i > 0 {
                        return Ok(i);
                    } else {
                        return Err(Error::Busy);
                    }
                }
                Err(nb::Error::Other(_)) => {
                    return Err(Error::Other);
                }
            };
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        for _ in 0..self.flush_retry_times {
            if self.uart.is_tx_empty() && self.uart.is_tx_complete() {
                return Ok(());
            }
            os::yield_cpu();
        }
        Err(Error::Other)
    }
}

// RX -------------------------------------------------------------------------

pub struct UartPollRx<U> {
    uart: U,
    continue_receive_retry_times: u32,
}

impl<U: UartPeripheral> UartPollRx<U> {
    pub(super) fn new(uart: U, continue_receive_retry_times: u32) -> Self {
        Self {
            uart,
            continue_receive_retry_times,
        }
    }
}

impl<U: UartPeripheral> ErrorType for UartPollRx<U> {
    type Error = Error;
}

impl<U: UartPeripheral> Read for UartPollRx<U> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut retry = self.continue_receive_retry_times;

        let mut i = 0;
        while i < buf.len() {
            match self.uart.read() {
                Ok(byte) => {
                    buf[i] = byte as u8;
                    i += 1;
                    retry = 0;
                }
                Err(nb::Error::WouldBlock) => {
                    retry += 1;
                    if retry > self.continue_receive_retry_times {
                        return Ok(i);
                    }
                    os::yield_cpu();
                }
                Err(nb::Error::Other(e)) => return Err(e),
            }
        }
        Ok(buf.len())
    }
}
