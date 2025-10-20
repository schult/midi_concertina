#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_stm32::{
    bind_interrupts,
    adc,
    gpio,
    peripherals,
    rcc,
};
use embassy_time::{
    Duration,
    Ticker,
};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    ADC1_COMP => adc::InterruptHandler<peripherals::ADC1>;
});

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let mut config = embassy_stm32::Config::default();
    config.rcc.hsi = true;
    config.rcc.sys = rcc::Sysclk::HSI;
    let p = embassy_stm32::init(config);

    let mut button_en = [
        gpio::Output::new(p.PA10, gpio::Level::Low, gpio::Speed::Low),  // 6
        gpio::Output::new(p.PA9, gpio::Level::Low, gpio::Speed::Low),  // 1
        gpio::Output::new(p.PA8, gpio::Level::Low, gpio::Speed::Low),  // 1a
        gpio::Output::new(p.PB15, gpio::Level::Low, gpio::Speed::Low),  // 7
        gpio::Output::new(p.PB14, gpio::Level::Low, gpio::Speed::Low),  // 2
        gpio::Output::new(p.PB13, gpio::Level::Low, gpio::Speed::Low),  // 8
        gpio::Output::new(p.PB12, gpio::Level::Low, gpio::Speed::Low),  // 2a
        gpio::Output::new(p.PB11, gpio::Level::Low, gpio::Speed::Low),  // 3
        gpio::Output::new(p.PA5, gpio::Level::Low, gpio::Speed::Low),  // 10
        gpio::Output::new(p.PA6, gpio::Level::Low, gpio::Speed::Low),  // 5
        gpio::Output::new(p.PA7, gpio::Level::Low, gpio::Speed::Low),  // 5a
        gpio::Output::new(p.PB0, gpio::Level::Low, gpio::Speed::Low),  // 4a
        gpio::Output::new(p.PB1, gpio::Level::Low, gpio::Speed::Low),  // 4
        gpio::Output::new(p.PB2, gpio::Level::Low, gpio::Speed::Low),  // 9
        gpio::Output::new(p.PB10, gpio::Level::Low, gpio::Speed::Low),  // 3a
    ];

    let mut mux = [
        gpio::Output::new(p.PA3, gpio::Level::High, gpio::Speed::Medium),
        gpio::Output::new(p.PA4, gpio::Level::High, gpio::Speed::Medium),
        gpio::Output::new(p.PA1, gpio::Level::High, gpio::Speed::Medium),
        gpio::Output::new(p.PA0, gpio::Level::High, gpio::Speed::Medium),
    ];

    let mut adc1 = adc::Adc::new(p.ADC1, Irqs);
    adc1.set_sample_time(adc::SampleTime::CYCLES160_5);
    let mut adc_pin = p.PA2;

    // Allow for DRV5053 power-up. Duration is max turn-on time divided by number of simultaneously
    // powered sensors.
    let mut ticker = Ticker::every(Duration::from_micros(25));

    let mut i = 0;
    button_en[i].set_high();
    ticker.next().await;

    let mut vals = [0u16; 15];

    loop {
        let next_i = (i + 1) % button_en.len();
        button_en[next_i].set_high();
        ticker.next().await;

        for bit in 0..mux.len() {
            mux[bit].set_level(match (i >> bit) & 1 {
                0 => gpio::Level::Low,
                _ => gpio::Level::High,
            });
        }

        vals[i] = adc1.read(&mut adc_pin).await;
        info!("{}", vals);

        button_en[i].set_low();
        i = next_i;
    }
}
