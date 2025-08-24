pub mod pwm;
pub use pwm::*;
pub mod counter;
pub use counter::*;
pub mod fix_timer;
pub use fix_timer::*;
pub mod delay;
pub use delay::*;

use crate::time::Hertz;

pub trait PwmChannel: embedded_hal::pwm::SetDutyCycle {
    fn config(&mut self, mode: PwmMode, polarity: PwmPolarity);
    fn set_enable(&mut self, en: bool);
}

// ----------------------------------------------------------------------------

pub trait GeneralTimer {
    fn reset_config(&mut self);
    fn enable_counter(&mut self);
    fn disable_counter(&mut self);
    fn is_counter_enabled(&self) -> bool;
    fn reset_counter(&mut self);
    fn max_auto_reload() -> u32;
    unsafe fn set_auto_reload_unchecked(&mut self, arr: u32);
    fn set_auto_reload(&mut self, arr: u32) -> Result<(), Error>;
    fn read_auto_reload(&self) -> u32;
    fn set_prescaler(&mut self, psc: u16);
    fn read_prescaler(&self) -> u16;
    fn read_count(&self) -> u32;
    fn trigger_update(&mut self);
    fn stop_in_debug(&mut self, state: bool);
    fn config_freq(&mut self, clock: Hertz, count_freq: Hertz, update_freq: Hertz);

    fn clear_interrupt_flag(&mut self, event: Event);
    fn listen_interrupt(&mut self, event: Event, b: bool);
    fn get_interrupt_flag(&self) -> Event;
    fn start_one_pulse(&mut self);
}

pub trait TimerDirection: GeneralTimer {
    fn set_count_direction(&mut self, dir: CountDirection);
}

pub trait TimerWithPwm: GeneralTimer {
    fn start_pwm(&mut self);
    fn stop_pwm(&mut self);

    fn preload_output_channel_in_mode(&mut self, channel: Channel, mode: PwmMode);
    fn set_polarity(&mut self, channel: Channel, polarity: PwmPolarity);
}

pub trait TimerWithPwm1Ch: TimerWithPwm {
    fn enable_ch1(&mut self, en: bool);
    fn set_ch1_cc_value(&mut self, value: u32);
    fn get_ch1_cc_value(&self) -> u32;
}

pub trait TimerWithPwm2Ch: TimerWithPwm1Ch {
    fn enable_ch2(&mut self, en: bool);
    fn set_ch2_cc_value(&mut self, value: u32);
    fn get_ch2_cc_value(&self) -> u32;
}

pub trait TimerWithPwm3Ch: TimerWithPwm2Ch {
    fn enable_ch3(&mut self, en: bool);
    fn set_ch3_cc_value(&mut self, value: u32);
    fn get_ch3_cc_value(&self) -> u32;
}

pub trait TimerWithPwm4Ch: TimerWithPwm3Ch {
    fn enable_ch4(&mut self, en: bool);
    fn set_ch4_cc_value(&mut self, value: u32);
    fn get_ch4_cc_value(&self) -> u32;
}

// Enumerate ------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    C1,
    C2,
    C3,
    C4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CountDirection {
    Up,
    Down,
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

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Error {
    /// Timer is disabled
    Disabled,
    WrongAutoReload,
}

/// Interrupt events
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SysEvent {
    /// [Timer] timed out / count down ended
    Update,
}

bitflags::bitflags! {
    pub struct Event: u32 {
        const Update  = 1 << 0;
        const C1 = 1 << 1;
        const C2 = 1 << 2;
        const C3 = 1 << 3;
        const C4 = 1 << 4;
    }
}
