use crate::resources::DfuResources;
use embassy_boot_stm32::{AlignedBuffer, FirmwareUpdater, FirmwareUpdaterConfig};
use embassy_embedded_hal::adapter::BlockingAsync;
use embassy_sync::channel;
use embassy_sync::mutex::Mutex;
use midi::util::FileDumpReceiver;

#[embassy_executor::task]
pub async fn task(
    r: DfuResources,
    midi_in_channel: channel::DynamicReceiver<'static, midi::Message>,
    midi_out_channel: channel::DynamicSender<'static, midi::Message>,
) {
    let flash = embassy_stm32::flash::Flash::new_blocking(r.flash);
    let flash = Mutex::new(BlockingAsync::new(flash));
    let updater_config = FirmwareUpdaterConfig::from_linkerfile(&flash, &flash);

    let mut aligned_buffer = AlignedBuffer([0; embassy_stm32::flash::WRITE_SIZE]);
    let mut updater = FirmwareUpdater::new(updater_config, aligned_buffer.as_mut());
    updater.mark_booted().await.unwrap();

    const SYSEX_DEVICE_ID: u8 = 0x01;
    let mut dfu_writer = util::DfuWriter::new(updater);
    let mut message_adapter = MessageChannelAdapter {
        sender: midi_out_channel,
    };
    let mut receiver = FileDumpReceiver::new(
        &mut dfu_writer,
        &mut message_adapter,
        SYSEX_DEVICE_ID,
        "BIN ",
    );

    loop {
        let message = midi_in_channel.receive().await;
        receiver.process(message).await;
    }
}

struct MessageChannelAdapter {
    sender: channel::DynamicSender<'static, midi::Message>,
}

impl midi::util::MessageSender for MessageChannelAdapter {
    async fn send(&mut self, message: midi::Message) {
        self.sender.send(message).await;
    }
}
