#![no_std]
#![no_main]

use core::cell::RefCell;

use cortex_m_rt::entry;
use embassy_boot_stm32::{
    BootLoader,
    BootLoaderConfig,
};
use embassy_stm32::{
    flash,
    rcc,
};
use embassy_sync::blocking_mutex::Mutex;
use panic_reset as _;

#[entry]
fn main() -> ! {
    let mut config = embassy_stm32::Config::default();
    config.rcc.hsi = true;
    config.rcc.sys = rcc::Sysclk::HSI;
    let p = embassy_stm32::init(config);

    let flash_layout = flash::Flash::new_blocking(p.FLASH).into_blocking_regions();
    let flash = Mutex::new(RefCell::new(flash_layout.bank1_region));

    let config = BootLoaderConfig::from_linkerfile_blocking(&flash, &flash, &flash);
    let active_offset = config.active.offset();
    let bl = BootLoader::prepare::<_, _, _, 2048>(config);

    unsafe { bl.load(flash::BANK1_REGION.base + active_offset) }
}
