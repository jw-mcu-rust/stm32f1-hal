use super::*;
use crate::common::{dma::*, os::*};
use embedded_io::{ErrorType, Read, Write};

// TX -------------------------------------------------------------------------

pub struct UartDmaBufTx<U, CH, T> {
    _uart: U,
    w: DmaRingbufTxWriter<u8, CH>,
    timeout: T,
    flush_timeout: T,
}

impl<U, CH, T> UartDmaBufTx<U, CH, T>
where
    U: UartPeriph,
    CH: DmaChannel,
    T: Timeout,
{
    pub fn new(
        mut uart: U,
        dma_ch: CH,
        buf_size: usize,
        timeout: T,
        flush_timeout: T,
    ) -> (Self, DmaRingbufTxLoader<u8, CH>) {
        uart.enable_dma_tx(true);
        let (w, l) = DmaRingbufTx::new(dma_ch, uart.get_tx_data_reg_addr(), buf_size);
        (
            Self {
                _uart: uart,
                w,
                timeout,
                flush_timeout,
            },
            l,
        )
    }
}

impl<U, CH, T> ErrorType for UartDmaBufTx<U, CH, T>
where
    U: UartPeriph,
    CH: DmaChannel,
    T: Timeout,
{
    type Error = Error;
}

impl<U, CH, T> Write for UartDmaBufTx<U, CH, T>
where
    U: UartPeriph,
    CH: DmaChannel,
    T: Timeout,
{
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        if buf.len() == 0 {
            return Err(Error::Other);
        }

        let mut t = self.timeout.start();
        loop {
            if let n @ 1.. = self.w.write(buf) {
                return Ok(n);
            } else if t.timeout() {
                break;
            }
            t.interval();
        }
        Err(Error::Busy)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        let mut t = self.flush_timeout.start();
        loop {
            if !self.w.in_progress() {
                return Ok(());
            } else if t.timeout() {
                break;
            }
            t.interval();
        }
        Err(Error::Other)
    }
}

// RX -------------------------------------------------------------------------

pub struct UartDmaRx<U, CH, T> {
    _uart: U,
    ch: DmaCircularBufferRx<u8, CH>,
    timeout: T,
}

impl<U, CH, T> UartDmaRx<U, CH, T>
where
    U: UartPeriph,
    CH: DmaChannel,
    T: Timeout,
{
    pub fn new(mut uart: U, dma_ch: CH, buf_size: usize, timeout: T) -> Self {
        let ch = DmaCircularBufferRx::<u8, CH>::new(dma_ch, uart.get_rx_data_reg_addr(), buf_size);
        uart.enable_dma_rx(true);
        Self {
            _uart: uart,
            ch,
            timeout,
        }
    }
}

impl<U, CH, T> ErrorType for UartDmaRx<U, CH, T>
where
    U: UartPeriph,
    CH: DmaChannel,
    T: Timeout,
{
    type Error = Error;
}

impl<U, CH, T> Read for UartDmaRx<U, CH, T>
where
    U: UartPeriph,
    CH: DmaChannel,
    T: Timeout,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if buf.len() == 0 {
            return Err(Error::Other);
        }

        let mut t = self.timeout.start();
        loop {
            if let Some(d) = self.ch.read(buf.len()) {
                buf[..d.len()].copy_from_slice(d);
                return Ok(d.len());
            } else if t.timeout() {
                break;
            }
            t.interval();
        }
        Err(Error::Other)
    }
}
