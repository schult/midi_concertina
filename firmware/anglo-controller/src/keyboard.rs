#[cfg(feature = "defmt")]
use defmt::panic;

use crate::i2c;
use crate::i2c_transfer; // TODO: Incorporate into this module
use embassy_futures::yield_now;
use embassy_sync::watch;
use embassy_time::Timer;
use version::FirmwareVersion;

pub async fn update_firmware<'a>(
    i2c_mutex: &'a i2c::I2cMutex,
    address: u8,
    version: &FirmwareVersion<'a>,
    firmware: &[u8],
) {
    let i2c_wrapper = i2c::Wrapper::new(i2c_mutex);
    let mut i2c_controller = i2c_proto::Controller::new(i2c_wrapper);

    let mut session = i2c_transfer::Session::new(address);

    // Retry with a short delay in case keyboard is still starting up.
    const RETRY_COUNT: usize = 3;
    for _ in 0..RETRY_COUNT {
        if let Ok(keyboard_version) = i2c_controller.get_version(address).await {
            #[cfg(feature = "defmt")]
            defmt::info!("Keyboard({}) version: {}", address, keyboard_version);

            if keyboard_version != *version || keyboard_version.prerelease.is_some() {
                session.begin(firmware);
            }
            break;
        }

        Timer::after_millis(10).await;
    }

    loop {
        match session.poll(&mut i2c_controller).await {
            Ok(i2c_transfer::Status::InProgress) => (),
            Err(_) => panic!("I2C communication error"),
            _ => break,
        }
        yield_now().await;
    }
}

#[embassy_executor::task(pool_size = 2)]
pub async fn task(
    i2c_mutex: &'static i2c::I2cMutex,
    address: u8,
    buttons: watch::DynSender<'static, u16>,
) {
    let i2c_wrapper = i2c::Wrapper::new(i2c_mutex);
    let mut i2c_controller = i2c_proto::Controller::new(i2c_wrapper);

    loop {
        if let Ok(new_state) = i2c_controller.get_buttons(address).await {
            buttons.send(new_state);
        }
        yield_now().await;
    }
}
