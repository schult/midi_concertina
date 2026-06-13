#[cfg(feature = "defmt")]
use defmt::{error, info};

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
    #[cfg(feature = "defmt")]
    error!("Keyboard({:02X}) not responding", address);
    Err(())
}

pub async fn update_firmware<'a>(
    i2c_mutex: &'a i2c::I2cMutex,
    address: u8,
    version: &FirmwareVersion<'a>,
    firmware: &[u8],
) -> Result<FirmwareVersion<'static>, ()> {
    let i2c_wrapper = i2c::Wrapper::new(i2c_mutex);
    let mut i2c_controller = i2c_proto::Controller::new(i2c_wrapper);

    let keyboard_version = get_keyboard_version(&mut i2c_controller, address).await?;
    #[cfg(feature = "defmt")]
    info!("Keyboard({:02X}) version: {}", address, keyboard_version);
    if keyboard_version == *version {
        return Ok(keyboard_version);
    }

    #[cfg(feature = "defmt")]
    info!("Keyboard({:02X}) receiving update...", address);
    let mut session = TransferSession::new(address, firmware);
    loop {
        match session.poll(&mut i2c_controller).await {
            Ok(TransferStatus::Finished) => break,
            Ok(TransferStatus::InProgress) => (),
            Ok(TransferStatus::Failed) | Err(_) => {
                #[cfg(feature = "defmt")]
                error!("Keyboard({:02X}) update transfer failed", address);
                return Err(());
            }
        }
        yield_now().await;
    }

    #[cfg(feature = "defmt")]
    info!("Keyboard({:02X}) applying update...", address);
    let keyboard_version = get_keyboard_version(&mut i2c_controller, address).await?;
    #[cfg(feature = "defmt")]
    info!("Keyboard({:02X}) update complete", address);
    #[cfg(feature = "defmt")]
    info!("Keyboard({:02X}) version: {}", address, keyboard_version);
    Ok(keyboard_version)
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
    fn new(address: u8, data: &'a [u8]) -> Self {
        let beginning = Some(data.chunks(i2c_proto::PACKET_MAX_PAYLOAD_SIZE));
        Self {
            address,
            beginning: beginning.clone(),
            iter: beginning.clone(),
            retries_remaining: 5,
            status: TransferStatus::InProgress,
        }
    }

    async fn poll(
        &mut self,
        i2c: &mut i2c_proto::Controller<crate::i2c::Wrapper<'a>>,
    ) -> Result<TransferStatus, i2c_proto::ControllerError<i2c::Error>> {
        if self.status != TransferStatus::InProgress {
            return Ok(self.status.clone());
        }

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

        Ok(self.status.clone())
    }
}
