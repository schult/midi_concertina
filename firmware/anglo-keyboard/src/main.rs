#![no_std]
#![no_main]

mod buttons;
mod state;

#[cfg(feature = "defmt")]
use defmt_rtt as _;
#[cfg(feature = "defmt")]
use panic_probe as _;
#[cfg(not(feature = "defmt"))]
use panic_reset as _;

use embassy_stm32::adc::AdcChannel;
use embassy_stm32::{adc, bind_interrupts, gpio, i2c, peripherals, rcc, time::khz};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::watch::Watch;
use state::Mode;
use version::FirmwareVersion;

bind_interrupts!(struct Irqs {
    ADC1_COMP => adc::InterruptHandler<peripherals::ADC1>;
    I2C1 => i2c::EventInterruptHandler<peripherals::I2C1>, i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

enum Chirality {
    Left,
    Right,
}

static MODE: Watch<ThreadModeRawMutex, Mode, 2> = Watch::new_with(Mode::ScanButtons);
static BUTTON_STATE: Watch<ThreadModeRawMutex, u16, 1> = Watch::new();

struct I2cWrapper<'a> {
    i2c: i2c::I2c<'a, embassy_stm32::mode::Async, i2c::mode::MultiMaster>,
}

impl<'a> i2c_proto::DeviceIo for I2cWrapper<'a> {
    type Command = i2c::SlaveCommand;
    type SendStatus = i2c::SendStatus;
    type Error = i2c::Error;

    async fn listen(&mut self) -> Result<Self::Command, Self::Error> {
        self.i2c.listen().await
    }

    async fn respond_to_read(&mut self, write: &[u8]) -> Result<Self::SendStatus, Self::Error> {
        self.i2c.respond_to_read(write).await
    }

    async fn respond_to_write(&mut self, read: &mut [u8]) -> Result<usize, Self::Error> {
        self.i2c.respond_to_write(read).await
    }
}

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    let mut config = embassy_stm32::Config::default();
    config.rcc.hsi = true;
    config.rcc.sys = rcc::Sysclk::HSI;
    let p = embassy_stm32::init(config);

    let version = FirmwareVersion {
        major: env!("FIRMWARE_MAJOR_VERSION").parse().unwrap(),
        minor: env!("FIRMWARE_MINOR_VERSION").parse().unwrap(),
        patch: env!("FIRMWARE_PATCH_VERSION").parse().unwrap(),
        prerelease: option_env!("FIRMWARE_PRERELEASE_VERSION"),
    };

    let chirality_in = gpio::Input::new(p.PB4, gpio::Pull::None);
    let chirality = if chirality_in.is_low() {
        Chirality::Left
    } else {
        Chirality::Right
    };

    // Right hand mapping
    let buttons = match chirality {
        Chirality::Left => [
            buttons::ButtonConfig::new(p.PB12, 6),  // 1a
            buttons::ButtonConfig::new(p.PB10, 14), // 2a
            buttons::ButtonConfig::new(p.PB0, 11),  // 3a
            buttons::ButtonConfig::new(p.PA6, 9),   // 4a
            buttons::ButtonConfig::new(p.PA5, 8),   // 5a
            buttons::ButtonConfig::new(p.PA9, 1),   // 1
            buttons::ButtonConfig::new(p.PB15, 3),  // 2
            buttons::ButtonConfig::new(p.PB11, 7),  // 3
            buttons::ButtonConfig::new(p.PB1, 12),  // 4
            buttons::ButtonConfig::new(p.PA7, 10),  // 5
            buttons::ButtonConfig::new(p.PA10, 0),  // 6
            buttons::ButtonConfig::new(p.PA8, 2),   // 7
            buttons::ButtonConfig::new(p.PB14, 4),  // 8
            buttons::ButtonConfig::new(p.PB13, 5),  // 9
            buttons::ButtonConfig::new(p.PB2, 13),  // 10
        ],
        Chirality::Right => [
            buttons::ButtonConfig::new(p.PA8, 2),   // 1a
            buttons::ButtonConfig::new(p.PB12, 6),  // 2a
            buttons::ButtonConfig::new(p.PB10, 14), // 3a
            buttons::ButtonConfig::new(p.PB0, 11),  // 4a
            buttons::ButtonConfig::new(p.PA7, 10),  // 5a
            buttons::ButtonConfig::new(p.PA9, 1),   // 1
            buttons::ButtonConfig::new(p.PB14, 4),  // 2
            buttons::ButtonConfig::new(p.PB11, 7),  // 3
            buttons::ButtonConfig::new(p.PB1, 12),  // 4
            buttons::ButtonConfig::new(p.PA6, 9),   // 5
            buttons::ButtonConfig::new(p.PA10, 0),  // 6
            buttons::ButtonConfig::new(p.PB15, 3),  // 7
            buttons::ButtonConfig::new(p.PB13, 5),  // 8
            buttons::ButtonConfig::new(p.PB2, 13),  // 9
            buttons::ButtonConfig::new(p.PA5, 8),   // 10
        ],
    };

    let mux = buttons::Mux::new([p.PA3.into(), p.PA4.into(), p.PA1.into(), p.PA0.into()]);

    let mut adc1 = adc::Adc::new(p.ADC1, Irqs);
    adc1.set_sample_time(adc::SampleTime::CYCLES160_5);
    let adc_pin = p.PA2;

    spawner
        .spawn(buttons::scan_task(
            MODE.receiver().unwrap(),
            BUTTON_STATE.sender(),
            buttons,
            mux,
            adc1,
            adc_pin.degrade_adc(),
        ))
        .unwrap();

    let mut i2c_config = i2c::Config::default();
    // TODO: Increase frequency and disable pullups after fixing controller hardware.
    i2c_config.sda_pullup = true;
    i2c_config.scl_pullup = true;
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
    let i2c_slave = i2c_master.into_slave_multimaster(slave_config);
    let i2c_wrapper = I2cWrapper { i2c: i2c_slave };
    let mut i2c_device = i2c_proto::Device::new(i2c_wrapper);

    let mut mode_receiver = MODE.receiver().unwrap();
    let mut button_state_receiver = BUTTON_STATE.receiver().unwrap();

    loop {
        if let Ok(i2c::SlaveCommand {
            kind: i2c::SlaveCommandKind::Read,
            address: _,
        }) = i2c_device.listen().await
        {
            let button_state = button_state_receiver.get().await;
            let _ = i2c_device.send_buttons(button_state).await;
        }
    }
}
