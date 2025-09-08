//! It doesn't depend on DMA or interrupts, relying instead on continuous polling.

use super::*;
use crate::common::os::*;
use embedded_hal_nb as e_nb;
use embedded_io as e_io;

// TX -------------------------------------------------------------------------

pub struct UartPollTx<U, T> {
    uart: U,
    timeout: T,
    flush_retry_times: u32,
}

impl<U: UartPeriph, T: Timeout> UartPollTx<U, T> {
    pub fn new(uart: U, timeout: T, flush_retry_times: u32) -> Self {
        Self {
            uart,
            timeout,
            flush_retry_times,
        }
    }
}

impl<U: UartPeriph, T: Timeout> e_nb::serial::ErrorType for UartPollTx<U, T> {
    type Error = Error;
}
impl<U: UartPeriph, T: Timeout> e_io::ErrorType for UartPollTx<U, T> {
    type Error = Error;
}

// NB Write ----

impl<U: UartPeriph, T: Timeout> e_nb::serial::Write<u16> for UartPollTx<U, T> {
    #[inline]
    fn write(&mut self, word: u16) -> nb::Result<(), Self::Error> {
        self.uart.write(word)
    }

    #[inline]
    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        if self.uart.is_tx_empty() && self.uart.is_tx_complete() {
            return Ok(());
        }
        Err(nb::Error::WouldBlock)
    }
}

// IO Write ----

impl<U: UartPeriph, T: Timeout> e_io::Write for UartPollTx<U, T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        if buf.len() == 0 {
            return Err(Error::Other);
        }

        // try first data
        let mut rst = Err(nb::Error::WouldBlock);
        let mut t = self.timeout.start();
        while !t.timeout() {
            rst = self.uart.write(buf[0] as u16);
            if let Err(nb::Error::WouldBlock) = rst {
                t.interval();
            } else {
                break;
            }
        }

        match rst {
            Ok(()) => (),
            Err(nb::Error::WouldBlock) => return Err(Error::Busy),
            Err(nb::Error::Other(_)) => return Err(Error::Other),
        }

        // write rest data
        for (i, &data) in buf[1..buf.len()].iter().enumerate() {
            if self.uart.write(data as u16).is_err() {
                return Ok(i + 1);
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        for _ in 0..=self.flush_retry_times {
            if self.uart.is_tx_empty() && self.uart.is_tx_complete() {
                return Ok(());
            }
            yield_cpu();
        }
        Err(Error::Other)
    }
}

// RX -------------------------------------------------------------------------

pub struct UartPollRx<U, T> {
    uart: U,
    timeout: T,
    continue_retry_times: u32,
}

impl<U: UartPeriph, T: Timeout> UartPollRx<U, T> {
    pub fn new(uart: U, timeout: T, continue_retry_times: u32) -> Self {
        Self {
            uart,
            timeout,
            continue_retry_times,
        }
    }
}

impl<U: UartPeriph, T: Timeout> e_nb::serial::ErrorType for UartPollRx<U, T> {
    type Error = Error;
}
impl<U: UartPeriph, T: Timeout> e_io::ErrorType for UartPollRx<U, T> {
    type Error = Error;
}

// NB Read ----

impl<U: UartPeriph, T: Timeout> e_nb::serial::Read<u16> for UartPollRx<U, T> {
    #[inline]
    fn read(&mut self) -> nb::Result<u16, Self::Error> {
        self.uart.read()
    }
}

// IO Read ----

impl<U: UartPeriph, T: Timeout> e_io::Read for UartPollRx<U, T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if buf.len() == 0 {
            return Err(Error::Other);
        }

        // try first data
        let mut rst = Err(nb::Error::WouldBlock);
        let mut t = self.timeout.start();
        while !t.timeout() {
            rst = self.uart.read();
            if let Err(nb::Error::WouldBlock) = rst {
                t.interval();
            } else {
                break;
            }
        }

        match rst {
            Ok(data) => buf[0] = data as u8,
            _ => return Err(Error::Other),
        }

        let mut retry = 0;
        let mut n = 1;
        while n < buf.len() {
            match self.uart.read() {
                Ok(data) => {
                    buf[n] = data as u8;
                    n += 1;
                    retry = 0;
                }
                Err(nb::Error::Other(_)) => return Ok(n),
                Err(nb::Error::WouldBlock) => {
                    if retry >= self.continue_retry_times {
                        return Ok(n);
                    }
                    retry += 1;
                    yield_cpu();
                }
            }
        }
        Ok(buf.len())
    }
}
