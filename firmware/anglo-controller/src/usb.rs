#[cfg(feature = "defmt")]
use defmt::panic;

use crate::resources::{Irqs, UsbResources};
use circular_buffer::CircularBuffer;
use embassy_futures::join::join3;
use embassy_stm32::usb;
use embassy_sync::channel;
use embassy_usb::class::midi::MidiClass;
use embassy_usb::driver::EndpointError;

#[embassy_executor::task]
pub async fn task(
    r: UsbResources,
    midi_in_channel: channel::DynamicSender<'static, midi::Message>,
    midi_out_channel: channel::DynamicReceiver<'static, midi::Message>,
) {
    let usb_driver = usb::Driver::new(r.usb, Irqs, r.dp, r.dm);

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
    let usb_future = usb_device.run();

    const USB_MIDI_CABLE: u8 = 0;

    let sender_future = async {
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

    let receiver_future = async {
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

    join3(usb_future, sender_future, receiver_future).await;
}
