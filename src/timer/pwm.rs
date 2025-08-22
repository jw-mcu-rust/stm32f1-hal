use super::*;
use crate::{
    Mcu, Steal, afio,
    afio::{RemapMode, timer_remap::*},
    common::{timer::*, wrap_trait::*},
    pac,
    rcc::{BusClock, BusTimerClock, Enable, Reset},
    time::Hertz,
    timer::Channel,
};

pub trait RegisterBlock: WithPwm {}

#[allow(private_bounds)]
#[allow(unused_variables)]
impl<REG: RegisterBlock> Timer<REG> {
    pub fn into_pwm<REMAP: RemapMode<REG>>(
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
    ) {
        REMAP::remap(&mut mcu.afio);
    }
}
