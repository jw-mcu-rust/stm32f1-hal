use crate::embedded_hal::digital::StatefulOutputPin;

pub struct LedTask<P> {
    led: P,
    freq: u32,
    count: u32,
}

impl<P: StatefulOutputPin> LedTask<P> {
    pub fn new(led: P, freq: u32) -> Self {
        Self {
            led,
            freq,
            count: 0,
        }
    }

    pub fn poll(&mut self) {
        self.count += 1;
        if self.count >= self.freq {
            self.led.toggle().ok();
            self.count = 0;
        }
    }
}
