//! It doesn't depend on DMA or interrupts, relying instead on continuous polling.

use super::*;
use embedded_io::{ErrorType, Read, Write};

pub struct UartPollTx<U> {
    uart: U,
}

impl<U: UartReg> UartPollTx<U> {
    pub(super) fn new(uart: U) -> Self {
        Self { uart }
    }
}

impl<U: UartReg> ErrorType for UartPollTx<U> {
    type Error = Error;
}

impl<U: UartReg> Write for UartPollTx<U> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        for (i, &b) in buf.iter().enumerate() {
            match self.uart.write(b as u16) {
                Ok(()) => {}
                Err(nb::Error::WouldBlock) => {
                    return Ok(i);
                }
                Err(nb::Error::Other(_)) => {
                    return Err(Error::Other);
                }
            };
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        for _ in 0..8_000_000 {
            if self.uart.is_tx_empty() {
                return Ok(());
            }
        }
        Err(Error::Other)
    }
}

pub struct UartPollRx<U: 'static> {
    uart: U,
}

impl<U: UartReg> UartPollRx<U> {
    pub(super) fn new(uart: U) -> Self {
        Self { uart }
    }
}

impl<U: UartReg> ErrorType for UartPollRx<U> {
    type Error = Error;
}

impl<U: UartReg> Read for UartPollRx<U> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        for i in 0..buf.len() {
            match self.uart.read() {
                Ok(byte) => {
                    buf[i] = byte as u8;
                }
                Err(nb::Error::WouldBlock) => return Ok(i),
                Err(nb::Error::Other(e)) => return Err(e),
            }
        }
        Ok(buf.len())
    }
}
