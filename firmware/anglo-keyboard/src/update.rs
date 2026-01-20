use embassy_boot_stm32::{AlignedBuffer, FirmwareUpdater, FirmwareUpdaterConfig};
use embassy_embedded_hal::adapter::BlockingAsync;
use embassy_stm32::flash::{self, Flash};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel;
use embassy_sync::mutex::Mutex;

pub enum Command {
    Begin,
    Write{ data: [u8;i2c_proto::PACKET_MAX_PAYLOAD_SIZE], length: usize },
    End,
}

#[embassy_executor::task]
pub async fn update_task(
    command_receiver: channel::Receiver<'static, ThreadModeRawMutex, Command, 4>,
    flash: Flash<'static, flash::Blocking>,
) {
    let flash = Mutex::new(BlockingAsync::new(flash));
    let updater_config = FirmwareUpdaterConfig::from_linkerfile(&flash, &flash);

    let mut aligned_buffer = AlignedBuffer([0; flash::WRITE_SIZE]);
    let mut updater = FirmwareUpdater::new(updater_config, aligned_buffer.as_mut());
    updater.mark_booted().await.unwrap();

    let mut dfu_writer = util::DfuWriter::new(updater);

    loop {
        match command_receiver.receive().await {
            Command::Begin => {
                if dfu_writer.open().await.is_err() {
                    // TODO: Error
                }
            }
            Command::Write { data, length } => {
                if dfu_writer.write(&data[..length]).await.is_err() {
                    // TODO: Error
                }
            }
            Command::End => {
                if dfu_writer.close().await.is_err() {
                    // TODO: Error
                }
            }
        }
    }
}
