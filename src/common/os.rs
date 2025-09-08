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

pub trait Timeout {
    fn start(&mut self) -> impl TimeoutInstance;
}

pub trait TimeoutInstance {
    fn timeout(&mut self) -> bool;
    fn restart(&mut self);
    fn interval(&self);
}

// Retry ----------------------------------

pub struct RetryTimes {
    retry_times: usize,
}
impl RetryTimes {
    pub fn new(retry_times: usize) -> Self {
        Self { retry_times }
    }
}
impl Timeout for RetryTimes {
    #[inline]
    fn start(&mut self) -> impl TimeoutInstance {
        RetryTimesInstance {
            count: 0,
            retry_times: self.retry_times,
        }
    }
}

pub struct RetryTimesInstance {
    count: usize,
    retry_times: usize,
}
impl TimeoutInstance for RetryTimesInstance {
    #[inline]
    fn timeout(&mut self) -> bool {
        if self.count <= self.retry_times {
            self.count = self.count.wrapping_add(1);
            false
        } else {
            true
        }
    }

    #[inline(always)]
    fn restart(&mut self) {
        self.count = 0;
    }

    #[inline(always)]
    fn interval(&self) {}
}

// Always ----------------------------------

pub struct AlwaysTimeout {}
impl AlwaysTimeout {
    pub fn new() -> Self {
        Self {}
    }
}
impl Timeout for AlwaysTimeout {
    #[inline]
    fn start(&mut self) -> impl TimeoutInstance {
        AlwaysTimeoutInstance {}
    }
}

pub struct AlwaysTimeoutInstance {}
impl TimeoutInstance for AlwaysTimeoutInstance {
    #[inline(always)]
    fn timeout(&mut self) -> bool {
        true
    }

    #[inline(always)]
    fn restart(&mut self) {}

    #[inline(always)]
    fn interval(&self) {}
}
