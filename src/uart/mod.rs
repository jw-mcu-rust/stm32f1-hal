#[cfg(any(all(feature = "stm32f103", feature = "high"), feature = "connectivity"))]
pub(crate) mod uart;
pub(crate) mod usart;

pub use crate::common::uart::*;
