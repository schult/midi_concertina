#[cfg(feature = "defmt")]
use defmt::{error, info, panic};

use crate::i2c;
use core::slice::Chunks;
use embassy_futures::yield_now;
use embassy_sync::watch;
use embassy_time::Timer;
use version::FirmwareVersion;

async fn get_keyboard_version<'a>(
    i2c_controller: &mut i2c_proto::Controller<i2c::Wrapper<'a>>,
    address: u8,
) -> Result<FirmwareVersion<'static>, ()> {
    // Retry count and delay chosen to allow > 2 minutes for keyboard to finish applying update.
    const RETRY_COUNT: usize = 1200;
    for _ in 0..RETRY_COUNT {
        if let Ok(keyboard_version) = i2c_controller.get_version(address).await {
            return Ok(keyboard_version);
        }

        Timer::after_millis(100).await;
    }
    Err(())
}

pub async fn update_firmware<'a>(
    i2c_mutex: &'a i2c::I2cMutex,
    address: u8,
    version: &FirmwareVersion<'a>,
    firmware: &[u8],
) {
    let i2c_wrapper = i2c::Wrapper::new(i2c_mutex);
    let mut i2c_controller = i2c_proto::Controller::new(i2c_wrapper);

    let mut session = TransferSession::new(address);

    if let Ok(keyboard_version) = get_keyboard_version(&mut i2c_controller, address).await {
        #[cfg(feature = "defmt")]
        info!("Keyboard({:02X}) version: {}", address, keyboard_version);
        if keyboard_version != *version || keyboard_version.prerelease.is_some() {
            #[cfg(feature = "defmt")]
            info!("Keyboard({:02X}) receiving update...", address);
            session.begin(firmware);
            loop {
                match session.poll(&mut i2c_controller).await {
                    Ok(TransferStatus::InProgress) => (),
                    Err(_) => panic!("I2C communication error"),
                    _ => break,
                }
                yield_now().await;
            }
        }
    } else {
        #[cfg(feature = "defmt")]
        error!("Keyboard({:02X}) not responding at startup", address);
        return;
    }

    #[cfg(feature = "defmt")]
    info!("Keyboard({:02X}) applying update...", address);
    let _new_version = get_keyboard_version(&mut i2c_controller, address).await;
    #[cfg(feature = "defmt")]
    if let Ok(keyboard_version) = _new_version {
        info!("Keyboard({:02X}) update complete", address);
        info!("Keyboard({:02X}) version: {}", address, keyboard_version);
    } else {
        error!("Keyboard({:02X}) not responding after update", address);
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
        // TODO: Send zero and pause if MODE != Mode::Ready
        if let Ok(new_state) = i2c_controller.get_buttons(address).await {
            buttons.send(new_state);
        }
        yield_now().await;
    }
}

#[derive(Clone, PartialEq)]
enum TransferStatus {
    InProgress,
    Finished,
    Failed,
}

struct TransferSession<'a> {
    address: u8,
    beginning: Option<Chunks<'a, u8>>,
    iter: Option<Chunks<'a, u8>>,
    retries_remaining: u8,
    status: TransferStatus,
}

impl<'a> TransferSession<'a> {
    fn new(address: u8) -> Self {
        Self {
            address,
            beginning: None,
            iter: None,
            retries_remaining: 0,
            status: TransferStatus::Finished,
        }
    }

    fn begin(&mut self, data: &'a [u8]) {
        self.beginning = Some(data.chunks(i2c_proto::PACKET_MAX_PAYLOAD_SIZE));
        self.retries_remaining = 5;
        self.status = TransferStatus::InProgress;
    }

    async fn poll(
        &mut self,
        i2c: &mut i2c_proto::Controller<crate::i2c::Wrapper<'a>>,
    ) -> Result<TransferStatus, i2c_proto::ControllerError<i2c::Error>> {
        if self.status == TransferStatus::InProgress {
            match i2c.get_write_status(self.address).await {
                Ok(i2c_proto::WriteStatus::Ready) => {
                    if let Some(chunk) = self.iter.as_mut().unwrap().next() {
                        i2c.write_packet(self.address, chunk).await?;
                    } else {
                        i2c.write_end(self.address).await?;
                        self.status = TransferStatus::Finished;
                    }
                }
                Ok(i2c_proto::WriteStatus::Cancel) | Err(_) => {
                    if self.retries_remaining > 0 {
                        self.iter = self.beginning.clone();
                        self.retries_remaining -= 1;
                        i2c.write_begin(self.address).await?;
                    } else {
                        self.status = TransferStatus::Failed;
                    }
                }
                _ => (),
            }
        }
        Ok(self.status.clone())
    }
}
