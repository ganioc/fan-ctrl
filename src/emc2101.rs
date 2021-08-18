use crate::{thread, Duration, Error, I2c, I2cError};
use crc8::Crc8;
use std::{error, fmt};
use std::result;

const EMC2101_REG_CONFIG: u8 = 0x3;
const EMC2101_TEMP_FORCE: u8 = 0x0C;
const EMC2101_WHOAMI: u8 = 0xFD;
const EMC2101_FAN_CONFIG: u8 = 0x4A;
const EMC2101_PWM_FREQ: u8 = 0x4D;
const EMC2101_REG_FAN_SETTING: u8 = 0x4C;
const EMC2101_TACH_LSB: u8 = 0x46;
const EMC2101_TACH_MSB: u8 = 0x47;
const EMC2101_TACH_LIMIT_LSB: u8 = 0x48;
const EMC2101_TACH_LIMIT_MSB: u8 = 0x49;
const EMC2101_LUT_START: u8 = 0x50;

const MAX_LUT_SPEED:u8 = 0x3F;
const EMC2101_FAN_RPM_NUMERATOR: u32 = 5400000;

pub struct Emc2101 {
    bus: u8,
    addr: u16,
    i2c: I2c,
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
            Emc2101Error::AhtI2c(ref err) => write!(f, "I2c Error {}", err),
            Emc2101Error::InvalidDeviceId => write!(f, "InvalidDeviceId"),
            // The wrapped error contains additional information and is available
            // via the source() method.
            Emc2101Error::UnkonwnStatus => write!(f, "UnkonwnStatus"),
        }
    }
}

impl error::Error for Emc2101Error {}

pub type Result<T> = result::Result<T, Emc2101Error>;

impl Emc2101 {
    pub fn new(bus: u8, addr: u16) -> Result<Emc2101> {
        let mut i2c = I2c::with_bus(bus)?;
        i2c.set_slave_address(addr)?;
        Ok(Emc2101 { bus, addr, i2c })
    }

    pub fn enable_tach(&mut self, enable: bool) -> Result<()> {
        let mut data = self.i2c.smbus_read_byte(EMC2101_REG_CONFIG)?;
        if (enable) {
            data = data | 1u8 << 2;
        } else {
            data = data & !(1u8 << 2);
        }
        println!("write data is => {:02X}", data);
        self.i2c.smbus_write_byte(EMC2101_REG_CONFIG, data)?;
        data = self.i2c.smbus_read_byte(EMC2101_REG_CONFIG)?;
        println!("read data is => {:02X}", data);
        Ok(())
    }

    pub fn invert_fan_speed(&mut self, invert: bool) -> Result<()> {
        let mut data = self.i2c.smbus_read_byte(EMC2101_FAN_CONFIG)?;
        if (invert) {
            data = data | 1u8 << 4;
        } else {
            data = data & !(1u8 << 4);
        }
        self.i2c.smbus_write_byte(EMC2101_FAN_CONFIG, data)?;
        Ok(())
    }

    pub fn set_pwm_frequency(&mut self, freq: u8) -> Result<()> {
        self.i2c.smbus_write_byte(EMC2101_PWM_FREQ, freq)?;
        Ok(())
    }

    pub fn set_pwm_clock(&mut self, clksel: bool, clkovr: bool) -> Result<()> {
        let mut data = self.i2c.smbus_read_byte(EMC2101_FAN_CONFIG)?;

        data = data.set_bit(clksel, 3);
        data = data.set_bit(clkovr, 2);
        self.i2c.smbus_write_byte(EMC2101_FAN_CONFIG, data)?;
        Ok(())
    }

    pub fn enable_program(&mut self, enable: bool) -> Result<()> {
        let mut data = self.i2c.smbus_read_byte(EMC2101_FAN_CONFIG)?;

        data = data.set_bit(enable, 5);
        self.i2c.smbus_write_byte(EMC2101_FAN_CONFIG, data)?;
        data = self.i2c.smbus_read_byte(EMC2101_FAN_CONFIG)?;
        Ok(())
    }

    pub fn set_duty_cycle(&mut self, duty: u8) -> Result<()> {
        let mut to_reg: u8 = ((duty as u16 * 64) / 100 as u16) as u8;

        if (to_reg > 63) {
            to_reg = 63;
        }
        self.i2c.smbus_write_byte(EMC2101_REG_FAN_SETTING, to_reg)?;
        Ok(())
    }

    pub fn set_lut(&mut self, index: u8, temp: u8, fan_pwm: u8) -> Result<()> {
        let fan_data = ((fan_pwm as u32 * MAX_LUT_SPEED as u32) as f32 / 100.0) as u8;
        let offset = EMC2101_LUT_START + index * 2;

        self.enable_program(true)?;
        self.i2c.smbus_write_byte(offset, temp)?;
        self.i2c.smbus_write_byte(offset+1, fan_data)?;
        println!("offset => {:02X} temp => {:02X} fan_data => {:02X}", offset, temp, fan_data);
        self.enable_program(false)?;
        Ok(())
    }

    pub fn set_min_rpm(&mut self, min_rpm: u16) -> Result<()> {
        let lsb_value: u16 = (EMC2101_FAN_RPM_NUMERATOR / min_rpm as u32) as u16;

        self.i2c.smbus_write_byte(EMC2101_TACH_LIMIT_LSB, (lsb_value & 0xFF) as u8)?;
        self.i2c.smbus_write_byte(EMC2101_TACH_LIMIT_MSB, ((lsb_value >> 8) & 0xFF) as u8)?;
        Ok(())
    }

    pub fn enable_force_temp(&mut self, force: bool) -> Result<()> {
        let mut data = self.i2c.smbus_read_byte(EMC2101_FAN_CONFIG)?;
        data = data.set_bit(force, 6);
        self.i2c.smbus_write_byte(EMC2101_FAN_CONFIG, data)?;
        Ok(())
    }

    pub fn get_fan_speed(&mut self) -> Result<u16> {
        let data_lsb = self.i2c.smbus_read_byte(EMC2101_TACH_LSB)?;
        let data_msb = self.i2c.smbus_read_byte(EMC2101_TACH_MSB)?;

        println!("msb=>  {:02X} lsb {:02X}", data_msb, data_lsb);
        if (data_lsb == 0xFF && data_msb == 0xFF) {
            return Ok((0));
        }
        let raw_data: u16 = (data_lsb as u16) | ((data_msb & 0x3F) as u16) << 8;
        return Ok((5400000 as u32 / raw_data as u32) as u16);
    }

    pub fn get_temp(&mut self) -> Result<u8> {
        let data = self.i2c.smbus_read_byte(EMC2101_TEMP_FORCE)?;
        Ok(data)
    }

    pub fn set_default_config(&mut self, fan_duty: u8)-> Result<()> {
        self.enable_tach(true)?;
        self.invert_fan_speed(false)?;
        self.set_pwm_frequency(0x1F)?;
        self.enable_force_temp(true)?;
        self.set_pwm_clock(false, false)?;
        self.enable_program(true)?;
        self.set_duty_cycle(fan_duty)?;
        self.set_min_rpm(150)?;
        Ok(())
    }

    pub fn init(&mut self) -> Result<()> {
        let id = self.i2c.smbus_read_byte(EMC2101_WHOAMI)?;
        if (id != 0x16 && id != 0x28) {
            return Err(Emc2101Error::InvalidDeviceId);
        }
        Ok(())
    }
}
