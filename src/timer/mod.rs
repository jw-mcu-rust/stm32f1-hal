use crate::{
    Mcu, Steal,
    afio::{RemapMode, timer_remap::*},
    pac::DBGMCU as DBG,
    rcc,
    time::Hertz,
};

pub use crate::common::timer::*;

#[cfg(feature = "rtic")]
pub mod monotonic;
#[cfg(feature = "rtic")]
pub use monotonic::*;
pub mod syst;
pub use syst::*;
#[cfg(any(feature = "stm32f100", feature = "stm32f103", feature = "connectivity"))]
pub mod timer1;
#[cfg(feature = "xl")]
pub mod timer10;
#[cfg(feature = "xl")]
pub mod timer11;
#[cfg(any(feature = "xl", all(feature = "stm32f100", feature = "high",)))]
pub mod timer12;
#[cfg(any(feature = "xl", all(feature = "stm32f100", feature = "high",)))]
pub mod timer13;
#[cfg(any(feature = "xl", all(feature = "stm32f100", feature = "high",)))]
pub mod timer14;
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
#[cfg(feature = "xl")]
pub mod timer9;

pub trait Instance: rcc::Enable + rcc::Reset + rcc::BusTimerClock + GeneralTimer {}

// Initialize -----------------------------------------------------------------

pub trait TimerInit<TIM> {
    fn constrain(self, mcu: &mut Mcu) -> Timer<TIM>;
}

/// Timer wrapper
pub struct Timer<TIM> {
    tim: TIM,
    clk: Hertz,
}

impl<TIM: Instance + Steal> Timer<TIM> {
    /// Initialize timer
    pub fn new(tim: TIM, mcu: &mut Mcu) -> Self {
        // Enable and reset the timer peripheral
        mcu.rcc.enable(&tim);
        mcu.rcc.reset(&tim);

        Self {
            clk: mcu.rcc.get_timer_clock(&tim),
            tim,
        }
    }

    /// Non-blocking [Counter] with custom fixed precision
    pub fn counter<const FREQ: u32>(self) -> Counter<TIM, FREQ> {
        FTimer::new(self.tim, self.clk).counter()
    }

    /// Non-blocking [Counter] with fixed precision of 1 ms (1 kHz sampling)
    ///
    /// Can wait from 2 ms to 65 sec for 16-bit timer and from 2 ms to 49 days for 32-bit timer.
    ///
    /// NOTE: don't use this if your system frequency more than 65 MHz
    pub fn counter_ms(self) -> CounterMs<TIM> {
        self.counter::<1_000>()
    }

    /// Non-blocking [Counter] with fixed precision of 1 μs (1 MHz sampling)
    ///
    /// Can wait from 2 μs to 65 ms for 16-bit timer and from 2 μs to 71 min for 32-bit timer.
    pub fn counter_us(self) -> CounterUs<TIM> {
        self.counter::<1_000_000>()
    }

    /// Non-blocking [Counter] with dynamic precision which uses `Hertz` as Duration units
    pub fn counter_hz(self) -> CounterHz<TIM> {
        CounterHz {
            tim: self.tim,
            clk: self.clk,
        }
    }

    /// Blocking [Delay] with custom fixed precision
    pub fn delay<const FREQ: u32>(self) -> Delay<TIM, FREQ> {
        FTimer::new(self.tim, self.clk).delay()
    }

