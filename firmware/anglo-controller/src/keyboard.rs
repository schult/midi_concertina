#[cfg(feature = "defmt")]
use defmt::{error, info};

use crate::i2c;
use core::slice::Chunks;
use embassy_time::Timer;
use version::FirmwareVersion;

#[derive(Clone, Copy, PartialEq)]
pub enum TransferState {
    Begin,
    InProgress,
    Done,
}

pub struct Transfer<'a> {
    state: TransferState,
    address: u8,
    version: FirmwareVersion<'a>,
    firmware: &'a [u8],
    chunks: Chunks<'a, u8>,
}

impl<'a> Transfer<'a> {
    pub fn new(address: u8, version: FirmwareVersion<'a>, firmware: &'a [u8]) -> Self {
        Self {
            state: TransferState::Begin,
            address,
            version,
            firmware,
            chunks: firmware.chunks(i2c_proto::PACKET_MAX_PAYLOAD_SIZE),
        }
    }

    pub async fn poll(
        &mut self,
        i2c: &mut i2c_proto::Controller<crate::i2c::Wrapper>,
    ) -> Result<TransferState, ()> {
        match self.state {
            TransferState::Begin => self.state_begin(i2c).await,
            TransferState::InProgress => self.state_in_progress(i2c).await,
            TransferState::Done => Ok(self.state),
        }
    }

    async fn state_begin(
        &mut self,
        i2c: &mut i2c_proto::Controller<crate::i2c::Wrapper>,
    ) -> Result<TransferState, ()> {
        let keyboard_version = match get_version(i2c, self.address).await {
            Ok(version) => version,
            Err(_) => {
                #[cfg(feature = "defmt")]
                error!("Keyboard({:02X}) error getting version", self.address);
                return Err(());
            }
        };
        #[cfg(feature = "defmt")]
        info!(
            "Keyboard({:02X}) version: {}",
            self.address, keyboard_version
        );
        if keyboard_version != self.version {
            self.chunks = self.firmware.chunks(i2c_proto::PACKET_MAX_PAYLOAD_SIZE);
            #[cfg(feature = "defmt")]
            info!("Keyboard({:02X}) receiving update...", self.address);
            if i2c.write_begin(self.address).await.is_err() {
                #[cfg(feature = "defmt")]
                error!("Keyboard({:02X}) write begin failed", self.address);
                return Err(());
            }
            self.state = TransferState::InProgress;
        }

        Ok(self.state)
    }

    async fn state_in_progress(
        &mut self,
        i2c: &mut i2c_proto::Controller<crate::i2c::Wrapper>,
    ) -> Result<TransferState, ()> {
        match i2c.get_write_status(self.address).await {
            Ok(i2c_proto::WriteStatus::Ready) => {
                if let Some(chunk) = self.chunks.next() {
                    if i2c.write_packet(self.address, chunk).await.is_err() {
                        #[cfg(feature = "defmt")]
                        error!("Keyboard({:02X}) write packet failed", self.address);
                        self.state = TransferState::Begin;
                        return Err(());
                    }
                } else {
                    if i2c.write_end(self.address).await.is_err() {
                        #[cfg(feature = "defmt")]
                        error!("Keyboard({:02X}) write end failed", self.address);
                        self.state = TransferState::Begin;
                        return Err(());
                    }
                    #[cfg(feature = "defmt")]
                    info!("Keyboard({:02X}) applying update...", self.address);
                    let _keyboard_version = match get_version(i2c, self.address).await {
                        Ok(version) => version,
                        Err(_) => {
                            #[cfg(feature = "defmt")]
                            error!("Keyboard({:02X}) error verifying version", self.address);
                            self.state = TransferState::Begin;
                            return Err(());
                        }
                    };
                    #[cfg(feature = "defmt")]
                    info!("Keyboard({:02X}) update complete", self.address);
                    #[cfg(feature = "defmt")]
                    info!(
                        "Keyboard({:02X}) version: {}",
                        self.address, _keyboard_version
                    );
                    self.state = TransferState::Done;
                }
            }
            Ok(i2c_proto::WriteStatus::Busy) => (),
            Ok(i2c_proto::WriteStatus::Cancel) => {
                #[cfg(feature = "defmt")]
                error!("Keyboard({:02X}) update transfer cancelled", self.address);
                self.state = TransferState::Begin;
                return Err(());
            }
            Err(_e) => {
                #[cfg(feature = "defmt")]
                error!("Keyboard({:02X}) update transfer error: {}", self.address, defmt::Debug2Format(&_e));
                self.state = TransferState::Begin;
                return Err(());
            }
        }

        Ok(self.state)
    }
}

async fn get_version(
    i2c: &mut i2c_proto::Controller<i2c::Wrapper>,
    address: u8,
) -> Result<FirmwareVersion<'static>, ()> {
    // Retry count and delay chosen to allow > 2 minutes for keyboard to finish applying update.
    const RETRY_COUNT: usize = 1200;
    for _ in 0..RETRY_COUNT {
        if let Ok(keyboard_version) = i2c.get_version(address).await {
            return Ok(keyboard_version);
        }
        Timer::after_millis(100).await;
    }
    #[cfg(feature = "defmt")]
    error!("Keyboard({:02X}) not responding", address);
    Err(())
}
