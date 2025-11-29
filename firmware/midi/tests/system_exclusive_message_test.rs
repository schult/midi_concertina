use circular_buffer::CircularBuffer;
use hex_literal::hex;
use midi::system_exclusive_message::*;

#[test]
fn sysex_read_returns_none_if_no_data() {
    let mut data = CircularBuffer::<64, u8>::new();
    let result = SystemExclusiveMessage::read(&mut data);
    assert_eq!(result, None);
}

#[test]
fn sysex_read_returns_none_if_no_sysex_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("90 3c 7f 80 3c 7f"));
    let result = SystemExclusiveMessage::read(&mut data);
    assert_eq!(result, None);
}

#[test]
fn sysex_read_discards_bytes_if_no_sysex_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("90 3c 7f 80 3c 7f"));
    let _ = SystemExclusiveMessage::read(&mut data);
    assert_eq!(data, []);
}

#[test]
fn sysex_read_returns_none_if_sysex_data_is_incomplete() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7f"));
    let result = SystemExclusiveMessage::read(&mut data);
    assert_eq!(result, None);
}

#[test]
fn sysex_read_does_not_consume_incomplete_sysex_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e"));
    let _ = SystemExclusiveMessage::read(&mut data);
    assert_eq!(data, hex!("f0 7e"));

    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7f"));
    let _ = SystemExclusiveMessage::read(&mut data);
    assert_eq!(data, hex!("f0 7e 03 7f"));
}

#[test]
fn sysex_read_consumes_sysex_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7f 3b f7"));
    let _ = SystemExclusiveMessage::read(&mut data);
    assert_eq!(data, []);
}

#[test]
fn sysex_read_consumes_only_one_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7f 3b f7   f0 7e 03 7f 3c f7"));
    let _ = SystemExclusiveMessage::read(&mut data);
    assert_eq!(data, hex!("f0 7e 03 7f 3c f7"));
}

#[test]
fn sysex_read_identifies_ack_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7f 3b f7"));
    let result = SystemExclusiveMessage::read(&mut data);
    assert!(matches!(result, Some(SystemExclusiveMessage::Ack(_))));
}

#[test]
fn sysex_reads_message_after_ignored_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("90 3c 7f   f0 7e 03 7f 3b f7"));
    let result = SystemExclusiveMessage::read(&mut data);
    assert!(matches!(result, Some(SystemExclusiveMessage::Ack(_))));
    assert_eq!(data, []);
}

#[test]
fn sysex_reads_message_after_incomplete_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7f   f0 7e 03 7f 3b f7"));
    let result = SystemExclusiveMessage::read(&mut data);
    assert!(matches!(result, Some(SystemExclusiveMessage::Ack(_))));
    assert_eq!(data, []);
}

#[test]
fn sysex_ignores_non_universal_messages() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 55 03 7f 3b f7"));
    let result = SystemExclusiveMessage::read(&mut data);
    assert!(matches!(result, None));
    assert_eq!(data, []);

    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 00 10 56 03 7f 3b f7"));
    let result = SystemExclusiveMessage::read(&mut data);
    assert!(matches!(result, None));
    assert_eq!(data, []);
}

#[test]
fn sysex_reads_message_after_non_universal_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 55 03 7f 3b f7   f0 7e 03 7f 3b f7"));
    let result = SystemExclusiveMessage::read(&mut data);
    assert!(matches!(result, Some(SystemExclusiveMessage::Ack(_))));
    assert_eq!(data, []);
}

#[test]
fn sysex_read_parses_ack_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7f 3b f7"));
    let result = SystemExclusiveMessage::read(&mut data);
    let expected = Some(SystemExclusiveMessage::ack(0x03, 0x3b));
    assert_eq!(result, expected);
}

#[test]
fn sysex_read_identifies_nak_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7e 3b f7"));
    let result = SystemExclusiveMessage::read(&mut data);
    assert!(matches!(result, Some(SystemExclusiveMessage::Nak(_))));
}

#[test]
fn sysex_read_parses_nak_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7e 3b f7"));
    let result = SystemExclusiveMessage::read(&mut data);
    let expected = Some(SystemExclusiveMessage::nak(0x03, 0x3b));
    assert_eq!(result, expected);
}

