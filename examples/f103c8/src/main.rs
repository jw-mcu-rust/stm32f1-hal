#![no_std]
#![no_main]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use core::{mem::MaybeUninit, panic::PanicInfo};
use cortex_m::asm;
use cortex_m_rt::entry;
use jw_stm32f1_hal as hal;
use jw_stm32f1_hal::{
    Heap, Mcu,
    afio::{NONE_PIN, RemapDefault},
    embedded_hal, embedded_io,
    gpio::PinState,
    nvic_scb::PriorityGrouping,
    pac::{self, Interrupt},
    prelude::*,
    rcc,
    timer::*,
    uart::{self, UartPeriph},
};

mod led_task;
use led_task::LedTask;
mod uart_task;
use uart_task::UartPollTask;

#[global_allocator]
static HEAP: Heap = Heap::empty();
const HEAP_SIZE: usize = 10 * 1024;
static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let mut scb = cp.SCB.constrain();
    // Set it as early as possible
    scb.set_priority_grouping(PriorityGrouping::Group4);
    // Initialize the heap BEFORE you use it
    unsafe { HEAP.init(&raw mut HEAP_MEM as usize, HEAP_SIZE) }

    let dp = pac::Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let sysclk = 72.MHz();
    let cfg = rcc::Config::hse(8.MHz()).sysclk(sysclk);
    let mut rcc = dp.RCC.constrain().freeze(cfg, &mut flash.acr);
    assert_eq!(rcc.clocks.sysclk(), sysclk);

    let mut gpioa = dp.GPIOA.split(&mut rcc);
    let mut gpiob = dp.GPIOB.split(&mut rcc);

    let afio = dp.AFIO.constrain(&mut rcc);
    let mut mcu = Mcu {
        scb,
        nvic: cp.NVIC.constrain(),
        rcc,
        afio,
    };

    // Keep them in one place for easier management
    mcu.nvic.set_priority(Interrupt::USART1, 1);

    // UART -------------------------------------

    // let pin_tx = Some(gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh));
    // let pin_rx = Some(gpioa.pa10.into_pull_up_input(&mut gpioa.crh));
    let pin_tx = Some(gpiob.pb6.into_alternate_push_pull(&mut gpiob.crl));
    let pin_rx = Some(gpiob.pb7.into_pull_up_input(&mut gpiob.crl));
    // let pin_rx = hal::afio::NONE_PIN;

    let config = uart::Config::default();
    let uart1 = dp.USART1.constrain();
    let (Some(uart_tx), Some(uart_rx)) = uart1.into_tx_rx((pin_tx, pin_rx), config, &mut mcu)
    else {
        panic!()
    };

    // let mut uart_task = uart_poll_init(uart_tx, uart_rx);
    let mut uart_task = uart_interrupt_init(
        uart_tx,
        uart_rx,
        &mut mcu,
        pac::interrupt::USART1,
        &all_it::USART1_CB,
    );

    // LED --------------------------------------

    let mut led = gpiob
        .pb0
        .into_open_drain_output_with_state(&mut gpiob.crl, PinState::High);
    let mut timer = cp.SYST.counter_hz(&mcu.rcc.clocks);
    let freq = 100.Hz();
    timer.start(freq).unwrap();
    let mut led_task = LedTask::new(led, freq.raw());

    // PWM --------------------------------------

    let c1 = gpioa.pa8.into_alternate_push_pull(&mut gpioa.crh);
    let (mut bt, Some(mut ch1), _, _, _) = dp.TIM1.constrain().into_pwm::<RemapDefault<_>>(
        (Some(c1), NONE_PIN, NONE_PIN, NONE_PIN),
        CountDirection::Up,
        true,
        &mut mcu,
    ) else {
        panic!()
    };
    bt.config_freq(1.MHz(), 20.kHz());

    ch1.config(
        PwmMode::Mode1,
        PwmPolarity::ActiveHigh,
        bt.get_max_duty() / 2,
    );

    bt.start();

    loop {
        if timer.wait().is_ok() {
            led_task.poll();
        }
        uart_task.poll();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    asm::bkpt();
    loop {}
}

fn uart_poll_init<U: UartPeriph>(
    tx: uart::Tx<U>,
    rx: uart::Rx<U>,
) -> UartPollTask<impl embedded_io::Write, impl embedded_io::Read> {
    let (uart_tx, uart_rx) = (tx.into_poll(0, 10_000), rx.into_poll(0, 1_000));
    UartPollTask::new(32, uart_tx, uart_rx)
}

fn uart_interrupt_init<U: UartPeriph + 'static>(
    tx: uart::Tx<U>,
    rx: uart::Rx<U>,
    mcu: &mut Mcu,
    it_line: pac::interrupt,
    interrupt_callback: &hal::interrupt::Callback,
) -> UartPollTask<impl embedded_io::Write + use<U>, impl embedded_io::Read + use<U>> {
    mcu.nvic.enable(it_line, false);
    let (tx, mut tx_it) = tx.into_interrupt(64, 0, 10_000);
    let (rx, mut rx_it) = rx.into_interrupt(64, 0);
    interrupt_callback.set(mcu, move || {
        rx_it.handler();
        tx_it.handler();
    });
    UartPollTask::new(32, tx, rx)
}

mod all_it {
    use super::hal::{interrupt_handler, pac::interrupt};
    interrupt_handler!((USART1, USART1_CB), (EXTI1, EXTI1_CB),);
}
