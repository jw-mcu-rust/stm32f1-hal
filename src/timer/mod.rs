#![allow(dead_code)]
#![allow(private_bounds)]
#![allow(non_upper_case_globals)]

use crate::{
    Mcu, Steal,
    pac::DBGMCU as DBG,
    rcc::{self, Clocks},
    time::Hertz,
};

use core::convert::TryFrom;

#[cfg(feature = "rtic")]
pub mod monotonic;
#[cfg(feature = "rtic")]
pub use monotonic::*;
pub mod delay;
pub use delay::*;
pub mod counter;
pub use counter::*;
pub mod syst;
pub use syst::*;
pub mod pwm;
#[cfg(any(feature = "stm32f100", feature = "stm32f103", feature = "connectivity"))]
pub mod timer1;
#[cfg(feature = "stm32f100")]
pub mod timer15;
#[cfg(feature = "stm32f100")]
pub mod timer16;
#[cfg(feature = "stm32f100")]
pub mod timer17;
pub mod timer2;
pub mod timer3;
#[cfg(feature = "medium")]
pub mod timer4;
#[cfg(any(feature = "high", feature = "connectivity"))]
pub mod timer5;
#[cfg(any(feature = "stm32f100", feature = "high", feature = "connectivity"))]
pub mod timer6;
#[cfg(any(
    all(feature = "high", any(feature = "stm32f101", feature = "stm32f103")),
    any(feature = "stm32f100", feature = "connectivity")
))]
pub mod timer7;
#[cfg(all(feature = "stm32f103", feature = "high"))]
pub mod timer8;

mod impl_hal;

pub use crate::common::timer::*;

pub trait Instance: rcc::Enable + rcc::Reset + rcc::BusTimerClock + GeneralTimer {}

pub trait TimerInit<TIM> {
    fn constrain(self, mcu: &mut Mcu) -> Timer<TIM>;
}

/// Timer wrapper
pub struct Timer<TIM> {
    pub(super) tim: TIM,
    pub(super) clk: Hertz,
}

impl<TIM: Instance + Steal> Timer<TIM> {
    /// Initialize timer
    pub fn new(tim: TIM, mcu: &mut Mcu) -> Self {
        // Enable and reset the timer peripheral
        TIM::enable(&mut mcu.rcc);
        TIM::reset(&mut mcu.rcc);

        Self {
            clk: TIM::timer_clock(&mcu.rcc.clocks),
            tim,
        }
    }

    pub fn configure(&mut self, clocks: &Clocks) {
        self.clk = TIM::timer_clock(clocks);
    }

    /// Non-blocking [Counter] with custom fixed precision
    pub fn counter<const FREQ: u32>(self, mcu: &mut Mcu) -> Counter<TIM, FREQ> {
        FTimer::new(self.tim, mcu).counter()
    }

    /// Non-blocking [Counter] with fixed precision of 1 ms (1 kHz sampling)
    ///
    /// Can wait from 2 ms to 65 sec for 16-bit timer and from 2 ms to 49 days for 32-bit timer.
    ///
    /// NOTE: don't use this if your system frequency more than 65 MHz
    pub fn counter_ms(self, mcu: &mut Mcu) -> CounterMs<TIM> {
        self.counter::<1_000>(mcu)
    }

    /// Non-blocking [Counter] with fixed precision of 1 μs (1 MHz sampling)
    ///
    /// Can wait from 2 μs to 65 ms for 16-bit timer and from 2 μs to 71 min for 32-bit timer.
    pub fn counter_us(self, mcu: &mut Mcu) -> CounterUs<TIM> {
        self.counter::<1_000_000>(mcu)
    }

    /// Non-blocking [Counter] with dynamic precision which uses `Hertz` as Duration units
    pub fn counter_hz(self) -> CounterHz<TIM> {
        CounterHz(self)
    }

    /// Blocking [Delay] with custom fixed precision
    pub fn delay<const FREQ: u32>(self, mcu: &mut Mcu) -> Delay<TIM, FREQ> {
        FTimer::new(self.tim, mcu).delay()
    }

