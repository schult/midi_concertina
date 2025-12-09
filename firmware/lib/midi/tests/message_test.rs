use circular_buffer::CircularBuffer;
use hex_literal::hex;
use midi::*;

#[test]
fn message_note_on_constructor() {
    let value = Message::note_on(2, Note::Ab4, 64);

    assert!(matches!(value, Message::NoteOn(_)));
    if let Message::NoteOn(d) = value {
        assert_eq!(d.channel, 2);
        assert_eq!(d.note, Note::Ab4);
        assert_eq!(d.velocity, 64);
    }
}

#[test]
#[should_panic]
fn message_note_on_constructor_panics_if_channel_over_4_bits() {
    let _ = Message::note_on(1 << 4, Note::Ab4, 0);
}

#[test]
#[should_panic]
fn message_note_on_constructor_panics_if_velocity_over_7_bits() {
    let _ = Message::note_on(2, Note::Ab4, 1 << 7);
}

#[test]
fn message_note_off_constructor() {
    let value = Message::note_off(2, Note::Ab4, 64);

    assert!(matches!(value, Message::NoteOff(_)));
    if let Message::NoteOff(d) = value {
        assert_eq!(d.channel, 2);
        assert_eq!(d.note, Note::Ab4);
        assert_eq!(d.velocity, 64);
    }
}

#[test]
#[should_panic]
fn message_note_off_constructor_panics_if_channel_over_4_bits() {
    let _ = Message::note_off(1 << 4, Note::Ab4, 0);
}

#[test]
#[should_panic]
fn message_note_off_constructor_panics_if_velocity_over_7_bits() {
    let _ = Message::note_off(2, Note::Ab4, 1 << 7);
}

#[test]
fn message_control_change_constructor() {
    let value = Message::control_change(2, 31, 64);

    assert!(matches!(value, Message::ControlChange(_)));
    if let Message::ControlChange(d) = value {
        assert_eq!(d.channel, 2);
        assert_eq!(d.control, 31);
        assert_eq!(d.value, 64);
    }
}

#[test]
#[should_panic]
fn message_control_change_constructor_panics_if_channel_over_4_bits() {
    let _ = Message::control_change(1 << 4, 31, 64);
}

#[test]
#[should_panic]
fn message_control_change_constructor_panics_if_control_over_7_bits() {
    let _ = Message::control_change(2, 1 << 7, 64);
}

#[test]
#[should_panic]
fn message_control_change_constructor_panics_if_value_over_7_bits() {
    let _ = Message::control_change(2, 31, 1 << 7);
}

#[test]
fn message_ack_constructor() {
    let value = Message::ack(1, 14);

    assert!(matches!(value, Message::Ack(_)));
    if let Message::Ack(d) = value {
        assert_eq!(d.device_id, 1);
        assert_eq!(d.packet_num, 14);
    }
}

#[test]
#[should_panic]
fn message_ack_constructor_panics_if_device_id_over_7_bits() {
    let _ = Message::ack(1 << 7, 14);
}

#[test]
#[should_panic]
fn message_ack_constructor_panics_if_packet_num_over_7_bits() {
    let _ = Message::ack(1, 1 << 7);
}

#[test]
fn message_nak_constructor() {
    let value = Message::nak(1, 14);

    assert!(matches!(value, Message::Nak(_)));
    if let Message::Nak(d) = value {
        assert_eq!(d.device_id, 1);
        assert_eq!(d.packet_num, 14);
    }
}

#[test]
#[should_panic]
fn message_nak_constructor_panics_if_device_id_over_7_bits() {
    let _ = Message::nak(1 << 7, 14);
}

#[test]
#[should_panic]
fn message_nak_constructor_panics_if_packet_num_over_7_bits() {
    let _ = Message::nak(1, 1 << 7);
}

