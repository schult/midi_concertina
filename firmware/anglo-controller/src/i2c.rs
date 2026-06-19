use crate::resources::I2cResources;
use crate::bellows;
use crate::mode::Mode;
use core::ops::DerefMut;
use embassy_futures::yield_now;
use embassy_stm32::i2c::{Config, I2c, Master};
use embassy_stm32::time::khz;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::{Mutex, MutexGuard};
use embassy_sync::watch;
use embassy_time::Duration;
use i2c_proto::ControllerIo;
use static_cell::StaticCell;
use version::FirmwareVersion;

pub use embassy_stm32::i2c::Error;

const LEFT_KEYBOARD_ADDRESS: u8 = 0x22;
const RIGHT_KEYBOARD_ADDRESS: u8 = 0x23;
const BELLOWS_ADDRESS: u8 = 0x7F;

pub type I2cMutex = Mutex<NoopRawMutex, I2c<'static, embassy_stm32::mode::Blocking, Master>>;
pub type I2cMutexGuard<'a> =
    MutexGuard<'a, NoopRawMutex, I2c<'static, embassy_stm32::mode::Blocking, Master>>;

// TODO:
// - Make this a Wrapper constructor instead
// - Move firmware update to a new Wrapper method.
// - Add power control to Wrapper
// - Switch back to async i2c
pub fn init(r: I2cResources, controller_version: FirmwareVersion) -> &'static mut I2cMutex {
    let mut config = Config::default();
    config.frequency = khz(100);
    config.timeout = Duration::from_millis(25);

    static I2C: StaticCell<I2cMutex> = StaticCell::new();
    let i2c = I2C.init(Mutex::new(I2c::new_blocking(
        r.i2c, r.scl, r.sda, config,
    )));

    // TODO: Don't split i2c across futures
    /*
    loop {
        #[cfg(feature = "defmt")]
        info!("I2C power on");
        i2c_power.set_low();

        const KEYBOARD_FIRMWARE: &[u8] = include_bytes!("../../build/anglo-keyboard.bin");
        let left_keyboard_update = keyboard::update_firmware(
            i2c,
            LEFT_KEYBOARD_ADDRESS,
            &controller_version,
            KEYBOARD_FIRMWARE,
        );
        let right_keyboard_update = keyboard::update_firmware(
            i2c,
            RIGHT_KEYBOARD_ADDRESS,
            &controller_version,
            KEYBOARD_FIRMWARE,
        );
        if let (Ok(_), Ok(_)) = join(left_keyboard_update, right_keyboard_update).await {
            break;
        }

        #[cfg(feature = "defmt")]
        info!("I2C power off");
        i2c_power.set_high();
        Timer::after_millis(100).await;
    }
    */

    i2c
}

#[embassy_executor::task(pool_size = 2)]
pub async fn task(
    i2c_mutex: &'static I2cMutex,
    mut mode: watch::DynReceiver<'static, Mode>,
    left_buttons: watch::DynSender<'static, u16>,
    right_buttons: watch::DynSender<'static, u16>,
    bellows: watch::DynSender<'static, bellows::State>,
) {
    let i2c_wrapper = Wrapper::new(i2c_mutex);
    let mut i2c_controller = i2c_proto::Controller::new(i2c_wrapper);

    loop {
        if mode.get().await != Mode::Ready {
            left_buttons.send(0);
            right_buttons.send(0);
            bellows.send(bellows::State::default());
            mode.get_and(|x| *x == Mode::Ready).await;
        }

        if let Ok(new_state) = i2c_controller.get_buttons(LEFT_KEYBOARD_ADDRESS).await {
            left_buttons.send(new_state);
        } else {
            // TODO: power cycle
        }

        if let Ok(new_state) = i2c_controller.get_buttons(RIGHT_KEYBOARD_ADDRESS).await {
            right_buttons.send(new_state);
        } else {
            // TODO: power cycle
        }

        if let Ok(new_state) = bellows::State::read(&mut i2c_controller.io, BELLOWS_ADDRESS).await {
            bellows.send(new_state);
        } else {
            // TODO: power cycle
        }

        yield_now().await;
    }
}

const RETRY_COUNT: usize = 20;

pub struct Wrapper<'a> {
    i2c: &'a I2cMutex,
}

impl<'a> Wrapper<'a> {
    pub fn new(i2c: &'a I2cMutex) -> Self {
        Wrapper { i2c }
    }
}

impl<'a> ControllerIo for Wrapper<'a> {
    type Error = Error;

    async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        let mut result = Ok(());
        for _ in 0..RETRY_COUNT {
            let mut guard: I2cMutexGuard<'_> = self.i2c.lock().await;
            let i2c = guard.deref_mut();
            result = i2c.blocking_read(address, read);
            drop(guard);
            if result.is_ok() {
                return result;
            }
            yield_now().await;
        }
        result
    }

    async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        let mut result = Ok(());
        for _ in 0..RETRY_COUNT {
            let mut guard: I2cMutexGuard<'_> = self.i2c.lock().await;
            let i2c = guard.deref_mut();
            result = i2c.blocking_write(address, write);
            drop(guard);
            if result.is_ok() {
                return result;
            }
            yield_now().await;
        }
        result
    }

    async fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        let mut result = Ok(());
        for _ in 0..RETRY_COUNT {
            let mut guard: I2cMutexGuard<'_> = self.i2c.lock().await;
            let i2c = guard.deref_mut();
            result = i2c.blocking_write_read(address, write, read);
            drop(guard);
            if result.is_ok() {
                return result;
            }
            yield_now().await;
        }
        result
    }
}
