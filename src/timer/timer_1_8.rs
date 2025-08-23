#![allow(dead_code)]
#![allow(unused_imports)]

use crate::pac::tim1;

use super::{Channel, CountDirection, freq_to_presc_arr};
use crate::{
    Mcu, Steal, afio,
    afio::{RemapMode, timer_remap::*},
    common::{timer::*, wrap_trait::*},
    pac,
    rcc::{BusClock, BusTimerClock, Enable, Reset},
    time::Hertz,
};

// Initialization -------------------------------------------------------------

macro_rules! impl_timer_init {
    ($($reg:ty),+) => {$(
        impl RegisterBlock for $reg {}
        impl TimerInit<$reg> for $reg {
            fn constrain(self) -> Timer<$reg> {
                Timer { reg: self }
            }
        }
    )+};
}

pub trait TimerInit<REG: RegisterBlock> {
    fn constrain(self) -> Timer<REG>;
}

pub trait RegisterBlock: RegisterBlockWrap + BusTimerClock + Enable + Reset {}

pub struct Timer<REG> {
    reg: REG,
}

#[allow(private_bounds)]
#[allow(unused_variables)]
impl<REG: RegisterBlock + Steal> Timer<REG> {
    fn steal(&self) -> Self {
        Self {
            reg: unsafe { self.reg.steal() },
        }
    }

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
    ) -> (
        impl BasicTimer,
        Option<impl TimerChannel>,
        Option<impl TimerChannel>,
        Option<impl TimerChannel>,
        Option<impl TimerChannel>,
    ) {
        self.config(mcu);
        REMAP::remap(&mut mcu.afio);
        self.set_count_direction(dir);

        // The reference manual is a bit ambiguous about when enabling this bit is really
        // necessary, but since we MUST enable the preload for the output channels then we
        // might as well enable for the auto-reload too
        self.reg.cr1().modify(|_, w| w.arpe().bit(preload));

        self.reg.bdtr().modify(|_, w| w.moe().set_bit());

        (
            BasicTimerImpl {
                reg: unsafe { self.reg.steal() },
                clk: REG::timer_clock(&mcu.rcc.clocks),
            },
            pins.0.map(|_| PwmChannel1 {
                reg: unsafe { self.reg.steal() },
            }),
            pins.1.map(|_| PwmChannel2 {
                reg: unsafe { self.reg.steal() },
            }),
            pins.2.map(|_| PwmChannel3 {
                reg: unsafe { self.reg.steal() },
            }),
            pins.3.map(|_| PwmChannel4 {
                reg: unsafe { self.reg.steal() },
            }),
        )
    }

    fn config(&mut self, mcu: &mut Mcu) {
        REG::enable(&mut mcu.rcc);
        REG::reset(&mut mcu.rcc);
    }

    #[inline]
    fn set_one_pulse_mode(&mut self, en: bool) {
        self.reg.cr1().modify(|_, w| w.opm().bit(en));
    }

    #[inline]
    fn master_mode(&mut self, mode: u8) {
        // tim6::cr2::MMS_A::Update as u8;
        unsafe { self.reg.cr2().modify(|_, w| w.mms().bits(mode)) };
    }

    #[inline]
    fn set_count_direction(&mut self, dir: CountDirection) {
        self.reg
            .cr1()
            .modify(|_, w| w.dir().bit(dir == CountDirection::Down));
    }
}

// Implement Basic Timer -------------------------------------------------------

struct BasicTimerImpl<REG> {
    reg: REG,
    clk: Hertz,
}
impl<REG: RegisterBlock> BasicTimerImpl<REG> {
    #[inline]
    fn set_prescaler(&mut self, psc: u32) {
        unsafe { self.reg.psc().write(|w| w.bits(psc)) };
    }

    #[inline]
    fn set_auto_reload_value(&mut self, value: u32) {
        // Note: Make it impossible to set the ARR value to 0, since this
        // would cause an infinite loop.
        if value > 0 {
            unsafe { self.reg.arr().write(|w| w.bits(value)) };
        }
    }

    #[inline]
    fn get_auto_reload_value(&self) -> u16 {
        self.reg.arr().read().arr().bits()
    }

    #[inline]
    fn trigger_update(&self) {
        // Sets the URS bit to prevent an interrupt from being triggered by
        // the UG bit
        self.reg.cr1().modify(|_, w| w.urs().set_bit());
        self.reg.egr().write(|w| w.ug().set_bit());
        self.reg.cr1().modify(|_, w| w.urs().clear_bit());
    }

    #[inline]
    fn set_interrupt(&self, en: bool) {
        self.reg.dier().modify(|_, w| w.uie().bit(en));
    }

