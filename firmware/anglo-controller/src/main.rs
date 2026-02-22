#![no_std]
#![no_main]

#[cfg(feature = "defmt")]
use defmt::panic;
#[cfg(feature = "defmt")]
use defmt_rtt as _;
#[cfg(feature = "defmt")]
use panic_probe as _;
#[cfg(not(feature = "defmt"))]
use panic_reset as _;

use circular_buffer::CircularBuffer;
use embassy_boot_stm32::{AlignedBuffer, FirmwareUpdater, FirmwareUpdaterConfig};
use embassy_embedded_hal::adapter::BlockingAsync;
use embassy_stm32::flash::Flash;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::{bind_interrupts, dma, flash, gpio, i2c, peripherals, rcc, time::khz, usb};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_sync::mutex::Mutex;
use embassy_sync::watch::{self, Watch};
use embassy_time::Timer;
use embassy_usb::class::midi::MidiClass;
use embassy_usb::driver::EndpointError;
use midi::util::FileDumpReceiver;
use version::FirmwareVersion;

mod bellows;
mod file_dump_io;
mod i2c_transfer;
mod keymap;

bind_interrupts!(struct Irqs {
    USB => usb::InterruptHandler<peripherals::USB>;
    I2C1 => i2c::EventInterruptHandler<peripherals::I2C1>, i2c::ErrorInterruptHandler<peripherals::I2C1>;
    DMA1_CHANNEL2_3 => dma::InterruptHandler<peripherals::DMA1_CH2>, dma::InterruptHandler<peripherals::DMA1_CH3>;
});

struct I2cWrapper<'a> {
    i2c: i2c::I2c<'a, embassy_stm32::mode::Async, i2c::mode::Master>,
}

impl<'a> i2c_proto::ControllerIo for I2cWrapper<'a> {
    type Error = i2c::Error;

    async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c.read(address, read).await
    }

    async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        self.i2c.write(address, write).await
    }

    async fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.i2c.write_read(address, write, read).await
    }
}

type MessageChannel = Channel<ThreadModeRawMutex, midi::Message, 8>;
type MessageSender = Sender<'static, ThreadModeRawMutex, midi::Message, 8>;
type MessageReceiver = Receiver<'static, ThreadModeRawMutex, midi::Message, 8>;

static MIDI_OUT_CHANNEL: MessageChannel = MessageChannel::new();
static MIDI_IN_CHANNEL: MessageChannel = MessageChannel::new();