#[test]
fn message_wait_constructor() {
    let value = Message::wait(1, 14);

    assert!(matches!(value, Message::Wait(_)));
    if let Message::Wait(d) = value {
        assert_eq!(d.device_id, 1);
        assert_eq!(d.packet_num, 14);
    }
}

#[test]
#[should_panic]
fn message_wait_constructor_panics_if_device_id_over_7_bits() {
    let _ = Message::wait(1 << 7, 14);
}

#[test]
#[should_panic]
fn message_wait_constructor_panics_if_packet_num_over_7_bits() {
    let _ = Message::wait(1, 1 << 7);
}

#[test]
fn message_cancel_constructor() {
    let value = Message::cancel(1, 14);

    assert!(matches!(value, Message::Cancel(_)));
    if let Message::Cancel(d) = value {
        assert_eq!(d.device_id, 1);
        assert_eq!(d.packet_num, 14);
    }
}

#[test]
#[should_panic]
fn message_cancel_constructor_panics_if_device_id_over_7_bits() {
    let _ = Message::cancel(1 << 7, 14);
}

#[test]
#[should_panic]
fn message_cancel_constructor_panics_if_packet_num_over_7_bits() {
    let _ = Message::cancel(1, 1 << 7);
}

#[test]
fn message_eof_constructor() {
    let value = Message::eof(1, 14);

    assert!(matches!(value, Message::Eof(_)));
    if let Message::Eof(d) = value {
        assert_eq!(d.device_id, 1);
        assert_eq!(d.packet_num, 14);
    }
}

#[test]
#[should_panic]
fn message_eof_constructor_panics_if_device_id_over_7_bits() {
    let _ = Message::eof(1 << 7, 14);
}

#[test]
#[should_panic]
fn message_eof_constructor_panics_if_packet_num_over_7_bits() {
    let _ = Message::eof(1, 1 << 7);
}

#[test]
fn message_identity_request_constructor() {
    let value = Message::identity_request(1);

    assert!(matches!(value, Message::IdentityRequest(_)));
    if let Message::IdentityRequest(d) = value {
        assert_eq!(d.device_id, 1);
    }
}

#[test]
#[should_panic]
fn message_identity_request_constructor_panics_if_device_id_over_7_bits() {
    let _ = Message::identity_request(1 << 7);
}

#[test]
fn message_identity_reply_constructor() {
    let value = Message::identity_reply(3, 0x7F, 0x1253, 0x2430, &[1, 9, 8, 5]);

    assert!(matches!(value, Message::IdentityReply(_)));
    if let Message::IdentityReply(d) = value {
        assert_eq!(d.device_id, 3);
        assert_eq!(d.manufacturer_id, 0x7F);
        assert_eq!(d.device_family_code, 0x1253);
        assert_eq!(d.device_family_member_code, 0x2430);
        assert_eq!(d.software_rev, [1, 9, 8, 5]);
    }
}

#[test]
#[should_panic]
fn message_identity_reply_constructor_panics_if_device_id_over_7_bits() {
    let _ = Message::identity_reply(1 << 7, 0x7F, 0x1253, 0x2430, &[1, 9, 8, 5]);
}

#[test]
#[should_panic]
fn message_identity_reply_constructor_panics_if_manufacturer_id_over_7_bits() {
    let _ = Message::identity_reply(3, 1 << 7, 0x1253, 0x2430, &[1, 9, 8, 5]);
}

#[test]
#[should_panic]
fn message_identity_reply_constructor_panics_if_device_family_code_over_14_bits() {
    let _ = Message::identity_reply(3, 0x7F, 1 << 14, 0x2430, &[1, 9, 8, 5]);
}

#[test]
#[should_panic]
fn message_identity_reply_constructor_panics_if_device_family_member_code_over_14_bits() {
    let _ = Message::identity_reply(3, 0x7F, 0x1253, 1 << 14, &[1, 9, 8, 5]);
}