    #[inline]
    fn is_interrupted(&self) -> bool {
        if self.reg.sr().read().uif().bit_is_set() {
            self.reg.sr().modify(|_, w| w.uif().clear_bit());
            true
        } else {
            false
        }
    }
}
impl<REG: RegisterBlock> BasicTimer for BasicTimerImpl<REG> {
    #[inline]
    fn start(&mut self) {
        self.reg.cnt().reset();
        self.reg.bdtr().modify(|_, w| w.aoe().set_bit());
        self.reg.cr1().modify(|_, w| w.cen().set_bit());
    }

    #[inline]
    fn stop(&mut self) {
        self.reg.cnt().reset();
        self.reg.cr1().modify(|_, w| w.cen().clear_bit());
    }

    #[inline]
    fn get_count_value(&self) -> u32 {
        self.reg.cnt().read().bits()
    }

    #[inline]
    fn get_max_duty(&self) -> u32 {
        (self.get_auto_reload_value() as u32).wrapping_add(1)
    }

    #[inline]
    fn config_freq(&mut self, count_freq: Hertz, update_freq: Hertz) {
        let (prescaler, arr) =
            freq_to_presc_arr(self.clk.raw(), count_freq.raw(), update_freq.raw());
        self.set_prescaler(prescaler);
        self.set_auto_reload_value(arr);
        // Trigger update event to load the registers
        self.trigger_update();
    }
}

// Implement PWM Channel -------------------------------------------------------

macro_rules! impl_pwm_channel {
    ($channel:ident, $ccmrx_output:ident, $ocxpe:ident, $ccxm:ident, $ccxp:ident, $ccxe:ident, $ccrx:ident) => {
        struct $channel<REG> {
            reg: REG,
        }
        impl<REG: RegisterBlock> TimerChannel for $channel<REG> {
            #[inline]
            fn config(&mut self, mode: PwmMode, polarity: PwmPolarity, duty: u32) {
                unsafe {
                    self.reg
                        .$ccmrx_output()
                        .modify(|_, w| w.$ocxpe().set_bit().$ccxm().bits(mode.into()));
                }
                self.reg
                    .ccer()
                    .modify(|_, w| w.$ccxp().bit(polarity.into()).$ccxe().set_bit());
                self.set_duty(duty);
            }

            #[inline(always)]
            fn set_enable(&mut self, en: bool) {
                self.reg.ccer().modify(|_, w| w.$ccxe().bit(en));
            }

            #[inline(always)]
            fn set_duty(&mut self, duty: u32) {
                unsafe { self.reg.$ccrx().write(|w| w.bits(duty)) };
            }
        }
    };
}
pub(crate) use impl_pwm_channel;

impl_pwm_channel!(PwmChannel1, ccmr1_output, oc1pe, oc1m, cc1p, cc1e, ccr1);
impl_pwm_channel!(PwmChannel2, ccmr1_output, oc2pe, oc2m, cc2p, cc2e, ccr2);
impl_pwm_channel!(PwmChannel3, ccmr2_output, oc3pe, oc3m, cc3p, cc3e, ccr3);
impl_pwm_channel!(PwmChannel4, ccmr2_output, oc4pe, oc4m, cc4p, cc4e, ccr4);

impl_timer_init!(pac::TIM1);
#[cfg(all(feature = "stm32f103", feature = "high"))]
impl_timer_init!(pac::TIM8);
wrap_trait_deref!(
    (pac::TIM1, pac::TIM8,),
    pub trait RegisterBlockWrap {
        fn cr1(&self) -> &tim1::CR1;
        fn cr2(&self) -> &tim1::CR2;
        fn smcr(&self) -> &tim1::SMCR;
        fn dier(&self) -> &tim1::DIER;
        fn sr(&self) -> &tim1::SR;
        fn egr(&self) -> &tim1::EGR;
        fn ccmr1_input(&self) -> &tim1::CCMR1_INPUT;
        fn ccmr1_output(&self) -> &tim1::CCMR1_OUTPUT;
        fn ccmr2_input(&self) -> &tim1::CCMR2_INPUT;
        fn ccmr2_output(&self) -> &tim1::CCMR2_OUTPUT;
        fn ccer(&self) -> &tim1::CCER;
        fn cnt(&self) -> &tim1::CNT;
        fn psc(&self) -> &tim1::PSC;
        fn arr(&self) -> &tim1::ARR;
        fn rcr(&self) -> &tim1::RCR;
        fn ccr(&self, n: usize) -> &tim1::CCR;
        fn ccr1(&self) -> &tim1::CCR;
        fn ccr2(&self) -> &tim1::CCR;
        fn ccr3(&self) -> &tim1::CCR;
        fn ccr4(&self) -> &tim1::CCR;
        fn bdtr(&self) -> &tim1::BDTR;
        fn dcr(&self) -> &tim1::DCR;
        fn dmar(&self) -> &tim1::DMAR;
        fn ccr_iter(&self) -> impl Iterator<Item = &tim1::CCR>;
    }
);
