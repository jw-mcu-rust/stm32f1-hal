use super::*;
use crate::timer::Channel;

// Implement Traits -----------------------------------------------------------

pub struct PwmChannel1<TIM> {
    tim: TIM,
}
impl<TIM> PwmChannel1<TIM> {
    pub fn new(tim: TIM) -> Self {
        Self { tim }
    }
}
impl<TIM: TimerWithPwm1Ch> PwmChannel for PwmChannel1<TIM> {
    #[inline(always)]
    fn config(&mut self, mode: PwmMode, polarity: PwmPolarity) {
        self.tim
            .preload_output_channel_in_mode(Channel::C1, mode.into());
        self.tim.set_polarity(Channel::C1, polarity);
    }

    #[inline(always)]
    fn set_enable(&mut self, en: bool) {
        self.tim.enable_ch1(en);
    }

    #[inline(always)]
    fn get_max_duty(&self) -> u32 {
        (self.tim.read_auto_reload() as u32).wrapping_add(1)
    }

    #[inline(always)]
    fn set_duty(&mut self, duty: u32) {
        self.tim.set_ch1_cc_value(duty);
    }
}

pub struct PwmChannel2<TIM> {
    pub(super) tim: TIM,
}
impl<TIM> PwmChannel2<TIM> {
    pub fn new(tim: TIM) -> Self {
        Self { tim }
    }
}
impl<TIM: TimerWithPwm2Ch> PwmChannel for PwmChannel2<TIM> {
    #[inline(always)]
    fn config(&mut self, mode: PwmMode, polarity: PwmPolarity) {
        self.tim
            .preload_output_channel_in_mode(Channel::C2, mode.into());
        self.tim.set_polarity(Channel::C2, polarity);
    }

    #[inline(always)]
    fn set_enable(&mut self, en: bool) {
        self.tim.enable_ch2(en);
    }

    #[inline(always)]
    fn get_max_duty(&self) -> u32 {
        (self.tim.read_auto_reload() as u32).wrapping_add(1)
    }

    #[inline(always)]
    fn set_duty(&mut self, duty: u32) {
        self.tim.set_ch2_cc_value(duty);
    }
}

pub struct PwmChannel3<TIM> {
    pub(super) tim: TIM,
}
impl<TIM> PwmChannel3<TIM> {
    pub fn new(tim: TIM) -> Self {
        Self { tim }
    }
}
impl<TIM: TimerWithPwm4Ch> PwmChannel for PwmChannel3<TIM> {
    #[inline(always)]
    fn config(&mut self, mode: PwmMode, polarity: PwmPolarity) {
        self.tim
            .preload_output_channel_in_mode(Channel::C3, mode.into());
        self.tim.set_polarity(Channel::C3, polarity);
    }

    #[inline(always)]
    fn set_enable(&mut self, en: bool) {
        self.tim.enable_ch3(en);
    }

    #[inline(always)]
    fn get_max_duty(&self) -> u32 {
        (self.tim.read_auto_reload() as u32).wrapping_add(1)
    }

    #[inline(always)]
    fn set_duty(&mut self, duty: u32) {
        self.tim.set_ch3_cc_value(duty);
    }
}

pub struct PwmChannel4<TIM> {
    pub(super) tim: TIM,
}
impl<TIM> PwmChannel4<TIM> {
    pub fn new(tim: TIM) -> Self {
        Self { tim }
    }
}
impl<TIM: TimerWithPwm4Ch> PwmChannel for PwmChannel4<TIM> {
    #[inline(always)]
    fn config(&mut self, mode: PwmMode, polarity: PwmPolarity) {
        self.tim
            .preload_output_channel_in_mode(Channel::C4, mode.into());
        self.tim.set_polarity(Channel::C4, polarity);
    }

    #[inline(always)]
    fn set_enable(&mut self, en: bool) {
        self.tim.enable_ch4(en);
    }

    #[inline(always)]
    fn get_max_duty(&self) -> u32 {
        (self.tim.read_auto_reload() as u32).wrapping_add(1)
    }

    #[inline(always)]
    fn set_duty(&mut self, duty: u32) {
        self.tim.set_ch4_cc_value(duty);
    }
}