#[test]
#[should_panic]
fn message_identity_reply_constructor_panics_if_version_contains_value_over_7_bits() {
    let _ = Message::identity_reply(3, 0x7F, 0x1253, 0x2430, &[1, 9, 1 << 7, 5]);
}

#[test]
#[should_panic]
fn message_identity_reply_constructor_panics_if_version_too_short() {
    let _ = Message::identity_reply(3, 0x7F, 0x1253, 0x2430, &[1, 9, 8]);
}

#[test]
#[should_panic]
fn message_identity_reply_constructor_panics_if_version_too_long() {
    let _ = Message::identity_reply(3, 0x7F, 0x1253, 0x2430, &[1, 9, 8, 5, 6]);
}

#[test]
fn message_file_dump_header_constructor() {
    let value = Message::file_dump_header(2, 1, 1_382_550, "TEXT");

    assert!(matches!(value, Message::FileDumpHeader(_)));
    if let Message::FileDumpHeader(d) = value {
        assert_eq!(d.device_id, 2);
        assert_eq!(d.source_id, 1);
        assert_eq!(d.length, 1_382_550);
        assert_eq!(d.raw_file_type, hex!("54 45 58 54"));
    }
}

#[test]
#[should_panic]
fn message_file_dump_header_constructor_panics_if_device_id_over_7_bits() {
    let _ = Message::file_dump_header(1 << 7, 1, 1_382_550, "TEXT");
}

#[test]
#[should_panic]
fn message_file_dump_header_constructor_panics_if_source_id_over_7_bits() {
    let _ = Message::file_dump_header(2, 1 << 7, 1_382_550, "TEXT");
}

#[test]
#[should_panic]
fn message_file_dump_header_constructor_panics_if_length_over_28_bits() {
    let _ = Message::file_dump_header(2, 1, 1 << 28, "TEXT");
}

#[test]
#[should_panic]
fn message_file_dump_header_constructor_panics_if_file_type_too_short() {
    let _ = Message::file_dump_header(2, 1, 1_382_550, "BIN");
}

#[test]
#[should_panic]
fn message_file_dump_header_constructor_panics_if_file_type_too_long() {
    let _ = Message::file_dump_header(2, 1, 1_382_550, "MIDIEX");
}

#[test]
#[should_panic]
fn message_file_dump_header_constructor_panics_if_file_type_not_ascii() {
    let _ = Message::file_dump_header(2, 1, 1_382_550, "ÀBI");
}

#[test]
fn message_file_dump_packet_constructor() {
    let value = Message::file_dump_packet(2, 110, &hex!("7f 00 ff 56"));
    let mut data = [0; 112];
    data[..4].copy_from_slice(&hex!("7f 00 ff 56"));

    assert!(matches!(value, Message::FileDumpPacket(_)));
    if let Message::FileDumpPacket(d) = value {
        assert_eq!(d.device_id, 2);
        assert_eq!(d.packet_num, 110);
        assert!(d.checksum_ok);
        assert_eq!(d.data, data);
        assert_eq!(d.data_size, 4);
    }
}

#[test]
#[should_panic]
fn message_file_dump_packet_constructor_device_id_over_7_bits() {
    let _ = Message::file_dump_packet(1 << 7, 110, &hex!("7f 00 ff 56"));
}

#[test]
#[should_panic]
fn message_file_dump_packet_constructor_packet_num_over_7_bits() {
    let _ = Message::file_dump_packet(2, 1 << 7, &hex!("7f 00 ff 56"));
}

#[test]
#[should_panic]
fn message_file_dump_packet_constructor_panics_if_data_too_long() {
    let _ = Message::file_dump_packet(2, 110, &[0xCC; 113]);
}

#[test]
fn message_write_note_on() {
    let mut output = CircularBuffer::<{ Message::MAX_LENGTH }, u8>::new();
    let message = Message::note_on(1, Note::C4, 64);
    let result = message.write(&mut output);
    assert_eq!(result, Ok(()));
    assert_eq!(output, hex!("91 3c 40"));
}

