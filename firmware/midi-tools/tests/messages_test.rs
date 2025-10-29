use circular_buffer::CircularBuffer;
use hex_literal::hex;
use midi_tools::messages::sysex::{HandshakeData, SysEx};

#[test]
fn sysex_read_returns_none_if_no_data() {
    let mut data = CircularBuffer::<64, u8>::new();
    let result = SysEx::read(&mut data);
    assert_eq!(result, None);
}

#[test]
fn sysex_read_returns_none_if_no_sysex_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("90 3c 7f 80 3c 7f"));
    let result = SysEx::read(&mut data);
    assert_eq!(result, None);
}

#[test]
fn sysex_read_discards_bytes_if_no_sysex_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("90 3c 7f 80 3c 7f"));
    let _ = SysEx::read(&mut data);
    assert_eq!(data, []);
}

#[test]
fn sysex_read_returns_none_if_sysex_data_is_incomplete() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7f"));
    let result = SysEx::read(&mut data);
    assert_eq!(result, None);
}

#[test]
fn sysex_read_does_not_consume_incomplete_sysex_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e"));
    let _ = SysEx::read(&mut data);
    assert_eq!(data, hex!("f0 7e"));

    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7f"));
    let _ = SysEx::read(&mut data);
    assert_eq!(data, hex!("f0 7e 03 7f"));
}

#[test]
fn sysex_read_consumes_sysex_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7f 3b"));
    let _ = SysEx::read(&mut data);
    assert_eq!(data, []);
}

#[test]
fn sysex_read_consumes_only_one_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7f 3b   f0 7e 03 7f 3c"));
    let _ = SysEx::read(&mut data);
    assert_eq!(data, hex!("f0 7e 03 7f 3c"));
}

#[test]
fn sysex_read_identifies_ack_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7f 3b"));
    let result = SysEx::read(&mut data);
    assert!(matches!(result, Some(SysEx::Ack(_))));
}

#[test]
fn sysex_reads_message_after_ignored_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("90 3c 7f   f0 7e 03 7f 3b"));
    let result = SysEx::read(&mut data);
    assert!(matches!(result, Some(SysEx::Ack(_))));
    assert_eq!(data, []);
}

#[test]
fn sysex_ignores_non_universal_messages() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 55 03 7f 3b"));
    let result = SysEx::read(&mut data);
    assert!(matches!(result, None));
    assert_eq!(data, []);

    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 00 10 56 03 7f 3b"));
    let result = SysEx::read(&mut data);
    assert!(matches!(result, None));
    assert_eq!(data, []);
}

#[test]
fn sysex_read_parses_ack_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7f 3b"));
    let result = SysEx::read(&mut data);
    let expected = Some(SysEx::Ack(HandshakeData{
        device_id: 0x03,
        packet_num: 0x3b,
    }));
    assert_eq!(result, expected);
}

#[test]
fn sysex_read_identifies_nak_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7e 3b"));
    let result = SysEx::read(&mut data);
    assert!(matches!(result, Some(SysEx::Nak(_))));
}

#[test]
fn sysex_read_parses_nak_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7e 3b"));
    let result = SysEx::read(&mut data);
    let expected = Some(SysEx::Nak(HandshakeData{
        device_id: 0x03,
        packet_num: 0x3b,
    }));
    assert_eq!(result, expected);
}

#[test]
fn sysex_read_ignores_system_real_time_messages() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 f8 7e fa 03 fb fc 7f fe ff 3b"));
    let result = SysEx::read(&mut data);
    let expected = Some(SysEx::Ack(HandshakeData{
        device_id: 0x03,
        packet_num: 0x3b,
    }));
    assert_eq!(result, expected);
    assert_eq!(data, []);
}

#[test]
fn sysex_read_discards_unrecognized_messages() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 33 3b"));
    let result = SysEx::read(&mut data);
    assert_eq!(result, None);
    assert_eq!(data, []);
}
