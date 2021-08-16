use std::{error, fmt};
use crc8::Crc8;
use crate::{I2c, I2cError, Error, Duration, thread};


const EMC2101_WHOAMI: u8 = 0xFD;
pub struct Emc2101 {
    bus: u8,
    addr: u16,
    i2c: I2c
}

#[derive(Debug)]
pub enum Emc2101Error {
    AhtI2c(I2cError),
    InvalidDeviceId,
    UnkonwnStatus,
}

impl From<I2cError> for Emc2101Error {
    fn from(err: I2cError) -> Self {
        Emc2101Error::AhtI2c(err)
    }
}

impl fmt::Display for Emc2101Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Emc2101Error::AhtI2c(ref err)=>
                write!(f, "I2c Error {}", err),
            Emc2101Error::InvalidDeviceId =>
                write!(f, "InvalidDeviceId"),
            // The wrapped error contains additional information and is available
            // via the source() method.
            Emc2101Error::UnkonwnStatus =>
                write!(f, "UnkonwnStatus"),
        }
    }
}

impl error::Error for Emc2101Error {}

impl Emc2101 {
    pub fn new(bus: u8, addr: u16) -> Result<Emc2101, Emc2101Error> {
        let mut i2c = I2c::with_bus(bus)?;
        i2c.set_slave_address(addr)?;
        Ok(Emc2101 {
            bus,
            addr,
            i2c,
        })
    }

    pub fn init(&mut self) -> Result<(), Emc2101Error> {
        let id = self.i2c.smbus_read_byte(EMC2101_WHOAMI)?;
        if (id == 0x16 || id == 0x28) {
            Ok(())
        } else {
            Err(Emc2101Error::InvalidDeviceId)
        }
    }
}
