#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_stm32::adc::AdcChannel;
use embassy_stm32::{Peri, adc, bind_interrupts, gpio, i2c, peripherals, rcc, time::khz};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Ticker};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    ADC1_COMP => adc::InterruptHandler<peripherals::ADC1>;
    I2C1 => i2c::EventInterruptHandler<peripherals::I2C1>, i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

const BUTTON_COUNT: usize = 15;
const MUX_PINS: usize = 4;

enum Chirality {
    Left,
    Right,
}

struct Mux<'a, const N: usize> {
    out: [gpio::Output<'a>; N],
}

impl<'a, const N: usize> Mux<'a, N> {
    pub fn new(pins: [Peri<'static, gpio::AnyPin>; N]) -> Self {
        Self {
            out: pins.map(|p| gpio::Output::new(p, gpio::Level::Low, gpio::Speed::Medium)),
        }
    }

    pub fn select(&mut self, index: usize) {
        for bit in 0..self.out.len() {
            self.out[bit].set_level(match (index >> bit) & 1 {
                0 => gpio::Level::Low,
                _ => gpio::Level::High,
            });
        }
    }
}

struct ButtonConfig<'a> {
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

    pub fn enable(&mut self) {
        self.power.set_high();
    }

    pub fn disable(&mut self) {
        self.power.set_low();
    }

    pub fn index(&mut self) -> usize {
        self.index
    }
}

static BUTTON_STATE_SIGNAL: Signal<ThreadModeRawMutex, u16> = Signal::new();

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    let mut config = embassy_stm32::Config::default();
    config.rcc.hsi = true;
    config.rcc.sys = rcc::Sysclk::HSI;
    let p = embassy_stm32::init(config);

    let chirality_in = gpio::Input::new(p.PB4, gpio::Pull::None);
    let chirality = if chirality_in.is_low() {
        Chirality::Left
    } else {
        Chirality::Right
    };

    // Right hand mapping
    let buttons = match chirality {
        Chirality::Left => [
            ButtonConfig::new(p.PB12, 6),  // 1a
            ButtonConfig::new(p.PB10, 14), // 2a
            ButtonConfig::new(p.PB0, 11),  // 3a
            ButtonConfig::new(p.PA6, 9),   // 4a
            ButtonConfig::new(p.PA5, 8),   // 5a
            ButtonConfig::new(p.PA9, 1),   // 1
            ButtonConfig::new(p.PB15, 3),  // 2
            ButtonConfig::new(p.PB11, 7),  // 3
            ButtonConfig::new(p.PB1, 12),  // 4
            ButtonConfig::new(p.PA7, 10),  // 5
            ButtonConfig::new(p.PA10, 0),  // 6
            ButtonConfig::new(p.PA8, 2),   // 7
            ButtonConfig::new(p.PB14, 4),  // 8
            ButtonConfig::new(p.PB13, 5),  // 9
            ButtonConfig::new(p.PB2, 13),  // 10
        ],
        Chirality::Right => [
            ButtonConfig::new(p.PA8, 2),   // 1a
            ButtonConfig::new(p.PB12, 6),  // 2a
            ButtonConfig::new(p.PB10, 14), // 3a
            ButtonConfig::new(p.PB0, 11),  // 4a
            ButtonConfig::new(p.PA7, 10),  // 5a
            ButtonConfig::new(p.PA9, 1),   // 1
            ButtonConfig::new(p.PB14, 4),  // 2
            ButtonConfig::new(p.PB11, 7),  // 3
            ButtonConfig::new(p.PB1, 12),  // 4
            ButtonConfig::new(p.PA6, 9),   // 5
            ButtonConfig::new(p.PA10, 0),  // 6
            ButtonConfig::new(p.PB15, 3),  // 7
            ButtonConfig::new(p.PB13, 5),  // 8
            ButtonConfig::new(p.PB2, 13),  // 9
            ButtonConfig::new(p.PA5, 8),   // 10
        ],
    };

    let mux = Mux::new([p.PA3.into(), p.PA4.into(), p.PA1.into(), p.PA0.into()]);

    let mut adc1 = adc::Adc::new(p.ADC1, Irqs);
    adc1.set_sample_time(adc::SampleTime::CYCLES160_5);
    let adc_pin = p.PA2;

    spawner
        .spawn(button_scan_task(buttons, mux, adc1, adc_pin.degrade_adc()))
        .unwrap();

    let mut i2c_config = i2c::Config::default();
    i2c_config.sda_pullup = false;
    i2c_config.scl_pullup = false;
    // TODO: Increase frequency after fixing controller hardware.
    i2c_config.frequency = khz(10);

    let scl_pin = p.PB8;
    let sda_pin = p.PB9;
    let tx_dma = p.DMA1_CH2;
    let rx_dma = p.DMA1_CH3;
    let i2c_master = i2c::I2c::new(p.I2C1, scl_pin, sda_pin, Irqs, tx_dma, rx_dma, i2c_config);

    let i2c_addr = match chirality {
        Chirality::Left => 0x22,
        Chirality::Right => 0x23,
    };
    let slave_config = i2c::SlaveAddrConfig::basic(i2c_addr);
    let mut i2c_slave = i2c_master.into_slave_multimaster(slave_config);

    let mut response_buffer = [0u8; 2];
    loop {
        match i2c_slave.listen().await {
            Ok(i2c::SlaveCommand {
                kind: i2c::SlaveCommandKind::Read,
                address: _,
            }) => {
                let update = BUTTON_STATE_SIGNAL.try_take();
                if let Some(state) = update {
                    response_buffer = state.to_be_bytes();
                }
                let _ = i2c_slave.respond_to_read(&response_buffer).await;
            }
            _ => (),
        }
    }
}

#[embassy_executor::task]
async fn button_scan_task(
    mut buttons: [ButtonConfig<'static>; BUTTON_COUNT],
    mut mux: Mux<'static, MUX_PINS>,
    mut adc: adc::Adc<'static, peripherals::ADC1>,
    mut adc_pin: adc::AnyAdcChannel<peripherals::ADC1>,
) {
    // Allow for DRV5053 power-up. Duration is max turn-on time divided by number of simultaneously
    // powered sensors.
    let mut ticker = Ticker::every(Duration::from_micros(25));

    let mut i = 0;
    buttons[i].enable();
    ticker.next().await;

    let mut state = 0u16;

    loop {
        let next_i = (i + 1) % buttons.len();
        buttons[next_i].enable();
        ticker.next().await;

        mux.select(buttons[i].index());

        let raw = adc.read(&mut adc_pin).await;
        if raw < 600 {
            state |= 1 << i;
        } else {
            state &= !(1 << i);
        }
        BUTTON_STATE_SIGNAL.signal(state);

        buttons[i].disable();
        i = next_i;
    }
}