    /// Blocking [Delay] with fixed precision of 1 ms (1 kHz sampling)
    ///
    /// Can wait from 2 ms to 49 days.
    ///
    /// NOTE: don't use this if your system frequency more than 65 MHz
    pub fn delay_ms(self, mcu: &mut Mcu) -> DelayMs<TIM> {
        self.delay::<1_000>(mcu)
    }
    /// Blocking [Delay] with fixed precision of 1 μs (1 MHz sampling)
    ///
    /// Can wait from 2 μs to 71 min.
    pub fn delay_us(self, mcu: &mut Mcu) -> DelayUs<TIM> {
        self.delay::<1_000_000>(mcu)
    }

    pub fn release(self) -> TIM {
        self.tim
    }

    /// Starts listening for an `event`
    ///
    /// Note, you will also have to enable the TIM2 interrupt in the NVIC to start
    /// receiving events.
    pub fn listen(&mut self, event: Event) {
        self.tim.listen_interrupt(event, true);
    }

    /// Clears interrupt associated with `event`.
    ///
    /// If the interrupt is not cleared, it will immediately retrigger after
    /// the ISR has finished.
    pub fn clear_interrupt(&mut self, event: Event) {
        self.tim.clear_interrupt_flag(event);
    }

    pub fn get_interrupt(&mut self) -> Event {
        self.tim.get_interrupt_flag()
    }

    /// Stops listening for an `event`
    pub fn unlisten(&mut self, event: Event) {
        self.tim.listen_interrupt(event, false);
    }

    /// Stopping timer in debug mode can cause troubles when sampling the signal
    pub fn stop_in_debug(&mut self, dbg: &mut DBG, state: bool) {
        self.tim.stop_in_debug(dbg, state);
    }
}

impl<TIM: Instance + MasterTimer> Timer<TIM> {
    pub fn set_master_mode(&mut self, mode: TIM::Mms) {
        self.tim.master_mode(mode)
    }
}

// Fixed precision timers -----------------------------------------------------

/// Timer wrapper for fixed precision timers.
///
/// Uses `fugit::TimerDurationU32` for most of operations
pub struct FTimer<TIM, const FREQ: u32> {
    tim: TIM,
}

/// `FTimer` with precision of 1 μs (1 MHz sampling)
pub type FTimerUs<TIM> = FTimer<TIM, 1_000_000>;

/// `FTimer` with precision of 1 ms (1 kHz sampling)
///
/// NOTE: don't use this if your system frequency more than 65 MHz
pub type FTimerMs<TIM> = FTimer<TIM, 1_000>;

impl<TIM: Instance, const FREQ: u32> FTimer<TIM, FREQ> {
    /// Initialize timer
    pub fn new(tim: TIM, mcu: &mut Mcu) -> Self {
        // Enable and reset the timer peripheral
        TIM::enable(&mut mcu.rcc);
        TIM::reset(&mut mcu.rcc);

        let mut t = Self { tim };
        t.configure(&mcu.rcc.clocks);
        t
    }

    /// Calculate prescaler depending on `Clocks` state
    pub fn configure(&mut self, clocks: &Clocks) {
        let clk = TIM::timer_clock(clocks);
        assert!(clk.raw() % FREQ == 0);
        let psc = clk.raw() / FREQ;
        self.tim.set_prescaler(u16::try_from(psc - 1).unwrap());
    }

    /// Creates `Counter` that imlements [embedded_hal_02::timer::CountDown]
    pub fn counter(self) -> Counter<TIM, FREQ> {
        Counter(self)
    }

    /// Creates `Delay` that imlements [embedded_hal_02::blocking::delay] traits
    pub fn delay(self) -> Delay<TIM, FREQ> {
        Delay(self)
    }

    /// Releases the TIM peripheral
    pub fn release(self) -> TIM {
        self.tim
    }

    /// Starts listening for an `event`
    ///
    /// Note, you will also have to enable the TIM2 interrupt in the NVIC to start
    /// receiving events.
    pub fn listen(&mut self, event: Event) {
        self.tim.listen_interrupt(event, true);
    }