#[test]
fn message_write_note_off() {
    let mut output = CircularBuffer::<{ Message::MAX_LENGTH }, u8>::new();
    let message = Message::note_off(1, Note::C4, 64);
    let result = message.write(&mut output);
    assert_eq!(result, Ok(()));
    assert_eq!(output, hex!("81 3c 40"));
}

#[test]
fn message_write_control_change() {
    let mut output = CircularBuffer::<{ Message::MAX_LENGTH }, u8>::new();
    let message = Message::control_change(1, 7, 32);
    let result = message.write(&mut output);
    assert_eq!(result, Ok(()));
    assert_eq!(output, hex!("B1 07 20"));
}

#[test]
fn message_write_ack() {
    let mut output = CircularBuffer::<64, u8>::new();
    let message = Message::ack(5, 31);
    let result = message.write(&mut output);
    assert_eq!(result, Ok(()));
    assert_eq!(output, hex!("f0 7e 05 7f 1f f7"));
    assert_eq!(Some(message), Message::read_sysex(&mut output));
}

#[test]
fn message_write_nak() {
    let mut output = CircularBuffer::<64, u8>::new();
    let message = Message::nak(5, 31);
    let result = message.write(&mut output);
    assert_eq!(result, Ok(()));
    assert_eq!(output, hex!("f0 7e 05 7e 1f f7"));
    assert_eq!(Some(message), Message::read_sysex(&mut output));
}

#[test]
fn message_write_wait() {
    let mut output = CircularBuffer::<64, u8>::new();
    let message = Message::wait(5, 31);
    let result = message.write(&mut output);
    assert_eq!(result, Ok(()));
    assert_eq!(output, hex!("f0 7e 05 7c 1f f7"));
    assert_eq!(Some(message), Message::read_sysex(&mut output));
}

#[test]
fn message_write_cancel() {
    let mut output = CircularBuffer::<64, u8>::new();
    let message = Message::cancel(5, 31);
    let result = message.write(&mut output);
    assert_eq!(result, Ok(()));
    assert_eq!(output, hex!("f0 7e 05 7d 1f f7"));
    assert_eq!(Some(message), Message::read_sysex(&mut output));
}

#[test]
fn message_write_eof() {
    let mut output = CircularBuffer::<64, u8>::new();
    let message = Message::eof(5, 31);
    let result = message.write(&mut output);
    assert_eq!(result, Ok(()));
    assert_eq!(output, hex!("f0 7e 05 7b 1f f7"));
    assert_eq!(Some(message), Message::read_sysex(&mut output));
}

#[test]
fn message_write_file_dump_header() {
    let mut output = CircularBuffer::<64, u8>::new();
    const DEVICE_ID: u8 = 3;
    const SOURCE_ID: u8 = 4;
    const LENGTH: u32 = 0x03FA;
    let message = Message::file_dump_header(DEVICE_ID, SOURCE_ID, LENGTH, "TYPE");
    let result = message.write(&mut output);
    assert_eq!(result, Ok(()));
    assert_eq!(
        output,
        hex!("f0 7e 03 07 01 04  54 59 50 45  7a 07 00 00  f7")
    );
    assert_eq!(Some(message), Message::read_sysex(&mut output));
}

#[test]
fn message_write_file_dump_packet_with_payload_divisible_by_7_bytes() {
    let mut output = CircularBuffer::<64, u8>::new();
    const DEVICE_ID: u8 = 3;
    const PACKET_NUM: u8 = 0x3A;
    let payload = hex!("20 03 40 0c d0 0e f0");
    let message = Message::file_dump_packet(DEVICE_ID, PACKET_NUM, &payload);
    let result = message.write(&mut output);
    assert_eq!(result, Ok(()));
    assert_eq!(
        output,
        hex!("f0 7e 03 07 02 3a  07  05 20 03 40 0c 50 0e 70  01 f7")
    );
    assert_eq!(Some(message), Message::read_sysex(&mut output));
}

