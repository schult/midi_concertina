use crate::mode::Mode;
use crate::resources::{ButtonResources, Irqs};
use embassy_stm32::{exti, gpio};
use embassy_sync::channel;
use embassy_time::{Duration, with_timeout};

#[embassy_executor::task]
pub async fn task(r: ButtonResources, mode_request: channel::DynamicSender<'static, Mode>) {
    let mut button = exti::ExtiInput::new(r.pin, r.channel, gpio::Pull::Up, Irqs);

    loop {
        button.wait_for_falling_edge().await;
        if with_timeout(Duration::from_secs(5), button.wait_for_rising_edge()).await.is_err() {
            mode_request.send(Mode::WaitingForFirmware).await;
            // button.wait_for_rising_edge().await;
        }
    }
}
