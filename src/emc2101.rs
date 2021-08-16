use std::{error, fmt};
use crc8::Crc8;
use crate::{I2c, I2cError, Error, Duration, thread};

const EMC2101_REG_CONFIG: u8 = 0x3;
const EMC2101_WHOAMI: u8 = 0xFD;
const EMC2101_FAN_CONFIG: u8 = 0x4A;
const EMC2101_PWM_FREQ: u8 = 0x4D;
const EMC2101_REG_FAN_SETTING: u8 = 0x4C;

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

pub trait RegData {
    fn set_bit(&self, value: bool, bit: u8) -> u8;
}

impl RegData for u8 {
    fn set_bit(&self, value: bool, bit: u8) -> u8 {

        if (value) {
            *self | (1u8 << bit)
        } else {
            *self & !(1u8 << bit)
        }
    }
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

    pub fn enable_tach(&mut self, enable: bool) -> Result<(), Emc2101Error> {
        let mut data = self.i2c.smbus_read_byte(EMC2101_REG_CONFIG)?;
        if (enable) {
            data = data | 1u8 << 2;
        } else {
            data = data & !(1u8 << 2);
        }
        println!("data is {:02X}", data);
        self.i2c.smbus_write_byte(EMC2101_REG_CONFIG, data)?;
        Ok(())
    }

    pub fn invert_fan_speed(&mut self, invert: bool) -> Result<(), Emc2101Error> {
        let mut data = self.i2c.smbus_read_byte(EMC2101_FAN_CONFIG)?;
        if (invert) {
            data = data | 1u8 << 4;
        } else {
            data = data & !(1u8 << 4);
        }
        self.i2c.smbus_write_byte(EMC2101_FAN_CONFIG, data)?;
        Ok(())
    }

    pub fn set_pwm_frequency(&mut self, freq: u8) -> Result<(), Emc2101Error> {
        self.i2c.smbus_write_byte(EMC2101_PWM_FREQ, freq)?;
        Ok(())
    }

    pub fn set_pwm_clock(&mut self, clksel:bool, clkovr:bool) -> Result<(), Emc2101Error> {
        let mut data = self.i2c.smbus_read_byte(EMC2101_FAN_CONFIG)?;

        data = data.set_bit(clksel, 3);
        data = data.set_bit(clkovr, 2);
        println!("set pwm clock data is {:02X}", data);
        self.i2c.smbus_write_byte(EMC2101_FAN_CONFIG, data)?;
        Ok(())
    }

    pub fn enable_lut(&mut self, enable:bool) -> Result<(), Emc2101Error> {
        let mut data = self.i2c.smbus_read_byte(EMC2101_FAN_CONFIG)?;

        data = data.set_bit(enable, 5);
        self.i2c.smbus_write_byte(EMC2101_FAN_CONFIG, data)?;
        Ok(())
    }

    pub fn set_duty_cycle(&mut self, duty: u8) -> Result<(), Emc2101Error> {
        let mut to_reg: u8 = ((duty as u16 * 64) / 100 as u16) as u8;

        if (to_reg > 63) {
            to_reg = 63;
        }
        self.i2c.smbus_write_byte(EMC2101_REG_FAN_SETTING, to_reg)?;
        Ok(())
    }

    pub fn enable_force_temp(&mut self, force: bool) -> Result<(), Emc2101Error> {
        let mut data = self.i2c.smbus_read_byte(EMC2101_FAN_CONFIG)?;
        data = data.set_bit(force, 6);
        self.i2c.smbus_write_byte(EMC2101_FAN_CONFIG, data)?;
        Ok(())
    }

    pub fn init(&mut self) -> Result<(), Emc2101Error> {
        let id = self.i2c.smbus_read_byte(EMC2101_WHOAMI)?;
        if (id != 0x16 && id != 0x28) {
            return Err(Emc2101Error::InvalidDeviceId);
        }

        self.enable_tach(true)?;
        self.invert_fan_speed(false)?;
        self.set_pwm_frequency(0x1F)?;
        self.set_pwm_clock(true, false)?;
        self.enable_lut(false)?;
        self.set_duty_cycle(100)?;
        self.enable_force_temp(false)?;
        println!("after set fan");
        Ok(())
    }
}
