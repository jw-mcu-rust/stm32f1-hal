#![no_std]
#![no_main]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use core::panic::PanicInfo;
use cortex_m::asm;
use cortex_m_rt::entry;
use jw_stm32f1_hal::{Mcu, gpio::PinState, io, pac, prelude::*, rcc, timer::Timer, uart};

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let sysclk = 16.MHz();
    let cfg = rcc::Config::hse(8.MHz()).sysclk(sysclk);
    let mut rcc = dp.RCC.constrain().freeze(cfg, &mut flash.acr);
    assert_eq!(rcc.clocks.sysclk(), sysclk);

    let mut gpioa = dp.GPIOA.split(&mut rcc);
    let mut gpiob = dp.GPIOB.split(&mut rcc);

    let afio = dp.AFIO.constrain(&mut rcc);
    let mut mcu = Mcu { rcc, afio };

    // UART ---------------------------------------

    let config = uart::Config::default();
    let pin_tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let pin_rx = gpioa.pa10.into_pull_up_input(&mut gpioa.crh);
    let (Some(uart_tx), Some(uart_rx)) =
        dp.USART1
            .constrain()
            .into_tx_rx((Some(pin_tx), Some(pin_rx)), config, &mut mcu)
    else {
        panic!()
    };
    let mut uart_task = UartPollTask::new(uart_tx.into_poll(), uart_rx.into_poll());

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

struct UartPollTask<W, R> {
    tx: W,
    rx: R,
    buf: [u8; 32],
    tx_i: usize,
    rx_i: usize,
}

impl<W, R> UartPollTask<W, R>
where
    W: io::Write,
    R: io::Read,
{
    fn new(tx: W, rx: R) -> Self {
        Self {
            tx,
            rx,
            buf: [0; 32],
            tx_i: 0,
            rx_i: 0,
        }
    }

    fn poll(&mut self) {
        let mut i = 1;
        while i > 0 && self.rx_i < 30 {
            if let Ok(size) = self.rx.read(&mut self.buf[self.rx_i..]) {
                self.rx_i += size;
                if size > 0 {
                    // continually receive
                    i = 100;
                }
            }
            i -= 1;
        }

        // loopback
        if self.rx_i > self.tx_i
            && let Ok(size) = self.tx.write(&self.buf[self.tx_i..self.rx_i])
        {
            self.tx_i += size;
        }

        if self.rx_i == self.tx_i {
            self.rx_i = 0;
            self.tx_i = 0;
        }
    }
}
