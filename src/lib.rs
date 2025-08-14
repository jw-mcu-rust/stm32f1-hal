#![no_std]

pub mod backup_domain;
pub mod bb;
pub mod flash;
pub mod prelude;
pub mod rcc;
pub mod time;

pub use embedded_hal as hal;
pub use fugit::HertzU32 as Hertz;
#[cfg(feature = "stm32f100")]
pub use stm32f1::stm32f100 as pac;
#[cfg(feature = "stm32f101")]
pub use stm32f1::stm32f101 as pac;
#[cfg(feature = "stm32f103")]
pub use stm32f1::stm32f103 as pac;
#[cfg(any(feature = "stm32f105", feature = "stm32f107"))]
pub use stm32f1::stm32f107 as pac;

use stm32f1::Periph;

mod sealed {
    pub trait Sealed {}
}
use sealed::Sealed;

impl<RB, const A: usize> Sealed for Periph<RB, A> {}

pub trait Ptr: Sealed {
    /// RegisterBlock structure
    type RB;
    /// Return the pointer to the register block
    fn ptr() -> *const Self::RB;
}

impl<RB, const A: usize> Ptr for Periph<RB, A> {
    type RB = RB;
    fn ptr() -> *const Self::RB {
        Self::ptr()
    }
}

pub trait Steal: Sealed {
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
    unsafe fn steal() -> Self;
}

impl<RB, const A: usize> Steal for Periph<RB, A> {
    unsafe fn steal() -> Self {
        unsafe { Self::steal() }
    }
}
