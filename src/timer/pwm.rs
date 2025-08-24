use super::*;
use crate::{
    Mcu, Steal,
    afio::{RemapMode, timer_remap::*},
    common::timer::*,
    timer::Channel,
};

impl<TIM: Instance + TimerWithPwm4Ch + TimerDirection + Steal> Timer<TIM> {
    pub fn into_pwm4<REMAP: RemapMode<TIM>>(
        mut self,
        pins: (
            Option<impl TimCh1Pin<REMAP>>,
            Option<impl TimCh2Pin<REMAP>>,
            Option<impl TimCh3Pin<REMAP>>,
            Option<impl TimCh4Pin<REMAP>>,
        ),
        dir: CountDirection,
        preload: bool,
        mcu: &mut Mcu,
    ) -> (
        impl PwmTimer,
        Option<impl PwmChannel>,
        Option<impl PwmChannel>,
        Option<impl PwmChannel>,
        Option<impl PwmChannel>,
    ) {
        REMAP::remap(&mut mcu.afio);
        self.tim.enable_preload(preload);
        self.tim.set_count_direction(dir);

        let c1 = pins.0.map(|_| PwmChannel1::<TIM> {
            tim: unsafe { self.tim.steal() },
        });
        let c2 = pins.1.map(|_| PwmChannel2::<TIM> {
            tim: unsafe { self.tim.steal() },
        });
        let c3 = pins.2.map(|_| PwmChannel3::<TIM> {
            tim: unsafe { self.tim.steal() },
        });
        let c4 = pins.3.map(|_| PwmChannel4::<TIM> {
            tim: unsafe { self.tim.steal() },
        });

        (self, c1, c2, c3, c4)
    }
}

impl<TIM: Instance + TimerWithPwm> PwmTimer for Timer<TIM> {
    #[inline(always)]
    fn start(&mut self) {
        self.tim.start_pwm();
    }

    #[inline(always)]
    fn stop(&mut self) {
        self.tim.stop();
    }

    #[inline]
    fn get_count_value(&self) -> u32 {
        self.tim.read_count().into()
    }

    #[inline]
    fn get_max_duty(&self) -> u32 {
        (self.tim.read_auto_reload() as u32).wrapping_add(1)
    }

    #[inline]
    fn config_freq(&mut self, count_freq: Hertz, update_freq: Hertz) {
        let (prescaler, arr) =
            freq_to_presc_arr(self.clk.raw(), count_freq.raw(), update_freq.raw());
        self.tim.set_prescaler(prescaler as u16);
        unsafe {
            self.tim.set_auto_reload_unchecked(arr);
        }
        // Trigger update event to load the registers
        self.tim.trigger_update();
    }
}

pub struct PwmChannel1<TIM> {
    pub(super) tim: TIM,
}
impl<TIM: Instance + TimerWithPwm1Ch> PwmChannel for PwmChannel1<TIM> {
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
impl<TIM: Instance + TimerWithPwm2Ch> PwmChannel for PwmChannel2<TIM> {
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
impl<TIM: Instance + TimerWithPwm4Ch> PwmChannel for PwmChannel3<TIM> {
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
impl<TIM: Instance + TimerWithPwm4Ch> PwmChannel for PwmChannel4<TIM> {
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
