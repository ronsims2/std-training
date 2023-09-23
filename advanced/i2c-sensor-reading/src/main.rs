use anyhow::Result;
use embedded_hal::blocking::delay::DelayMs;
use esp_idf_hal::{
    delay,
    delay::FreeRtos,
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    prelude::*,
};
use shtcx::{self, shtc3, PowerMode};
// If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_idf_sys as _;
use esp_idf_svc as _;
use log::{error, info, warn};
use icm42670::{Address, Icm42670, PowerMode as imuPowerMode};
use shared_bus::BusManagerSimple;

// Goals of this exercise:
// - Part1: Instantiate i2c peripheral
// - Part1: Implement one sensor, print sensor values
// - Part2: Implement second sensor on same bus to solve an ownership problem

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let pins = peripherals.pins;
    let sda = pins.gpio10;
    let scl = pins.gpio8;
    let config = I2cConfig::new().baudrate(400.kHz().into());
    let i2c = peripherals.i2c0;
    let i2c = I2cDriver::new(i2c, sda, scl, &config)?;
    // Need this since 2 sensors can't own the i2c
    let bus = shared_bus::BusManagerSimple::new(i2c);
    let mut proxy_1 = bus.acquire_i2c();
    let mut proxy_2 = bus.acquire_i2c();

    let mut temp_sensor = shtc3(proxy_1);
    let mut delay = delay::Ets;

    let mut icm = Icm42670::new(proxy_2, Address::Primary).unwrap();
    icm.set_power_mode(imuPowerMode::GyroLowNoise);


    // 4. Read and print the sensor's device ID.

    loop {
        // 5. This loop initiates measurements, reads values and prints humidity in % and Temperature in °C.
        let temp_sensor_id = temp_sensor.raw_id_register().unwrap();
        info!("Temp sensor ID: {}", temp_sensor_id);
        let temp = temp_sensor
            .measure_temperature(PowerMode::NormalMode, &mut delay)
            .unwrap()
            .as_degrees_celsius();

        info!("Board temp: {}°C", temp);

        let humidity = temp_sensor
            .measure_humidity(PowerMode::NormalMode, &mut delay)
            .unwrap()
            .as_percent();

        info!("Ambient humidity: {:.1}%", humidity);

        let gyro_data = icm.gyro_raw().unwrap();
        info!("Gyroscope data: {:?}", gyro_data);

        FreeRtos.delay_ms(500u32);
    }
}
