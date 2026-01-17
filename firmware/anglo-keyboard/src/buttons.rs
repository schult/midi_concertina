use crate::state::Mode;
use embassy_stm32::{Peri, adc, gpio, peripherals};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::watch;
use embassy_time::{Duration, Ticker};

pub const BUTTON_COUNT: usize = 15;
pub const MUX_PIN_COUNT: usize = 4;

pub struct Mux<'a, const N: usize> {
    out: [gpio::Output<'a>; N],
}

impl<'a, const N: usize> Mux<'a, N> {
    pub fn new(pins: [Peri<'a, gpio::AnyPin>; N]) -> Self {
        Self {
            out: pins.map(|p| gpio::Output::new(p, gpio::Level::Low, gpio::Speed::Medium)),
        }
    }

    fn select(&mut self, index: usize) {
        for bit in 0..self.out.len() {
            self.out[bit].set_level(match (index >> bit) & 1 {
                0 => gpio::Level::Low,
                _ => gpio::Level::High,
            });
        }
    }
}

pub struct ButtonConfig<'a> {
    power: gpio::Output<'a>,
    index: usize,
}

impl<'a> ButtonConfig<'a> {
    pub fn new(power_pin: Peri<'a, impl gpio::Pin>, index: usize) -> Self {
        Self {
            power: gpio::Output::new(power_pin, gpio::Level::Low, gpio::Speed::Low),
            index,
        }
    }

    fn enable(&mut self) {
        self.power.set_high();
    }

    fn disable(&mut self) {
        self.power.set_low();
    }

    fn index(&mut self) -> usize {
        self.index
    }
}

#[embassy_executor::task]
pub async fn scan_task(
    mut mode_receiver: watch::Receiver<'static, ThreadModeRawMutex, Mode, 2>,
    button_state_sender: watch::Sender<'static, ThreadModeRawMutex, u16, 1>,
    mut buttons: [ButtonConfig<'static>; BUTTON_COUNT],
    mut mux: Mux<'static, MUX_PIN_COUNT>,
    mut adc: adc::Adc<'static, peripherals::ADC1>,
    mut adc_pin: adc::AnyAdcChannel<peripherals::ADC1>,
) {
    // Allow for DRV5053 power-up. Duration is max turn-on time divided by number of simultaneously
    // powered sensors.
    let mut ticker = Ticker::every(Duration::from_micros(25));

    let mut i = 0;
    buttons[i].enable();
    ticker.next().await;

    let mut button_state = 0u16;

    loop {
        let _ = mode_receiver.get_and(|x| *x == Mode::ScanButtons).await;

        let next_i = (i + 1) % buttons.len();
        buttons[next_i].enable();
        ticker.next().await;

        mux.select(buttons[i].index());

        let raw = adc.read(&mut adc_pin).await;
        if raw < 600 {
            button_state |= 1 << i;
        } else {
            button_state &= !(1 << i);
        }
        button_state_sender.send(button_state);

        buttons[i].disable();
        i = next_i;
    }
}
