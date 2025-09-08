//! SysTick: System Timer

use super::*;
use crate::{Mcu, os::*, time::Hertz};
use core::ops::{Deref, DerefMut};
use cortex_m::peripheral::{SYST, syst::SystClkSource};
use embedded_hal::delay::DelayNs;
use fugit::{ExtU32Ceil, MicrosDurationU32, TimerDurationU32, TimerInstantU32};

pub trait SysTimerInit: Sized {
    /// Creates timer which takes [Hertz] as Duration
    fn counter_hz(self, mcu: &Mcu) -> SysCounterHz;

    /// Creates timer with custom precision (core frequency recommended is known)
    fn counter<const FREQ: u32>(self, mcu: &Mcu) -> SysCounter<FREQ>;
    /// Creates timer with precision of 1 μs (1 MHz sampling)
    fn counter_us(self, mcu: &Mcu) -> SysCounterUs {
        self.counter::<1_000_000>(mcu)
    }
    /// Blocking [Delay] with custom precision
    fn delay(self, mcu: &Mcu) -> SysDelay;
}

impl SysTimerInit for SYST {
    fn counter_hz(self, mcu: &Mcu) -> SysCounterHz {
        SystemTimer::syst(self, mcu).counter_hz()
    }
    fn counter<const FREQ: u32>(self, mcu: &Mcu) -> SysCounter<FREQ> {
        SystemTimer::syst(self, mcu).counter()
    }
    fn delay(self, mcu: &Mcu) -> SysDelay {
        SystemTimer::syst_external(self, mcu).delay()
    }
}

pub struct SystemTimer {
    pub(super) syst: SYST,
    pub(super) clk: Hertz,
}
impl SystemTimer {
    /// Initialize SysTick timer
    pub fn syst(mut syst: SYST, mcu: &Mcu) -> Self {
        syst.set_clock_source(SystClkSource::Core);
        Self {
            syst,
            clk: mcu.rcc.clocks.hclk(),
        }
    }

    /// Initialize SysTick timer and set it frequency to `HCLK / 8`
    pub fn syst_external(mut syst: SYST, mcu: &Mcu) -> Self {
        syst.set_clock_source(SystClkSource::External);
        Self {
            syst,
            clk: mcu.rcc.clocks.hclk() / 8,
        }
    }

    pub fn release(self) -> SYST {
        self.syst
    }

    /// Starts listening for an `event`
    pub fn listen(&mut self, event: SysEvent) {
        match event {
            SysEvent::Update => self.syst.enable_interrupt(),
        }
    }

    /// Stops listening for an `event`
    pub fn unlisten(&mut self, event: SysEvent) {
        match event {
            SysEvent::Update => self.syst.disable_interrupt(),
        }
    }

    /// Resets the counter
    pub fn reset(&mut self) {
        // According to the Cortex-M3 Generic User Guide, the interrupt request is only generated
        // when the counter goes from 1 to 0, so writing zero should not trigger an interrupt
        self.syst.clear_current();
    }
}

// Counter --------------------------------------------------------------------

impl SystemTimer {
    /// Creates [SysCounterHz] which takes [Hertz] as Duration
    pub fn counter_hz(self) -> SysCounterHz {
        SysCounterHz(self)
    }

    /// Creates [SysCounter] with custom precision (core frequency recommended is known)
    pub fn counter<const FREQ: u32>(self) -> SysCounter<FREQ> {
        SysCounter(self)
    }

    /// Creates [SysCounter] 1 microsecond precision
    pub fn counter_us(self) -> SysCounterUs {
        SysCounter(self)
    }
}

/// Hardware timers
pub struct SysCounterHz(SystemTimer);

