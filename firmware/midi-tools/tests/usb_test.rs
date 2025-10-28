use hex_literal::hex;
use midi_tools::usb;

#[test]
fn cin_u8_conversions_are_consistent() {
    for i in 0..16u8 {
        let cin = usb::Cin::from(i);
        assert_eq!(i, cin as u8);
    }
}

#[test]
#[should_panic]
fn cin_from_oversize_u8_panics() {
    let _ = usb::Cin::from(16u8);
}

#[test]
fn event_packet_parse_gets_no_packets_from_empty_buffer() {
    let buffer = [];
    let packets = usb::EventPacket::parse(&buffer);
    assert_eq!(packets.count(), 0);
}

#[test]
fn event_packet_parse_gets_one_packet_per_four_bytes() {
    let buffer = hex!(
        "09 90 3c 7f"
        "08 80 3c 7f"
        "19 90 48 7f"
        "18 80 48 7f"
    );
    let packets = usb::EventPacket::parse(&buffer);
    assert_eq!(packets.count(), 4);
}

#[test]
fn event_packet_extracts_cable_number() {
    let buffer = hex!(
        "80 12 34 56"
        "91 12 34 56"
        "a2 f3 34 00"
        "b3 f2 34 56"
        "c4 f0 7e 01"
        "d5 12 00 00"
        "e6 12 34 00"
        "f7 12 34 56"
        "08 82 34 56"
        "19 92 34 56"
        "2a a2 34 56"
        "3b b2 34 56"
        "4c c2 34 00"
        "5d d2 34 00"
        "6e e2 34 56"
        "7f 12 00 00"
    );
    let mut packets = usb::EventPacket::parse(&buffer);
    assert_eq!(packets.next().unwrap().cable(), 0x8);
    assert_eq!(packets.next().unwrap().cable(), 0x9);
    assert_eq!(packets.next().unwrap().cable(), 0xa);
    assert_eq!(packets.next().unwrap().cable(), 0xb);
    assert_eq!(packets.next().unwrap().cable(), 0xc);
    assert_eq!(packets.next().unwrap().cable(), 0xd);
    assert_eq!(packets.next().unwrap().cable(), 0xe);
    assert_eq!(packets.next().unwrap().cable(), 0xf);
    assert_eq!(packets.next().unwrap().cable(), 0x0);
    assert_eq!(packets.next().unwrap().cable(), 0x1);
    assert_eq!(packets.next().unwrap().cable(), 0x2);
    assert_eq!(packets.next().unwrap().cable(), 0x3);
    assert_eq!(packets.next().unwrap().cable(), 0x4);
    assert_eq!(packets.next().unwrap().cable(), 0x5);
    assert_eq!(packets.next().unwrap().cable(), 0x6);
    assert_eq!(packets.next().unwrap().cable(), 0x7);
}

#[test]
fn event_packet_extracts_cin() {
    let buffer = hex!(
        "08 82 34 56"
        "19 92 34 56"
        "2a a2 34 56"
        "3b b2 34 56"
        "4c c2 34 00"
        "5d d2 34 00"
        "6e e2 34 56"
        "7f 12 00 00"
        "80 12 34 56"
        "91 12 34 56"
        "a2 f3 34 00"
        "b3 f2 34 56"
        "c4 f0 7e 01"
        "d5 12 00 00"
        "e6 12 34 00"
        "f7 12 34 56"
    );
    let mut packets = usb::EventPacket::parse(&buffer);
    assert_eq!(packets.next().unwrap().cin(), usb::Cin::NoteOff);
    assert_eq!(packets.next().unwrap().cin(), usb::Cin::NoteOn);
    assert_eq!(packets.next().unwrap().cin(), usb::Cin::PolyKeyPress);
    assert_eq!(packets.next().unwrap().cin(), usb::Cin::ControlChange);
    assert_eq!(packets.next().unwrap().cin(), usb::Cin::ProgramChange);
    assert_eq!(packets.next().unwrap().cin(), usb::Cin::ChannelPressure);
    assert_eq!(packets.next().unwrap().cin(), usb::Cin::PitchBendChange);
    assert_eq!(packets.next().unwrap().cin(), usb::Cin::SingleByte);
    assert_eq!(packets.next().unwrap().cin(), usb::Cin::Misc);
    assert_eq!(packets.next().unwrap().cin(), usb::Cin::CableEvent);
    assert_eq!(packets.next().unwrap().cin(), usb::Cin::SysCommon2Byte);
    assert_eq!(packets.next().unwrap().cin(), usb::Cin::SysCommon3Byte);
    assert_eq!(packets.next().unwrap().cin(), usb::Cin::SysEx);
    assert_eq!(packets.next().unwrap().cin(), usb::Cin::Sys1Byte);
    assert_eq!(packets.next().unwrap().cin(), usb::Cin::SysExEnd2Byte);
    assert_eq!(packets.next().unwrap().cin(), usb::Cin::SysExEnd3Byte);
}

#[test]
fn event_packet_extracts_payload() {
    let buffer = hex!(
        "08 82 34 56"
        "19 92 34 56"
        "2a a2 34 56"
        "3b b2 34 56"
        "4c c2 34 00"
        "5d d2 34 00"
        "6e e2 34 56"
        "7f 12 00 00"
        "80 12 34 56"
        "91 12 34 56"
        "a2 f3 34 00"
        "b3 f2 34 56"
        "c4 f0 7e 01"
        "d5 12 00 00"
        "e6 12 34 00"
        "f7 12 34 56"
    );
    let mut packets = usb::EventPacket::parse(&buffer);
    assert_eq!(packets.next().unwrap().payload(), hex!("82 34 56"));
    assert_eq!(packets.next().unwrap().payload(), hex!("92 34 56"));
    assert_eq!(packets.next().unwrap().payload(), hex!("a2 34 56"));
    assert_eq!(packets.next().unwrap().payload(), hex!("b2 34 56"));
    assert_eq!(packets.next().unwrap().payload(), hex!("c2 34"));
    assert_eq!(packets.next().unwrap().payload(), hex!("d2 34"));
    assert_eq!(packets.next().unwrap().payload(), hex!("e2 34 56"));
    assert_eq!(packets.next().unwrap().payload(), hex!("12"));
    assert_eq!(packets.next().unwrap().payload(), hex!("12 34 56"));
    assert_eq!(packets.next().unwrap().payload(), hex!("12 34 56"));
    assert_eq!(packets.next().unwrap().payload(), hex!("f3 34"));
    assert_eq!(packets.next().unwrap().payload(), hex!("f2 34 56"));
    assert_eq!(packets.next().unwrap().payload(), hex!("f0 7e 01"));
    assert_eq!(packets.next().unwrap().payload(), hex!("12"));
    assert_eq!(packets.next().unwrap().payload(), hex!("12 34"));
    assert_eq!(packets.next().unwrap().payload(), hex!("12 34 56"));
}
