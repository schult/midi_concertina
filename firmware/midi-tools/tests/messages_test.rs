use circular_buffer::CircularBuffer;
use hex_literal::hex;
use midi_tools::messages::sysex::SysEx;

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
    assert!(data.is_empty());
}

#[test]
fn sysex_read_returns_none_if_sysex_data_is_incomplete() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 00 7f"));
    let result = SysEx::read(&mut data);
    assert_eq!(result, None);
}

#[test]
fn sysex_read_does_not_consume_incomplete_sysex_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 00 7f"));
    let _ = SysEx::read(&mut data);
    assert_eq!(data, hex!("f0 7e 00 7f"));
}
