use crate::{embedded_hal::digital::StatefulOutputPin, waiter_trait::WaiterStatus};

pub struct LedTask<P, T> {
    led: P,
    timeout: T,
}

impl<P: StatefulOutputPin, T: WaiterStatus> LedTask<P, T> {
    pub fn new(led: P, timeout: T) -> Self {
        Self { led, timeout }
    }

    pub fn poll(&mut self) {
        if self.timeout.timeout() {
            self.led.toggle().ok();
        }
    }
}
