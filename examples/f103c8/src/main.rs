#![no_std]
#![no_main]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use core::{mem::MaybeUninit, panic::PanicInfo};
use cortex_m::asm;
use cortex_m_rt::entry;
use embedded_alloc::LlffHeap as Heap;
use jw_stm32f1_hal as hal;
use jw_stm32f1_hal::{
    Mcu,
    gpio::PinState,
    nvic_scb::PriorityGrouping,
    pac::{self, Interrupt},
    prelude::*,
    rcc,
    timer::Timer,
    uart,
    uart::UartEvent,
};

mod uart_task;
use uart_task::UartPollTask;

#[global_allocator]
static HEAP: Heap = Heap::empty();
const HEAP_SIZE: usize = 15 * 1024;
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
    let sysclk = 16.MHz();
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
    mcu.nvic.set_priority(Interrupt::USART1, 10);

    // UART ---------------------------------------

    let uart1 = dp.USART1.constrain();
    let mut uart1_it = uart1.get_interrupt_handler();

    // let pin_tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    // let pin_rx = gpioa.pa10.into_pull_up_input(&mut gpioa.crh);
    let pin_tx = gpiob.pb6.into_alternate_push_pull(&mut gpiob.crl);
    let pin_rx = gpiob.pb7.into_pull_up_input(&mut gpiob.crl);
    let config = uart::Config::default();
    let (uart_tx, uart_rx) = uart1.into_tx_rx((pin_tx, pin_rx), config, &mut mcu);
    let (uart_tx, uart_rx) = (uart_tx.into_poll(), uart_rx.into_poll());
    let mut uart_task = UartPollTask::new(uart_tx, uart_rx);

    uart1_it.listen(UartEvent::Idle);
    all_it::USART1_CB.set(
        &mut mcu,
        move || {
            if uart1_it.is_interrupted(UartEvent::Idle) {}
        },
    );

    // LED ----------------------------------------

    let mut led = gpiob
        .pb0
        .into_open_drain_output_with_state(&mut gpiob.crl, PinState::High);

    let mut timer = Timer::syst(cp.SYST, &mcu.rcc.clocks).counter_hz();
    timer.start(2.Hz()).unwrap();

    loop {
        if timer.wait().is_ok() {
            led.toggle();
        }
        uart_task.poll();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    asm::bkpt();
    loop {}
}

mod all_it {
    use super::hal::{interrupt_callback, pac::interrupt};

    interrupt_callback!(USART1, USART1_CB);
}
