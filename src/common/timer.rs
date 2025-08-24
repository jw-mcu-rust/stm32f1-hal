use crate::time::Hertz;

pub trait PwmTimer {
    fn start(&mut self);
    fn stop(&mut self);
    fn config_freq(&mut self, count_freq: Hertz, update_freq: Hertz);
    fn get_max_duty(&self) -> u32;
    fn get_count_value(&self) -> u32;
}

pub trait PwmChannel {
    fn config(&mut self, mode: PwmMode, polarity: PwmPolarity);
    fn set_enable(&mut self, en: bool);
    fn get_max_duty(&self) -> u32;
    /// Remember to use `get_max_duty()`
    fn set_duty(&mut self, duty: u32);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PwmMode {
    Mode1,
    Mode2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PwmPolarity {
    ActiveHigh,
    ActiveLow,
}
