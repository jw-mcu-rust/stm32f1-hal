pub mod uart_poll;

pub use core::convert::Infallible;
pub use uart_poll::*;

use embedded_io::ErrorKind;
// pub mod uart_dma_tx;
// pub use uart_dma_tx::*;
// pub mod uart_dma_ringbuf_rx;
// pub use uart_dma_ringbuf_rx::*;
// pub mod uart_dma_ringbuf_tx;
// pub use uart_dma_ringbuf_tx::*;

/// UART Transmitter
pub struct Tx<U: UartReg> {
    uart: U,
}

impl<U: UartReg> Tx<U> {
    pub(crate) fn new(uart: U) -> Self {
        Self { uart }
    }

    // pub fn get_interrupt_handler(&self) -> UartInterrupt<U> {
    //     UartInterrupt::new(unsafe { self.uart.steal_mut() })
    // }

    pub fn into_poll(self) -> UartPollTx<U> {
        UartPollTx::<U>::new(self.uart)
    }

    // pub fn into_dma<CH>(self, dma_ch: CH) -> UartDmaTx<U, CH>
    // where
    //     CH: BindDmaTx<U>,
    // {
    //     UartDmaTx::<U, CH>::new(self.uart, dma_ch)
    // }

    // pub fn into_dma_ringbuf<CH>(self, dma_ch: CH, buf_size: usize) -> UartDmaBufTx<U, CH>
    // where
    //     CH: BindDmaTx<U>,
    // {
    //     UartDmaBufTx::<U, CH>::new(self.uart, dma_ch, buf_size)
    // }
}

// ------------------------------------------------------------------------------------------------

/// UART Receiver
pub struct Rx<U: UartReg> {
    uart: U,
}

impl<U: UartReg> Rx<U> {
    pub(crate) fn new(uart: U) -> Self {
        Self { uart }
    }

    pub fn into_poll(self) -> UartPollRx<U> {
        UartPollRx::<U>::new(self.uart)
    }

    // pub fn into_dma_circle<CH>(self, dma_ch: CH, buf_size: usize) -> UartDmaBufRx<U, CH>
    // where
    //     CH: BindDmaRx<U>,
    // {
    //     UartDmaBufRx::<U, CH>::new(self.uart, dma_ch, buf_size)
    // }
}

// ------------------------------------------------------------------------------------------------

/// UART interrupt handler
// pub struct UartInterrupt<U: UartReg> {
//     uart: U,
// }

// impl<U: UartReg> UartInterrupt<U> {
//     pub(crate) fn new(uart: &'static mut U) -> Self {
//         Self { uart }
//     }

//     #[inline]
//     pub fn is_interrupted(&mut self, event: UartEvent) -> bool {
//         self.uart.is_interrupted(event)
//     }

//     #[inline]
//     pub fn listen(&mut self, event: UartEvent) {
//         self.uart.set_interrupt(event, true);
//     }

//     #[inline]
//     pub fn unlisten(&mut self, event: UartEvent) {
//         self.uart.set_interrupt(event, false);
//     }
// }

// ----------------------------------------------------------------------------

pub trait UartReg {
    fn set_dma_tx(&mut self, enable: bool);
    fn set_dma_rx(&mut self, enable: bool);

    fn get_tx_data_reg_addr(&self) -> u32;
    fn write(&mut self, word: u16) -> nb::Result<(), Infallible>;
    fn is_tx_empty(&self) -> bool;

    fn get_rx_data_reg_addr(&self) -> u32;
    fn read(&mut self) -> nb::Result<u16, Error>;
    fn is_rx_not_empty(&self) -> bool;

    fn set_interrupt(&mut self, event: UartEvent, enable: bool);
    fn is_interrupted(&mut self, event: UartEvent) -> bool;

    fn clear_pe_flag(&self);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UartEvent {
    /// New data can be sent
    TxEmpty,
    /// New data has been received
    RxNotEmpty,
    /// Idle line state detected
    Idle,
}

/// UART error
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The peripheral receive buffer was overrun.
    Overrun,
    /// Received data does not conform to the peripheral configuration.
    /// Can be caused by a misconfigured device on either end of the serial line.
    FrameFormat,
    /// Parity check failed.
    Parity,
    /// UART line is too noisy to read valid data.
    Noise,
    /// UART is busy and cannot accept new data.
    Busy,
    /// A different error occurred. The original error may contain more information.
    Other,
}

impl embedded_io::Error for Error {
    fn kind(&self) -> ErrorKind {
        match self {
            Error::Overrun => ErrorKind::InvalidData,
            Error::FrameFormat => ErrorKind::InvalidData,
            Error::Parity => ErrorKind::InvalidData,
            Error::Noise => ErrorKind::InvalidData,
            Error::Busy => ErrorKind::Interrupted,
            Error::Other => ErrorKind::Other,
        }
    }
}

pub enum WordLength {
    /// When parity is enabled, a word has 7 data bits + 1 parity bit,
    /// otherwise 8 data bits.
    Bits8,
    /// When parity is enabled, a word has 8 data bits + 1 parity bit,
    /// otherwise 9 data bits.
    Bits9,
}

pub enum Parity {
    ParityNone,
    ParityEven,
    ParityOdd,
}

pub enum StopBits {
    /// 1 stop bit
    STOP1,
    /// 0.5 stop bits
    STOP0P5,
    /// 2 stop bits
    STOP2,
    /// 1.5 stop bits
    STOP1P5,
}

pub struct Config {
    pub baudrate: u32,
    pub word_length: WordLength,
    pub parity: Parity,
    pub stop_bits: StopBits,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            baudrate: 115_200,
            word_length: WordLength::Bits8,
            parity: Parity::ParityNone,
            stop_bits: StopBits::STOP1,
        }
    }
}

impl Config {
    pub fn baudrate(mut self, baudrate: u32) -> Self {
        self.baudrate = baudrate;
        self
    }

    pub fn word_length(mut self, wordlength: WordLength) -> Self {
        self.word_length = wordlength;
        self
    }

    pub fn word_length_8bits(mut self) -> Self {
        self.word_length = WordLength::Bits8;
        self
    }

    pub fn word_length_9bits(mut self) -> Self {
        self.word_length = WordLength::Bits9;
        self
    }

    pub fn parity(mut self, parity: Parity) -> Self {
        self.parity = parity;
        self
    }

    pub fn parity_none(mut self) -> Self {
        self.parity = Parity::ParityNone;
        self
    }

    pub fn parity_even(mut self) -> Self {
        self.parity = Parity::ParityEven;
        self
    }

    pub fn parity_odd(mut self) -> Self {
        self.parity = Parity::ParityOdd;
        self
    }

    pub fn stop_bits(mut self, stop_bits: StopBits) -> Self {
        self.stop_bits = stop_bits;
        self
    }
}
