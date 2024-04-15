#![no_std]
#![no_main]

extern crate alloc;
use core::mem::MaybeUninit;
use esp_backtrace as _;
use esp_hal::{clock::ClockControl, peripherals::Peripherals, prelude::*, Delay, IO};
use esp_println::println;
use esp_hal as hal;

use esp_wifi::{current_millis, initialize, EspWifiInitFor, esp_now::{BROADCAST_ADDRESS, PeerInfo}};

use esp_hal::Rng;
mod util;

const TOGGLE_LED: u8 = 1 << 0;


#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

const SEND_DELAY: u64 = 300;

#[entry]
fn main() -> ! {
    #[cfg(feature = "log")]
    esp_println::logger::init_logger(log::LevelFilter::Info);

    let peripherals = Peripherals::take();

    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::max(system.clock_control).freeze();

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
    let input_pin = io.pins.gpio1.into_pull_down_input();
    let mut led = io.pins.gpio0.into_push_pull_output();

    led.set_high();


    println!("esp-now version {}", esp_now.get_version().unwrap());

    let mut next_send_time = current_millis() + SEND_DELAY;
    let mut data:[u8; 1] = [0u8; 1];
    println!("data: {:?}", &data);

    loop {
        if input_pin.is_input_high() {
            data[0] = data[0] | TOGGLE_LED;
            //println!("input is high, adding TOGGLE_LED to data {:?}", &data);
        } else {
            data[0] = (data[0] >> TOGGLE_LED) & 1;
            //println!("input is low, removing TOGGLE_LED from data {:?}", &data);
        }
        let r = esp_now.receive();
        if let Some(r) = r {
            println!("Received {:?}", r);

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
                let status = esp_now
                    .send(&r.info.src_address, b"Hello Peer")
                    .unwrap()
                    .wait();
                println!("Send hello to peer status: {:?}", status);
            }
        }

        if current_millis() >= next_send_time {
            next_send_time = current_millis() + SEND_DELAY;
            println!("Send {:?}", &data);
            println!("toggling led");
            led.toggle();
            let status = esp_now
                .send(&BROADCAST_ADDRESS, &data)
                .unwrap()
                .wait();
            println!("Send broadcast status: {:?}", status)
        }
    }
}