#[test]
fn sysex_read_ignores_system_real_time_messages() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 f8 7e fa 03 fb fc 7f fe ff 3b f7"));
    let result = SystemExclusiveMessage::read(&mut data);
    let expected = Some(SystemExclusiveMessage::ack(0x03, 0x3b));
    assert_eq!(result, expected);
    assert_eq!(data, []);
}

#[test]
fn sysex_read_discards_unrecognized_messages() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 33 3b f7"));
    let result = SystemExclusiveMessage::read(&mut data);
    assert_eq!(result, None);
    assert_eq!(data, []);
}

#[test]
fn sysex_reads_message_unrecognized_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 33 3b f7   f0 7e 03 7f 3b f7"));
    let result = SystemExclusiveMessage::read(&mut data);
    assert!(matches!(result, Some(SystemExclusiveMessage::Ack(_))));
    assert_eq!(data, []);
}

#[test]
fn sysex_read_identifies_wait_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7c 3b f7"));
    let result = SystemExclusiveMessage::read(&mut data);
    assert!(matches!(result, Some(SystemExclusiveMessage::Wait(_))));
}

#[test]
fn sysex_read_parses_wait_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7c 3b f7"));
    let result = SystemExclusiveMessage::read(&mut data);
    let expected = Some(SystemExclusiveMessage::wait(0x03, 0x3b));
    assert_eq!(result, expected);
}

#[test]
fn sysex_read_identifies_cancel_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7d 3b f7"));
    let result = SystemExclusiveMessage::read(&mut data);
    assert!(matches!(result, Some(SystemExclusiveMessage::Cancel(_))));
}

#[test]
fn sysex_read_parses_cancel_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7d 3b f7"));
    let result = SystemExclusiveMessage::read(&mut data);
    let expected = Some(SystemExclusiveMessage::cancel(0x03, 0x3b));
    assert_eq!(result, expected);
}

#[test]
fn sysex_read_identifies_eof_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7b 3b f7"));
    let result = SystemExclusiveMessage::read(&mut data);
    assert!(matches!(result, Some(SystemExclusiveMessage::Eof(_))));
}

#[test]
fn sysex_read_parses_eof_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 7b 3b f7"));
    let result = SystemExclusiveMessage::read(&mut data);
    let expected = Some(SystemExclusiveMessage::eof(0x03, 0x3b));
    assert_eq!(result, expected);
}

#[test]
fn sysex_read_identifies_file_dump_header_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!(
        "f0 7e 03 07 01 00  42 49 4e 20  1e 64 77 32  66 69 6c 65 2e 62 69 6e f7"
    ));
    let result = SystemExclusiveMessage::read(&mut data);
    assert!(matches!(
        result,
        Some(SystemExclusiveMessage::FileDumpHeader(_))
    ));
}

#[test]
fn sysex_read_parses_file_dump_header_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!(
        "f0 7e 03 07 01 01  42 49 4e 20  1e 24 77 32  66 69 6c 65 2e 62 69 6e f7"
    ));
    let result = SystemExclusiveMessage::read(&mut data);
    let expected = Some(SystemExclusiveMessage::file_dump_header(
        3, 1, 0x65DD21E, "BIN ",
    ));
    assert_eq!(result, expected);
}

#[test]
fn sysex_read_consumes_all_file_dump_header_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!(
        "f0 7e 03 07 01 01  42 49 4e 20  1e 24 77 32  66 69 6c 65 2e 62 69 6e f7   f0 7e"
    ));
    let _ = SystemExclusiveMessage::read(&mut data);
    assert_eq!(data, hex!("f0 7e"));
}

#[test]
fn file_dump_header_data_file_type_as_str() {
    let data = SystemExclusiveMessage::file_dump_header(0, 0, 0, "BIN ");
    if let SystemExclusiveMessage::FileDumpHeader(d) = data {
        assert_eq!(d.raw_file_type, hex!("42 49 4e 20"));
        assert_eq!(d.file_type(), "BIN ");
    }
}

#[test]
fn sysex_read_identifies_file_dump_packet_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!(
        "f0 7e 03 07 02 71 0f  00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60  5c f7"
    ));
    let result = SystemExclusiveMessage::read(&mut data);
    assert!(matches!(
        result,
        Some(SystemExclusiveMessage::FileDumpPacket(_))
    ));
}