    /// Blocking [Delay] with fixed precision of 1 ms (1 kHz sampling)
    ///
    /// Can wait from 2 ms to 49 days.
    ///
    /// NOTE: don't use this if your system frequency more than 65 MHz
    pub fn delay_ms(self) -> DelayMs<TIM> {
        self.delay::<1_000>()
    }
    /// Blocking [Delay] with fixed precision of 1 μs (1 MHz sampling)
    ///
    /// Can wait from 2 μs to 71 min.
    pub fn delay_us(self) -> DelayUs<TIM> {
        self.delay::<1_000_000>()
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
    pub fn stop_in_debug(&mut self, state: bool) {
        self.tim.stop_in_debug(state);
    }
}

impl<TIM: Instance + MasterTimer> Timer<TIM> {
    pub fn set_master_mode(&mut self, mode: MasterMode) {
        self.tim.master_mode(mode)
    }
}

impl<TIM: Instance + TimerDirection> Timer<TIM> {
    pub fn set_count_direction(&mut self, dir: CountDirection) {
        self.tim.set_count_direction(dir);
    }
}

// Initialize PWM -------------------------------------------------------------

impl<'a, TIM: Instance + TimerWithPwm1Ch + Steal + 'a> Timer<TIM> {
    pub fn into_pwm1<REMAP: RemapMode<TIM>>(
        mut self,
        _pin: impl TimCh1Pin<REMAP> + 'a,
        update_freq: Hertz,
        preload: bool,
        mcu: &mut Mcu,
    ) -> (PwmTimer<TIM>, impl PwmChannel) {
        REMAP::remap(&mut mcu.afio);
        self.tim.enable_preload(preload);
        self.tim.config_freq(self.clk, update_freq);

        let c1 = PwmChannel1::new(unsafe { self.tim.steal() });
        let t = PwmTimer::new(self.tim, self.clk);
        (t, c1)
    }
}

impl<'a, TIM: Instance + TimerWithPwm2Ch + Steal + 'a> Timer<TIM> {
    pub fn into_pwm2<REMAP: RemapMode<TIM>>(
        mut self,
        pins: (Option<impl TimCh1Pin<REMAP>>, Option<impl TimCh2Pin<REMAP>>),
        update_freq: Hertz,
        preload: bool,
        mcu: &mut Mcu,
    ) -> (
        PwmTimer<TIM>,
        Option<impl PwmChannel + 'a>,
        Option<impl PwmChannel + 'a>,
    ) {
        REMAP::remap(&mut mcu.afio);
        self.tim.enable_preload(preload);
        self.tim.config_freq(self.clk, update_freq);

        let c1 = pins
            .0
            .map(|_| PwmChannel1::new(unsafe { self.tim.steal() }));
        let c2 = pins
            .1
            .map(|_| PwmChannel2::new(unsafe { self.tim.steal() }));
        let t = PwmTimer::new(self.tim, self.clk);
        (t, c1, c2)
    }
}

impl<'a, TIM: Instance + TimerWithPwm4Ch + Steal + 'a> Timer<TIM> {
    pub fn into_pwm4<REMAP: RemapMode<TIM>>(
        mut self,
        pins: (
            Option<impl TimCh1Pin<REMAP>>,
            Option<impl TimCh2Pin<REMAP>>,
            Option<impl TimCh3Pin<REMAP>>,
            Option<impl TimCh4Pin<REMAP>>,
        ),
        update_freq: Hertz,
        preload: bool,
        mcu: &mut Mcu,
    ) -> (
        PwmTimer<TIM>,
        Option<impl PwmChannel + 'a>,
        Option<impl PwmChannel + 'a>,
        Option<impl PwmChannel + 'a>,
        Option<impl PwmChannel + 'a>,
    ) {
        REMAP::remap(&mut mcu.afio);
        self.tim.enable_preload(preload);
        self.tim.config_freq(self.clk, update_freq);

        let c1 = pins
            .0
            .map(|_| PwmChannel1::new(unsafe { self.tim.steal() }));
        let c2 = pins
            .1
            .map(|_| PwmChannel2::new(unsafe { self.tim.steal() }));
        let c3 = pins
            .2
            .map(|_| PwmChannel3::new(unsafe { self.tim.steal() }));
        let c4 = pins
            .3
            .map(|_| PwmChannel4::new(unsafe { self.tim.steal() }));
        let t = PwmTimer::new(self.tim, self.clk);
        (t, c1, c2, c3, c4)
    }
}

// Destroy --------------------------------------------------------------------

pub fn destroy_counter_hz<TIM: GeneralTimer>(mut counter: CounterHz<TIM>) -> Timer<TIM> {
    // stop timer
    counter.tim.reset_config();
    Timer {
        tim: counter.tim,
        clk: counter.clk,
    }
}

#[cfg(feature = "rtic")]
pub fn destroy_mono_timer<TIM: GeneralTimer, const FREQ: u32>(
    mut timer: MonoTimer<TIM, FREQ>,
) -> FTimer<TIM, FREQ> {
    timer.tim.reset_config();
    timer.timer
}

// Enumerate ------------------------------------------------------------------

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

const fn compute_prescaler_arr(timer_clk: u32, update_freq: u32) -> (u32, u32) {
    let ticks = timer_clk / update_freq;
    let prescaler = (ticks - 1) / (1 << 16);
    let arr = ticks / (prescaler + 1) - 1;
    (prescaler, arr)
}
