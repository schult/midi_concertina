use embassy_sync::{channel, watch};

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Startup,
    KeyboardInit,
    WaitingForFirmware,
    ReceivingFirmware,
    Ready,
}

#[embassy_executor::task]
pub async fn task(
    mode_request: channel::DynamicReceiver<'static, Mode>,
    mode: watch::DynSender<'static, Mode>,
) {
    let mut internal_mode = Mode::Startup;
    mode.send(internal_mode);

    loop {
        let requested_mode = mode_request.receive().await;
        let accept_request = match internal_mode {
            Mode::Startup => requested_mode == Mode::KeyboardInit,
            Mode::KeyboardInit => requested_mode == Mode::Ready,
            Mode::WaitingForFirmware => {
                requested_mode == Mode::ReceivingFirmware || requested_mode == Mode::Ready
            }
            Mode::ReceivingFirmware => requested_mode == Mode::Ready,
            Mode::Ready => requested_mode == Mode::WaitingForFirmware,
        };

        if accept_request {
            internal_mode = requested_mode;
            mode.send(internal_mode);
        }
    }
}
