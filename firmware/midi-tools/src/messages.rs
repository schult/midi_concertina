pub mod sysex {
    use embedded_io::BufRead;

    #[derive(Debug, PartialEq)]
    pub struct HandshakeData {
        pub device_id: u8,
        pub packet_num: u8,
    }

    #[derive(Debug, PartialEq)]
    pub struct IdentityRequestData {
        pub device_id: u8,
    }

    #[derive(Debug, PartialEq)]
    pub struct IdentityReplyData {
        pub device_id: u8,
        pub manufacturer_id: u8,
        pub device_family_code: u16,
        pub device_family_member_code: u16,
        pub software_rev: [u8; 4],
    }

    #[derive(Debug, PartialEq)]
    pub struct FileDumpHeaderData {
        pub device_id: u8,
        pub source_id: u8,
        pub length: u32,

        file_type: [u8; 4],
    }

    impl FileDumpHeaderData {
        pub fn file_type(&self) -> &str {
            core::str::from_utf8(&self.file_type).unwrap()
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct FileDumpPacketData {
        pub device_id: u8,
        pub packet_num: u8,
        pub checksum_ok: bool,

        data: [u8; 112],
        data_size: usize,
    }

    impl FileDumpPacketData {
        pub fn data(&self) -> &[u8] {
            &self.data[..self.data_size]
        }
    }

    #[derive(Debug, PartialEq)]
    pub enum SysEx {
        Ack(HandshakeData), // 7f
        Nak(HandshakeData), // 7e
        Wait(HandshakeData), // 7c
        Cancel(HandshakeData), // 7d
        Eof(HandshakeData), // 7b
        IdentityRequest(IdentityRequestData), // 06 01
        IdentityReply(IdentityReplyData), // 06 02
        FileDumpHeader(FileDumpHeaderData), // 07 01
        FileDumpPacket(FileDumpPacketData), // 07 02
        ShowControl, // 02
    }

    impl SysEx {
        pub fn read(reader: &mut impl BufRead) -> Option<Self> {
            if let Ok(buffer) = reader.fill_buf() {
                let begin = match buffer.iter().position(|x| *x == 0xF0) {
                    Some(i) => i,
                    None => buffer.len(),
                };
                let length = match buffer[begin..].iter().skip(1).position(|x| (*x & 0x80) != 0) {
                    Some(i) => i + 1,
                    None => buffer[begin..].len(),
                };
                let end = begin + length;
                let raw = &buffer[begin..end];

                if raw.len() < 5 {
                    reader.consume(begin);
                    return None;
                }

                if raw[1] != 0x7E {
                    reader.consume(end);
                    return None;
                }

                let message = match raw[3] {
                    0x7F => Some(SysEx::Ack(HandshakeData { device_id: raw[2], packet_num: raw[4] })),
                    0x7E => Some(SysEx::Nak(HandshakeData { device_id: raw[2], packet_num: raw[4] })),
                    _ => None,
                };
                reader.consume(end);
                return message;
            }

            None
        }
    }

} // mod sysex
