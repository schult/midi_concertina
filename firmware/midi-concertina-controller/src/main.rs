#![no_std]
#![no_main]

use circular_buffer::CircularBuffer;
use defmt::{info, warn, panic};
use defmt_rtt as _;
use embassy_boot_stm32::{AlignedBuffer, FirmwareUpdater, FirmwareUpdaterConfig};
use embassy_embedded_hal::adapter::BlockingAsync;
use embassy_stm32::flash::{self, Flash};
use embassy_stm32::{bind_interrupts, gpio, peripherals, rcc, usb};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
use embassy_usb::class::midi;
use embassy_usb::driver::EndpointError;
use midi_tools::messages::sysex::SysEx;
use midi_tools::usb_midi;
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USB => usb::InterruptHandler<peripherals::USB>;
});

const USB_MIDI_CABLE: u8 = 0;
const SYSEX_DEVICE_ID: u8 = 0x01;
const SYSEX_ANY_DEVICE: u8 = 0x7F;

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

    // TODO: Do actual work here?
    let forever = embassy_sync::signal::Signal::<NoopRawMutex, ()>::new();
    forever.wait().await;
}

#[embassy_executor::task]
async fn firmware_task(
    flash: Flash::<'static, flash::Blocking>,
    midi_in_channel: EventPacketReceiver,
    midi_out_channel: EventPacketSender,
) {
    let flash = Mutex::new(BlockingAsync::new(flash));
    let updater_config = FirmwareUpdaterConfig::from_linkerfile(&flash, &flash);

    const PAGE_SIZE: usize = flash::MAX_ERASE_SIZE;
    let max_firmware_size: usize = updater_config.dfu.size() as usize - PAGE_SIZE;
    let mut aligned_buffer = AlignedBuffer([0; flash::WRITE_SIZE]);
    let mut updater = FirmwareUpdater::new(updater_config, aligned_buffer.as_mut());
    updater.mark_booted().await.unwrap();

    let send = async |sysex: SysEx| {
        for event_packet in usb_midi::EventPacket::encode_sysex(USB_MIDI_CABLE, &sysex) {
            midi_out_channel.send(event_packet).await;
        }
    };

    struct FileDumpProgress {
        source_id: u8,
        next_packet: u8,
        offset: usize,
    }
    let mut state: Option<FileDumpProgress> = None;

    let mut midi_buffer = CircularBuffer::<512, u8>::new();
    let mut file_buffer = CircularBuffer::<256, u8>::new();

    loop {
        let event_packet = midi_in_channel.receive().await;
        midi_buffer.extend_from_slice(&event_packet.payload());

        let sysex = match SysEx::read(&mut midi_buffer) {
            Some(x) => x,
            None => continue,
        };

        if let SysEx::FileDumpHeader(header) = sysex {
            if header.device_id != SYSEX_DEVICE_ID && header.device_id != SYSEX_ANY_DEVICE {
                continue;
            }

            if header.file_type() != "BIN " {
                warn!("CANCEL");
                send(SysEx::cancel(header.source_id, 0)).await;
                continue;
            }

            if header.length as usize > max_firmware_size {
                warn!("CANCEL");
                send(SysEx::cancel(header.source_id, 0)).await;
                continue;
            }

            file_buffer.clear();
            /*
            let result = updater.prepare_update().await;
            if result.is_ok() {
                send(SysEx::ack(header.source_id, 0)).await;
                state = Some(FileDumpProgress{ source_id: header.source_id, next_packet: 0, offset: 0 });
            } else {
                warn!("CANCEL");
                send(SysEx::cancel(header.source_id, 0)).await;
            }
            */
            send(SysEx::ack(header.source_id, 0)).await;
            state = Some(FileDumpProgress{ source_id: header.source_id, next_packet: 0, offset: 0 });
        } else if let Some(progress) = &mut state {
            match sysex {
                SysEx::FileDumpPacket(packet) => {
                    if packet.device_id != SYSEX_DEVICE_ID && packet.device_id != SYSEX_ANY_DEVICE {
                        continue;
                    }

                    if packet.packet_num != progress.next_packet {
                        warn!("CANCEL");
                        send(SysEx::cancel(progress.source_id, packet.packet_num)).await;
                        state = None;
                        continue;
                    }

                    if !packet.checksum_ok {
                        send(SysEx::nak(progress.source_id, packet.packet_num)).await;
                        continue;
                    }

                    file_buffer.extend_from_slice(packet.data());
                    if file_buffer.len() >= PAGE_SIZE {
                        if progress.offset + PAGE_SIZE > max_firmware_size {
                            warn!("CANCEL");
                            send(SysEx::cancel(progress.source_id, packet.packet_num)).await;
                            state = None;
                            continue;
                        }

                        let mut buffer = [0; PAGE_SIZE];
                        for i in 0..PAGE_SIZE {
                            buffer[i] = file_buffer.pop_front().unwrap();
                        }
                        // info!("{:x}", buffer);
                        let result = updater.write_firmware(progress.offset, &buffer).await;
                        if result.is_err() {
                            warn!("CANCEL");
                            send(SysEx::cancel(progress.source_id, packet.packet_num)).await;
                            state = None;
                            continue;
                        }
                        progress.offset += buffer.len();
                    }

                    progress.next_packet = (progress.next_packet + 1) % 0x7F;

                    send(SysEx::ack(progress.source_id, packet.packet_num)).await;
                },
                SysEx::Eof(_) => {
                    if !file_buffer.is_empty() {
                        let mut buffer = [0; PAGE_SIZE];
                        for i in 0..file_buffer.len() {
                            buffer[i] = file_buffer.pop_front().unwrap();
                        }
                        // info!("{:x}", buffer);
                        let result = updater.write_firmware(progress.offset, &buffer).await;
                        if result.is_err() {
                            state = None;
                            continue;
                        }
                    }

                    info!("MARKING UPDATED");
                    Timer::after_secs(1).await;
                    let _ = updater.mark_updated().await;
                    info!("RESET");
                    Timer::after_secs(1).await;
                    cortex_m::peripheral::SCB::sys_reset();
                },
                _ => (),
            }
        }
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
        // playing = !playing;
        playing = button.is_high();

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
    let midi_class = midi::MidiClass::new(&mut usb_builder, 1, 1, MAX_MIDI_PACKET_SIZE as u16);
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
