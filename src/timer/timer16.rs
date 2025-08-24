type TimerX = pac::TIM16;
const CH_NUMBER: u8 = 1;

// Do NOT manually modify the code between begin and end!
// It's synced by scripts/sync_code.py.
// sync general begin

use super::*;
use crate::{Mcu, pac};

impl Instance for TimerX {}

impl TimerInit<TimerX> for TimerX {
    fn constrain(self, mcu: &mut Mcu) -> Timer<TimerX> {
        Timer::new(self, mcu)
    }
}

impl GeneralTimer for TimerX {
    type Width = u16;

    #[inline(always)]
    fn start(&mut self) {
        self.cnt().reset();
        self.cr1().modify(|_, w| w.cen().set_bit());
    }

    #[inline(always)]
    fn stop(&mut self) {
        self.cnt().reset();
        self.cr1().modify(|_, w| w.cen().clear_bit());
    }

    #[inline(always)]
    fn max_auto_reload() -> u32 {
        Self::Width::MAX as u32
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
    fn enable_preload(&mut self, b: bool) {
        self.cr1().modify(|_, w| w.arpe().bit(b));
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
    fn set_prescaler(&mut self, psc: u16) {
        self.psc().write(|w| w.psc().set(psc));
    }

    #[inline(always)]
    fn read_prescaler(&self) -> u16 {
        self.psc().read().psc().bits()
    }

    #[inline(always)]
    fn trigger_update(&mut self) {
        // Sets the URS bit to prevent an interrupt from being triggered by
        // the UG bit
        self.cr1().modify(|_, w| w.urs().set_bit());
        self.egr().write(|w| w.ug().set_bit());
        self.cr1().modify(|_, w| w.urs().clear_bit());
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
    fn read_count(&self) -> Self::Width {
        self.cnt().read().bits() as Self::Width
    }

    #[inline(always)]
    fn start_one_pulse(&mut self) {
        self.cr1().modify(|_, w| w.opm().set_bit().cen().set_bit());
    }

    #[inline(always)]
    fn cr1_reset(&mut self) {
        self.cr1().reset();
    }

    #[inline(always)]
    fn stop_in_debug(&mut self, dbg: &mut DBG, state: bool) {
        dbg.cr().modify(|_, w| w.dbg_tim1_stop().bit(state));
    }
}

// sync general end
// sync pwm begin
// PWM ------------------------------------------------------------------------

impl TimerWithPwm for TimerX {
    // sync pwm end
    // sync start_pwm2 begin

    #[inline(always)]
    fn start_pwm(&mut self) {
        self.start();
    }

    // sync start_pwm2 end
    // sync pwm_cfg begin

    #[inline(always)]
    fn preload_output_channel_in_mode(&mut self, channel: Channel, mode: Ocm) {
        assert!((channel as u8) < CH_NUMBER);
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
        assert!((channel as u8) < CH_NUMBER);
        match channel {
            Channel::C1 => {
                self.ccer()
                    .modify(|_, w| w.cc1p().bit(polarity == PwmPolarity::ActiveLow));
            }
            _ => (),
        }
    }

    // sync pwm_cfg end
    // sync pwm_ch1 begin
}

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

// sync pwm_ch1 end
