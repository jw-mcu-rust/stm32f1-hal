cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        pub use std::sync::Arc;
    } else {
        pub use alloc::vec::Vec;
        pub use alloc::boxed::Box;
        pub use alloc::sync::Arc;
    }
}

#[inline(always)]
pub fn yield_cpu() {
    // TODO yield
}
