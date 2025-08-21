pub use rtrb::{
    Consumer, Producer, RingBuffer,
    chunks::{ReadChunk, WriteChunkUninit},
};

pub trait ProducerExt<T> {
    fn get_write_chunk_uninit(&mut self) -> Option<WriteChunkUninit<'_, T>>;
}
impl<T> ProducerExt<T> for Producer<T> {
    fn get_write_chunk_uninit(&mut self) -> Option<WriteChunkUninit<'_, T>> {
        let n = self.slots();
        if n > 0 {
            if let Ok(chunk) = self.write_chunk_uninit(n) {
                return Some(chunk);
            }
        }
        None
    }
}

pub trait WriteChunkExt<T> {
    fn get_mut_slice(&mut self) -> &mut [T];
    fn get_mut_slices(&mut self) -> (&mut [T], &mut [T]);
    fn copy_from_slice(self, buf: &[T]) -> usize;
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

    fn copy_from_slice(mut self, buf: &[T]) -> usize {
        let (c1, c2) = self.get_mut_slices();

        if c1.len() == 0 {
            return 0;
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
            self.commit(n);
        }
        n
    }
}

pub trait ConsumerExt<T> {
    fn get_read_chunk(&mut self) -> Option<ReadChunk<'_, T>>;
}
impl<T> ConsumerExt<T> for Consumer<T> {
    fn get_read_chunk(&mut self) -> Option<ReadChunk<'_, T>> {
        let n = self.slots();
        if n > 0 {
            if let Ok(chunk) = self.read_chunk(n) {
                return Some(chunk);
            }
        }
        None
    }
}

pub trait ReadChunkExt<T> {
    fn get_slice(&self) -> &[T];
    fn copy_to_slice(self, buf: &mut [T]) -> usize;
}
impl<T: Copy> ReadChunkExt<T> for ReadChunk<'_, T> {
    fn get_slice(&self) -> &[T] {
        let (buf, _) = self.as_slices();
        buf
    }

    fn copy_to_slice(self, buf: &mut [T]) -> usize {
        let (c1, c2) = self.as_slices();
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
        self.commit(n);
        n
    }
}
