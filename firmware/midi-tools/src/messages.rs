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
            let len = match reader.fill_buf() {
                Ok(buffer) => buffer.len(),
                Err(_) => 0,
            };
            reader.consume(len);

            None
        }
    }

} // mod sysex