static UPDATE_COMPLETE: Watch<ThreadModeRawMutex, bool, 1> = Watch::new_with(false);

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    let mut config = embassy_stm32::Config::default();
    config.rcc.hsi = true;
    config.rcc.sys = rcc::Sysclk::HSI;
    config.rcc.hsi48 = Some(rcc::Hsi48Config {
        sync_from_usb: true,
    });
    config.rcc.mux.clk48sel = rcc::mux::Clk48sel::HSI48;
    let p = embassy_stm32::init(config);

    let controller_version = FirmwareVersion {
        major: env!("FIRMWARE_MAJOR_VERSION").parse().unwrap(),
        minor: env!("FIRMWARE_MINOR_VERSION").parse().unwrap(),
        patch: env!("FIRMWARE_PATCH_VERSION").parse().unwrap(),
        prerelease: option_env!("FIRMWARE_PRERELEASE_VERSION"),
    };
    #[cfg(feature = "defmt")]
    defmt::info!("Controller version: {}", controller_version);

    let flash = Flash::new_blocking(p.FLASH);
    spawner.spawn(
        firmware_task(flash, MIDI_IN_CHANNEL.receiver(), MIDI_OUT_CHANNEL.sender()).unwrap(),
    );

    let led_pin = PwmPin::new(p.PB4, gpio::OutputType::PushPull);
    let led_pwm = SimplePwm::new(
        p.TIM3,
        Some(led_pin),
        None,
        None,
        None,
        khz(30),
        Default::default(),
    );
    let button = gpio::Input::new(p.PB5, gpio::Pull::Up);
    spawner
        .spawn(control_panel_task(UPDATE_COMPLETE.receiver().unwrap(), led_pwm, button).unwrap());

    let usb_driver = usb::Driver::new(p.USB, Irqs, p.PA12, p.PA11);
    spawner.spawn(
        usb_task(
            usb_driver,
            MIDI_IN_CHANNEL.sender(),
            MIDI_OUT_CHANNEL.receiver(),
        )
        .unwrap(),
    );

    let mut i2c_config = i2c::Config::default();
    i2c_config.frequency = khz(100);

    let scl_pin = p.PB8;
    let sda_pin = p.PB9;
    let tx_dma = p.DMA1_CH2;
    let rx_dma = p.DMA1_CH3;
    let i2c_master = i2c::I2c::new(p.I2C1, scl_pin, sda_pin, tx_dma, rx_dma, Irqs, i2c_config);
    let i2c_wrapper = I2cWrapper { i2c: i2c_master };
    let mut i2c_controller = i2c_proto::Controller::new(i2c_wrapper);

    const BELLOWS_ADDR: u8 = 0x28;
    const LEFT_ADDR: u8 = 0x22;
    const RIGHT_ADDR: u8 = 0x23;
    let mut left_buttons: u16 = 0;
    let mut right_buttons: u16 = 0;

    let mut bellows_state = bellows::BellowsState::default();

    const MIDI_CHANNEL: u8 = 0;

    // Remove keyboard upgrade code/data when defmt is enabled to make more room in flash
    // TODO: Check if this is still necessary on 192kb part
    // #[cfg(not(feature = "defmt"))]
    {
        Timer::after_secs(5).await;

        let keyboard_firmware = include_bytes!("../../build/anglo-keyboard.bin");
        let mut transfer_sessions = [
            i2c_transfer::Session::new(LEFT_ADDR),
            i2c_transfer::Session::new(RIGHT_ADDR),
        ];
        for session in &mut transfer_sessions {
            const RETRY_COUNT: usize = 3;
            for _ in 0..RETRY_COUNT {
                if let Ok(keyboard_version) = i2c_controller.get_version(session.address()).await {
                    #[cfg(feature = "defmt")]
                    defmt::info!(
                        "Keyboard({}) version: {}",
                        session.address(),
                        keyboard_version
                    );

                    /*
                    if keyboard_version != controller_version
                        || keyboard_version.prerelease.is_some()
                    {
                        session.begin(keyboard_firmware);
                    }
                    */
                    break;
                }
            }
        }
        /*
        let mut transfer_in_progress = true;
        while transfer_in_progress {
            transfer_in_progress = false;
            for session in &mut transfer_sessions {
                transfer_in_progress |= match session.poll(&mut i2c_controller).await {
                    Ok(i2c_transfer::Status::InProgress) => true,
                    Err(_) => panic!("I2C communication error"),
                    _ => false,
                }
            }
        }
        */
    }

    UPDATE_COMPLETE.sender().send(true);

    loop {
        let left_notes = match bellows_state.direction {
            bellows::BellowsDirection::Push => &keymap::LEFT_PUSH[..],
            bellows::BellowsDirection::Pull => &keymap::LEFT_PULL[..],
            bellows::BellowsDirection::None => &[],
        };

        let right_notes = match bellows_state.direction {
            bellows::BellowsDirection::Push => &keymap::RIGHT_PUSH[..],
            bellows::BellowsDirection::Pull => &keymap::RIGHT_PULL[..],
            bellows::BellowsDirection::None => &[],
        };

        if let Ok(new_state) = i2c_controller.get_buttons(LEFT_ADDR).await {
            let changes = left_buttons ^ new_state;
            for (i, note) in left_notes.iter().enumerate() {
                if (changes >> i) & 1 == 1 {
                    if (new_state >> i) & 1 == 1 {
                        let message = midi::Message::note_on(MIDI_CHANNEL, *note, 127);
                        MIDI_OUT_CHANNEL.send(message).await;
                    } else {
                        let message = midi::Message::note_off(MIDI_CHANNEL, *note, 127);
                        MIDI_OUT_CHANNEL.send(message).await;
                    }
                }
            }

            left_buttons = new_state;
        }

        if let Ok(new_state) = i2c_controller.get_buttons(RIGHT_ADDR).await {
            let changes = right_buttons ^ new_state;
            for (i, note) in right_notes.iter().enumerate() {
                if (changes >> i) & 1 == 1 {
                    if (new_state >> i) & 1 == 1 {
                        let message = midi::Message::note_on(MIDI_CHANNEL, *note, 127);
                        MIDI_OUT_CHANNEL.send(message).await;
                    } else {
                        let message = midi::Message::note_off(MIDI_CHANNEL, *note, 127);
                        MIDI_OUT_CHANNEL.send(message).await;
                    }
                }
            }

            right_buttons = new_state;
        }

        if let Ok(new_state) = i2c_controller.get_bellows(BELLOWS_ADDR).await {
            let new_bellows_state = bellows::BellowsState::new(new_state);
            if new_bellows_state.direction != bellows_state.direction {
                for (i, note) in left_notes.iter().enumerate() {
                    if (left_buttons >> i) & 1 == 1 {
                        let message = midi::Message::note_off(MIDI_CHANNEL, *note, 127);
                        MIDI_OUT_CHANNEL.send(message).await;
                    }
                }

                for (i, note) in right_notes.iter().enumerate() {
                    if (right_buttons >> i) & 1 == 1 {
                        let message = midi::Message::note_off(MIDI_CHANNEL, *note, 127);
                        MIDI_OUT_CHANNEL.send(message).await;
                    }
                }

                left_buttons = 0;
                right_buttons = 0;
            }

            bellows_state = new_bellows_state;
        }

        Timer::after_micros(1).await;
    }
}

