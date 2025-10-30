use circular_buffer::CircularBuffer;
use hex_literal::hex;
use midi_tools::messages::sysex::{FileDumpHeaderData, HandshakeData, SysEx};

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

#[test]
fn sysex_read_identifies_wait_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7c 3b"));
    let result = SysEx::read(&mut data);
    assert!(matches!(result, Some(SysEx::Wait(_))));
}

#[test]
fn sysex_read_parses_wait_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7c 3b"));
    let result = SysEx::read(&mut data);
    let expected = Some(SysEx::Wait(HandshakeData{
        device_id: 0x03,
        packet_num: 0x3b,
    }));
    assert_eq!(result, expected);
}

#[test]
fn sysex_read_identifies_cancel_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7d 3b"));
    let result = SysEx::read(&mut data);
    assert!(matches!(result, Some(SysEx::Cancel(_))));
}

#[test]
fn sysex_read_parses_cancel_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7d 3b"));
    let result = SysEx::read(&mut data);
    let expected = Some(SysEx::Cancel(HandshakeData{
        device_id: 0x03,
        packet_num: 0x3b,
    }));
    assert_eq!(result, expected);
}

#[test]
fn sysex_read_identifies_eof_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7b 3b"));
    let result = SysEx::read(&mut data);
    assert!(matches!(result, Some(SysEx::Eof(_))));
}

#[test]
fn sysex_read_parses_eof_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7b 3b"));
    let result = SysEx::read(&mut data);
    let expected = Some(SysEx::Eof(HandshakeData{
        device_id: 0x03,
        packet_num: 0x3b,
    }));
    assert_eq!(result, expected);
}

#[test]
fn sysex_read_discards_truncated_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7f   f0 7e 03 7f 3c"));
    let result = SysEx::read(&mut data);
    assert!(matches!(result, None));
    assert_eq!(data, hex!("f0 7e 03 7f 3c"));
}

#[test]
fn sysex_read_identifies_file_dump_header_message() {
    let mut data = CircularBuffer::<64, u8>::from(
        hex!("f0 7e 03 07 01 00  42 49 4e 20  1e 64  66 69 6c 65 2e 62 69 6e"));
    let result = SysEx::read(&mut data);
    assert!(matches!(result, Some(SysEx::FileDumpHeader(_))));
}

#[test]
fn sysex_read_parses_file_dump_header_data() {
    let mut data = CircularBuffer::<64, u8>::from(
        hex!("f0 7e 03 07 01 01  42 49 4e 20  1e 24  66 69 6c 65 2e 62 69 6e"));
    let result = SysEx::read(&mut data);
    let expected = Some(SysEx::FileDumpHeader(FileDumpHeaderData{
        device_id: 3,
        source_id: 1,
        length: 0x121e,
        raw_file_type: hex!("42 49 4e 20"),
    }));
    assert_eq!(result, expected);
}

#[test]
fn sysex_read_consumes_all_file_dump_header_data() {
    let mut data = CircularBuffer::<64, u8>::from(
        hex!("f0 7e 03 07 01 01  42 49 4e 20  1e 24  66 69 6c 65 2e 62 69 6e   f0 7e"));
    let _ = SysEx::read(&mut data);
    assert_eq!(data, hex!("f0 7e"));
}

#[test]
fn file_dump_header_data_file_type_as_str() {
    let data = FileDumpHeaderData{
        device_id: 0,
        source_id: 0,
        length: 0,
        raw_file_type: hex!("42 49 4e 20"),
    };
    assert_eq!(data.file_type(), "BIN ");
}
