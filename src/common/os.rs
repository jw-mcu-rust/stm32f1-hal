cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        pub use std::sync::Arc;
    } else {
        pub use alloc::vec::Vec;
        pub use alloc::boxed::Box;
        pub use alloc::sync::Arc;
    }
}

pub use waiter_trait::{Waiter, WaiterStatus};
