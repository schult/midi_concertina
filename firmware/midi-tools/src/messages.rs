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

    fn parse_handshake<'a>(it: &mut impl Iterator<Item = &'a u8>, sub_id_1: u8) -> Option<HandshakeData> {
        if *it.next()? != 0xF0 { return None; }
        if *it.next()? != 0x7E { return None; }
        let device_id = *it.next()?;
        if *it.next()? != sub_id_1 { return None; }
        let packet_num = *it.next()?;
        Some(HandshakeData { device_id, packet_num })
    }

    fn parse_ack<'a>(it: &mut impl Iterator<Item = &'a u8>) -> Option<SysEx> {
        Some(SysEx::Ack(parse_handshake(it, 0x7F)?))
    }

    fn parse_nak<'a>(it: &mut impl Iterator<Item = &'a u8>) -> Option<SysEx> {
        Some(SysEx::Nak(parse_handshake(it, 0x7E)?))
    }

    fn parse_wait<'a>(it: &mut impl Iterator<Item = &'a u8>) -> Option<SysEx> {
        Some(SysEx::Wait(parse_handshake(it, 0x7C)?))
    }

    fn parse_cancel<'a>(it: &mut impl Iterator<Item = &'a u8>) -> Option<SysEx> {
        Some(SysEx::Cancel(parse_handshake(it, 0x7D)?))
    }

    fn parse_eof<'a>(it: &mut impl Iterator<Item = &'a u8>) -> Option<SysEx> {
        Some(SysEx::Eof(parse_handshake(it, 0x7B)?))
    }

    fn is_data(x: &u8) -> bool {
        return (*x & 0x80) == 0;
    }

    fn is_sys_rt(x: &u8) -> bool {
        return (*x & 0xF8) == 0xF8;
    }

    impl SysEx {
        pub fn read(reader: &mut impl BufRead) -> Option<Self> {
            if let Ok(buffer) = reader.fill_buf() {
                let begin = match buffer.iter().position(|x| *x == 0xF0) {
                    Some(i) => i,
                    None => buffer.len(),
                };
                let length = match buffer[begin..].iter().skip(1).position(|x| !is_data(x) && !is_sys_rt(x)) {
                    Some(i) => i + 1,
                    None => buffer[begin..].len(),
                };
                let end = begin + length;
                let raw = &buffer[begin..end];
                let mut raw_it = raw.iter().filter(|x| !is_sys_rt(x));

                let sysex_id = raw_it.clone().nth(1);
                if sysex_id.is_some_and(|x| *x != 0x7E) {
                    // Ignore non-universal SysEx messages
                    reader.consume(end);
                    return None;
                }

                let sub_id_1 = raw_it.clone().nth(3);
                let message = match sub_id_1 {
                    Some(0x7F) => parse_ack(&mut raw_it),
                    Some(0x7E) => parse_nak(&mut raw_it),
                    Some(0x7C) => parse_wait(&mut raw_it),
                    Some(0x7D) => parse_cancel(&mut raw_it),
                    Some(0x7B) => parse_eof(&mut raw_it),
                    None => None,
                    _ => {
                        // Ignore unrecognized message types
                        reader.consume(end);
                        return None;
                    },
                };

                reader.consume(if message.is_some() { end } else { begin });
                return message;
            }

            None
        }
    }

} // mod sysex