    /// Clears interrupt associated with `event`.
    ///
    /// If the interrupt is not cleared, it will immediately retrigger after
    /// the ISR has finished.
    pub fn clear_interrupt(&mut self, event: Event) {
        self.tim.clear_interrupt_flag(event);
    }

    pub fn get_interrupt(&mut self) -> Event {
        self.tim.get_interrupt_flag()
    }

    /// Stops listening for an `event`
    pub fn unlisten(&mut self, event: Event) {
        self.tim.listen_interrupt(event, false);
    }

    /// Stopping timer in debug mode can cause troubles when sampling the signal
    pub fn stop_in_debug(&mut self, dbg: &mut DBG, state: bool) {
        self.tim.stop_in_debug(dbg, state);
    }
}

impl<TIM: Instance + MasterTimer, const FREQ: u32> FTimer<TIM, FREQ> {
    pub fn set_master_mode(&mut self, mode: TIM::Mms) {
        self.tim.master_mode(mode)
    }
}

// Enumerate ------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Channel {
    C1 = 0,
    C2 = 1,
    C3 = 2,
    C4 = 3,
}

impl From<u8> for Channel {
    fn from(value: u8) -> Self {
        match value {
            3 => Channel::C4,
            2 => Channel::C3,
            1 => Channel::C2,
            _ => Channel::C1,
        }
    }
}

