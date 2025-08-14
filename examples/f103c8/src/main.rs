#![no_std]
#![no_main]

use core::panic::PanicInfo;
use cortex_m::asm;
use cortex_m_rt::entry;
use jw_stm32f1_hal::{gpio::PinState, pac, prelude::*, rcc, timer::Timer};
use nb::block;

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let sysclk = 16.MHz();
    let cfg = rcc::Config::hse(8.MHz()).sysclk(sysclk);
    let mut rcc = dp.RCC.constrain().freeze(cfg, &mut flash.acr);
    assert_eq!(rcc.clocks.sysclk(), sysclk);

    let mut gpiob = dp.GPIOB.split(&mut rcc);
    let mut led = gpiob
        .pb0
        .into_open_drain_output_with_state(&mut gpiob.crl, PinState::High);

    let mut timer = Timer::syst(cp.SYST, &rcc.clocks).counter_hz();
    timer.start(2.Hz()).unwrap();

    loop {
        block!(timer.wait()).unwrap();
        led.set_low();
        block!(timer.wait()).unwrap();
        led.set_high();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    asm::bkpt();
    loop {}
}