impl Deref for SysCounterHz {
    type Target = SystemTimer;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SysCounterHz {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SysCounterHz {
    pub fn start(&mut self, timeout: Hertz) -> Result<(), Error> {
        let rvr = self.clk.raw() / timeout.raw() - 1;

        if rvr >= (1 << 24) {
            return Err(Error::WrongAutoReload);
        }

        self.syst.set_reload(rvr);
        self.syst.clear_current();
        self.syst.enable_counter();

        Ok(())
    }

    pub fn wait(&mut self) -> nb::Result<(), Error> {
        if self.syst.has_wrapped() {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    pub fn cancel(&mut self) -> Result<(), Error> {
        if !self.syst.is_counter_enabled() {
            return Err(Error::Disabled);
        }

        self.syst.disable_counter();
        Ok(())
    }
}

pub type SysCounterUs = SysCounter<1_000_000>;

/// SysTick timer with precision of 1 μs (1 MHz sampling)
pub struct SysCounter<const FREQ: u32>(SystemTimer);

impl<const FREQ: u32> Deref for SysCounter<FREQ> {
    type Target = SystemTimer;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const FREQ: u32> DerefMut for SysCounter<FREQ> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const FREQ: u32> SysCounter<FREQ> {
    /// Starts listening for an `event`
    pub fn listen(&mut self, event: SysEvent) {
        match event {
            SysEvent::Update => self.syst.enable_interrupt(),
        }
    }

    /// Stops listening for an `event`
    pub fn unlisten(&mut self, event: SysEvent) {
        match event {
            SysEvent::Update => self.syst.disable_interrupt(),
        }
    }

    pub fn now(&self) -> TimerInstantU32<FREQ> {
        TimerInstantU32::from_ticks(SYST::get_current() / (self.clk.raw() / FREQ))
    }

    pub fn start(&mut self, timeout: TimerDurationU32<FREQ>) -> Result<(), Error> {
        let rvr = timeout.ticks() * (self.clk.raw() / FREQ) - 1;

        if rvr >= (1 << 24) {
            return Err(Error::WrongAutoReload);
        }

        self.syst.set_reload(rvr);
        self.syst.clear_current();
        self.syst.enable_counter();

        Ok(())
    }

    pub fn wait(&mut self) -> nb::Result<(), Error> {
        if self.syst.has_wrapped() {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    pub fn cancel(&mut self) -> Result<(), Error> {
        if !self.syst.is_counter_enabled() {
            return Err(Error::Disabled);
        }

        self.syst.disable_counter();
        Ok(())
    }
}

impl<const FREQ: u32> fugit_timer::Timer<FREQ> for SysCounter<FREQ> {
    type Error = Error;

    fn now(&mut self) -> TimerInstantU32<FREQ> {
        Self::now(self)
    }

    fn start(&mut self, duration: TimerDurationU32<FREQ>) -> Result<(), Self::Error> {
        self.start(duration)
    }

    fn wait(&mut self) -> nb::Result<(), Self::Error> {
        self.wait()
    }

    fn cancel(&mut self) -> Result<(), Self::Error> {
        self.cancel()
    }
}

// Delay ----------------------------------------------------------------------

/// Timer as a delay provider (SysTick by default)
pub struct SysDelay(SystemTimer);

impl Deref for SysDelay {
    type Target = SystemTimer;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SysDelay {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SysDelay {
    /// Releases the timer resource
    pub fn release(self) -> SystemTimer {
        self.0
    }
}

impl SystemTimer {
    pub fn delay(self) -> SysDelay {
        SysDelay(self)
    }
}

impl SysDelay {
    pub fn delay(&mut self, us: MicrosDurationU32) {
        // The SysTick Reload Value register supports values between 1 and 0x00FFFFFF.
        const MAX_RVR: u32 = 0x00FF_FFFF;

        let mut total_rvr = us.ticks() * (self.clk.raw() / 1_000_000);

        while total_rvr != 0 {
            let current_rvr = total_rvr.min(MAX_RVR);

            self.syst.set_reload(current_rvr);
            self.syst.clear_current();
            self.syst.enable_counter();

            // Update the tracking variable while we are waiting...
            total_rvr -= current_rvr;

            while !self.syst.has_wrapped() {}

            self.syst.disable_counter();
        }
    }
}

impl fugit_timer::Delay<1_000_000> for SysDelay {
    type Error = core::convert::Infallible;

    fn delay(&mut self, duration: MicrosDurationU32) -> Result<(), Self::Error> {
        self.delay(duration);
        Ok(())
    }
}

impl DelayNs for SysDelay {
    fn delay_ns(&mut self, ns: u32) {
        self.delay(ns.nanos_at_least());
    }

    fn delay_ms(&mut self, ms: u32) {
        self.delay(ms.millis_at_least());
    }
}

// ----------------------------------------------------------------------------

/// SysTick must be set to 1 kHz frequency
pub struct SysTickTimeout {
    timeout_us: usize,
}
impl SysTickTimeout {
    pub fn new(timeout_us: usize) -> Self {
        Self { timeout_us }
    }
}
impl Timeout for SysTickTimeout {
    fn start(&mut self) -> impl TimeoutInstance {
        let now = SYST::get_current() as usize;
        let reload = SYST::get_reload() as usize;
        let round = self.timeout_us / 1000;
        let us = self.timeout_us % 1000;

        SysTickTimeoutInstance {
            former_tick: now,
            timeout_tick: us * reload / 1000,
            elapsed_tick: 0,
            round_backup: round,
            round,
        }
    }
}

pub struct SysTickTimeoutInstance {
    former_tick: usize,
    timeout_tick: usize,
    elapsed_tick: usize,
    round: usize,
    round_backup: usize,
}
impl SysTickTimeoutInstance {
    fn elapsed(&mut self) -> usize {
        let now = SYST::get_current() as usize;
        let elapsed = if now <= self.former_tick {
            self.former_tick - now
        } else {
            self.former_tick + (SYST::get_reload() as usize - now)
        };
        self.former_tick = now;
        elapsed
    }
}
impl TimeoutInstance for SysTickTimeoutInstance {
    fn timeout(&mut self) -> bool {
        self.elapsed_tick += self.elapsed();

        if self.round == 0 {
            if self.elapsed_tick >= self.timeout_tick {
                self.elapsed_tick -= self.timeout_tick;
                self.round = self.round_backup;
                return true;
            }
        } else {
            let reload = SYST::get_reload() as usize;
            if self.elapsed_tick >= reload {
                self.elapsed_tick -= reload;
                self.round -= 1;
            }
        }
        false
    }

    #[inline(always)]
    fn restart(&mut self) {
        self.round = self.round_backup;
        self.elapsed_tick = 0;
    }

    #[inline(always)]
    fn interval(&self) {}
}
