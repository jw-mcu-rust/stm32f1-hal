type TimerX = pac::TIM8;
type Width = u16;

// Do NOT manually modify the code between begin and end!
// It's synced by scripts/sync_code.py.
// sync begin

use super::*;
use crate::{Mcu, pac};

impl Instance for TimerX {}

impl TimerInit<TimerX> for TimerX {
    fn constrain(self, mcu: &mut Mcu) -> Timer<TimerX> {
        Timer::new(self, mcu)
    }
}

impl GeneralTimer for TimerX {
    #[inline(always)]
    fn reset_config(&mut self) {
        self.cr1().reset();
    }

    #[inline(always)]
    fn enable_counter(&mut self) {
        self.cr1().modify(|_, w| w.cen().set_bit());
    }

    #[inline(always)]
    fn disable_counter(&mut self) {
        self.cr1().modify(|_, w| w.cen().clear_bit());
    }

    #[inline(always)]
    fn is_counter_enabled(&self) -> bool {
        self.cr1().read().cen().is_enabled()
    }

    #[inline(always)]
    fn reset_counter(&mut self) {
        self.cnt().reset();
    }

    #[inline(always)]
    fn max_auto_reload() -> u32 {
        Width::MAX as u32
    }

    #[inline(always)]
    unsafe fn set_auto_reload_unchecked(&mut self, arr: u32) {
        unsafe {
            self.arr().write(|w| w.bits(arr));
        }
    }

    #[inline(always)]
    fn set_auto_reload(&mut self, arr: u32) -> Result<(), Error> {
        // Note: Make it impossible to set the ARR value to 0, since this
        // would cause an infinite loop.
        if arr > 0 && arr <= Self::max_auto_reload() {
            Ok(unsafe { self.set_auto_reload_unchecked(arr) })
        } else {
            Err(Error::WrongAutoReload)
        }
    }

    #[inline(always)]
    fn read_auto_reload(&self) -> u32 {
        self.arr().read().bits()
    }

    #[inline(always)]
    fn set_prescaler(&mut self, psc: u16) {
        self.psc().write(|w| w.psc().set(psc));
    }

    #[inline(always)]
    fn read_prescaler(&self) -> u16 {
        self.psc().read().psc().bits()
    }

    #[inline(always)]
    fn read_count(&self) -> u32 {
        self.cnt().read().bits() as u32
    }

    #[inline(always)]
    fn trigger_update(&mut self) {
        // Sets the URS bit to prevent an interrupt from being triggered by
        // the UG bit
        self.cr1().modify(|_, w| w.urs().set_bit());
        self.egr().write(|w| w.ug().set_bit());
        self.cr1().modify(|_, w| w.urs().clear_bit());
    }

    #[inline]
    fn config_freq(&mut self, clock: Hertz, update_freq: Hertz) {
        let (prescaler, arr) = compute_prescaler_arr(clock.raw(), update_freq.raw());
        self.set_prescaler(prescaler as u16);
        self.set_auto_reload(arr).unwrap();
        // Trigger update event to load the registers
        self.trigger_update();
    }

    #[inline(always)]
    fn clear_interrupt_flag(&mut self, event: Event) {
        self.sr()
            .write(|w| unsafe { w.bits(0xffff & !event.bits()) });
    }

    #[inline(always)]
    fn listen_interrupt(&mut self, event: Event, b: bool) {
        self.dier().modify(|r, w| unsafe {
            w.bits(if b {
                r.bits() | event.bits()
            } else {
                r.bits() & !event.bits()
            })
        });
    }

    #[inline(always)]
    fn get_interrupt_flag(&self) -> Event {
        Event::from_bits_truncate(self.sr().read().bits())
    }

    #[inline(always)]
    fn start_one_pulse(&mut self) {
        self.cr1().modify(|_, w| w.opm().set_bit().cen().set_bit());
    }

    #[inline(always)]
    fn stop_in_debug(&mut self, state: bool) {
        let dbg = unsafe { DBG::steal() };
        // sync dbg_t8
        dbg.cr().modify(|_, w| w.dbg_tim8_stop().bit(state));
        // sync dbg_end
    }

    #[inline(always)]
    fn enable_preload(&mut self, b: bool) {
        self.cr1().modify(|_, w| w.arpe().bit(b));
    }
}

// sync pwm
// PWM ------------------------------------------------------------------------

impl TimerWithPwm for TimerX {
    fn stop_pwm(&mut self) {
        self.disable_counter();
    }

    // sync start_pwm_aoe