#[test]
fn message_write_file_dump_packet_with_payload_not_divisible_by_7_bytes() {
    let mut output = CircularBuffer::<64, u8>::new();
    const DEVICE_ID: u8 = 3;
    const PACKET_NUM: u8 = 0x3A;
    let payload = hex!("20 03 40 0c d0 0e f0  f0 80 77");
    let message = Message::file_dump_packet(DEVICE_ID, PACKET_NUM, &payload);
    let result = message.write(&mut output);
    assert_eq!(result, Ok(()));
    assert_eq!(
        output,
        hex!("f0 7e 03 07 02 3a  0b  05 20 03 40 0c 50 0e 70  60 70 00 77  6a f7")
    );
    assert_eq!(Some(message), Message::read_sysex(&mut output));
}

#[test]
fn message_read_sysex_returns_none_if_no_data() {
    let mut data = CircularBuffer::<64, u8>::new();
    let result = Message::read_sysex(&mut data);
    assert_eq!(result, None);
}

#[test]
fn message_read_sysex_returns_none_if_no_sysex_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("90 3c 7f 80 3c 7f"));
    let result = Message::read_sysex(&mut data);
    assert_eq!(result, None);
}

#[test]
fn message_read_sysex_discards_bytes_if_no_sysex_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("90 3c 7f 80 3c 7f"));
    let _ = Message::read_sysex(&mut data);
    assert_eq!(data, []);
}

#[test]
fn message_read_sysex_returns_none_if_sysex_data_is_incomplete() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7f"));
    let result = Message::read_sysex(&mut data);
    assert_eq!(result, None);
}

#[test]
fn message_read_sysex_does_not_consume_incomplete_sysex_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e"));
    let _ = Message::read_sysex(&mut data);
    assert_eq!(data, hex!("f0 7e"));

    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7f"));
    let _ = Message::read_sysex(&mut data);
    assert_eq!(data, hex!("f0 7e 03 7f"));
}

#[test]
fn message_read_sysex_consumes_sysex_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7f 3b f7"));
    let _ = Message::read_sysex(&mut data);
    assert_eq!(data, []);
}

#[test]
fn message_read_sysex_consumes_only_one_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7f 3b f7   f0 7e 03 7f 3c f7"));
    let _ = Message::read_sysex(&mut data);
    assert_eq!(data, hex!("f0 7e 03 7f 3c f7"));
}

#[test]
fn message_read_sysex_identifies_ack_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7f 3b f7"));
    let result = Message::read_sysex(&mut data);
    assert!(matches!(result, Some(Message::Ack(_))));
}

#[test]
fn message_read_sysex_reads_message_after_ignored_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("90 3c 7f   f0 7e 03 7f 3b f7"));
    let result = Message::read_sysex(&mut data);
    assert!(matches!(result, Some(Message::Ack(_))));
    assert_eq!(data, []);
}

#[test]
fn message_read_sysex_reads_message_after_incomplete_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7f   f0 7e 03 7f 3b f7"));
    let result = Message::read_sysex(&mut data);
    assert!(matches!(result, Some(Message::Ack(_))));
    assert_eq!(data, []);
}

#[test]
fn message_read_sysex_ignores_non_universal_messages() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 55 03 7f 3b f7"));
    let result = Message::read_sysex(&mut data);
    assert!(result.is_none());
    assert_eq!(data, []);

    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 00 10 56 03 7f 3b f7"));
    let result = Message::read_sysex(&mut data);
    assert!(result.is_none());
    assert_eq!(data, []);
}

#[test]
fn message_read_sysex_reads_message_after_non_universal_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 55 03 7f 3b f7   f0 7e 03 7f 3b f7"));
    let result = Message::read_sysex(&mut data);
    assert!(matches!(result, Some(Message::Ack(_))));
    assert_eq!(data, []);
}

