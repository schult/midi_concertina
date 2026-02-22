#![no_std]
#![no_main]

mod buttons;
mod state;
mod update;

#[cfg(feature = "defmt")]
use defmt_rtt as _;
#[cfg(feature = "defmt")]
use panic_probe as _;
#[cfg(not(feature = "defmt"))]
use panic_reset as _;

use embassy_stm32::adc::AdcChannel;
use embassy_stm32::flash::Flash;
use embassy_stm32::{adc, bind_interrupts, dma, gpio, i2c, peripherals, rcc, time::khz};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::watch::Watch;
use state::Mode;
use version::FirmwareVersion;

bind_interrupts!(struct Irqs {
    ADC1_COMP => adc::InterruptHandler<peripherals::ADC1>;
    I2C1 => i2c::EventInterruptHandler<peripherals::I2C1>, i2c::ErrorInterruptHandler<peripherals::I2C1>;
    DMA1_CHANNEL2_3 => dma::InterruptHandler<peripherals::DMA1_CH2>, dma::InterruptHandler<peripherals::DMA1_CH3>;
});

enum Chirality {
    Left,
    Right,
}

static MODE: Watch<ThreadModeRawMutex, Mode, 2> = Watch::new_with(Mode::ScanButtons);
static BUTTON_STATE: Watch<ThreadModeRawMutex, u16, 1> = Watch::new();
static UPDATE_COMMANDS: Channel<ThreadModeRawMutex, update::Command, 4> = Channel::new();
static UPDATE_STATUS: Watch<ThreadModeRawMutex, update::Status, 1> = Watch::new();

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
    #[cfg(feature = "defmt")]
    defmt::info!("Version: {}", version);

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

    let adc1 = adc::Adc::new(p.ADC1, Irqs);
    let adc_pin = p.PA2;

    spawner.spawn(
        buttons::scan_task(
            MODE.receiver().unwrap(),
            BUTTON_STATE.sender(),
            buttons,
            mux,
            adc1,
            adc_pin.degrade_adc(),
        )
        .unwrap(),
    );

    let flash = Flash::new_blocking(p.FLASH);

    spawner.spawn(
        update::update_task(UPDATE_COMMANDS.receiver(), UPDATE_STATUS.sender(), flash).unwrap(),
    );

    let mut i2c_config = i2c::Config::default();
    i2c_config.frequency = khz(100);

    let scl_pin = p.PB8;
    let sda_pin = p.PB9;
    let tx_dma = p.DMA1_CH2;
    let rx_dma = p.DMA1_CH3;
    let i2c_master = i2c::I2c::new(p.I2C1, scl_pin, sda_pin, tx_dma, rx_dma, Irqs, i2c_config);

    let i2c_addr = match chirality {
        Chirality::Left => 0x22,
        Chirality::Right => 0x23,
    };
    let slave_config = i2c::SlaveAddrConfig::basic(i2c_addr);
    let i2c_slave = i2c_master.into_slave_multimaster(slave_config);
    let i2c_wrapper = I2cWrapper { i2c: i2c_slave };
    let mut i2c_device = i2c_proto::Device::new(i2c_wrapper);

    let mode_sender = MODE.sender();
    let mut button_state_receiver = BUTTON_STATE.receiver().unwrap();
    let mut update_status_receiver = UPDATE_STATUS.receiver().unwrap();

    enum ReadResponse {
        Buttons,
        Version,
        WriteStatus,
    }
    let mut next_read = ReadResponse::Buttons;

    loop {
        match i2c_device.listen().await {
            Ok(i2c::SlaveCommand {
                kind: i2c::SlaveCommandKind::Read,
                address: _,
            }) => {
                match next_read {
                    ReadResponse::Buttons => {
                        let button_state = button_state_receiver.get().await;
                        let _ = i2c_device.send_buttons(button_state).await;
                    }
                    ReadResponse::Version => {
                        let _ = i2c_device.send_version(&version).await;
                    }
                    ReadResponse::WriteStatus => {
                        let mode = mode_sender.try_get().unwrap();
                        let update_status = update_status_receiver.get().await;
                        let write_status = match (mode, update_status) {
                            (Mode::UpgradeFirmware, update::Status::Ok) => {
                                match UPDATE_COMMANDS.is_full() {
                                    true => i2c_proto::WriteStatus::Busy,
                                    false => i2c_proto::WriteStatus::Ready,
                                }
                            }
                            _ => i2c_proto::WriteStatus::Cancel,
                        };
                        let _ = i2c_device.send_write_status(write_status).await;
                    }
                }
                next_read = ReadResponse::Buttons;
            }
            Ok(i2c::SlaveCommand {
                kind: i2c::SlaveCommandKind::Write,
                address: _,
            }) => {
                match i2c_device.receive_command().await {
                    Ok(i2c_proto::Command::GetVersion) => {
                        next_read = ReadResponse::Version;
                    }
                    Ok(i2c_proto::Command::GetWriteStatus) => {
                        next_read = ReadResponse::WriteStatus;
                    }
                    Ok(i2c_proto::Command::WriteBegin) => {
                        mode_sender.send(Mode::UpgradeFirmware);
                        UPDATE_COMMANDS.send(update::Command::Begin).await;
                        update_status_receiver
                            .get_and(|x| *x == update::Status::Ok)
                            .await;
                    }
                    Ok(i2c_proto::Command::WritePacket { data, length }) => {
                        if mode_sender.try_get().unwrap() == Mode::UpgradeFirmware {
                            UPDATE_COMMANDS
                                .send(update::Command::Write { data, length })
                                .await;
                        }
                    }
                    Ok(i2c_proto::Command::WriteEnd) => {
                        if mode_sender.try_get().unwrap() == Mode::UpgradeFirmware {
                            UPDATE_COMMANDS.send(update::Command::End).await;
                        }
                    }
                    Err(_) => mode_sender.send(Mode::ScanButtons), // Cancel in-progress update
                }
            }
            Err(_) => (),
        }
    }
}
