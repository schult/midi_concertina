use crate::mode::Mode;
use crate::resources::LedResources;
use core::f32::consts;
use embassy_stm32::timer::GeneralInstance4Channel;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm, SimplePwmChannel};
use embassy_stm32::{gpio, time::khz};
use embassy_sync::watch;
use embassy_time::{Instant, Timer};
use micromath::F32Ext;

#[embassy_executor::task]
pub async fn task(
    r: LedResources,
    mut mode: watch::DynReceiver<'static, Mode>,
    mut button_held: watch::DynReceiver<'static, bool>,
) {
    let led_pin = PwmPin::new(r.pin, gpio::OutputType::PushPull);
    let mut led_pwm = SimplePwm::new(
        r.timer,
        Some(led_pin),
        None,
        None,
        None,
        khz(30),
        Default::default(),
    );

    let mut led = led_pwm.ch1();
    let led_max = led.max_duty_cycle() / 2;
    let led_min = led.max_duty_cycle() / 64;
    led.enable();

    loop {
        match mode.get().await {
            Mode::Startup => {
                led.set_duty_cycle(led_max);
                blink(&mut led, 2).await;
                Timer::after_secs(1).await;
            }
            Mode::KeyboardInit => {
                led.set_duty_cycle(led_max);
                blink(&mut led, 3).await;
                Timer::after_secs(1).await;
            }
            Mode::WaitingForFirmware => {
                led.set_duty_cycle(led_max);
                blink(&mut led, 4).await;
                Timer::after_secs(1).await;
            }
            Mode::ReceivingFirmware => {
                led.set_duty_cycle(led_max);
                blink(&mut led, 1).await;
            }
            Mode::Ready => {
                let brightness = if button_held.get().await {
                    led_max
                } else {
                    const INTERVAL: u64 = 4000; // milliseconds
                    let t = (Instant::now().as_millis() % INTERVAL) as f32;
                    let x = (t / INTERVAL as f32) * 2.0 * consts::PI;
                    let y = x.sin().exp();
                    let y_norm = (y - (1.0 / consts::E)) / (consts::E - (1.0 / consts::E));
                    (y_norm * (led_max - led_min) as f32) as u32 + led_min
                };
                led.set_duty_cycle(brightness);
                led.enable();
                Timer::after_millis(30).await;
            }
        }
    }
}

async fn blink<'a, T: GeneralInstance4Channel>(led: &mut SimplePwmChannel<'a, T>, count: u32) {
    for _ in 0..count {
        led.enable();
        Timer::after_millis(100).await;
        led.disable();
        Timer::after_millis(200).await;
    }
}
