use super::*;
use crate::common::os::*;

/// A buffer used for DMA cyclic data reception, continuously read by the user.
pub struct DmaCircularBufferRx<T: Sized, CH> {
    ch: CH,
    buf: CircularBuffer<T>,
}

impl<T, CH> DmaCircularBufferRx<T, CH>
where
    T: Sized + Copy,
    CH: DmaChannel,
{
    pub fn new(mut ch: CH, peripheral_addr: usize, buf_size: usize) -> Self {
        let buf = CircularBuffer::<T>::new(buf_size);
        ch.stop();
        ch.set_memory_buf_for_peripheral(buf.as_slice());
        ch.set_peripheral_address::<T>(peripheral_addr, false, false, true);
        ch.start();
        Self { ch, buf }
    }

    pub fn read(&mut self, max: usize) -> Option<&[T]> {
        self.buf.read(self.ch.get_left_len(), max)
    }
}

pub struct CircularBuffer<T: Sized> {
    recv_buf: Vec<T>,
    read_idx: usize,
}

impl<T> CircularBuffer<T> {
    fn new(buf_size: usize) -> Self {
        let mut recv_buf = Vec::<T>::with_capacity(buf_size);
        unsafe { recv_buf.set_len(buf_size) }

        Self {
            recv_buf,
            read_idx: 0,
        }
    }

    fn read(&mut self, left_len: usize, max: usize) -> Option<&[T]> {
        let dma_recv_idx = if left_len == 0 {
            0
        } else {
            self.recv_buf.len() - left_len
        };

        if self.read_idx == dma_recv_idx {
            return None;
        }

        let ret;
        if dma_recv_idx < self.read_idx {
            if max > self.recv_buf.len() - self.read_idx {
                ret = Some(&self.recv_buf[self.read_idx..]);
                self.read_idx = 0;
            } else {
                let end = self.read_idx + max;
                ret = Some(&self.recv_buf[self.read_idx..end]);
                self.read_idx = end;
            }
        } else {
            if max > dma_recv_idx - self.read_idx {
                ret = Some(&self.recv_buf[self.read_idx..dma_recv_idx]);
                self.read_idx = dma_recv_idx;
            } else {
                let end = self.read_idx + max;
                ret = Some(&self.recv_buf[self.read_idx..end]);
                self.read_idx = end;
            };
        }

        return ret;
    }

    fn as_slice(&self) -> &[T] {
        self.recv_buf.as_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circular_buffer() {
        let buf_size = 13;
        let mut buf = CircularBuffer::new(buf_size);
        assert_eq!(buf.recv_buf.len(), buf_size);

        for i in 0..buf_size {
            buf.recv_buf[i] = i as u8;
        }

        assert_eq!(
            buf.read(5, usize::MAX),
            Some([0u8, 1, 2, 3, 4, 5, 6, 7].as_slice())
        );
        assert_eq!(buf.read(5, usize::MAX), None);
        // Single wraparound
        assert_eq!(
            buf.read(0, usize::MAX),
            Some([8u8, 9, 10, 11, 12].as_slice())
        );
        assert_eq!(buf.read(0, usize::MAX), None);
        assert_eq!(buf.read(buf_size, usize::MAX), None);
        // small max
        assert_eq!(buf.read(5, 5), Some([0u8, 1, 2, 3, 4].as_slice()));
        assert_eq!(buf.read(5, 5), Some([5u8, 6, 7].as_slice()));
        assert_eq!(buf.read(5, 5), None);
        assert_eq!(
            buf.read(0, usize::MAX),
            Some([8u8, 9, 10, 11, 12].as_slice())
        );
        // Multiple wraparounds
        assert_eq!(
            buf.read(5, usize::MAX),
            Some([0u8, 1, 2, 3, 4, 5, 6, 7].as_slice())
        );
        assert_eq!(
            buf.read(10, usize::MAX),
            Some([8u8, 9, 10, 11, 12].as_slice())
        );
        assert_eq!(buf.read(10, usize::MAX), Some([0u8, 1, 2].as_slice()));
        assert_eq!(buf.read(10, usize::MAX), None);
    }
}