#[test]
fn sysex_read_parses_file_dump_packet_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!(
        "f0 7e 03 07 02 71 0f  00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60  5c f7"
    ));
    let result = SystemExclusiveMessage::read(&mut data);

    let expected = Some(SystemExclusiveMessage::file_dump_packet(
        3,
        113,
        &hex!("01 20 03 40 05 60 07  80 09 A0 0B C0 0D E0"),
    ));
    assert_eq!(result, expected);
}

#[test]
fn sysex_read_parses_max_length_file_dump_packet() {
    let mut data = CircularBuffer::<{ SystemExclusiveMessage::MAX_LENGTH }, u8>::from(hex!(
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
    let result = SystemExclusiveMessage::read(&mut data);

    let expected = Some(SystemExclusiveMessage::file_dump_packet(
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
fn sysex_read_rejects_byte_count_over_127() {
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
    let result = SystemExclusiveMessage::read(&mut data);
    assert_eq!(result, None);
    assert_eq!(data, []);
}

#[test]
fn sysex_read_parses_odd_length_file_dump_packet() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 07 02 71 03  00 01 20 03  28 f7"));
    let result = SystemExclusiveMessage::read(&mut data);

    let expected = Some(SystemExclusiveMessage::file_dump_packet(
        3,
        113,
        &hex!("01 20 03"),
    ));
    assert_eq!(result, expected);
}

#[test]
fn sysex_read_detects_file_dump_packet_checksum_validity() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 07 02 71 03  00 01 20 03  28 f7"));
    let result = SystemExclusiveMessage::read(&mut data).unwrap();
    assert!(matches!(
        result,
        SystemExclusiveMessage::FileDumpPacket(FileDumpPacketData {
            checksum_ok: true,
            ..
        })
    ));

    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 07 02 71 03  00 01 20 03  29 f7"));
    let result = SystemExclusiveMessage::read(&mut data).unwrap();
    assert!(matches!(
        result,
        SystemExclusiveMessage::FileDumpPacket(FileDumpPacketData {
            checksum_ok: false,
            ..
        })
    ));
}

#[test]
fn sysex_ack_constructor() {
    let value = SystemExclusiveMessage::ack(1, 14);

    assert!(matches!(value, SystemExclusiveMessage::Ack(_)));
    if let SystemExclusiveMessage::Ack(d) = value {
        assert_eq!(d.device_id, 1);
        assert_eq!(d.packet_num, 14);
    }
}

#[test]
#[should_panic]
fn sysex_ack_constructor_panics_if_device_id_over_7_bits() {
    let _ = SystemExclusiveMessage::ack(1 << 7, 14);
}

#[test]
#[should_panic]
fn sysex_ack_constructor_panics_if_packet_num_over_7_bits() {
    let _ = SystemExclusiveMessage::ack(1, 1 << 7);
}

#[test]
fn sysex_nak_constructor() {
    let value = SystemExclusiveMessage::nak(1, 14);

    assert!(matches!(value, SystemExclusiveMessage::Nak(_)));
    if let SystemExclusiveMessage::Nak(d) = value {
        assert_eq!(d.device_id, 1);
        assert_eq!(d.packet_num, 14);
    }
}

#[test]
#[should_panic]
fn sysex_nak_constructor_panics_if_device_id_over_7_bits() {
    let _ = SystemExclusiveMessage::nak(1 << 7, 14);
}

#[test]
#[should_panic]
fn sysex_nak_constructor_panics_if_packet_num_over_7_bits() {
    let _ = SystemExclusiveMessage::nak(1, 1 << 7);
}

#[test]
fn sysex_wait_constructor() {
    let value = SystemExclusiveMessage::wait(1, 14);

    assert!(matches!(value, SystemExclusiveMessage::Wait(_)));
    if let SystemExclusiveMessage::Wait(d) = value {
        assert_eq!(d.device_id, 1);
        assert_eq!(d.packet_num, 14);
    }
}

#[test]
#[should_panic]
fn sysex_wait_constructor_panics_if_device_id_over_7_bits() {
    let _ = SystemExclusiveMessage::wait(1 << 7, 14);
}

#[test]
#[should_panic]
fn sysex_wait_constructor_panics_if_packet_num_over_7_bits() {
    let _ = SystemExclusiveMessage::wait(1, 1 << 7);
}

#[test]
fn sysex_cancel_constructor() {
    let value = SystemExclusiveMessage::cancel(1, 14);

    assert!(matches!(value, SystemExclusiveMessage::Cancel(_)));
    if let SystemExclusiveMessage::Cancel(d) = value {
        assert_eq!(d.device_id, 1);
        assert_eq!(d.packet_num, 14);
    }
}

#[test]
#[should_panic]
fn sysex_cancel_constructor_panics_if_device_id_over_7_bits() {
    let _ = SystemExclusiveMessage::cancel(1 << 7, 14);
}

#[test]
#[should_panic]
fn sysex_cancel_constructor_panics_if_packet_num_over_7_bits() {
    let _ = SystemExclusiveMessage::cancel(1, 1 << 7);
}

#[test]
fn sysex_eof_constructor() {
    let value = SystemExclusiveMessage::eof(1, 14);

    assert!(matches!(value, SystemExclusiveMessage::Eof(_)));
    if let SystemExclusiveMessage::Eof(d) = value {
        assert_eq!(d.device_id, 1);
        assert_eq!(d.packet_num, 14);
    }
}

#[test]
#[should_panic]
fn sysex_eof_constructor_panics_if_device_id_over_7_bits() {
    let _ = SystemExclusiveMessage::eof(1 << 7, 14);
}

#[test]
#[should_panic]
fn sysex_eof_constructor_panics_if_packet_num_over_7_bits() {
    let _ = SystemExclusiveMessage::eof(1, 1 << 7);
}

#[test]
fn sysex_identity_request_constructor() {
    let value = SystemExclusiveMessage::identity_request(1);

    assert!(matches!(value, SystemExclusiveMessage::IdentityRequest(_)));
    if let SystemExclusiveMessage::IdentityRequest(d) = value {
        assert_eq!(d.device_id, 1);
    }
}

#[test]
#[should_panic]
fn sysex_identity_request_constructor_panics_if_device_id_over_7_bits() {
    let _ = SystemExclusiveMessage::identity_request(1 << 7);
}

#[test]
fn sysex_identity_reply_constructor() {
    let value = SystemExclusiveMessage::identity_reply(3, 0x7F, 0x1253, 0x2430, &[1, 9, 8, 5]);

    assert!(matches!(value, SystemExclusiveMessage::IdentityReply(_)));
    if let SystemExclusiveMessage::IdentityReply(d) = value {
        assert_eq!(d.device_id, 3);
        assert_eq!(d.manufacturer_id, 0x7F);
        assert_eq!(d.device_family_code, 0x1253);
        assert_eq!(d.device_family_member_code, 0x2430);
        assert_eq!(d.software_rev, [1, 9, 8, 5]);
    }
}

#[test]
#[should_panic]
fn sysex_identity_reply_constructor_panics_if_device_id_over_7_bits() {
    let _ = SystemExclusiveMessage::identity_reply(1 << 7, 0x7F, 0x1253, 0x2430, &[1, 9, 8, 5]);
}

#[test]
#[should_panic]
fn sysex_identity_reply_constructor_panics_if_manufacturer_id_over_7_bits() {
    let _ = SystemExclusiveMessage::identity_reply(3, 1 << 7, 0x1253, 0x2430, &[1, 9, 8, 5]);
}

#[test]
#[should_panic]
fn sysex_identity_reply_constructor_panics_if_device_family_code_over_14_bits() {
    let _ = SystemExclusiveMessage::identity_reply(3, 0x7F, 1 << 14, 0x2430, &[1, 9, 8, 5]);
}

#[test]
#[should_panic]
fn sysex_identity_reply_constructor_panics_if_device_family_member_code_over_14_bits() {
    let _ = SystemExclusiveMessage::identity_reply(3, 0x7F, 0x1253, 1 << 14, &[1, 9, 8, 5]);
}

#[test]
#[should_panic]
fn sysex_identity_reply_constructor_panics_if_version_contains_value_over_7_bits() {
    let _ = SystemExclusiveMessage::identity_reply(3, 0x7F, 0x1253, 0x2430, &[1, 9, 1 << 7, 5]);
}

#[test]
#[should_panic]
fn sysex_identity_reply_constructor_panics_if_version_too_short() {
    let _ = SystemExclusiveMessage::identity_reply(3, 0x7F, 0x1253, 0x2430, &[1, 9, 8]);
}

#[test]
#[should_panic]
fn sysex_identity_reply_constructor_panics_if_version_too_long() {
    let _ = SystemExclusiveMessage::identity_reply(3, 0x7F, 0x1253, 0x2430, &[1, 9, 8, 5, 6]);
}

#[test]
fn sysex_file_dump_header_constructor() {
    let value = SystemExclusiveMessage::file_dump_header(2, 1, 1_382_550, &"TEXT");

    assert!(matches!(value, SystemExclusiveMessage::FileDumpHeader(_)));
    if let SystemExclusiveMessage::FileDumpHeader(d) = value {
        assert_eq!(d.device_id, 2);
        assert_eq!(d.source_id, 1);
        assert_eq!(d.length, 1_382_550);
        assert_eq!(d.raw_file_type, hex!("54 45 58 54"));
    }
}

#[test]
#[should_panic]
fn sysex_file_dump_header_constructor_panics_if_device_id_over_7_bits() {
    let _ = SystemExclusiveMessage::file_dump_header(1 << 7, 1, 1_382_550, &"TEXT");
}

#[test]
#[should_panic]
fn sysex_file_dump_header_constructor_panics_if_source_id_over_7_bits() {
    let _ = SystemExclusiveMessage::file_dump_header(2, 1 << 7, 1_382_550, &"TEXT");
}

#[test]
#[should_panic]
fn sysex_file_dump_header_constructor_panics_if_length_over_28_bits() {
    let _ = SystemExclusiveMessage::file_dump_header(2, 1, 1 << 28, &"TEXT");
}

#[test]
#[should_panic]
fn sysex_file_dump_header_constructor_panics_if_file_type_too_short() {
    let _ = SystemExclusiveMessage::file_dump_header(2, 1, 1_382_550, &"BIN");
}

#[test]
#[should_panic]
fn sysex_file_dump_header_constructor_panics_if_file_type_too_long() {
    let _ = SystemExclusiveMessage::file_dump_header(2, 1, 1_382_550, &"MIDIEX");
}

#[test]
#[should_panic]
fn sysex_file_dump_header_constructor_panics_if_file_type_not_ascii() {
    let _ = SystemExclusiveMessage::file_dump_header(2, 1, 1_382_550, &"ÀBI");
}

#[test]
fn sysex_file_dump_packet_constructor() {
    let value = SystemExclusiveMessage::file_dump_packet(2, 110, &hex!("7f 00 ff 56"));
    let mut data = [0; 112];
    data[..4].copy_from_slice(&hex!("7f 00 ff 56"));

    assert!(matches!(value, SystemExclusiveMessage::FileDumpPacket(_)));
    if let SystemExclusiveMessage::FileDumpPacket(d) = value {
        assert_eq!(d.device_id, 2);
        assert_eq!(d.packet_num, 110);
        assert_eq!(d.checksum_ok, true);
        assert_eq!(d.data, data);
        assert_eq!(d.data_size, 4);
    }
}

#[test]
#[should_panic]
fn sysex_file_dump_packet_constructor_device_id_over_7_bits() {
    let _ = SystemExclusiveMessage::file_dump_packet(1 << 7, 110, &hex!("7f 00 ff 56"));
}

#[test]
#[should_panic]
fn sysex_file_dump_packet_constructor_packet_num_over_7_bits() {
    let _ = SystemExclusiveMessage::file_dump_packet(2, 1 << 7, &hex!("7f 00 ff 56"));
}

#[test]
#[should_panic]
fn sysex_file_dump_packet_constructor_panics_if_data_too_long() {
    let _ = SystemExclusiveMessage::file_dump_packet(2, 110, &[0xCC; 113]);
}

#[test]
fn sysex_write_ack() {
    let mut output = CircularBuffer::<64, u8>::new();
    let message = SystemExclusiveMessage::ack(5, 31);
    let result = message.write(&mut output);
    assert_eq!(result, Ok(()));
    assert_eq!(output, hex!("f0 7e 05 7f 1f f7"));
    assert_eq!(Some(message), SystemExclusiveMessage::read(&mut output));
}

#[test]
fn sysex_write_nak() {
    let mut output = CircularBuffer::<64, u8>::new();
    let message = SystemExclusiveMessage::nak(5, 31);
    let result = message.write(&mut output);
    assert_eq!(result, Ok(()));
    assert_eq!(output, hex!("f0 7e 05 7e 1f f7"));
    assert_eq!(Some(message), SystemExclusiveMessage::read(&mut output));
}

#[test]
fn sysex_write_wait() {
    let mut output = CircularBuffer::<64, u8>::new();
    let message = SystemExclusiveMessage::wait(5, 31);
    let result = message.write(&mut output);
    assert_eq!(result, Ok(()));
    assert_eq!(output, hex!("f0 7e 05 7c 1f f7"));
    assert_eq!(Some(message), SystemExclusiveMessage::read(&mut output));
}

#[test]
fn sysex_write_cancel() {
    let mut output = CircularBuffer::<64, u8>::new();
    let message = SystemExclusiveMessage::cancel(5, 31);
    let result = message.write(&mut output);
    assert_eq!(result, Ok(()));
    assert_eq!(output, hex!("f0 7e 05 7d 1f f7"));
    assert_eq!(Some(message), SystemExclusiveMessage::read(&mut output));
}

#[test]
fn sysex_write_eof() {
    let mut output = CircularBuffer::<64, u8>::new();
    let message = SystemExclusiveMessage::eof(5, 31);
    let result = message.write(&mut output);
    assert_eq!(result, Ok(()));
    assert_eq!(output, hex!("f0 7e 05 7b 1f f7"));
    assert_eq!(Some(message), SystemExclusiveMessage::read(&mut output));
}

#[test]
fn sysex_write_file_dump_header() {
    let mut output = CircularBuffer::<64, u8>::new();
    const DEVICE_ID: u8 = 3;
    const SOURCE_ID: u8 = 4;
    const LENGTH: u32 = 0x03FA;
    let message = SystemExclusiveMessage::file_dump_header(DEVICE_ID, SOURCE_ID, LENGTH, "TYPE");
    let result = message.write(&mut output);
    assert_eq!(result, Ok(()));
    assert_eq!(
        output,
        hex!("f0 7e 03 07 01 04  54 59 50 45  7a 07 00 00  f7")
    );
    assert_eq!(Some(message), SystemExclusiveMessage::read(&mut output));
}

#[test]
fn sysex_write_file_dump_packet_with_payload_divisible_by_7_bytes() {
    let mut output = CircularBuffer::<64, u8>::new();
    const DEVICE_ID: u8 = 3;
    const PACKET_NUM: u8 = 0x3A;
    let payload = hex!("20 03 40 0c d0 0e f0");
    let message = SystemExclusiveMessage::file_dump_packet(DEVICE_ID, PACKET_NUM, &payload);
    let result = message.write(&mut output);
    assert_eq!(result, Ok(()));
    assert_eq!(
        output,
        hex!("f0 7e 03 07 02 3a  07  05 20 03 40 0c 50 0e 70  01 f7")
    );
    assert_eq!(Some(message), SystemExclusiveMessage::read(&mut output));
}

#[test]
fn sysex_write_file_dump_packet_with_payload_not_divisible_by_7_bytes() {
    let mut output = CircularBuffer::<64, u8>::new();
    const DEVICE_ID: u8 = 3;
    const PACKET_NUM: u8 = 0x3A;
    let payload = hex!("20 03 40 0c d0 0e f0  f0 80 77");
    let message = SystemExclusiveMessage::file_dump_packet(DEVICE_ID, PACKET_NUM, &payload);
    let result = message.write(&mut output);
    assert_eq!(result, Ok(()));
    assert_eq!(
        output,
        hex!("f0 7e 03 07 02 3a  0b  05 20 03 40 0c 50 0e 70  60 70 00 77  6a f7")
    );
    assert_eq!(Some(message), SystemExclusiveMessage::read(&mut output));
}