#[test]
fn message_read_sysex_parses_ack_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7f 3b f7"));
    let result = Message::read_sysex(&mut data);
    let expected = Some(Message::ack(0x03, 0x3b));
    assert_eq!(result, expected);
}

#[test]
fn message_read_sysex_identifies_nak_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7e 3b f7"));
    let result = Message::read_sysex(&mut data);
    assert!(matches!(result, Some(Message::Nak(_))));
}

#[test]
fn message_read_sysex_parses_nak_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7e 3b f7"));
    let result = Message::read_sysex(&mut data);
    let expected = Some(Message::nak(0x03, 0x3b));
    assert_eq!(result, expected);
}

#[test]
fn message_read_sysex_ignores_system_real_time_messages() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 f8 7e fa 03 fb fc 7f fe ff 3b f7"));
    let result = Message::read_sysex(&mut data);
    let expected = Some(Message::ack(0x03, 0x3b));
    assert_eq!(result, expected);
    assert_eq!(data, []);
}

#[test]
fn message_read_sysex_discards_unrecognized_messages() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 33 3b f7"));
    let result = Message::read_sysex(&mut data);
    assert_eq!(result, None);
    assert_eq!(data, []);
}

#[test]
fn message_read_sysex_reads_message_unrecognized_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 33 3b f7   f0 7e 03 7f 3b f7"));
    let result = Message::read_sysex(&mut data);
    assert!(matches!(result, Some(Message::Ack(_))));
    assert_eq!(data, []);
}

#[test]
fn message_read_sysex_identifies_wait_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7c 3b f7"));
    let result = Message::read_sysex(&mut data);
    assert!(matches!(result, Some(Message::Wait(_))));
}

#[test]
fn message_read_sysex_parses_wait_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7c 3b f7"));
    let result = Message::read_sysex(&mut data);
    let expected = Some(Message::wait(0x03, 0x3b));
    assert_eq!(result, expected);
}

#[test]
fn message_read_sysex_identifies_cancel_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7d 3b f7"));
    let result = Message::read_sysex(&mut data);
    assert!(matches!(result, Some(Message::Cancel(_))));
}

#[test]
fn message_read_sysex_parses_cancel_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7d 3b f7"));
    let result = Message::read_sysex(&mut data);
    let expected = Some(Message::cancel(0x03, 0x3b));
    assert_eq!(result, expected);
}

#[test]
fn message_read_sysex_identifies_eof_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7b 3b f7"));
    let result = Message::read_sysex(&mut data);
    assert!(matches!(result, Some(Message::Eof(_))));
}

#[test]
fn message_read_sysex_parses_eof_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7b 3b f7"));
    let result = Message::read_sysex(&mut data);
    let expected = Some(Message::eof(0x03, 0x3b));
    assert_eq!(result, expected);
}

#[test]
fn message_read_sysex_identifies_file_dump_header_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!(
        "f0 7e 03 07 01 00  42 49 4e 20  1e 64 77 32  66 69 6c 65 2e 62 69 6e f7"
    ));
    let result = Message::read_sysex(&mut data);
    assert!(matches!(
        result,
        Some(Message::FileDumpHeader(_))
    ));
}

#[test]
fn message_read_sysex_parses_file_dump_header_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!(
        "f0 7e 03 07 01 01  42 49 4e 20  1e 24 77 32  66 69 6c 65 2e 62 69 6e f7"
    ));
    let result = Message::read_sysex(&mut data);
    let expected = Some(Message::file_dump_header(
        3, 1, 0x65DD21E, "BIN ",
    ));
    assert_eq!(result, expected);
}

#[test]
fn message_read_sysex_consumes_all_file_dump_header_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!(
        "f0 7e 03 07 01 01  42 49 4e 20  1e 24 77 32  66 69 6c 65 2e 62 69 6e f7   f0 7e"
    ));
    let _ = Message::read_sysex(&mut data);
    assert_eq!(data, hex!("f0 7e"));
}

