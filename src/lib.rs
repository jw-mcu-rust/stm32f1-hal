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
