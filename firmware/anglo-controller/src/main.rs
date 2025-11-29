#![no_std]
#![no_main]

use circular_buffer::CircularBuffer;
use defmt::{info, panic};
use defmt_rtt as _;
use embassy_boot_stm32::{AlignedBuffer, FirmwareUpdater, FirmwareUpdaterConfig};
use embassy_embedded_hal::adapter::BlockingAsync;
use embassy_stm32::flash::Flash;
use embassy_stm32::{bind_interrupts, flash, gpio, i2c, peripherals, rcc, time::khz, usb};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
use embassy_usb::class::midi::MidiClass;
use embassy_usb::driver::EndpointError;
use midi::messages::sysex::SysEx;
use midi::usb_midi;
use midi::util::FileDumpReceiver;
use panic_probe as _;
use static_cell::StaticCell;

mod file_dump_io;

bind_interrupts!(struct Irqs {
    USB => usb::InterruptHandler<peripherals::USB>;
    I2C1 => i2c::EventInterruptHandler<peripherals::I2C1>, i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

const USB_MIDI_CABLE: u8 = 0;
const SYSEX_DEVICE_ID: u8 = 0x01;

type EventPacketChannel = Channel<NoopRawMutex, usb_midi::EventPacket, 32>;
type EventPacketSender = Sender<'static, NoopRawMutex, usb_midi::EventPacket, 32>;
type EventPacketReceiver = Receiver<'static, NoopRawMutex, usb_midi::EventPacket, 32>;

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

    let midi_out_channel: &'static mut EventPacketChannel = {
        static CHANNEL: StaticCell<EventPacketChannel> = StaticCell::new();
        CHANNEL.init_with(|| EventPacketChannel::new())
    };
    let midi_in_channel: &'static mut EventPacketChannel = {
        static CHANNEL: StaticCell<EventPacketChannel> = StaticCell::new();
        CHANNEL.init_with(|| EventPacketChannel::new())
    };

    let flash = Flash::new_blocking(p.FLASH);
    spawner
        .spawn(firmware_task(
            flash,
            midi_in_channel.receiver(),
            midi_out_channel.sender(),
        ))
        .unwrap();

    let led = gpio::Output::new(p.PB4, gpio::Level::High, gpio::Speed::Low);
    let button = gpio::Input::new(p.PB5, gpio::Pull::Up);
    spawner
        .spawn(control_panel_task(led, button, midi_out_channel.sender()))
        .unwrap();

    let usb_driver = usb::Driver::new(p.USB, Irqs, p.PA12, p.PA11);
    spawner
        .spawn(usb_task(
            usb_driver,
            midi_in_channel.sender(),
            midi_out_channel.receiver(),
        ))
        .unwrap();

    let mut i2c_config = i2c::Config::default();
    // TODO: Disable pull-ups and increase frequency after fixing hardware.
    i2c_config.sda_pullup = true;
    i2c_config.scl_pullup = true;
    i2c_config.frequency = khz(10);

    let scl_pin = p.PB8;
    let sda_pin = p.PB9;
    let tx_dma = p.DMA1_CH2;
    let rx_dma = p.DMA1_CH3;
    let mut i2c_master = i2c::I2c::new(p.I2C1, scl_pin, sda_pin, Irqs, tx_dma, rx_dma, i2c_config);

    const LEFT_ADDR: u8 = 0x22;
    const RIGHT_ADDR: u8 = 0x23;
    let mut buffer = [0; 2];
    let mut left_buttons: u16 = 0;
    let mut right_buttons: u16 = 0;
    loop {
        if i2c_master.read(LEFT_ADDR, &mut buffer).await.is_ok() {
            left_buttons = u16::from_be_bytes(buffer);
        }
        match i2c_master.read(RIGHT_ADDR, &mut buffer).await {
            Ok(_) => right_buttons = u16::from_be_bytes(buffer),
            Err(e) => info!("{}", e),
        }
        info!(" ---- ---- ----");
        info!("{:015b}", left_buttons);
        info!("{:015b}", right_buttons);
        // Timer::after_millis(100).await;
    }
}

