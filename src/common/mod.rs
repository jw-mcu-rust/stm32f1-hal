pub mod uart;
pub mod wrap_trait;

#[macro_export]
macro_rules! static_ringbuf_init {
    ($t:ty, $size:literal) => {
        unsafe {
            static mut RB: MaybeUninit<StaticRb<$t, $size>> = MaybeUninit::uninit();
            RB.write(StaticRb::default());
            RB.assume_init_mut()
        }
    };
}
