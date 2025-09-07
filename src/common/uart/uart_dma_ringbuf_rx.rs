use super::*;
use crate::{common::dma::DmaChannel, os, ringbuf::*};

pub struct UartDmaBufRx<U, CH> {
    uart: U,
    ch: CH,
}

impl<U, CH> UartDmaBufRx<U, CH>
where
    U: UartPeriph,
    CH: DmaChannel,
{
    pub fn new(uart: U, dma_ch: CH, buf_size: usize) -> Self {
        // uart.set_dma_rx(true);
        // let ch = DmaRingbufRx::<u8, CH>::new(dma_ch, uart.get_rx_data_reg_addr(), buf_size);
        Self { uart, ch: dma_ch }
    }
}

// impl<U, CH> Read<u8> for UartDmaBufRx<U, CH>
// where
//     U: UartReg,
//     CH: DmaChannelReg,
// {
//     fn read(&mut self, buf: &mut [u8]) -> Result<usize, HalError> {
//         if let Some(d) = self.ch.read(buf.len()) {
//             buf[..d.len()].copy_from_slice(d);
//             Ok(d.len())
//         } else {
//             Err(HalError::Empty)
//         }
//     }
// }

// impl<U, CH> ReadGeneric<u8> for UartDmaBufRx<U, CH>
// where
//     U: UartReg,
//     CH: DmaChannelReg,
// {
//     fn read_closure(&mut self, mut f: impl FnMut(&[u8])) {
//         if let Some(d) = self.ch.read(usize::MAX) {
//             f(d);
//             // it needs to be read twice due to the internal circular feature
//             if let Some(d) = self.ch.read(usize::MAX) {
//                 f(d);
//             }
//         }
//     }
// }

// impl<U, CH> ReadBlock<u8> for UartDmaBufRx<U, CH>
// where
//     U: UartReg,
//     CH: DmaChannelReg,
// {
//     fn read_all(&mut self, buf: &mut [u8], timeout: u32) -> Result<(), HalError> {
//         let t = os::time_now();
//         let mut tmp = buf;
//         while tmp.len() > 0 {
//             let data = self.ch.read(tmp.len());
//             if let Some(d) = data {
//                 tmp[..d.len()].copy_from_slice(d);
//                 tmp = &mut tmp[d.len()..];
//             } else {
//                 os::sleep_ms(1);
//             }

//             if os::time_elapsed(t) >= timeout {
//                 return Err(HalError::Timeout);
//             }
//         }
//         Ok(())
//     }
// }
