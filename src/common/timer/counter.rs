use super::*;
use fugit::{HertzU32 as Hertz, MicrosDurationU32};

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
        self.tim.config_freq(clk, clk, timeout);

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
