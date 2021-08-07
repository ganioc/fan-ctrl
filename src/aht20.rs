use std::{error, fmt};
use crate::{I2c, I2cError, Error};

pub struct Aht20 {
    bus: u8,
    addr: u16,
    i2c: I2c
}

#[derive(Debug)]
pub enum Aht20Error {
    AhtI2c(I2cError),
    UnkonwnStatus,
}

impl From<I2cError> for Aht20Error {
    fn from(err: I2cError) -> Self {
        Aht20Error::AhtI2c(err)
    }
}

impl fmt::Display for Aht20Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Aht20Error::AhtI2c(..)=>
                write!(f, "I2c Error"),
            // The wrapped error contains additional information and is available
            // via the source() method.
            Aht20Error::UnkonwnStatus =>
                write!(f, "UnkonwnStatus"),
        }
    }
}

impl error::Error for Aht20Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Aht20Error::AhtI2c(ref err) => Some(err),
            // The cause is the underlying implementation error type. Is implicitly
            // cast to the trait object `&error::Error`. This works because the
            // underlying type already implements the `Error` trait.
            Aht20Error::UnkonwnStatus => None,
        }
    }
}

impl Aht20 {
    pub fn new(bus: u8, addr: u16) -> Result<Aht20, Aht20Error> {
        let mut i2c = I2c::with_bus(bus)?;
        i2c.set_slave_address(addr)?;
        Ok(Aht20 {
            bus,
            addr,
            i2c,
        })
    }

    fn get_status(&mut self) -> Result<u8, Aht20Error> {
        let data = [0x71 as u8; 1];
        self.i2c.write(&data)?;

        let mut reg = [0u8; 1];
        self.i2c.read(&mut reg)?;
        Ok(reg[0])
    }

    pub fn init(&mut self) -> Result<(), Aht20Error> {
        let status = self.get_status()?;
        println!("reg is {:?}", status);
        Ok(())
    }
}
