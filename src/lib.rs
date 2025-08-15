#![no_std]

pub mod afio;
pub mod backup_domain;
pub mod bb;
pub mod flash;
pub mod gpio;
pub mod prelude;
pub mod rcc;
pub mod time;
pub mod timer;
pub mod uart;

mod pac_ext_stf1;

pub use crate::pac_ext_stf1::Mcu;
pub use embedded_hal as hal;
pub use embedded_io as io;
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
