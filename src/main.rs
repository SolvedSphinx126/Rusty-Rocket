#![no_std]
#![no_main]

mod fmt;

use defmt::unwrap;
#[cfg(not(feature = "defmt"))]
use panic_halt as _;
#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_stm32::gpio::{AnyPin, Level, Output, Pin, Speed};
use embassy_time::{Duration, Timer};
use fmt::info;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let led = p.PB7.degrade();
    unwrap!(spawner.spawn(blinker(led, Duration::from_millis(100))));
}

#[embassy_executor::task]
async fn blinker(led: AnyPin, interval: Duration) {
    let mut led = Output::new(led, Level::Low, Speed::Low);
    info!("Blink task start");
    loop {
        led.set_high();
        Timer::after(interval).await;
        led.set_low();
        Timer::after(interval).await;
    }
}
