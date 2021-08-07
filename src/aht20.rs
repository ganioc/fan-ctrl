use crate::{I2c, Error as I2cError};

pub struct Aht20 {
    bus: u8,
    addr: u16,
    i2c: I2c
}

pub enum Error<'a> {
    I2c(&'a I2cError),
    UnkonwnStatus,
}

impl Aht20 {
    pub fn new(bus: u8, addr: u16) -> Result<Aht20, Box<dyn I2cError>> {
        let mut i2c = I2c::with_bus(bus)?;
        i2c.set_slave_address(addr)?;
        Ok(Aht20 {
            bus,
            addr,
            i2c,
        })
    }

    fn get_status(&mut self) -> Result<u8, Box<dyn I2cError>> {
        let data = [0x71 as u8; 1];
        self.i2c.write(&data)?;

        let mut reg = [0u8; 1];
        self.i2c.read(&mut reg)?;
        Ok(reg[0])
    }

    pub fn init(&mut self) -> Result<(), Box<dyn I2cError>> {
        let status = self.get_status()?;
        println!("reg is {:?}", status);
        Ok(())
    }
}
