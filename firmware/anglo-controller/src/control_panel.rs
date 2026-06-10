use crate::resources::ControlPanelResources;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::{gpio, time::khz};
use embassy_sync::watch;
use embassy_time::Timer;

#[embassy_executor::task]
pub async fn task(
    r: ControlPanelResources,
    mut update_complete: watch::DynReceiver<'static, bool>,
) {
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
    let button = gpio::Input::new(r.button, gpio::Pull::Up);

    let mut led = led_pwm.ch1();
    let led_max = led.max_duty_cycle() / 4;
    let led_step = led_max / 64;
    let mut led_duty = 0;
    let mut inc = true;
    led.enable();

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
            led_duty = led_duty + led_step
        } else {
            led_duty = led_duty - led_step
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
}
