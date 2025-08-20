#![no_std]

extern crate alloc;

pub mod afio;
pub mod backup_domain;
pub mod bb;
pub mod flash;
pub mod gpio;
pub mod interrupt;
pub mod nvic_scb;
pub mod prelude;
pub mod rcc;
pub mod time;
pub mod timer;
pub mod uart;

mod common;
mod os;

pub use embedded_hal;
pub use embedded_io;
pub use nb;
#[cfg(feature = "stm32f100")]
pub use stm32f1::stm32f100 as pac;
#[cfg(feature = "stm32f101")]
pub use stm32f1::stm32f101 as pac;
#[cfg(feature = "stm32f103")]
pub use stm32f1::stm32f103 as pac;
#[cfg(any(feature = "stm32f105", feature = "stm32f107"))]
pub use stm32f1::stm32f107 as pac;

pub(crate) trait Steal {
    /// Steal an instance of this peripheral
    ///
    /// # Safety
    ///
    /// Ensure that the new instance of the peripheral cannot be used in a way
    /// that may race with any existing instances, for example by only
    /// accessing read-only or write-only registers, or by consuming the
    /// original peripheral and using critical sections to coordinate
    /// access between multiple new instances.
    ///
    /// Additionally the HAL may rely on only one
    /// peripheral instance existing to ensure memory safety; ensure
    /// no stolen instances are passed to such software.
    unsafe fn steal(&self) -> Self;
}

impl<RB, const A: usize> Steal for stm32f1::Periph<RB, A> {
    unsafe fn steal(&self) -> Self {
        unsafe { Self::steal() }
    }
}

pub struct Mcu {
    // pub apb1: APB1,
    // pub apb2: APB2,
    // pub flash: pac::flash::Parts,
    pub scb: nvic_scb::Scb,
    pub nvic: nvic_scb::Nvic,
    pub rcc: rcc::Rcc,
    pub afio: afio::Afio,
}
