use std::thread;
use std::time::Duration;
use std::fs::File;
use std::io::prelude::*;
use std::result::Result;
use easy_error::{bail, ensure, Error, ResultExt, Terminator};


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

fn get_cpu_temp() -> Result<f32, Error> {
    let mut file = File::open("/sys/class/thermal/thermal_zone0/temp").context("fail to open file")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).context("fail to read")?;
    println!("contents is {}", contents);
    let temp = contents.trim_end().parse::<u32>().context("fail to parse")?;
    return Ok(temp as f32 / 1000.0);
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ffi::new_adc_client();
    let mut aht20 = Aht20::new(0, ADDR_AHT20)?;
    let mut fan_duty:u8 = 0;

    let mut emc2101 = Emc2101::new(0, 0x4C)?;
    aht20.init()?;
    emc2101.init()?;
    emc2101.set_default_config(fan_duty)?;

    //emc2101.set_lut(0, 10, 30)?;
    //emc2101.enable_program(false)?;
    //emc2101.enable_force_temp(true)?;
    //emc2101.set_lut(1, 20, 50)?;
    //emc2101.set_lut(2, 50, 90)?;
    loop {
        let mut new_fan_duty = fan_duty;
        aht20.get_sensor_data()?;
//        if let Ok(speed) = emc2101.get_fan_speed() {
//            println!("speed => {}", speed);
//        }
//        println!("temp in fan is {:?}", emc2101.get_temp());
        if let Ok((humi, temp)) = aht20.get_sensor_data() {
            println!("temp in aht20 is {} ", temp);
            println!("humi in aht20 is {} ", humi);
        }
        if let Ok(cpu_temp) = get_cpu_temp() {
            println!("cpu temp is {}", get_cpu_temp().unwrap());
            if (cpu_temp > 60.0) {
                new_fan_duty = 100;
            } else if (cpu_temp > 40.0 ) {
                new_fan_duty += 10;
            } else {
                new_fan_duty = 0;
            }
            if (new_fan_duty > 100) {
                new_fan_duty = 100;
            }
            if (new_fan_duty != fan_duty) {
                if let Ok(()) = emc2101.set_duty_cycle(new_fan_duty) {
                    println!("set new_fan_duty {}", new_fan_duty);
                    fan_duty = new_fan_duty;
                } else {
                    println!("fail to set duty cycel {}", new_fan_duty);
                }

            }
        } else {
            println!("Fail to get cpu_temp");
        }
        //for ch in 0..4 {
        //    let data = client.read(ch);
        //    println!("channel {} data is {}", ch, data);
        //    println!("{:?}", data.to_humman(ch));
        //}
        thread::sleep(Duration::from_secs(5));
    }
}
