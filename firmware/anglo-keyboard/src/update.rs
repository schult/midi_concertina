use crate::state::Mode;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel;
use embassy_sync::watch;

#[embassy_executor::task]
pub async fn update_task(
    command_receiver: channel::Receiver<'static, ThreadModeRawMutex, i2c_proto::Command, 4>,
    mode_sender: watch::Sender<'static, ThreadModeRawMutex, Mode, 2>,
) {
    loop {
        match command_receiver.receive().await {
            i2c_proto::Command::WriteBegin => {
                // TODO
            }
            i2c_proto::Command::WritePacket { data, length } => {
                // TODO
            }
            i2c_proto::Command::WriteEnd => {
                // TODO
            }
            _ => (), // TODO: Error?
        }
    }
}
