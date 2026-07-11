use crate::resources::I2cResources;
use crate::bellows;
use crate::keyboard;
use crate::mode::Mode;
use embassy_futures::yield_now;
use embassy_stm32::i2c::{Config, I2c, Master};
use embassy_stm32::{gpio, time::khz};
use embassy_sync::watch;
use embassy_time::{Duration, Timer};
use i2c_proto::ControllerIo;
use version::FirmwareVersion;

pub use embassy_stm32::i2c::Error;

const RETRY_COUNT: usize = 20;

const LEFT_KEYBOARD_ADDRESS: u8 = 0x22;
const RIGHT_KEYBOARD_ADDRESS: u8 = 0x23;
const BELLOWS_ADDRESS: u8 = 0x7F;

// TODO: Switch back to async i2c
pub struct Wrapper(I2c<'static, embassy_stm32::mode::Blocking, Master>);

impl ControllerIo for Wrapper {
    type Error = Error;

    async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        let mut result = Ok(());
        for _ in 0..RETRY_COUNT {
            result = self.0.blocking_read(address, read);
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
            result = self.0.blocking_write(address, write);
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
            result = self.0.blocking_write_read(address, write, read);
            if result.is_ok() {
                return result;
            }
            yield_now().await;
        }
        result
    }
}

pub struct Bus {
    controller: i2c_proto::Controller<Wrapper>,
    power: gpio::Output<'static>,
}

impl Bus {
    pub fn new(r: I2cResources) -> Self {
        let mut config = Config::default();
        config.frequency = khz(100);
        config.timeout = Duration::from_millis(25);
        let i2c = I2c::new_blocking(r.i2c, r.scl, r.sda, config);

        Bus {
            controller: i2c_proto::Controller::new(Wrapper(i2c)),
            power: gpio::Output::new(r.power, gpio::Level::High, gpio::Speed::Low),
        }
    }

    pub fn power_on(&mut self) {
        #[cfg(feature = "defmt")]
        defmt::info!("I2C power on");
        self.power.set_low();
    }

    pub fn power_off(&mut self) {
        #[cfg(feature = "defmt")]
        defmt::info!("I2C power off");
        self.power.set_high();
    }

    pub async fn reset(&mut self) {
        self.power_off();
        Timer::after_millis(1000).await;
        self.power_on();
        Timer::after_millis(1000).await;
    }

    pub async fn update_keyboards<'a>(&mut self, controller_version: FirmwareVersion<'a>, firmware: &[u8]) {
        // TODO: Don't split i2c across futures
        /*
        loop {
            let left_keyboard_update = keyboard::update_firmware(
                &mut self.controller,
                LEFT_KEYBOARD_ADDRESS,
                &controller_version,
                firmware,
            );
            let right_keyboard_update = keyboard::update_firmware(
                &mut self.controller,
                RIGHT_KEYBOARD_ADDRESS,
                &controller_version,
                firmware,
            );
            if let (Ok(_), Ok(_)) = join(left_keyboard_update, right_keyboard_update).await {
                break;
            }
            self.reset().await;
        }
        */
    }
}

#[embassy_executor::task]
pub async fn task(
    mut bus: Bus,
    mut mode: watch::DynReceiver<'static, Mode>,
    left_buttons: watch::DynSender<'static, u16>,
    right_buttons: watch::DynSender<'static, u16>,
    bellows: watch::DynSender<'static, bellows::State>,
) {
    loop {
        if mode.get().await != Mode::Ready {
            left_buttons.send(0);
            right_buttons.send(0);
            bellows.send(bellows::State::default());
            mode.get_and(|x| *x == Mode::Ready).await;
        }

        #[cfg(feature = "defmt")]
        defmt::info!("Left Keyboard");
        if let Ok(new_state) = bus.controller.get_buttons(LEFT_KEYBOARD_ADDRESS).await {
            left_buttons.send(new_state);
        } else {
            bus.reset().await;
        }

        #[cfg(feature = "defmt")]
        defmt::info!("Right Keyboard");
        if let Ok(new_state) = bus.controller.get_buttons(RIGHT_KEYBOARD_ADDRESS).await {
            right_buttons.send(new_state);
        } else {
            bus.reset().await;
        }

        #[cfg(feature = "defmt")]
        defmt::info!("Bellows");
        if let Ok(new_state) = bellows::State::read(&mut bus.controller.io, BELLOWS_ADDRESS).await {
            bellows.send(new_state);
        } else {
            bus.reset().await;
        }

        yield_now().await;
    }
}
