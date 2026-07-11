#[cfg(feature = "defmt")]
use defmt::{error, info};

use crate::i2c;
use core::slice::Chunks;
use embassy_futures::yield_now;
use embassy_time::Timer;
use version::FirmwareVersion;

async fn get_keyboard_version(
    i2c_controller: &mut i2c_proto::Controller<i2c::Wrapper>,
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
    i2c_controller: &mut i2c_proto::Controller<i2c::Wrapper>,
    address: u8,
    version: &FirmwareVersion<'a>,
    firmware: &[u8],
) -> Result<FirmwareVersion<'static>, ()> {
    let keyboard_version = get_keyboard_version(i2c_controller, address).await?;
    #[cfg(feature = "defmt")]
    info!("Keyboard({:02X}) version: {}", address, keyboard_version);
    if keyboard_version == *version {
        return Ok(keyboard_version);
    }

    #[cfg(feature = "defmt")]
    info!("Keyboard({:02X}) receiving update...", address);
    let mut transfer = Transfer::new(address, firmware);
    loop {
        match transfer.poll(i2c_controller).await {
            Ok(TransferStatus::InProgress) => (),
            Ok(TransferStatus::Finished) => break,
            Err(_) => {
                #[cfg(feature = "defmt")]
                error!("Keyboard({:02X}) update transfer failed", address);
                return Err(());
            }
        }
        yield_now().await;
    }

    #[cfg(feature = "defmt")]
    info!("Keyboard({:02X}) applying update...", address);
    let keyboard_version = get_keyboard_version(i2c_controller, address).await?;
    #[cfg(feature = "defmt")]
    info!("Keyboard({:02X}) update complete", address);
    #[cfg(feature = "defmt")]
    info!("Keyboard({:02X}) version: {}", address, keyboard_version);
    Ok(keyboard_version)
}

#[derive(Clone, PartialEq)]
enum TransferStatus {
    InProgress,
    Finished,
}

struct TransferError;

impl From<i2c_proto::ControllerError<i2c::Error>> for TransferError {
    fn from(_: i2c_proto::ControllerError<i2c::Error>) -> Self {
        Self {}
    }
}

struct Transfer<'a> {
    started: bool,
    address: u8,
    chunks: Chunks<'a, u8>,
}

impl<'a> Transfer<'a> {
    fn new(address: u8, firmware: &'a [u8]) -> Self {
        Self {
            started: false,
            address,
            chunks: firmware.chunks(i2c_proto::PACKET_MAX_PAYLOAD_SIZE),
        }
    }

    async fn poll(
        &mut self,
        i2c: &mut i2c_proto::Controller<crate::i2c::Wrapper>,
    ) -> Result<TransferStatus, TransferError> {
        if !self.started {
            i2c.write_begin(self.address).await?;
            self.started = true;
        }

        match i2c.get_write_status(self.address).await? {
            i2c_proto::WriteStatus::Ready => {
                if let Some(chunk) = self.chunks.next() {
                    i2c.write_packet(self.address, chunk).await?;
                } else {
                    i2c.write_end(self.address).await?;
                    return Ok(TransferStatus::Finished);
                }
            }
            i2c_proto::WriteStatus::Busy => (),
            i2c_proto::WriteStatus::Cancel => {
                return Err(TransferError);
            }
        }

        Ok(TransferStatus::InProgress)
    }
}
