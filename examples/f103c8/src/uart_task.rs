use crate::embedded_io::{Read, Write};

pub struct UartPollTask<W: Write, R: Read> {
    tx: W,
    rx: R,
    buf: [u8; 32],
    tx_i: usize,
    rx_i: usize,
}

impl<W, R> UartPollTask<W, R>
where
    W: Write,
    R: Read,
{
    pub fn new(tx: W, rx: R) -> Self {
        Self {
            tx,
            rx,
            buf: [0; 32],
            tx_i: 0,
            rx_i: 0,
        }
    }

    pub fn poll(&mut self) {
        if self.rx_i < 30 {
            if let Ok(size) = self.rx.read(&mut self.buf[self.rx_i..]) {
                self.rx_i += size;
            }
        }

        // loopback
        if self.rx_i > self.tx_i
            && let Ok(size) = self.tx.write(&self.buf[self.tx_i..self.rx_i])
        {
            self.tx_i += size;
        }

        if self.rx_i > 10 && self.rx_i == self.tx_i {
            self.rx_i = 0;
            self.tx_i = 0;
            self.tx.flush().unwrap();
        }
    }
}
