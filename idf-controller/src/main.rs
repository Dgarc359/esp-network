use esp_idf_svc::hal::ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver, LedcTimer};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::prelude::*;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

    let peripherals = Peripherals::take().unwrap();

    let timer_driver = LedcTimerDriver::new(peripherals.ledc.timer0, &TimerConfig::default().frequency(25.kHz().into())).unwrap();

    let mut driver = LedcDriver::new(peripherals.ledc.channel0, timer_driver, peripherals.pins.gpio9).unwrap();
    

    let max_duty = driver.get_max_duty();
    driver.set_duty(max_duty * 3 / 4);
}
