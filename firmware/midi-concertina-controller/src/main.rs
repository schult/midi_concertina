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
    let switch = gpio::Input::new(dp.PB5, gpio::Pull::Up);
    loop {
        led.set_level(
            match switch.get_level() {
                gpio::Level::Low => gpio::Level::High,
                gpio::Level::High => gpio::Level::Low,
            }
        );
    }
}
