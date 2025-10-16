#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embassy_stm32::gpio;
use panic_halt as _;

#[entry]
fn main() -> ! {
    let config = embassy_stm32::Config::default();
    let dp = embassy_stm32::init(config);

    let mut led = gpio::Output::new(dp.PB4, gpio::Level::High, gpio::Speed::Low);
    loop {
        cortex_m::asm::delay(3_000_000);
        led.toggle();
    }
}