    #[inline(always)]
    fn start_pwm(&mut self) {
        // self.bdtr().modify(|_, w| w.moe().set_bit());
        self.bdtr().modify(|_, w| w.aoe().set_bit());
        self.reset_counter();
        self.enable_counter();
    }

    // sync pwm_cfg_4

    #[inline(always)]
    fn preload_output_channel_in_mode(&mut self, channel: Channel, mode: PwmMode) {
        let mode = Ocm::from(mode);
        match channel {
            Channel::C1 => {
                self.ccmr1_output()
                    .modify(|_, w| w.oc1pe().set_bit().oc1m().set(mode as _));
            }
            Channel::C2 => {
                self.ccmr1_output()
                    .modify(|_, w| w.oc2pe().set_bit().oc2m().set(mode as _));
            }
            Channel::C3 => {
                self.ccmr2_output()
                    .modify(|_, w| w.oc3pe().set_bit().oc3m().set(mode as _));
            }
            Channel::C4 => {
                self.ccmr2_output()
                    .modify(|_, w| w.oc4pe().set_bit().oc4m().set(mode as _));
            }
        }
    }

    fn set_polarity(&mut self, channel: Channel, polarity: PwmPolarity) {
        match channel {
            Channel::C1 => {
                self.ccer()
                    .modify(|_, w| w.cc1p().bit(polarity == PwmPolarity::ActiveLow));
            }
            Channel::C2 => {
                self.ccer()
                    .modify(|_, w| w.cc2p().bit(polarity == PwmPolarity::ActiveLow));
            }
            Channel::C3 => {
                self.ccer()
                    .modify(|_, w| w.cc3p().bit(polarity == PwmPolarity::ActiveLow));
            }
            Channel::C4 => {
                self.ccer()
                    .modify(|_, w| w.cc4p().bit(polarity == PwmPolarity::ActiveLow));
            }
        }
    }
}

// sync pwm_ch1
// PWM Channels ---------------------------------------------------------------

impl TimerWithPwm1Ch for TimerX {
    #[inline(always)]
    fn enable_ch1(&mut self, en: bool) {
        self.ccer().modify(|_, w| w.cc1e().bit(en));
    }

    #[inline(always)]
    fn set_ch1_cc_value(&mut self, value: u32) {
        unsafe { self.ccr1().write(|w| w.bits(value)) };
    }

    #[inline(always)]
    fn get_ch1_cc_value(&self) -> u32 {
        self.ccr1().read().bits()
    }
}

// sync pwm_ch2

impl TimerWithPwm2Ch for TimerX {
    #[inline(always)]
    fn enable_ch2(&mut self, en: bool) {
        self.ccer().modify(|_, w| w.cc2e().bit(en));
    }

    #[inline(always)]
    fn set_ch2_cc_value(&mut self, value: u32) {
        unsafe { self.ccr2().write(|w| w.bits(value)) };
    }

    #[inline(always)]
    fn get_ch2_cc_value(&self) -> u32 {
        self.ccr2().read().bits()
    }
}

// sync pwm_ch4

impl TimerWithPwm3Ch for TimerX {
    #[inline(always)]
    fn enable_ch3(&mut self, en: bool) {
        self.ccer().modify(|_, w| w.cc3e().bit(en));
    }

    #[inline(always)]
    fn set_ch3_cc_value(&mut self, value: u32) {
        unsafe { self.ccr3().write(|w| w.bits(value)) };
    }

    #[inline(always)]
    fn get_ch3_cc_value(&self) -> u32 {
        self.ccr3().read().bits()
    }
}

impl TimerWithPwm4Ch for TimerX {
    #[inline(always)]
    fn enable_ch4(&mut self, en: bool) {
        self.ccer().modify(|_, w| w.cc4e().bit(en));
    }

    #[inline(always)]
    fn set_ch4_cc_value(&mut self, value: u32) {
        unsafe { self.ccr4().write(|w| w.bits(value)) };
    }

    #[inline(always)]
    fn get_ch4_cc_value(&self) -> u32 {
        self.ccr4().read().bits()
    }
}

// Other ----------------------------------------------------------------------

// sync master
impl MasterTimer for TimerX {
    #[inline(always)]
    fn master_mode(&mut self, mode: MasterMode) {
        self.cr2().modify(|_, w| w.mms().variant(mode.into()));
    }
}

// sync dir

impl TimerDirection for TimerX {
    #[inline(always)]
    fn set_count_direction(&mut self, dir: CountDirection) {
        self.cr1()
            .modify(|_, w| w.dir().bit(dir == CountDirection::Down));
    }
}

// sync end
