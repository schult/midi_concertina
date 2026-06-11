#![no_std]

use circular_buffer::CircularBuffer;
use embassy_boot::FirmwareUpdater;
use embassy_boot::FirmwareUpdaterError;
use embedded_storage_async::nor_flash::NorFlash;

const BUFFER_SIZE: usize = 256;

pub struct DfuWriter<'a, DFU: NorFlash, STATE: NorFlash> {
    updater: FirmwareUpdater<'a, DFU, STATE>,
    buffer: CircularBuffer<BUFFER_SIZE, u8>,
    offset: usize,
}

impl<'a, DFU: NorFlash, STATE: NorFlash> DfuWriter<'a, DFU, STATE> {
    pub const PAGE_SIZE: usize = DFU::ERASE_SIZE;

    pub fn new(updater: FirmwareUpdater<'a, DFU, STATE>) -> Self {
        let writer = DfuWriter {
            updater,
            buffer: CircularBuffer::<BUFFER_SIZE, u8>::new(),
            offset: 0,
        };
        assert!(writer.buffer.capacity() >= 2 * Self::PAGE_SIZE);
        writer
    }

    async fn write_page(&mut self) -> Result<(), FirmwareUpdaterError> {
        assert!(self.buffer.len() >= Self::PAGE_SIZE);
        self.updater
            .write_firmware(
                self.offset,
                &self.buffer.make_contiguous()[..Self::PAGE_SIZE],
            )
            .await?;
        drop(self.buffer.drain(..Self::PAGE_SIZE));
        self.offset += Self::PAGE_SIZE;
        Ok(())
    }
}

impl<'a, DFU: NorFlash, STATE: NorFlash> DfuWriter<'a, DFU, STATE> {
    pub async fn open(&mut self) -> Result<(), FirmwareUpdaterError> {
        self.buffer.clear();
        self.offset = 0;
        Ok(())
    }

    pub async fn write(&mut self, data: &[u8]) -> Result<(), FirmwareUpdaterError> {
        for chunk in data.chunks(Self::PAGE_SIZE) {
            self.buffer.extend_from_slice(chunk);
            while self.buffer.len() >= Self::PAGE_SIZE {
                self.write_page().await?;
            }
        }
        Ok(())
    }

    pub async fn close(&mut self) -> Result<(), FirmwareUpdaterError> {
        if !self.buffer.is_empty() {
            while self.buffer.len() < Self::PAGE_SIZE {
                self.buffer.push_back(0);
            }
            self.write_page().await?;
        }
        self.updater.mark_updated().await?;
        cortex_m::peripheral::SCB::sys_reset();
    }
}

#[cfg(feature = "midi")]
impl<'a, DFU: NorFlash, STATE: NorFlash> midi::util::FileWriter for DfuWriter<'a, DFU, STATE> {
    type ErrorType = FirmwareUpdaterError;

    async fn open(&mut self) -> Result<(), Self::ErrorType> {
        self.open().await
    }

    async fn write(&mut self, data: &[u8]) -> Result<(), Self::ErrorType> {
        self.write(data).await
    }

    async fn close(&mut self) -> Result<(), Self::ErrorType> {
        self.close().await
    }
}
