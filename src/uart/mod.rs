#[cfg(any(all(feature = "stm32f103", feature = "high"), feature = "connectivity"))]
pub(crate) mod uart;
pub(crate) mod usart;

pub use crate::common::uart::*;
#[cfg(any(all(feature = "stm32f103", feature = "high"), feature = "connectivity"))]
pub use uart::{Uart4, Uart5};
pub use usart::{Uart1, Uart2, Uart3};
