use crate::mode::Mode;
use crate::resources::LedResources;
use core::f32::consts;
use core::iter::Cycle;
use core::slice;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::{gpio, time::khz};
use embassy_sync::watch;
use embassy_time::{Duration, Ticker};
use micromath::F32Ext;
use static_cell::StaticCell;

const TICK_INTERVAL: Duration = Duration::from_millis(25);
const BREATH_INTERVAL: Duration = Duration::from_millis(4000);
const BREATH_STEP_COUNT: usize = (BREATH_INTERVAL.as_millis() / TICK_INTERVAL.as_millis()) as usize;

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
    led.set_duty_cycle_fully_off();
    led.enable();

    static BREATH_SEQUENCE: StaticCell<[(u32, u32); BREATH_STEP_COUNT]> = StaticCell::new();
    let breath_sequence = BREATH_SEQUENCE.init(generate_breath_sequence());
    let get_cycle = |mode: Mode| -> Cycle<slice::Iter<'static, (u32, u32)>> {
        match mode {
            Mode::Startup => [(1000, 4), (0, 8), (1000, 4), (0, 40)].iter().cycle(),
            Mode::KeyboardInit => [(1000, 4), (0, 8), (1000, 4), (0, 8), (1000, 4), (0, 40)]
                .iter()
                .cycle(),
            Mode::WaitingForFirmware => [(1000, 8), (0, 12)].iter().cycle(),
            Mode::ReceivingFirmware => [(1000, 2), (0, 4)].iter().cycle(),
            Mode::Ready => breath_sequence.iter().cycle(),
        }
    };

    let mut ticker = Ticker::every(TICK_INTERVAL);
    let mut local_mode = mode.get().await;
    let mut cycle = get_cycle(local_mode);
    let mut brightness = 0;
    let mut counter = 0;

    loop {
        let new_mode = mode.get().await;
        if local_mode != new_mode {
            local_mode = new_mode;
            cycle = get_cycle(local_mode);
            counter = 0;
        }

        if counter == 0 {
            (brightness, counter) = *cycle.next().unwrap();
        }
        counter -= 1;

        if local_mode == Mode::Ready && button_held.get().await {
            led.set_duty_cycle_fully_on();
        } else {
            // Run LED at half-brightness
            led.set_duty_cycle_fraction(brightness, 2000);
        }

        ticker.next().await;
    }
}

fn generate_breath_sequence() -> [(u32, u32); BREATH_STEP_COUNT] {
    let mut sequence = [(0, 1); BREATH_STEP_COUNT];
    for (i, step) in sequence.iter_mut().enumerate() {
        let x = (i as f32 / BREATH_STEP_COUNT as f32) * 2.0 * consts::PI;
        let y = x.sin().exp();
        let normalized_y = (y - (1.0 / consts::E)) / (consts::E - (1.0 / consts::E));
        const MAX_DUTY: u32 = 1000;
        const MIN_DUTY: u32 = 15;
        step.0 = (normalized_y * (MAX_DUTY - MIN_DUTY) as f32) as u32 + MIN_DUTY;
    }
    sequence
}
