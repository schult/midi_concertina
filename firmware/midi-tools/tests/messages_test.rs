use circular_buffer::CircularBuffer;
use hex_literal::hex;
use midi_tools::messages::sysex::*;

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
    let expected = Some(SysEx::Ack(HandshakeData {
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
    let expected = Some(SysEx::Nak(HandshakeData {
        device_id: 0x03,
        packet_num: 0x3b,
    }));
    assert_eq!(result, expected);
}

#[test]
fn sysex_read_ignores_system_real_time_messages() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 f8 7e fa 03 fb fc 7f fe ff 3b"));
    let result = SysEx::read(&mut data);
    let expected = Some(SysEx::Ack(HandshakeData {
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
    let expected = Some(SysEx::Wait(HandshakeData {
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
    let expected = Some(SysEx::Cancel(HandshakeData {
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
    let expected = Some(SysEx::Eof(HandshakeData {
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
    let mut data = CircularBuffer::<64, u8>::from(hex!(
        "f0 7e 03 07 01 00  42 49 4e 20  1e 64  66 69 6c 65 2e 62 69 6e"
    ));
    let result = SysEx::read(&mut data);
    assert!(matches!(result, Some(SysEx::FileDumpHeader(_))));
}

#[test]
fn sysex_read_parses_file_dump_header_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!(
        "f0 7e 03 07 01 01  42 49 4e 20  1e 24  66 69 6c 65 2e 62 69 6e"
    ));
    let result = SysEx::read(&mut data);
    let expected = Some(SysEx::FileDumpHeader(FileDumpHeaderData {
        device_id: 3,
        source_id: 1,
        length: 0x121e,
        raw_file_type: hex!("42 49 4e 20"),
    }));
    assert_eq!(result, expected);
}

#[test]
fn sysex_read_consumes_all_file_dump_header_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!(
        "f0 7e 03 07 01 01  42 49 4e 20  1e 24  66 69 6c 65 2e 62 69 6e   f0 7e"
    ));
    let _ = SysEx::read(&mut data);
    assert_eq!(data, hex!("f0 7e"));
}

#[test]
fn file_dump_header_data_file_type_as_str() {
    let data = FileDumpHeaderData {
        device_id: 0,
        source_id: 0,
        length: 0,
        raw_file_type: hex!("42 49 4e 20"),
    };
    assert_eq!(data.file_type(), "BIN ");
}

#[test]
fn sysex_read_identifies_file_dump_packet_message() {
    let mut data = CircularBuffer::<64, u8>::from(hex!(
        "f0 7e 03 07 02 71 0f  00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60  5c"
    ));
    let result = SysEx::read(&mut data);
    assert!(matches!(result, Some(SysEx::FileDumpPacket(_))));
}

#[test]
fn sysex_read_parses_file_dump_packet_data() {
    let mut data = CircularBuffer::<64, u8>::from(hex!(
        "f0 7e 03 07 02 71 0f  00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60  5c"
    ));
    let result = SysEx::read(&mut data);

    let mut expected_data = FileDumpPacketData {
        device_id: 3,
        packet_num: 113,
        checksum_ok: true,
        data: [0; 112],
        data_size: 14,
    };
    expected_data.data[..14].copy_from_slice(&hex!("01 20 03 40 05 60 07  80 09 A0 0B C0 0D E0"));
    let expected = Some(SysEx::FileDumpPacket(expected_data));
    assert_eq!(result, expected);
}

#[test]
fn sysex_read_parses_max_length_file_dump_packet() {
    let mut data = CircularBuffer::<137, u8>::from(hex!(
        "f0 7e 03 07 02 71 7f"
        "00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60"
        "00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60"
        "00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60"
        "00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60"
        "00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60"
        "00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60"
        "00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60"
        "00 01 20 03 40 05 60 07  55 00 09 20 0b 40 0d 60"
        "76"));
    let result = SysEx::read(&mut data);

    let expected_data = FileDumpPacketData {
        device_id: 3,
        packet_num: 113,
        checksum_ok: true,
        data: hex!(
            "01 20 03 40 05 60 07  80 09 A0 0B C0 0D E0"
            "01 20 03 40 05 60 07  80 09 A0 0B C0 0D E0"
            "01 20 03 40 05 60 07  80 09 A0 0B C0 0D E0"
            "01 20 03 40 05 60 07  80 09 A0 0B C0 0D E0"
            "01 20 03 40 05 60 07  80 09 A0 0B C0 0D E0"
            "01 20 03 40 05 60 07  80 09 A0 0B C0 0D E0"
            "01 20 03 40 05 60 07  80 09 A0 0B C0 0D E0"
            "01 20 03 40 05 60 07  80 09 A0 0B C0 0D E0"
        ),
        data_size: 112,
    };
    let expected = Some(SysEx::FileDumpPacket(expected_data));
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
        "76"));
    let result = SysEx::read(&mut data);
    assert_eq!(result, None);
    assert_eq!(
        data,
        hex!(
        "9f"
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
        "76")
    );
}

#[test]
fn sysex_read_parses_odd_length_file_dump_packet() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 07 02 71 03  00 01 20 03  28"));
    let result = SysEx::read(&mut data);

    let mut expected_data = FileDumpPacketData {
        device_id: 3,
        packet_num: 113,
        checksum_ok: true,
        data: [0; 112],
        data_size: 3,
    };
    expected_data.data[..3].copy_from_slice(&hex!("01 20 03"));
    let expected = Some(SysEx::FileDumpPacket(expected_data));
    assert_eq!(result, expected);
}

#[test]
fn sysex_read_detects_file_dump_packet_checksum_validity() {
    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 07 02 71 03  00 01 20 03  28"));
    let result = SysEx::read(&mut data).unwrap();
    assert!(matches!(
        result,
        SysEx::FileDumpPacket(FileDumpPacketData {
            checksum_ok: true,
            ..
        })
    ));

    let mut data = CircularBuffer::<64, u8>::from(hex!("f0 7e 03 07 02 71 03  00 01 20 03  29"));
    let result = SysEx::read(&mut data).unwrap();
    assert!(matches!(
        result,
        SysEx::FileDumpPacket(FileDumpPacketData {
            checksum_ok: false,
            ..
        })
    ));
}

#[test]
fn sysex_ack_constructor() {
    let value = SysEx::ack(1, 14);
    assert_eq!(value, SysEx::Ack(HandshakeData {
        device_id: 1,
        packet_num: 14,
    }));
}

#[test]
#[should_panic]
fn sysex_ack_constructor_panics_if_device_id_over_7_bits() {
    let _ = SysEx::ack(1 << 7, 14);
}

#[test]
#[should_panic]
fn sysex_ack_constructor_panics_if_packet_num_over_7_bits() {
    let _ = SysEx::ack(1, 1 << 7);
}

#[test]
fn sysex_nak_constructor() {
    let value = SysEx::nak(1, 14);
    assert_eq!(value, SysEx::Nak(HandshakeData {
        device_id: 1,
        packet_num: 14,
    }));
}

#[test]
#[should_panic]
fn sysex_nak_constructor_panics_if_device_id_over_7_bits() {
    let _ = SysEx::nak(1 << 7, 14);
}

#[test]
#[should_panic]
fn sysex_nak_constructor_panics_if_packet_num_over_7_bits() {
    let _ = SysEx::nak(1, 1 << 7);
}

#[test]
fn sysex_wait_constructor() {
    let value = SysEx::wait(1, 14);
    assert_eq!(value, SysEx::Wait(HandshakeData {
        device_id: 1,
        packet_num: 14,
    }));
}

#[test]
#[should_panic]
fn sysex_wait_constructor_panics_if_device_id_over_7_bits() {
    let _ = SysEx::wait(1 << 7, 14);
}

#[test]
#[should_panic]
fn sysex_wait_constructor_panics_if_packet_num_over_7_bits() {
    let _ = SysEx::wait(1, 1 << 7);
}

#[test]
fn sysex_cancel_constructor() {
    let value = SysEx::cancel(1, 14);
    assert_eq!(value, SysEx::Cancel(HandshakeData {
        device_id: 1,
        packet_num: 14,
    }));
}

#[test]
#[should_panic]
fn sysex_cancel_constructor_panics_if_device_id_over_7_bits() {
    let _ = SysEx::cancel(1 << 7, 14);
}

#[test]
#[should_panic]
fn sysex_cancel_constructor_panics_if_packet_num_over_7_bits() {
    let _ = SysEx::cancel(1, 1 << 7);
}

#[test]
fn sysex_eof_constructor() {
    let value = SysEx::eof(1, 14);
    assert_eq!(value, SysEx::Eof(HandshakeData {
        device_id: 1,
        packet_num: 14,
    }));
}

#[test]
#[should_panic]
fn sysex_eof_constructor_panics_if_device_id_over_7_bits() {
    let _ = SysEx::eof(1 << 7, 14);
}

#[test]
#[should_panic]
fn sysex_eof_constructor_panics_if_packet_num_over_7_bits() {
    let _ = SysEx::eof(1, 1 << 7);
}

#[test]
fn sysex_identity_request_constructor() {
    let value = SysEx::identity_request(1);
    assert_eq!(value, SysEx::IdentityRequest(IdentityRequestData {
        device_id: 1,
    }));
}

#[test]
#[should_panic]
fn sysex_identity_request_constructor_panics_if_device_id_over_7_bits() {
    let _ = SysEx::identity_request(1 << 7);
}

#[test]
fn sysex_identity_reply_constructor() {
    let value = SysEx::identity_reply(3, 0x7F, 0x1253, 0x2430, &[1, 9, 8, 5]);
    assert_eq!(value, SysEx::IdentityReply(IdentityReplyData {
        device_id: 3,
        manufacturer_id: 0x7F,
        device_family_code: 0x1253,
        device_family_member_code: 0x2430,
        software_rev: [1, 9, 8, 5],
    }));
}

#[test]
#[should_panic]
fn sysex_identity_reply_constructor_panics_if_device_id_over_7_bits() {
    let _ = SysEx::identity_reply(1 << 7, 0x7F, 0x1253, 0x2430, &[1, 9, 8, 5]);
}

#[test]
#[should_panic]
fn sysex_identity_reply_constructor_panics_if_manufacturer_id_over_7_bits() {
    let _ = SysEx::identity_reply(3, 1 << 7, 0x1253, 0x2430, &[1, 9, 8, 5]);
}

#[test]
#[should_panic]
fn sysex_identity_reply_constructor_panics_if_device_family_code_over_14_bits() {
    let _ = SysEx::identity_reply(3, 0x7F, 1 << 14, 0x2430, &[1, 9, 8, 5]);
}

#[test]
#[should_panic]
fn sysex_identity_reply_constructor_panics_if_device_family_member_code_over_14_bits() {
    let _ = SysEx::identity_reply(3, 0x7F, 0x1253, 1 << 14, &[1, 9, 8, 5]);
}

#[test]
#[should_panic]
fn sysex_identity_reply_constructor_panics_if_version_contains_value_over_7_bits() {
    let _ = SysEx::identity_reply(3, 0x7F, 0x1253, 0x2430, &[1, 9, 1 << 7, 5]);
}

#[test]
#[should_panic]
fn sysex_identity_reply_constructor_panics_if_version_too_short() {
    let _ = SysEx::identity_reply(3, 0x7F, 0x1253, 0x2430, &[1, 9, 8]);
}

#[test]
#[should_panic]
fn sysex_identity_reply_constructor_panics_if_version_too_long() {
    let _ = SysEx::identity_reply(3, 0x7F, 0x1253, 0x2430, &[1, 9, 8, 5, 6]);
}

#[test]
fn sysex_file_dump_header_constructor() {
    let value = SysEx::file_dump_header(2, 1, 1_382_550, &"TEXT");
    assert_eq!(value, SysEx::FileDumpHeader(FileDumpHeaderData {
        device_id: 2,
        source_id: 1,
        length: 1_382_550,
        raw_file_type: hex!("54 45 58 54"),
    }));
}

#[test]
#[should_panic]
fn sysex_file_dump_header_constructor_panics_if_device_id_over_7_bits() {
    let _ = SysEx::file_dump_header(1 << 7, 1, 1_382_550, &"TEXT");
}

#[test]
#[should_panic]
fn sysex_file_dump_header_constructor_panics_if_source_id_over_7_bits() {
    let _ = SysEx::file_dump_header(2, 1 << 7, 1_382_550, &"TEXT");
}

#[test]
#[should_panic]
fn sysex_file_dump_header_constructor_panics_if_length_over_28_bits() {
    let _ = SysEx::file_dump_header(2, 1, 1 << 28, &"TEXT");
}

#[test]
#[should_panic]
fn sysex_file_dump_header_constructor_panics_if_file_type_too_short() {
    let _ = SysEx::file_dump_header(2, 1, 1_382_550, &"BIN");
}

#[test]
#[should_panic]
fn sysex_file_dump_header_constructor_panics_if_file_type_too_long() {
    let _ = SysEx::file_dump_header(2, 1, 1_382_550, &"MIDIEX");
}

#[test]
#[should_panic]
fn sysex_file_dump_header_constructor_panics_if_file_type_not_ascii() {
    let _ = SysEx::file_dump_header(2, 1, 1_382_550, &"ÀBI");
}

#[test]
fn sysex_file_dump_packet_constructor() {
    let value = SysEx::file_dump_packet(2, 110, &hex!("7f 00 ff 56"));
    let mut data = [0; 112];
    data[..4].copy_from_slice(&hex!("7f 00 ff 56"));
    assert_eq!(value, SysEx::FileDumpPacket(FileDumpPacketData {
        device_id: 2,
        packet_num: 110,
        checksum_ok: true,
        data,
        data_size: 4,
    }));
}

#[test]
#[should_panic]
fn sysex_file_dump_packet_constructor_device_id_over_7_bits() {
    let _ = SysEx::file_dump_packet(1 << 7, 110, &hex!("7f 00 ff 56"));
}

#[test]
#[should_panic]
fn sysex_file_dump_packet_constructor_packet_num_over_7_bits() {
    let _ = SysEx::file_dump_packet(2, 1 << 7, &hex!("7f 00 ff 56"));
}

#[test]
#[should_panic]
fn sysex_file_dump_packet_constructor_panics_if_data_too_long() {
    let _ = SysEx::file_dump_packet(2, 110, &[0xCC; 113]);
}
