
#![no_std]
#![no_main]

extern crate alloc;
use core::mem::MaybeUninit;
use esp_backtrace as _;
use esp_hal::{clock::ClockControl, peripherals::Peripherals, prelude::*, Delay, IO, ledc::{timer::config::{Config as TimerConfig},LSGlobalClkSource, channel::{self, ChannelIFace}, LEDC, timer::{self, TimerIFace}, Speed, LowSpeed}};
use esp_println::println;
use esp_hal as hal;

use esp_wifi::{current_millis, initialize, EspWifiInitFor, esp_now::{BROADCAST_ADDRESS, PeerInfo}};

use esp_hal::Rng;
mod util;

const TOGGLE_LED: u8 = 1 << 0;

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

#[entry]
fn main() -> ! {
    #[cfg(feature = "log")]
    esp_println::logger::init_logger(log::LevelFilter::Info);

    let peripherals = Peripherals::take();

    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::max(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    #[cfg(target_arch = "xtensa")]
    let timer = hal::timer::TimerGroup::new(peripherals.TIMG1, &clocks).timer0;
    #[cfg(target_arch = "riscv32")]
    let timer = hal::systimer::SystemTimer::new(peripherals.SYSTIMER).alarm0;
    let init = initialize(
        EspWifiInitFor::Wifi,
        timer,
        Rng::new(peripherals.RNG),
        system.radio_clock_control,
        &clocks,
    )
    .unwrap();


    let wifi = peripherals.WIFI;
    let mut esp_now = esp_wifi::esp_now::EspNow::new(&init, wifi).unwrap();
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = io.pins.gpio0.into_push_pull_output();
    //let mut motor = io.pins.gpio9;

    let mut ledc = LEDC::new(peripherals.LEDC, &clocks);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);


    let mut lstimer0 = ledc.get_timer::<LowSpeed>(timer::Number::Timer0);
lstimer0
    .configure(timer::config::Config {
        duty: timer::config::Duty::Duty5Bit,
        clock_source: timer::LSClockSource::APBClk,
        frequency: esp_hal::prelude::_fugit_RateExtU32::kHz(24),
    })
    .unwrap();

    let mut channel0 = ledc.get_channel(channel::Number::Channel0, led);
channel0
    .configure(channel::config::Config {
        timer: &lstimer0,
        duty_pct: 75,
        pin_config: channel::config::PinConfig::PushPull,
    })
    .unwrap();

    //let mut motor = io.pins.gpio9.into_push_pull_output();

    //led.set_high();


    println!("esp-now version {}", esp_now.get_version().unwrap());
    //println!("Max duty: {}", channel0.max_duty_cycle());

    let mut next_send_time = current_millis() + 5 * 1000;
    loop {
        channel0.set_duty(50).ok();
        //delay.delay_ms(250u32);
        //channel0.set_duty(50).ok();
        //delay.delay_ms(250u32);
        //channel0.set_duty(75).ok();
        //delay.delay_ms(250u32);
        let r = esp_now.receive();
        if let Some(r) = r {

            let gotten_bit = r.data[0] & TOGGLE_LED;
            println!("got bit {:?} {:?}", &gotten_bit, TOGGLE_LED);
            if gotten_bit == TOGGLE_LED {
                //led.set_high();
                //motor.set_high();
            } else {
                //led.set_low();
                //motor.set_low();
            }

            if r.info.dst_address == BROADCAST_ADDRESS {
                if !esp_now.peer_exists(&r.info.src_address) {
                    esp_now
                        .add_peer(PeerInfo {
                            peer_address: r.info.src_address,
                            lmk: None,
                            channel: None,
                            encrypt: false,
                        })
                        .unwrap();
                }
                /*
                let status = esp_now
                    .send(&r.info.src_address, b"Hello Peer")
                    .unwrap()
                    .wait();
                println!("Send hello to peer status: {:?}", status);
                */
            }
        }

        if current_millis() >= next_send_time {
            next_send_time = current_millis() + 5 * 1000;
            println!("Send");
            //println!("toggling led");
            //led.toggle();
            /*
            let status = esp_now
                .send(&BROADCAST_ADDRESS, b"0123456789")
                .unwrap()
                .wait();

            println!("Send broadcast status: {:?}", status)
            */
        }
    }
}
