pub trait DmaChannel {
    fn start(&mut self);
    fn stop(&mut self);

    fn set_peripheral_address<T: Sized>(
        &mut self,
        address: usize,
        mem_to_periph: bool,
        increase: bool,
        circular: bool,
    );
    fn set_memory_address(&mut self, address: usize, increase: bool);
    fn set_transfer_length(&mut self, len: usize);
    fn set_memory_buf_for_peripheral<T: Sized>(&mut self, buf: &[T]) {
        self.set_memory_address(buf.as_ptr() as usize, true);
        self.set_transfer_length(buf.len());
    }

    fn set_memory_to_memory<T: Sized>(&mut self, src_addr: usize, dst_addr: usize, len: usize);

    fn get_left_len(&self) -> usize;
    fn in_progress(&self) -> bool;

    fn set_interrupt(&mut self, event: DmaEvent, enable: bool);
    fn is_interrupted(&mut self, event: DmaEvent) -> bool;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DmaEvent {
    TransferComplete,
    HalfTransfer,
}
