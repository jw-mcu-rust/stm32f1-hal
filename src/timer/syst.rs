//! SysTick: System Timer

use super::*;
use crate::{Mcu, time::Hertz};
use cortex_m::peripheral::SYST;
use cortex_m::peripheral::syst::SystClkSource;

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

    pub fn configure(&mut self, mcu: &Mcu) {
        self.syst.set_clock_source(SystClkSource::Core);
        self.clk = mcu.rcc.clocks.hclk();
    }

    pub fn configure_external(&mut self, mcu: &Mcu) {
        self.syst.set_clock_source(SystClkSource::External);
        self.clk = mcu.rcc.clocks.hclk() / 8;
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
