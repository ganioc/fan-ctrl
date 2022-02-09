extern crate serde_json;

#[macro_use]
use serde::Serialize;
use easy_error::{Error, ResultExt};
use std::default::Default;
use std::fs::File;
use std::io::prelude::*;
use std::result::Result;
use std::thread;
use std::time::Duration;
use sysfs_gpio::{Direction, Pin};

use clap::Parser;

use rppal::i2c::{Error as I2cError, I2c};

mod aht20;
mod emc2101;

use aht20::Aht20;
use emc2101::Emc2101;

const ADDR_AHT20: u16 = 0x38;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Name of the person to greet
    #[clap(short, long)]
    get_board_sensor_data: bool,

    #[clap(short, long)]
    power_on_adc: Option<bool>,

    /// Number of times to greet
    #[clap(long, default_value_t = 16)]
    power_pin: u8,

    #[clap(long)]
    deamon: bool,

    #[clap(long)]
    enable_adc: bool,
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum BoardAdcData {
    mA(f32),
    mV(f32),
}

pub trait BoardAdc {
    fn to_humman(&self, ch: u8) -> Option<BoardAdcData>;
    fn to_data(&self, ch: u8) -> f32;
}

#[derive(Serialize, Default)]
struct BoardSensorData {
    temperatue: f32,
    humid: f32,
    current_0: f32,
    current_1: f32,
    voltage_0: f32,
    voltage_1: f32,
}

impl BoardAdc for u16 {
    fn to_humman(&self, ch: u8) -> Option<BoardAdcData> {
        let v = *self as f32 * 6.144 / 2048.0;
        match ch {
            0 => Some(BoardAdcData::mA(v / 2.5 * 1000.0)),
            1 => Some(BoardAdcData::mA(v * 1000.0)),
            2 => Some(BoardAdcData::mV(v * 33.24 / 3.24)),
            3 => Some(BoardAdcData::mV(v * 2.0)),
            _ => None,
        }
    }

    fn to_data(&self, ch: u8) -> f32 {
        let v = *self as f32 * 6.144 / 2048.0;
        match ch {
            0 => v / 2.5 * 1000.0,
            1 => v * 1000.0,
            2 => v * 33.24 / 3.24,
            3 => v * 2.0,
            _ => panic!("Invalid {}", ch),
        }
    }
}

fn get_cpu_temp() -> Result<f32, Error> {
    let mut file =
        File::open("/sys/class/thermal/thermal_zone0/temp").context("fail to open file")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).context("fail to read")?;
    let temp = contents
        .trim_end()
        .parse::<u32>()
        .context("fail to parse")?;
    return Ok(temp as f32 / 1000.0);
}

fn show_board_sensor_data(aht20: &mut Aht20) {
    let mut report_data = BoardSensorData {
        ..Default::default()
    };
    let adc_reader = ffi::new_adc_client();

    let (humid, temperatue) = aht20.get_sensor_data().unwrap();

    report_data.temperatue = temperatue;
    report_data.humid = humid;
    report_data.current_0 = adc_reader.read(0).to_data(0);
    report_data.current_1 = adc_reader.read(1).to_data(1);
    report_data.voltage_0 = adc_reader.read(2).to_data(2);
    report_data.voltage_1 = adc_reader.read(3).to_data(3);

    println!("{}", serde_json::to_string(&report_data).unwrap());
}

fn power_adc(pin: u8, is_on: bool) {
    let adc_power = Pin::new(pin.into()); // number depends on chip, etc.
    println!("power in {pin} is_on {is_on}");
    adc_power
        .with_exported(|| {
            adc_power.set_direction(Direction::Out).unwrap();
            thread::sleep(Duration::from_millis(200));
            if is_on {
                adc_power.set_value(1).unwrap();
            } else {
                adc_power.set_value(0).unwrap();
            }
            Ok(())
        })
        .unwrap();
}

#[cxx::bridge(namespace = "ruff::adc")]
mod ffi {
    unsafe extern "C++" {
        include!("dev-monitor/include/adc.h");

        type AdcClient;

        fn new_adc_client() -> UniquePtr<AdcClient>;
        fn read(&self, channel: u8) -> u16;
    }
}

fn run_fan_daemon(
    aht20: &mut Aht20,
    emc2101: &mut Emc2101,
    adc_client: Option<cxx::UniquePtr<ffi::AdcClient>>,
) {
    let mut fan_duty = 30;
    loop {
        let mut new_fan_duty = fan_duty;
        if let Ok(speed) = emc2101.get_fan_speed() {
            println!("speed => {}", speed);
        }
        println!("temp in fan is {:?}", emc2101.get_temp());
        if let Ok((humi, temp)) = aht20.get_sensor_data() {
            println!("temp in aht20 is {} ", temp);
            println!("humi in aht20 is {} ", humi);
        }
        if let Ok(cpu_temp) = get_cpu_temp() {
            println!("cpu temp is {}", get_cpu_temp().unwrap());
            if cpu_temp > 80.0 {
                new_fan_duty = 100;
            } else if cpu_temp > 60.0 {
                new_fan_duty = 30;
            } else {
                new_fan_duty = 0;
            }
            if new_fan_duty >= 100 {
                new_fan_duty = 50;
            }
            if new_fan_duty != fan_duty {
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

        if let Some(ref client) = adc_client {
            for ch in 0..4 {
                let data = client.read(ch);
                println!("channel {} data is {}", ch, data);
                println!("{:?}", data.to_humman(ch));
            }
        }
        thread::sleep(Duration::from_secs(60));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let mut aht20 = Aht20::new(0, ADDR_AHT20)?;
    aht20.init()?;

    if let Some(power_on_adc) = cli.power_on_adc {
        power_adc(cli.power_pin, power_on_adc);
        return Ok(());
    }

    if cli.get_board_sensor_data {
        show_board_sensor_data(&mut aht20);
        return Ok(());
    }

    let adc_client = {
        if cli.enable_adc {
            Some(ffi::new_adc_client())
        } else {
            None
        }
    };

    if cli.deamon {
        let mut emc2101 = Emc2101::new(0, 0x4C)?;
        emc2101.init()?;
        run_fan_daemon(&mut aht20, &mut emc2101, adc_client);
    }
    return Ok(());
}