#[embassy_executor::task]
async fn firmware_task(
    flash: Flash<'static, flash::Blocking>,
    midi_in_channel: EventPacketReceiver,
    midi_out_channel: EventPacketSender,
) {
    let flash = Mutex::new(BlockingAsync::new(flash));
    let updater_config = FirmwareUpdaterConfig::from_linkerfile(&flash, &flash);

    let mut aligned_buffer = AlignedBuffer([0; flash::WRITE_SIZE]);
    let mut updater = FirmwareUpdater::new(updater_config, aligned_buffer.as_mut());
    updater.mark_booted().await.unwrap();

    let mut dfu_writer = file_dump_io::DfuWriter::new(updater);
    let mut sysex_adapter =
        file_dump_io::SysExChannelAdapter::new(midi_out_channel, USB_MIDI_CABLE);
    let mut receiver =
        FileDumpReceiver::new(&mut dfu_writer, &mut sysex_adapter, SYSEX_DEVICE_ID, "BIN ");

    let mut midi_buffer = CircularBuffer::<512, u8>::new();

    loop {
        let event_packet = midi_in_channel.receive().await;
        midi_buffer.extend_from_slice(&event_packet.payload());

        let sysex = match SysEx::read(&mut midi_buffer) {
            Some(x) => x,
            None => continue,
        };

        receiver.process(sysex).await;
    }
}

#[embassy_executor::task]
async fn control_panel_task(
    mut led: gpio::Output<'static>,
    button: gpio::Input<'static>,
    midi_out_channel: EventPacketSender,
) {
    let mut playing = false;

    loop {
        playing = !playing;
        // playing = button.is_high();

        // TODO: This is disabled so it doesn't interfere with sending sysex messages
        // let packet = match playing {
        //     false => usb_midi::EventPacket {
        //         raw: [0x08, 0x80, 69, 127],
        //     },
        //     true => usb_midi::EventPacket {
        //         raw: [0x09, 0x90, 69, 127],
        //     },
        // };
        // midi_out_channel.send(packet).await;

        led.set_level(if playing {
            gpio::Level::High
        } else {
            gpio::Level::Low
        });
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::task]
async fn usb_task(
    usb_driver: usb::Driver<'static, peripherals::USB>,
    midi_in_channel: EventPacketSender,
    midi_out_channel: EventPacketReceiver,
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

    let sender_fut = async {
        let mut usb_packet = [0; MAX_MIDI_PACKET_SIZE];
        loop {
            midi_sender.wait_connection().await;

            loop {
                // TODO: Accumulate multiple packets if available
                usb_packet[..4].copy_from_slice(&midi_out_channel.receive().await.raw);
                match midi_sender.write_packet(&usb_packet).await {
                    Ok(_) => (),
                    Err(EndpointError::BufferOverflow) => panic!("Buffer overflow"),
                    Err(EndpointError::Disabled) => break,
                }
            }
        }
    };

    let receiver_fut = async {
        let mut usb_packet = [0; MAX_MIDI_PACKET_SIZE];

        let accept_cins = [
            usb_midi::Cin::SysEx,
            usb_midi::Cin::Sys1Byte,
            usb_midi::Cin::SysExEnd2Byte,
            usb_midi::Cin::SysExEnd3Byte,
        ];

        loop {
            midi_reciever.wait_connection().await;

            loop {
                match midi_reciever.read_packet(&mut usb_packet).await {
                    Err(EndpointError::BufferOverflow) => panic!("Buffer overflow"),
                    Err(EndpointError::Disabled) => break,
                    Ok(len) => {
                        let packets = usb_midi::EventPacket::parse(&usb_packet[..len])
                            .filter(|x| x.cable() == USB_MIDI_CABLE)
                            .filter(|x| accept_cins.contains(&x.cin()));
                        for packet in packets {
                            midi_in_channel.send(packet).await;
                        }
                    }
                }
            }
        }
    };

    embassy_futures::join::join3(usb_fut, sender_fut, receiver_fut).await;
}
