#![no_std]
#![no_main]

use core::cell::RefCell;

use cortex_m_rt::entry;
use embassy_boot_stm32::{BootLoader, BootLoaderConfig};
use embassy_stm32::flash::{self, Flash};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::{gpio, rcc, time::hz};
use embassy_sync::blocking_mutex::Mutex;
use panic_reset as _;

#[entry]
fn main() -> ! {
    let mut config = embassy_stm32::Config::default();
    config.rcc.hsi = true;
    config.rcc.sys = rcc::Sysclk::HSI;
    let p = embassy_stm32::init(config);

    let led_pin = PwmPin::new(p.PB4, gpio::OutputType::PushPull);
    let mut led_pwm = SimplePwm::new(
        p.TIM3,
        Some(led_pin),
        None,
        None,
        None,
        hz(1),
        Default::default(),
    );
    let mut led = led_pwm.ch1();
    led.set_duty_cycle_percent(10);
    led.enable();

    let flash = Flash::new_blocking(p.FLASH);
    let flash = Mutex::new(RefCell::new(flash));

    let config = BootLoaderConfig::from_linkerfile_blocking(&flash, &flash, &flash);
    let boot_address = flash::BANK1_REGION.base() + config.active.offset();
    const PAGE_SIZE: usize = flash::MAX_ERASE_SIZE;
    let bl = BootLoader::prepare::<_, _, _, PAGE_SIZE>(config);

    drop(led);
    drop(led_pwm);

    unsafe { bl.load(boot_address) }
}