#[test]
fn file_dump_header_data_file_type_as_str() {
    let data = Message::file_dump_header(0, 0, 0, "BIN ");
    if let Message::FileDumpHeader(d) = data {
        assert_eq!(d.raw_file_type, hex!("42 49 4e 20"));
        assert_eq!(d.file_type(), "BIN ");
    }
}

#[test]
fn message_read_sysex_identifies_file_dump_packet_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!(
        "f0 7e 03 07 02 71 0f  00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60  5c f7"
    ));
    let result = Message::read_sysex(&mut data);
    assert!(matches!(
        result,
        Some(Message::FileDumpPacket(_))
    ));
}

#[test]
fn message_read_sysex_parses_file_dump_packet_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!(
        "f0 7e 03 07 02 71 0f  00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60  5c f7"
    ));
    let result = Message::read_sysex(&mut data);

    let expected = Some(Message::file_dump_packet(
        3,
        113,
        &hex!("01 20 03 40 05 60 07  80 09 A0 0B C0 0D E0"),
    ));
    assert_eq!(result, expected);
}

#[test]
fn message_read_sysex_parses_max_length_file_dump_packet() {
    let mut data = CircularBuffer::<{ Message::MAX_LENGTH }, u8>::from(hex!(
        "f0 7e 03 07 02 71 7f"
        "00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60"
        "00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60"
        "00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60"
        "00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60"
        "00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60"
        "00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60"
        "00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60"
        "00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60"
        "76 f7"));
    let result = Message::read_sysex(&mut data);

    let expected = Some(Message::file_dump_packet(
        3,
        113,
        &hex!(
            "01 20 03 40 05 60 07  80 09 A0 0B C0 0D E0"
            "01 20 03 40 05 60 07  80 09 A0 0B C0 0D E0"
            "01 20 03 40 05 60 07  80 09 A0 0B C0 0D E0"
            "01 20 03 40 05 60 07  80 09 A0 0B C0 0D E0"
            "01 20 03 40 05 60 07  80 09 A0 0B C0 0D E0"
            "01 20 03 40 05 60 07  80 09 A0 0B C0 0D E0"
            "01 20 03 40 05 60 07  80 09 A0 0B C0 0D E0"
            "01 20 03 40 05 60 07  80 09 A0 0B C0 0D E0"
        ),
    ));
    assert_eq!(result, expected);
}

#[test]
fn message_read_sysex_rejects_byte_count_over_127() {
    let mut data = CircularBuffer::<256, u8>::from(hex!(
        "f0 7e 03 07 02 71 9f"
        "00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60"
        "00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60"
        "00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60"
        "00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60"
        "00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60"
        "00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60"
        "00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60"
        "00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60"
        "00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60"
        "00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60"
        "76 f7"));
    let result = Message::read_sysex(&mut data);
    assert_eq!(result, None);
    assert_eq!(data, []);
}

#[test]
fn message_read_sysex_parses_odd_length_file_dump_packet() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 07 02 71 03  00 01 20 03  28 f7"));
    let result = Message::read_sysex(&mut data);

    let expected = Some(Message::file_dump_packet(
        3,
        113,
        &hex!("01 20 03"),
    ));
    assert_eq!(result, expected);
}

#[test]
fn message_read_sysex_detects_file_dump_packet_checksum_validity() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 07 02 71 03  00 01 20 03  28 f7"));
    let result = Message::read_sysex(&mut data).unwrap();
    assert!(matches!(
        result,
        Message::FileDumpPacket(FileDumpPacketData {
            checksum_ok: true,
            ..
        })
    ));

    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 07 02 71 03  00 01 20 03  29 f7"));
    let result = Message::read_sysex(&mut data).unwrap();
    assert!(matches!(
        result,
        Message::FileDumpPacket(FileDumpPacketData {
            checksum_ok: false,
            ..
        })
    ));
}
