pub mod uart;

use crate::*;
use stm32f1::Periph;

impl<RB, const A: usize> Steal for Periph<RB, A> {
    unsafe fn steal(&self) -> Self {
        unsafe { Self::steal() }
    }
}

pub struct Mcu {
    // pub apb1: APB1,
    // pub apb2: APB2,
    // pub flash: pac::flash::Parts,
    // pub afio: pac::afio::Parts,
    pub rcc: rcc::Rcc,
    // pub nvic: NVIC,
}
