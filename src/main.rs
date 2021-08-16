use std::error::Error;
use std::thread;
use std::time::Duration;

use rppal::i2c::{I2c, Error as I2cError};

mod aht20;
mod emc2101;

use aht20::{Aht20, Aht20Error};
use emc2101::{Emc2101, Emc2101Error};

const ADDR_AHT20: u16 = 0x38;

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum BoardAdcData {
    mA(f32),
    mV(f32),
}

pub trait BoardAdc {
    fn to_humman(&self, ch: u8) -> Option<BoardAdcData>;
}

impl BoardAdc for u16 {
    fn to_humman(&self, ch: u8) -> Option<BoardAdcData> {
        let v = *self as f32 * 6.144 / 2048.0;
        match ch {
            0 => Some(BoardAdcData::mA(v /2.5 * 1000.0)),
            1 => Some(BoardAdcData::mA(v * 1000.0)),
            2 => Some(BoardAdcData::mV(v * 33.24 / 3.24)),
            3 => Some(BoardAdcData::mV(v * 2.0)),
            _ => None
        }
    }
}

#[cxx::bridge(namespace = "ruff::adc")]
mod ffi {
    unsafe extern "C++" {
        include!("ruff-hnt-rs/include/adc.h");

        type AdcClient;

        fn new_adc_client() -> UniquePtr<AdcClient>;
        fn read(&self, channel: u8) -> u16;
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let client = ffi::new_adc_client();
    let mut aht20 = Aht20::new(0, ADDR_AHT20)?;

    let mut emc2101 = Emc2101::new(0, 0x4C)?;
    aht20.init()?;
    emc2101.init()?;

    loop {
        aht20.get_sensor_data()?;
        //for ch in 0..4 {
        //    let data = client.read(ch);
        //    println!("channel {} data is {}", ch, data);
        //    println!("{:?}", data.to_humman(ch));
        //}
        thread::sleep(Duration::from_secs(1));
    }
}
