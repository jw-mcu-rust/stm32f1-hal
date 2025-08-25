use super::{Error, Event, FTimer, GeneralTimer};
use core::ops::{Deref, DerefMut};
use fugit::{HertzU32 as Hertz, MicrosDurationU32, TimerDurationU32, TimerInstantU32};

/// Hardware timers
pub struct CounterHz<TIM> {
    pub(crate) tim: TIM,
    pub(crate) clk: Hertz,
}

impl<TIM: GeneralTimer> CounterHz<TIM> {
    pub fn start(&mut self, timeout: Hertz) -> Result<(), Error> {
        // pause
        self.tim.disable_counter();

        self.tim.clear_interrupt_flag(Event::Update);

        // reset counter
        self.tim.reset_counter();

        let clk = self.clk;
        self.tim.config_freq(clk, timeout);

        // start counter
        self.tim.enable_counter();

        Ok(())
    }

    pub fn wait(&mut self) -> nb::Result<(), Error> {
        if self.tim.get_interrupt_flag().contains(Event::Update) {
            self.tim.clear_interrupt_flag(Event::Update);
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    pub fn cancel(&mut self) -> Result<(), Error> {
        if !self.tim.is_counter_enabled() {
            return Err(Error::Disabled);
        }

        // disable counter
        self.tim.disable_counter();
        Ok(())
    }

    /// Restarts the timer in count down mode with user-defined prescaler and auto-reload register
    pub fn start_raw(&mut self, psc: u16, arr: u16) {
        // pause
        self.tim.disable_counter();

        self.tim.set_prescaler(psc);

        self.tim.set_auto_reload(arr as u32).unwrap();

        // Trigger an update event to load the prescaler value to the clock
        self.tim.trigger_update();

        // start counter
        self.tim.enable_counter();
    }

    /// Retrieves the content of the prescaler register. The real prescaler is this value + 1.
    pub fn psc(&self) -> u16 {
        self.tim.read_prescaler()
    }

    /// Retrieves the value of the auto-reload register.
    pub fn arr(&self) -> u16 {
        self.tim.read_auto_reload() as u16
    }

    /// Resets the counter
    pub fn reset(&mut self) {
        // Sets the URS bit to prevent an interrupt from being triggered by
        // the UG bit
        self.tim.trigger_update();
    }

    /// Returns the number of microseconds since the last update event.
    /// *NOTE:* This method is not a very good candidate to keep track of time, because
    /// it is very easy to lose an update event.
    pub fn now(&self) -> MicrosDurationU32 {
        let psc = self.tim.read_prescaler() as u32;

        // freq_divider is always bigger than 0, since (psc + 1) is always less than
        // timer_clock
        let freq_divider = (self.clk.raw() / (psc + 1)) as u64;
        let cnt: u32 = self.tim.read_count().into();
        let cnt = cnt as u64;

        // It is safe to make this cast, because the maximum timer period in this HAL is
        // 1s (1Hz), then 1 second < (2^32 - 1) microseconds
        MicrosDurationU32::from_ticks(u32::try_from(1_000_000 * cnt / freq_divider).unwrap())
    }
}

// ----------------------------------------------------------------------------

/// Periodic non-blocking timer that imlements [embedded_hal_02::timer::CountDown]
pub struct Counter<TIM, const FREQ: u32>(pub(super) FTimer<TIM, FREQ>);

impl<T, const FREQ: u32> Deref for Counter<T, FREQ> {
    type Target = FTimer<T, FREQ>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const FREQ: u32> DerefMut for Counter<T, FREQ> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// `Counter` with precision of 1 μs (1 MHz sampling)
pub type CounterUs<TIM> = Counter<TIM, 1_000_000>;

/// `Counter` with precision of of 1 ms (1 kHz sampling)
///
/// NOTE: don't use this if your system frequency more than 65 MHz
pub type CounterMs<TIM> = Counter<TIM, 1_000>;

impl<TIM: GeneralTimer, const FREQ: u32> Counter<TIM, FREQ> {
    /// Releases the TIM peripheral
    pub fn release(mut self) -> FTimer<TIM, FREQ> {
        // stop counter
        self.tim.reset_config();
        self.0
    }

    pub fn now(&self) -> TimerInstantU32<FREQ> {
        TimerInstantU32::from_ticks(self.tim.read_count().into())
    }

    pub fn start(&mut self, timeout: TimerDurationU32<FREQ>) -> Result<(), Error> {
        // pause
        self.tim.disable_counter();

        self.tim.clear_interrupt_flag(Event::Update);

        // reset counter
        self.tim.reset_counter();

        self.tim.set_auto_reload(timeout.ticks() - 1)?;

        // Trigger update event to load the registers
        self.tim.trigger_update();

        // start counter
        self.tim.enable_counter();

        Ok(())
    }

    pub fn wait(&mut self) -> nb::Result<(), Error> {
        if self.tim.get_interrupt_flag().contains(Event::Update) {
            self.tim.clear_interrupt_flag(Event::Update);
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    pub fn cancel(&mut self) -> Result<(), Error> {
        if !self.tim.is_counter_enabled() {
            return Err(Error::Disabled);
        }

        // disable counter
        self.tim.disable_counter();
        Ok(())
    }
}

impl<TIM: GeneralTimer, const FREQ: u32> fugit_timer::Timer<FREQ> for Counter<TIM, FREQ> {
    type Error = Error;

    fn now(&mut self) -> TimerInstantU32<FREQ> {
        Self::now(self)
    }

    fn start(&mut self, duration: TimerDurationU32<FREQ>) -> Result<(), Self::Error> {
        self.start(duration)
    }

    fn cancel(&mut self) -> Result<(), Self::Error> {
        self.cancel()
    }

    fn wait(&mut self) -> nb::Result<(), Self::Error> {
        self.wait()
    }
}
