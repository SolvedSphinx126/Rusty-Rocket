#![no_std]
#![no_main]

mod fmt;

use defmt::unwrap;
#[cfg(not(feature = "defmt"))]
use panic_halt as _;
#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_stm32::{bind_interrupts, dma::NoDma, gpio::{AnyPin, Level, Output, Pin, Speed}, i2c::{self, SclPin}, interrupt::{self, Interrupt}, peripherals, time::hz};
use embassy_stm32::i2c::I2c;
use embassy_time::{Delay, Duration, Instant, Timer};
use fmt::info;

use bme280::i2c::BME280;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    // LED Pin
    let led = p.PB7.degrade();

    // I2C Pins
    let scl = p.PA11;
    let sda = p.PA12;

    bind_interrupts!(struct Irqs {
        I2C2 => i2c::EventInterruptHandler<peripherals::I2C2>, i2c::ErrorInterruptHandler<peripherals::I2C2>;
    });

    let i2c: I2c<'static, peripherals::I2C2> = I2c::new(
        p.I2C2,
        scl,
        sda,
        Irqs,
        NoDma,
        NoDma,
        hz(100000),
        Default::default(),
    );

    let bme280: BME280<I2c<'static, peripherals::I2C2>> = BME280::new_primary(i2c);

    unwrap!(spawner.spawn(blinker(led, Duration::from_millis(100))));
    unwrap!(spawner.spawn(temp_reader(bme280)));
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

#[embassy_executor::task]
async fn temp_reader(mut bme280: BME280<I2c<'static, peripherals::I2C2>>) {
    let mut delay = Delay;
    bme280.init(&mut delay).unwrap();
    let mut time = Instant::now();
    loop {
        Timer::at(time + Duration::from_millis(100)).await;
        let measurements = bme280.measure(&mut delay).unwrap();
        info!("Temperature is {} at time {}", measurements.temperature, Instant::now().as_millis());
        time += Duration::from_millis(100);
    }
}

