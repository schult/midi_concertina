use crate::resources::I2cResources;
use core::ops::DerefMut;
use embassy_stm32::i2c::{Config, I2c, Master};
use embassy_stm32::time::khz;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::{Mutex, MutexGuard};
use embassy_time::Duration;
use i2c_proto::ControllerIo;
use static_cell::StaticCell;

pub use embassy_stm32::i2c::Error;

pub type I2cMutex = Mutex<NoopRawMutex, I2c<'static, embassy_stm32::mode::Blocking, Master>>;
pub type I2cMutexGuard<'a> =
    MutexGuard<'a, NoopRawMutex, I2c<'static, embassy_stm32::mode::Blocking, Master>>;

pub fn init(r: I2cResources) -> &'static mut I2cMutex {
    let mut config = Config::default();
    config.frequency = khz(100);
    config.timeout = Duration::from_millis(500);

    static I2C: StaticCell<I2cMutex> = StaticCell::new();
    I2C.init(Mutex::new(I2c::new_blocking(
        r.i2c, r.scl, r.sda, config,
    )))
}

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
        let mut guard: I2cMutexGuard<'_> = self.i2c.lock().await;
        let i2c = guard.deref_mut();
        i2c.blocking_read(address, read)
    }

    async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        let mut guard: I2cMutexGuard<'_> = self.i2c.lock().await;
        let i2c = guard.deref_mut();
        i2c.blocking_write(address, write)
    }

    async fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        let mut guard: I2cMutexGuard<'_> = self.i2c.lock().await;
        let i2c = guard.deref_mut();
        i2c.blocking_write_read(address, write, read)
    }
}
