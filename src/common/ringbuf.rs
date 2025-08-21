pub use rtrb::{
    Consumer, Producer, RingBuffer,
    chunks::{ReadChunk, WriteChunkUninit},
};

pub trait ProducerExt<T> {
    fn get_write_chunk_uninit(&mut self) -> Option<WriteChunkUninit<'_, T>>;
    fn push_slice(&mut self, buf: &[T]) -> Option<usize>;
}
impl<T: Copy> ProducerExt<T> for Producer<T> {
    fn get_write_chunk_uninit(&mut self) -> Option<WriteChunkUninit<'_, T>> {
        let n = self.slots();
        if n > 0 {
            if let Ok(chunk) = self.write_chunk_uninit(n) {
                return Some(chunk);
            }
        }
        None
    }

    fn push_slice(&mut self, buf: &[T]) -> Option<usize> {
        let n = self.slots();
        if n > 0 {
            let mut chunk = self.write_chunk_uninit(n).unwrap();
            let (c1, c2) = chunk.get_mut_slices();

            if c1.len() == 0 {
                return None;
            }

            let n = if c1.len() >= buf.len() {
                c1[..buf.len()].copy_from_slice(buf);
                buf.len()
            } else {
                let (b1, b2) = buf.split_at(c1.len());
                c1.copy_from_slice(b1);
                let min = b2.len().min(c2.len());
                c2[..min].copy_from_slice(&b2[..min]);
                c1.len() + min
            };
            unsafe {
                chunk.commit(n);
            }
            Some(n)
        } else {
            None
        }
    }
}

pub trait WriteChunkExt<T> {
    fn get_mut_slice(&mut self) -> &mut [T];
    fn get_mut_slices(&mut self) -> (&mut [T], &mut [T]);
}
impl<T: Copy> WriteChunkExt<T> for WriteChunkUninit<'_, T> {
    fn get_mut_slice(&mut self) -> &mut [T] {
        let (buf, _) = self.as_mut_slices();
        unsafe {
            let dst_ptr = buf.as_mut_ptr().cast();
            core::slice::from_raw_parts_mut(dst_ptr, buf.len())
        }
    }

    fn get_mut_slices(&mut self) -> (&mut [T], &mut [T]) {
        let (a, b) = self.as_mut_slices();
        unsafe {
            (
                core::slice::from_raw_parts_mut(a.as_mut_ptr().cast(), a.len()),
                core::slice::from_raw_parts_mut(b.as_mut_ptr().cast(), b.len()),
            )
        }
    }
}

pub trait ConsumerExt<T> {
    fn get_read_chunk(&mut self) -> Option<ReadChunk<'_, T>>;
    fn pop_slice(&mut self, elems: &mut [T]) -> Option<usize>;
}
impl<T: Copy> ConsumerExt<T> for Consumer<T> {
    fn get_read_chunk(&mut self) -> Option<ReadChunk<'_, T>> {
        let n = self.slots();
        if n > 0 {
            if let Ok(chunk) = self.read_chunk(n) {
                return Some(chunk);
            }
        }
        None
    }

    fn pop_slice(&mut self, buf: &mut [T]) -> Option<usize> {
        let n = self.slots();
        if n > 0 {
            let chunk = self.read_chunk(n).unwrap();
            let (c1, c2) = chunk.as_slices();
            let n = if c1.len() >= buf.len() {
                buf.copy_from_slice(&c1[..buf.len()]);
                buf.len()
            } else {
                let (b1, b2) = buf.split_at_mut(c1.len());
                b1.copy_from_slice(c1);
                let min = b2.len().min(c2.len());
                b2[..min].copy_from_slice(&c2[..min]);
                c1.len() + min
            };
            chunk.commit(n);
            Some(n)
        } else {
            None
        }
    }
}

pub trait ReadChunkExt<T> {
    fn get_slice(&self) -> &[T];
}
impl<T: Copy> ReadChunkExt<T> for ReadChunk<'_, T> {
    fn get_slice(&self) -> &[T] {
        let (buf, _) = self.as_slices();
        buf
    }
}
