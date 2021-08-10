use std::error::Error;
use std::thread;
use std::time::Duration;

use rppal::i2c::{I2c, Error as I2cError};

mod aht20;

use aht20::{Aht20, Aht20Error};

const ADDR_AHT20: u16 = 0x38;

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

    aht20.init()?;

    loop {
        aht20.get_sensor_data()?;
        for ch in 0..4 {
            let data = client.read(ch);
            println!("channel {} data is {}", ch, data);
        }
        thread::sleep(Duration::from_secs(1));
    }
}
