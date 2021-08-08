use std::{error, fmt};
use crate::{I2c, I2cError, Error, Duration, thread};

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
            Aht20Error::AhtI2c(ref err)=>
                write!(f, "I2c Error {}", err),
            // The wrapped error contains additional information and is available
            // via the source() method.
            Aht20Error::UnkonwnStatus =>
                write!(f, "UnkonwnStatus"),
        }
    }
}

impl error::Error for Aht20Error {}
//impl error::Error for Aht20Error {
//    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
//        match *self {
//            Aht20Error::AhtI2c(ref err) => Some(err),
//            // The cause is the underlying implementation error type. Is implicitly
//            // cast to the trait object `&error::Error`. This works because the
//            // underlying type already implements the `Error` trait.
//            Aht20Error::UnkonwnStatus => None,
//        }
//    }
//}

pub trait Aht20Decoder {
    fn to_human(&self) -> (f32, f32);
}

impl Aht20Decoder for [u8;6] {
    fn to_human(&self) -> (f32, f32) {
        let humid_data:u32 = (self[1] as u32) << 12 | (self[2] as u32) << 4 | (self[3] as u32) & 0xF0 >> 4;
        let temp_data:u32 = (self[3] as u32 & 0xF)<< 16 | (self[4] as u32) << 8 | self[5] as u32;

        (humid_data as f32 / 2_i32.pow(20) as f32 * 100.0, temp_data as f32 * ((200.0)/2_i32.pow(20) as f32) - 50.0)
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

    fn trigger_measure(&mut self) -> Result<(), Aht20Error> {
        let data:[u8;3] = [0xAC, 0x33, 0x00];
        self.i2c.write(&data)?;
        Ok(())
    }

    fn get_status(&mut self) -> Result<u8, Aht20Error> {
       // let data = [0x71 as u8; 1];
       // self.i2c.write(&data)?;

        let mut reg = [0u8; 1];
        self.i2c.read(&mut reg)?;
        Ok(reg[0])
    }

    pub fn get_sensor_data(&mut self) -> Result<(f32, f32), Aht20Error> {
        self.trigger_measure()?;
        let mut reg = [0u8; 6];
        let (humid_data, temp_data) =
        {
            loop {
                thread::sleep(Duration::from_millis(100));
                let status = self.get_status()?;
                if (status & 0x80u8 == 0) {
                    self.i2c.read(&mut reg)?;
                    break reg.to_human();
                } else {
                    println!("invalid status {}", status);
                }
            }
        };
        println!("humid_data is {} temp data is {}", humid_data, temp_data);
        Ok((humid_data, temp_data))
    }

    pub fn init(&mut self) -> Result<(), Aht20Error> {
        let status = self.get_status()?;
        println!("reg is {:?}", status);
        if (status & 0x16 == 0) {
            println!("send init command");
            let mut reg:[u8;2] = [0x08, 0x00];
            self.i2c.write(&reg)?;
            thread::sleep(Duration::from_micros(100));
        }
        Ok(())
    }
}