#[embassy_executor::task]
async fn firmware_task(
    flash: Flash<'static, flash::Blocking>,
    midi_in_channel: MessageReceiver,
    midi_out_channel: MessageSender,
) {
    let flash = Mutex::new(BlockingAsync::new(flash));
    let updater_config = FirmwareUpdaterConfig::from_linkerfile(&flash, &flash);

    let mut aligned_buffer = AlignedBuffer([0; flash::WRITE_SIZE]);
    let mut updater = FirmwareUpdater::new(updater_config, aligned_buffer.as_mut());
    updater.mark_booted().await.unwrap();

    const SYSEX_DEVICE_ID: u8 = 0x01;
    let mut dfu_writer = file_dump_io::DfuWriter::new(updater);
    let mut message_adapter = file_dump_io::MessageChannelAdapter::new(midi_out_channel);
    let mut receiver = FileDumpReceiver::new(
        &mut dfu_writer,
        &mut message_adapter,
        SYSEX_DEVICE_ID,
        "BIN ",
    );

    loop {
        let message = midi_in_channel.receive().await;
        receiver.process(message).await;
    }
}

#[embassy_executor::task]
async fn control_panel_task(
    mut update_complete: watch::Receiver<'static, ThreadModeRawMutex, bool, 1>,
    mut led_pwm: SimplePwm<'static, peripherals::TIM3>,
    button: gpio::Input<'static>,
) {
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

#[embassy_executor::task]
async fn usb_task(
    usb_driver: usb::Driver<'static, peripherals::USB>,
    midi_in_channel: MessageSender,
    midi_out_channel: MessageReceiver,
) {
    // TODO: Get IDs from https://pid.codes/howto/
    const USB_VID: u16 = 0xCAFE; // Default VID in TinyUSB
    const USB_PID: u16 = 0x4000 | (1 << 3); // PID for MIDI-only device in TinyUSB
    let mut usb_config = embassy_usb::Config::new(USB_VID, USB_PID);
    usb_config.manufacturer = Some("Bushel Basket");
    usb_config.product = Some("Anglo M");
    usb_config.serial_number = Some(embassy_stm32::uid::uid_hex());

    const MAX_MIDI_PACKET_SIZE: usize = 64;
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 256];

    let mut usb_builder = embassy_usb::Builder::new(
        usb_driver,
        usb_config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );
    let midi_class = MidiClass::new(&mut usb_builder, 1, 1, MAX_MIDI_PACKET_SIZE as u16);
    let (mut midi_sender, mut midi_reciever) = midi_class.split();
    let mut usb_device = usb_builder.build();
    let usb_fut = usb_device.run();

    const USB_MIDI_CABLE: u8 = 0;

    let sender_fut = async {
        let mut usb_packet = [0; MAX_MIDI_PACKET_SIZE];
        loop {
            midi_sender.wait_connection().await;

            'receive: loop {
                let message = &midi_out_channel.receive().await;
                let event_packets = midi::usb::EventPacket::encode(USB_MIDI_CABLE, message);
                for event_packet in event_packets {
                    usb_packet[..4].copy_from_slice(&event_packet.raw);
                    match midi_sender.write_packet(&usb_packet).await {
                        Err(EndpointError::BufferOverflow) => panic!("Buffer overflow"),
                        Err(EndpointError::Disabled) => break 'receive,
                        Ok(_) => (),
                    }
                }
            }
        }
    };

    let receiver_fut = async {
        let mut usb_packet = [0; MAX_MIDI_PACKET_SIZE];

        let accept_cins = [
            midi::usb::Cin::SysEx,
            midi::usb::Cin::Sys1Byte,
            midi::usb::Cin::SysExEnd2Byte,
            midi::usb::Cin::SysExEnd3Byte,
        ];

        let mut midi_buffer = CircularBuffer::<512, u8>::new();

        loop {
            midi_reciever.wait_connection().await;

            loop {
                match midi_reciever.read_packet(&mut usb_packet).await {
                    Err(EndpointError::BufferOverflow) => panic!("Buffer overflow"),
                    Err(EndpointError::Disabled) => break,
                    Ok(len) => {
                        let packets = midi::usb::EventPacket::parse(&usb_packet[..len])
                            .filter(|x| x.cable() == USB_MIDI_CABLE)
                            .filter(|x| accept_cins.contains(&x.cin()));
                        for packet in packets {
                            midi_buffer.extend_from_slice(packet.payload());
                            let sysex = match midi::Message::read_sysex(&mut midi_buffer) {
                                Some(x) => x,
                                None => continue,
                            };
                            midi_in_channel.send(sysex).await;
                        }
                    }
                }
            }
        }
    };

    embassy_futures::join::join3(usb_fut, sender_fut, receiver_fut).await;
}
