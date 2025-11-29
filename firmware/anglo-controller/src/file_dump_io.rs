use circular_buffer::CircularBuffer;
use embassy_boot::FirmwareUpdaterError;
use embassy_boot_stm32::FirmwareUpdater;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::Sender;
use embedded_storage_async::nor_flash::NorFlash;
use midi::SystemExclusiveMessage;

type EventPacketSender = Sender<'static, NoopRawMutex, midi::usb::EventPacket, 32>;

pub struct SysExChannelAdapter {
    channel: EventPacketSender,
    cable: u8,
}

impl SysExChannelAdapter {
    pub fn new(channel: EventPacketSender, cable: u8) -> Self {
        assert!(cable <= 0x0F);
        SysExChannelAdapter { channel, cable }
    }
}

impl midi::util::SysExOutput for SysExChannelAdapter {
    async fn send(&mut self, sysex: SystemExclusiveMessage) {
        for event_packet in midi::usb::EventPacket::encode_sysex(self.cable, &sysex) {
            self.channel.send(event_packet).await;
        }
    }
}

pub struct DfuWriter<'a, DFU: NorFlash, STATE: NorFlash> {
    updater: FirmwareUpdater<'a, DFU, STATE>,
    buffer: CircularBuffer<256, u8>,
    offset: usize,
}

impl<'a, DFU: NorFlash, STATE: NorFlash> DfuWriter<'a, DFU, STATE> {
    pub const PAGE_SIZE: usize = DFU::ERASE_SIZE;

    pub fn new(updater: FirmwareUpdater<'a, DFU, STATE>) -> Self {
        let writer = DfuWriter {
            updater,
            buffer: CircularBuffer::<256, u8>::new(),
            offset: 0,
        };
        assert!(writer.buffer.capacity() >= 2 * Self::PAGE_SIZE);
        writer
    }

    pub async fn write_len(&mut self, len: usize) -> Result<(), FirmwareUpdaterError> {
        assert!(len <= self.buffer.len());
        assert!(len <= Self::PAGE_SIZE);
        while self.buffer.len() < Self::PAGE_SIZE {
            self.buffer.push_back(0);
        }
        self.updater
            .write_firmware(
                self.offset,
                &self.buffer.make_contiguous()[..Self::PAGE_SIZE],
            )
            .await?;
        let _ = self.buffer.drain(..Self::PAGE_SIZE);
        self.offset += len;
        Ok(())
    }
}

impl<'a, DFU: NorFlash, STATE: NorFlash> midi::util::FileWriter for DfuWriter<'a, DFU, STATE> {
    type ErrorType = FirmwareUpdaterError;

    async fn open(&mut self) -> Result<(), Self::ErrorType> {
        self.buffer.clear();
        self.offset = 0;
        Ok(())
    }

    async fn write(&mut self, data: &[u8]) -> Result<(), Self::ErrorType> {
        self.buffer.extend_from_slice(data);
        while self.buffer.len() >= Self::PAGE_SIZE {
            self.write_len(Self::PAGE_SIZE).await?;
        }
        Ok(())
    }

    async fn close(&mut self) -> Result<(), Self::ErrorType> {
        if !self.buffer.is_empty() {
            self.write_len(self.buffer.len()).await?;
        }
        self.updater.mark_updated().await?;
        cortex_m::peripheral::SCB::sys_reset();
    }
}
