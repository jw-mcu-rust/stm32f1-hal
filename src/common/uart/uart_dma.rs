use super::*;
use crate::common::dma::*;
use embedded_io::{ErrorType, Read, Write};

// TX -------------------------------------------------------------------------

pub struct UartDmaBufTx<U, CH> {
    _uart: U,
    w: DmaRingbufTxWriter<u8, CH>,
}

impl<U, CH> UartDmaBufTx<U, CH>
where
    U: UartPeriph,
    CH: DmaChannel,
{
    pub fn new(mut uart: U, dma_ch: CH, buf_size: usize) -> (Self, DmaRingbufTxLoader<u8, CH>) {
        uart.enable_dma_tx(true);
        let (w, l) = DmaRingbufTx::new(dma_ch, uart.get_tx_data_reg_addr(), buf_size);
        (Self { _uart: uart, w }, l)
    }
}

impl<U, CH> ErrorType for UartDmaBufTx<U, CH>
where
    U: UartPeriph,
    CH: DmaChannel,
{
    type Error = Error;
}

impl<U, CH> Write for UartDmaBufTx<U, CH>
where
    U: UartPeriph,
    CH: DmaChannel,
{
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        // TODO block
        if let n @ 1.. = self.w.write(buf) {
            Ok(n)
        } else {
            Err(Error::Busy)
        }
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        // TODO
        Ok(())
    }
}

// RX -------------------------------------------------------------------------

pub struct UartDmaRx<U, CH> {
    _uart: U,
    ch: DmaCircularBufferRx<u8, CH>,
}

impl<U, CH> UartDmaRx<U, CH>
where
    U: UartPeriph,
    CH: DmaChannel,
{
    pub fn new(mut uart: U, dma_ch: CH, buf_size: usize) -> Self {
        let ch = DmaCircularBufferRx::<u8, CH>::new(dma_ch, uart.get_rx_data_reg_addr(), buf_size);
        uart.enable_dma_rx(true);
        Self { _uart: uart, ch }
    }
}

impl<U, CH> ErrorType for UartDmaRx<U, CH>
where
    U: UartPeriph,
    CH: DmaChannel,
{
    type Error = Error;
}

impl<U, CH> Read for UartDmaRx<U, CH>
where
    U: UartPeriph,
    CH: DmaChannel,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        // TODO block
        if let Some(d) = self.ch.read(buf.len()) {
            buf[..d.len()].copy_from_slice(d);
            Ok(d.len())
        } else {
            Err(Error::Other)
        }
    }
}
