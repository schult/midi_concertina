use crate::mode::Mode;
use crate::resources::DfuResources;
use embassy_boot_stm32::{AlignedBuffer, FirmwareUpdater, FirmwareUpdaterConfig};
use embassy_embedded_hal::adapter::BlockingAsync;
use embassy_sync::mutex::Mutex;
use embassy_sync::{channel, watch};
use midi::util::FileDumpReceiver;

#[embassy_executor::task]
pub async fn task(
    r: DfuResources,
    mut mode: watch::DynReceiver<'static, Mode>,
    mode_request: channel::DynamicSender<'static, Mode>,
    midi_in: channel::DynamicReceiver<'static, midi::Message>,
    midi_out: channel::DynamicSender<'static, midi::Message>,
) {
    let flash = embassy_stm32::flash::Flash::new_blocking(r.flash);
    let flash = Mutex::new(BlockingAsync::new(flash));
    let updater_config = FirmwareUpdaterConfig::from_linkerfile(&flash, &flash);

    let mut aligned_buffer = AlignedBuffer([0; embassy_stm32::flash::WRITE_SIZE]);
    let mut updater = FirmwareUpdater::new(updater_config, aligned_buffer.as_mut());
    updater.mark_booted().await.unwrap();

    const SYSEX_DEVICE_ID: u8 = 0x01;
    let mut dfu_writer = util::DfuWriter::new(updater);
    let mut message_adapter = MessageChannelAdapter { sender: midi_out };
    let mut receiver = FileDumpReceiver::new(
        &mut dfu_writer,
        &mut message_adapter,
        SYSEX_DEVICE_ID,
        "BIN ",
    );

    loop {
        let message = midi_in.receive().await;
        match mode.get().await {
            Mode::WaitingForFirmware => {
                receiver.process(message).await;
                if receiver.active() {
                    mode_request.send(Mode::ReceivingFirmware).await;
                }
                // TODO: Return to Mode::Ready after timeout
            }
            Mode::ReceivingFirmware => {
                receiver.process(message).await;
                if !receiver.active() {
                    mode_request.send(Mode::Ready).await;
                }
            }
            _ => (),
        }
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
