pub use crate::afio::AfioInit as _stm32_hal_afio_AfioInit;
pub use crate::afio::RFrom as _;
pub use crate::afio::RInto as _;
pub use crate::afio::Remap as _;
pub use crate::flash::FlashExt as _stm32_hal_flash_FlashExt;
pub use crate::gpio::GpioExt as _stm32_hal_gpio_GpioExt;
pub use crate::rcc::BkpExt as _;
pub use crate::rcc::RccExt as _stm32_hal_rcc_RccExt;
pub use crate::time::U32Ext as _stm32_hal_time_U32Ext;
#[cfg(feature = "rtic")]
pub use crate::timer::MonoTimerExt as _stm32f4xx_hal_timer_MonoTimerExt;
pub use crate::timer::SysTimerInit as _stm32_hal_timer_SysCounterInit;
pub use crate::timer::TimerInit as _stm32_hal_timer_TimerInit;
// pub use crate::timer::pwm_input::PwmInputExt as _;
// pub use crate::timer::pwm_input::QeiExt as _;
pub use crate::nvic_scb::NvicInit as _;
pub use crate::nvic_scb::ScbInit as _;
pub use crate::ringbuf::ConsumerExt;
pub use crate::ringbuf::ProducerExt;
pub use crate::ringbuf::ReadChunkExt;
pub use crate::ringbuf::WriteChunkExt;
// pub use crate::timer::timer_1_8::TimerInit as _;
#[cfg(any(all(feature = "stm32f103", feature = "high"), feature = "connectivity"))]
pub use crate::uart::UartInit as _;
pub use cortex_m;
pub use cortex_m_rt;
pub use fugit::ExtU32 as _fugit_ExtU32;
pub use fugit::RateExtU32 as _fugit_RateExtU32;
