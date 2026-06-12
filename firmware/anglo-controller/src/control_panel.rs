use crate::mode::Mode;
use crate::resources::ControlPanelResources;
use embassy_stm32::timer::GeneralInstance4Channel;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm, SimplePwmChannel};
use embassy_stm32::{gpio, time::khz};
use embassy_sync::watch;
use embassy_time::Timer;

#[embassy_executor::task]
pub async fn task(r: ControlPanelResources, mut mode: watch::DynReceiver<'static, Mode>) {
    let led_pin = PwmPin::new(r.led, gpio::OutputType::PushPull);
    let mut led_pwm = SimplePwm::new(
        r.timer,
        Some(led_pin),
        None,
        None,
        None,
        khz(30),
        Default::default(),
    );
    // TODO: Move button to separate task
    let button = gpio::Input::new(r.button, gpio::Pull::Up);

    let mut led = led_pwm.ch1();
    let led_max = led.max_duty_cycle() / 4;
    let led_step = led_max / 64;
    let mut led_duty = 0;
    let mut inc = true;
    led.enable();

    loop {
        // TODO: Tune led timing
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
                // TODO: Breath
                led.set_duty_cycle(led_max);
                led.enable();
                Timer::after_secs(1).await;
            }
        }
    }

    /*
        // TODO: Hold button to enter wait for firmware update
        // TODO: Indicate system state with LED

        while !update_complete.get().await {
            if inc && (led_max - led_duty < led_step) {
                led_duty = led_max;
                inc = !inc;
            } else if !inc && (led_duty < led_step) {
                led_duty = 0;
                inc = !inc;
            } else if inc {
                led_duty += led_step
            } else {
                led_duty -= led_step
            }
            led.set_duty_cycle(led_duty);

            Timer::after_millis(10).await;
        }

        led.set_duty_cycle(led_max);
        loop {
            match button.get_level() {
                gpio::Level::High => led.enable(),
                gpio::Level::Low => led.disable(),
            }
            Timer::after_millis(10).await;
        }

    */
}

async fn blink<'a, T: GeneralInstance4Channel>(led: &mut SimplePwmChannel<'a, T>, count: u32) {
    for _ in 0..count {
        led.enable();
        Timer::after_millis(100).await;
        led.disable();
        Timer::after_millis(100).await;
    }
}