impl From<PwmPolarity> for bool {
    fn from(value: PwmPolarity) -> Self {
        match value {
            PwmPolarity::ActiveHigh => false,
            PwmPolarity::ActiveLow => true,
        }
    }
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

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Error {
    /// Timer is disabled
    Disabled,
    WrongAutoReload,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CountDirection {
    Up,
    Down,
}

impl From<Channel> for u8 {
    fn from(value: Channel) -> Self {
        value as u8
    }
}

impl From<PwmMode> for u8 {
    fn from(value: PwmMode) -> Self {
        match value {
            PwmMode::Mode1 => 6,
            PwmMode::Mode2 => 7,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Ocm {
    Frozen = 0,
    ActiveOnMatch = 1,
    InactiveOnMatch = 2,
    Toggle = 3,
    ForceInactive = 4,
    ForceActive = 5,
    PwmMode1 = 6,
    PwmMode2 = 7,
}

impl From<PwmMode> for Ocm {
    fn from(value: PwmMode) -> Self {
        match value {
            PwmMode::Mode1 => Ocm::PwmMode1,
            PwmMode::Mode2 => Ocm::PwmMode2,
        }
    }
}

// Utilities ------------------------------------------------------------------

#[inline(always)]
const fn compute_arr_presc(freq: u32, clock: u32) -> (u16, u32) {
    let ticks = clock / freq;
    let psc = (ticks - 1) / (1 << 16);
    let arr = ticks / (psc + 1) - 1;
    (psc as u16, arr)
}

pub fn freq_to_presc_arr(timer_clk: u32, count_freq: u32, update_freq: u32) -> (u32, u32) {
    assert!(timer_clk >= count_freq);

    let prescaler = if count_freq > 0 {
        timer_clk / count_freq - 1
    } else {
        1
    };

    let period = if update_freq > 0 {
        count_freq / update_freq - 1
    } else {
        0
    };

    assert!(prescaler <= 0xFFFF);
    assert!(period <= 0xFFFF);
    (prescaler, period)
}

// Peripheral Traits ----------------------------------------------------------

pub trait GeneralTimer {
    type Width: Into<u32> + From<u16>;
    fn start(&mut self);
    fn stop(&mut self);
    fn max_auto_reload() -> u32;
    unsafe fn set_auto_reload_unchecked(&mut self, arr: u32);
    fn set_auto_reload(&mut self, arr: u32) -> Result<(), Error>;
    fn read_auto_reload(&self) -> u32;
    fn enable_preload(&mut self, b: bool);
    fn enable_counter(&mut self);
    fn disable_counter(&mut self);
    fn is_counter_enabled(&self) -> bool;
    fn reset_counter(&mut self);
    fn set_prescaler(&mut self, psc: u16);
    fn read_prescaler(&self) -> u16;
    fn trigger_update(&mut self);
    fn clear_interrupt_flag(&mut self, event: Event);
    fn listen_interrupt(&mut self, event: Event, b: bool);
    fn get_interrupt_flag(&self) -> Event;
    fn read_count(&self) -> Self::Width;
    fn start_one_pulse(&mut self);
    fn cr1_reset(&mut self);
    fn stop_in_debug(&mut self, dbg: &mut DBG, state: bool);
}

pub trait TimerWithPwm: GeneralTimer {
    fn start_pwm(&mut self);

    fn preload_output_channel_in_mode(&mut self, channel: Channel, mode: Ocm);
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

pub trait TimerWithPwm4Ch: TimerWithPwm2Ch {
    fn enable_ch3(&mut self, en: bool);
    fn set_ch3_cc_value(&mut self, value: u32);
    fn get_ch3_cc_value(&self) -> u32;
    fn enable_ch4(&mut self, en: bool);
    fn set_ch4_cc_value(&mut self, value: u32);
    fn get_ch4_cc_value(&self) -> u32;
}

pub trait MasterTimer: GeneralTimer {
    type Mms;
    fn master_mode(&mut self, mode: Self::Mms);
}

pub trait TimerDirection: GeneralTimer {
    fn set_count_direction(&mut self, dir: CountDirection);
}

// hal!(
//     pac::TIM2: [Timer2, u16, dbg_tim2_stop, c: (CH4), m: tim2, d: dir,],
//     pac::TIM3: [Timer3, u16, dbg_tim3_stop, c: (CH4), m: tim2, d: dir,],
// );

// #[cfg(any(feature = "stm32f100", feature = "stm32f103", feature = "connectivity"))]
// hal!(
//     pac::TIM1: [Timer1, u16, dbg_tim1_stop, c: (CH4, _aoe), m: tim1, d: dir,],
// );

// #[cfg(any(feature = "stm32f100", feature = "high", feature = "connectivity"))]
// hal! {
//     pac::TIM6: [Timer6, u16, dbg_tim6_stop, m: tim6,],
// }

// #[cfg(any(
//     all(feature = "high", any(feature = "stm32f101", feature = "stm32f103")),
//     any(feature = "stm32f100", feature = "connectivity")
// ))]
// hal! {
//     pac::TIM7: [Timer7, u16, dbg_tim7_stop, m: tim6,],
// }

// #[cfg(feature = "stm32f100")]
// hal! {
//     pac::TIM15: [Timer15, u16, dbg_tim15_stop, c: (CH2),],
//     pac::TIM16: [Timer16, u16, dbg_tim16_stop, c: (CH1),],
//     pac::TIM17: [Timer17, u16, dbg_tim17_stop, c: (CH1),],
// }

// #[cfg(feature = "medium")]
// hal! {
//     pac::TIM4: [Timer4, u16, dbg_tim4_stop, c: (CH4), m: tim2, d: dir,],
// }

// #[cfg(any(feature = "high", feature = "connectivity"))]
// hal! {
//     pac::TIM5: [Timer5, u16, dbg_tim5_stop, c: (CH4), m: tim2, d: dir,],
// }

// #[cfg(all(feature = "stm32f103", feature = "high"))]
// hal! {
//     pac::TIM8: [Timer8, u16, dbg_tim8_stop, c: (CH4, _aoe), m: tim1, d: dir,],
// }

//TODO: restore these timers once stm32-rs has been updated
/*
 *   dbg_tim(12-13)_stop fields missing from 103 xl in stm32-rs
 *   dbg_tim(9-10)_stop fields missing from 101 xl in stm32-rs
#[cfg(any(
    feature = "xl",
    all(
        feature = "stm32f100",
        feature = "high",
)))]
hal! {
    TIM12: (tim12, dbg_tim12_stop),
    TIM13: (tim13, dbg_tim13_stop),
    TIM14: (tim14, dbg_tim14_stop),
}
#[cfg(feature = "xl")]
hal! {
    TIM9: (tim9, dbg_tim9_stop),
    TIM10: (tim10, dbg_tim10_stop),
    TIM11: (tim11, dbg_tim11_stop),
}
*/
