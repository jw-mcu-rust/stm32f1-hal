use super::Afio;
use crate::{
    gpio::*,
    uart::{UartReg, *},
};

pub trait UartTxPin<U: UartReg> {
    fn set_remap_reg(&self, afio: &mut Afio) -> u8;
}
impl UartTxPin<Uart1> for PA9<Alternate<PushPull>> {
    fn set_remap_reg(&self, afio: &mut Afio) -> u8 {
        afio.reg.mapr().read().usart1_remap().bit() as u8
    }
}
impl UartTxPin<Uart1> for PB6<Alternate<PushPull>> {
    fn set_remap_reg(&self, afio: &mut Afio) -> u8 {
        afio.mapr.modify_mapr(|_, w| w.usart1_remap().set_bit());
        afio.reg.mapr().read().usart1_remap().bit() as u8
    }
}

pub trait UartRxPin<U: UartReg> {
    fn set_remap_reg(&self, afio: &mut Afio) -> u8;
}
impl<PULL: UpMode> UartRxPin<Uart1> for PA10<Input<PULL>> {
    fn set_remap_reg(&self, afio: &mut Afio) -> u8 {
        afio.reg.mapr().read().usart1_remap().bit() as u8
    }
}
impl<PULL: UpMode> UartRxPin<Uart1> for PB7<Input<PULL>> {
    fn set_remap_reg(&self, afio: &mut Afio) -> u8 {
        afio.mapr.modify_mapr(|_, w| w.usart1_remap().set_bit());
        afio.reg.mapr().read().usart1_remap().bit() as u8
    }
}
