#![no_std]

use embedded_io::{BufRead, Write};

pub mod usb;
pub mod util;

#[rustfmt::skip]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum Note
{
    C_, Db_, D_, Eb_, E_, F_, Gb_, G_, Ab_, A_, Bb_, B_,
    C0, Db0, D0, Eb0, E0, F0, Gb0, G0, Ab0, A0, Bb0, B0,
    C1, Db1, D1, Eb1, E1, F1, Gb1, G1, Ab1, A1, Bb1, B1,
    C2, Db2, D2, Eb2, E2, F2, Gb2, G2, Ab2, A2, Bb2, B2,
    C3, Db3, D3, Eb3, E3, F3, Gb3, G3, Ab3, A3, Bb3, B3,
    C4, Db4, D4, Eb4, E4, F4, Gb4, G4, Ab4, A4, Bb4, B4,
    C5, Db5, D5, Eb5, E5, F5, Gb5, G5, Ab5, A5, Bb5, B5,
    C6, Db6, D6, Eb6, E6, F6, Gb6, G6, Ab6, A6, Bb6, B6,
    C7, Db7, D7, Eb7, E7, F7, Gb7, G7, Ab7, A7, Bb7, B7,
    C8, Db8, D8, Eb8, E8, F8, Gb8, G8, Ab8, A8, Bb8, B8,
    C9, Db9, D9, Eb9, E9, F9, Gb9, G9,
}

pub mod cc {
    pub const CHANNEL_VOLUME_MSB: u8 = 0x07;
    pub const CHANNEL_VOLUME_LSB: u8 = 0x27;
}

pub const ALL_CALL_DEVICE_ID: u8 = 0x7F;

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub struct NoteData {
    pub channel: u8,
    pub note: Note,
    pub velocity: u8,
}

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub struct ControlData {
    pub channel: u8,
    pub control: u8,
    pub value: u8,
}

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub struct HandshakeData {
    pub device_id: u8,
    pub packet_num: u8,
}

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub struct IdentityRequestData {
    pub device_id: u8,
}

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub struct IdentityReplyData {
    pub device_id: u8,
    pub manufacturer_id: u8,
    pub device_family_code: u16,
    pub device_family_member_code: u16,
    pub software_rev: [u8; 4],
}

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub struct FileDumpHeaderData {
    pub device_id: u8,
    pub source_id: u8,
    pub length: u32,
    pub raw_file_type: [u8; 4],
}

impl FileDumpHeaderData {
    pub fn file_type(&self) -> &str {
        core::str::from_utf8(&self.raw_file_type).unwrap()
    }
}

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub struct FileDumpPacketData {
    pub device_id: u8,
    pub packet_num: u8,
    pub checksum_ok: bool,
    pub data: [u8; 112],
    pub data_size: usize,
}

impl FileDumpPacketData {
    pub fn data(&self) -> &[u8] {
        &self.data[..self.data_size]
    }
}

#[derive(Debug, PartialEq)]
pub enum Message {
    // Channel Voice
    NoteOn(NoteData),
    NoteOff(NoteData),
    ControlChange(ControlData),

    // System Exclusive
    Ack(HandshakeData),
    Nak(HandshakeData),
    Wait(HandshakeData),
    Cancel(HandshakeData),
    Eof(HandshakeData),
    IdentityRequest(IdentityRequestData),
    IdentityReply(IdentityReplyData),
    FileDumpHeader(FileDumpHeaderData),
    FileDumpPacket(FileDumpPacketData),
}

impl Message {
    pub const MAX_LENGTH: usize = 137;

    pub fn note_on(channel: u8, note: Note, velocity: u8) -> Self {
        assert!(channel <= 0x0F);
        assert!(velocity <= 0x7F);
        Self::NoteOn(NoteData {
            channel,
            note,
            velocity,
        })
    }

    pub fn note_off(channel: u8, note: Note, velocity: u8) -> Self {
        assert!(channel <= 0x0F);
        assert!(velocity <= 0x7F);
        Self::NoteOff(NoteData {
            channel,
            note,
            velocity,
        })
    }

    pub fn control_change(channel: u8, control: u8, value: u8) -> Self {
        assert!(channel <= 0x0F);
        assert!(control <= 0x7F);
        assert!(value <= 0x7F);
        Self::ControlChange(ControlData {
            channel,
            control,
            value,
        })
    }

    pub fn ack(device_id: u8, packet_num: u8) -> Self {
        assert!(device_id <= 0x7F);
        assert!(packet_num <= 0x7F);
        Message::Ack(HandshakeData {
            device_id,
            packet_num,
        })
    }

    pub fn nak(device_id: u8, packet_num: u8) -> Self {
        assert!(device_id <= 0x7F);
        assert!(packet_num <= 0x7F);
        Message::Nak(HandshakeData {
            device_id,
            packet_num,
        })
    }

    pub fn wait(device_id: u8, packet_num: u8) -> Self {
        assert!(device_id <= 0x7F);
        assert!(packet_num <= 0x7F);
        Message::Wait(HandshakeData {
            device_id,
            packet_num,
        })
    }

    pub fn cancel(device_id: u8, packet_num: u8) -> Self {
        assert!(device_id <= 0x7F);
        assert!(packet_num <= 0x7F);
        Message::Cancel(HandshakeData {
            device_id,
            packet_num,
        })
    }

    pub fn eof(device_id: u8, packet_num: u8) -> Self {
        assert!(device_id <= 0x7F);
        assert!(packet_num <= 0x7F);
        Message::Eof(HandshakeData {
            device_id,
            packet_num,
        })
    }

    pub fn identity_request(device_id: u8) -> Self {
        assert!(device_id <= 0x7F);
        Message::IdentityRequest(IdentityRequestData { device_id })
    }

    pub fn identity_reply(
        device_id: u8,
        manufacturer_id: u8,
        device_family_code: u16,
        device_family_member_code: u16,
        software_rev: &[u8],
    ) -> Self {
        assert!(device_id <= 0x7F);
        assert!(manufacturer_id <= 0x7F);
        assert!(device_family_code <= 0x3FFF);
        assert!(device_family_member_code <= 0x3FFF);
        assert!(software_rev.iter().all(|x| *x < 0x7F));
        let mut software_rev_array = [0; 4];
        software_rev_array.copy_from_slice(software_rev);
        Message::IdentityReply(IdentityReplyData {
            device_id,
            manufacturer_id,
            device_family_code,
            device_family_member_code,
            software_rev: software_rev_array,
        })
    }

    pub fn file_dump_header(device_id: u8, source_id: u8, length: u32, file_type: &str) -> Self {
        assert!(device_id <= 0x7F);
        assert!(source_id <= 0x7F);
        assert!(length <= 0x0FFFFFFF);
        let mut raw_file_type = [b' '; 4];
        raw_file_type.copy_from_slice(file_type.as_bytes());
        assert!(raw_file_type.iter().all(|x| *x < 0x7F));
        Message::FileDumpHeader(FileDumpHeaderData {
            device_id,
            source_id,
            length,
            raw_file_type,
        })
    }

    pub fn file_dump_packet(device_id: u8, packet_num: u8, data: &[u8]) -> Self {
        assert!(device_id <= 0x7F);
        assert!(packet_num <= 0x7F);
        let mut data_array = [0; 112];
        data_array[..data.len()].copy_from_slice(data);
        Message::FileDumpPacket(FileDumpPacketData {
            device_id,
            packet_num,
            checksum_ok: true,
            data: data_array,
            data_size: data.len(),
        })
    }

    pub fn write<T: Write>(&self, writer: &mut T) -> Result<(), T::Error> {
        match self {
            Message::NoteOn(d) => writer.write_all(&[0x90 | d.channel, d.note as u8, d.velocity]),
            Message::NoteOff(d) => writer.write_all(&[0x80 | d.channel, d.note as u8, d.velocity]),
            Message::ControlChange(d) => writer.write_all(&[0xB0 | d.channel, d.control, d.value]),
            Message::Ack(d) => write_ack(d, writer),
            Message::Nak(d) => write_nak(d, writer),
            Message::Wait(d) => write_wait(d, writer),
            Message::Cancel(d) => write_cancel(d, writer),
            Message::Eof(d) => write_eof(d, writer),
            Message::IdentityRequest(_) => Ok(()), // TODO
            Message::IdentityReply(_) => Ok(()),   // TODO
            Message::FileDumpHeader(d) => write_file_dump_header(d, writer),
            Message::FileDumpPacket(d) => write_file_dump_packet(d, writer),
        }
    }

    pub fn read_sysex(reader: &mut impl BufRead) -> Option<Self> {
        while let Ok(buffer) = reader.fill_buf() {
            match buffer.iter().position(|x| *x == 0xF0) {
                Some(0) => (),
                // Ignore non-SysEx messages
                Some(i) => {
                    reader.consume(i);
                    continue;
                }
                None => {
                    let length = buffer.len();
                    reader.consume(length);
                    break;
                }
            };

            let data_end = match buffer
                .iter()
                .skip(1)
                .position(|x| !is_data(x) && !is_sys_rt(x))
            {
                Some(i) => i + 1,
                None => break,
            };
            let end = match buffer.get(data_end) {
                Some(0xF7) => data_end + 1,
                _ => data_end,
            };

            let raw = &buffer[..end];
            let mut raw_it = raw.iter().filter(|x| !is_sys_rt(x));

            let sysex_id = raw_it.clone().nth(1);
            if sysex_id.is_some_and(|x| *x != 0x7E) {
                // Ignore non-universal SysEx messages
                reader.consume(end);
                continue;
            }

            let sub_id_1 = raw_it.clone().nth(3);
            let sub_id_2 = raw_it.clone().nth(4);
            let message = match (sub_id_1, sub_id_2) {
                (Some(0x7F), _) => parse_ack(&mut raw_it),
                (Some(0x7E), _) => parse_nak(&mut raw_it),
                (Some(0x7C), _) => parse_wait(&mut raw_it),
                (Some(0x7D), _) => parse_cancel(&mut raw_it),
                (Some(0x7B), _) => parse_eof(&mut raw_it),
                // TODO: (Some(0x06), Some(0x01)) => IdentityRequest
                // TODO: (Some(0x06), Some(0x02)) => IdentityReply
                (Some(0x07), Some(0x01)) => parse_file_dump_header(&mut raw_it),
                (Some(0x07), Some(0x02)) => parse_file_dump_packet(&mut raw_it),
                _ => None,
            };

            reader.consume(end);
            if message.is_some() {
                return message;
            }
        }

        None
    }
}

fn write_ack<T: Write>(data: &HandshakeData, writer: &mut T) -> Result<(), T::Error> {
    let raw_data = [0xF0, 0x7E, data.device_id, 0x7F, data.packet_num, 0xF7];
    writer.write_all(&raw_data)
}

fn write_nak<T: Write>(data: &HandshakeData, writer: &mut T) -> Result<(), T::Error> {
    let raw_data = [0xF0, 0x7E, data.device_id, 0x7E, data.packet_num, 0xF7];
    writer.write_all(&raw_data)
}

fn write_wait<T: Write>(data: &HandshakeData, writer: &mut T) -> Result<(), T::Error> {
    let raw_data = [0xF0, 0x7E, data.device_id, 0x7C, data.packet_num, 0xF7];
    writer.write_all(&raw_data)
}

fn write_cancel<T: Write>(data: &HandshakeData, writer: &mut T) -> Result<(), T::Error> {
    let raw_data = [0xF0, 0x7E, data.device_id, 0x7D, data.packet_num, 0xF7];
    writer.write_all(&raw_data)
}

fn write_eof<T: Write>(data: &HandshakeData, writer: &mut T) -> Result<(), T::Error> {
    let raw_data = [0xF0, 0x7E, data.device_id, 0x7B, data.packet_num, 0xF7];
    writer.write_all(&raw_data)
}

fn write_file_dump_header<T: Write>(
    data: &FileDumpHeaderData,
    writer: &mut T,
) -> Result<(), T::Error> {
    let raw_data = [
        0xF0,
        0x7E,
        data.device_id,
        0x07,
        0x01,
        data.source_id,
        data.raw_file_type[0],
        data.raw_file_type[1],
        data.raw_file_type[2],
        data.raw_file_type[3],
        (data.length & 0x7F) as u8,
        ((data.length >> 7) & 0x7F) as u8,
        ((data.length >> 14) & 0x7F) as u8,
        ((data.length >> 21) & 0x7F) as u8,
        0xF7,
    ];
    writer.write_all(&raw_data)
}

fn write_file_dump_packet<T: Write>(
    data: &FileDumpPacketData,
    writer: &mut T,
) -> Result<(), T::Error> {
    let unencoded = data.data();
    let byte_count = unencoded.len() + unencoded.len().div_ceil(7) - 1;
    let byte_count = byte_count as u8;

    let prelude = [
        0xF0,
        0x7E,
        data.device_id,
        0x07,
        0x02,
        data.packet_num,
        byte_count,
    ];
    writer.write_all(&prelude)?;

    let mut checksum = 0x7E ^ data.device_id ^ 0x07 ^ 0x02 ^ data.packet_num ^ byte_count;

    let chunks = unencoded.chunks_exact(7);
    let remainder = chunks.remainder();
    for chunk in chunks {
        let mut encoded = [0; 8];
        for (i, byte) in chunk.iter().enumerate() {
            encoded[0] |= (byte & 0x80) >> (i + 1);
            encoded[i + 1] = byte & 0x7F;
            checksum ^= encoded[i + 1];
        }
        checksum ^= encoded[0];
        writer.write_all(&encoded)?;
    }

    if !remainder.is_empty() {
        let mut encoded = [0; 8];
        for (i, byte) in remainder.iter().enumerate() {
            encoded[0] |= (byte & 0x80) >> (i + 1);
            encoded[i + 1] = byte & 0x7F;
            checksum ^= encoded[i + 1];
        }
        checksum ^= encoded[0];
        writer.write_all(&encoded[..=remainder.len()])?;
    }

    let postlude = [checksum, 0xF7];
    writer.write_all(&postlude)
}

fn is_data(x: &u8) -> bool {
    (*x & 0x80) == 0
}

fn is_sys_rt(x: &u8) -> bool {
    (*x & 0xF8) == 0xF8
}

fn parse_handshake<'a>(
    it: &mut impl Iterator<Item = &'a u8>,
    sub_id_1: u8,
) -> Option<HandshakeData> {
    if *it.next()? != 0xF0 {
        return None;
    }
    if *it.next()? != 0x7E {
        return None;
    }
    let device_id = *it.next()?;
    if *it.next()? != sub_id_1 {
        return None;
    }
    let packet_num = *it.next()?;
    if *it.next()? != 0xF7 {
        return None;
    }
    Some(HandshakeData {
        device_id,
        packet_num,
    })
}

fn parse_ack<'a>(it: &mut impl Iterator<Item = &'a u8>) -> Option<Message> {
    Some(Message::Ack(parse_handshake(it, 0x7F)?))
}

fn parse_nak<'a>(it: &mut impl Iterator<Item = &'a u8>) -> Option<Message> {
    Some(Message::Nak(parse_handshake(it, 0x7E)?))
}

fn parse_wait<'a>(it: &mut impl Iterator<Item = &'a u8>) -> Option<Message> {
    Some(Message::Wait(parse_handshake(it, 0x7C)?))
}

fn parse_cancel<'a>(it: &mut impl Iterator<Item = &'a u8>) -> Option<Message> {
    Some(Message::Cancel(parse_handshake(it, 0x7D)?))
}

fn parse_eof<'a>(it: &mut impl Iterator<Item = &'a u8>) -> Option<Message> {
    Some(Message::Eof(parse_handshake(it, 0x7B)?))
}

fn parse_file_dump_header<'a>(
    it: &mut impl Iterator<Item = &'a u8>,
) -> Option<Message> {
    if *it.next()? != 0xF0 {
        return None;
    }
    if *it.next()? != 0x7E {
        return None;
    }
    let device_id = *it.next()?;
    if *it.next()? != 0x07 {
        return None;
    }
    if *it.next()? != 0x01 {
        return None;
    }
    let source_id = *it.next()?;

    let mut raw_file_type = [0; 4];
    for byte in raw_file_type.iter_mut() {
        *byte = *it.next()?;
    }

    let mut length = 0;
    for i in 0..4 {
        let byte = *it.next()? as u32;
        length |= byte << (7 * i);
    }

    while *it.next()? != 0xF7 {}

    Some(Message::FileDumpHeader(FileDumpHeaderData {
        device_id,
        source_id,
        length,
        raw_file_type,
    }))
}

fn parse_file_dump_packet<'a>(
    it: &mut impl Iterator<Item = &'a u8>,
) -> Option<Message> {
    if *it.next()? != 0xF0 {
        return None;
    }
    if *it.next()? != 0x7E {
        return None;
    }
    let device_id = *it.next()?;
    if *it.next()? != 0x07 {
        return None;
    }
    if *it.next()? != 0x02 {
        return None;
    }
    let packet_num = *it.next()?;
    let encoded_size = *it.next()?;

    let mut checksum = 0x7B ^ device_id ^ packet_num ^ encoded_size;
    let mut data = [0; 112];
    let mut data_size = 0;

    let mut high_bits = 0;
    for i in 0..=encoded_size {
        let byte = *it.next()?;
        checksum ^= byte;
        if i % 8 == 0 {
            high_bits = byte;
        } else {
            high_bits <<= 1;
            data[data_size] = byte | (high_bits & 0x80);
            data_size += 1;
        }
    }

    let checksum_ok = *it.next()? == checksum;

    if *it.next()? != 0xF7 {
        return None;
    }

    Some(Message::FileDumpPacket(FileDumpPacketData {
        device_id,
        packet_num,
        checksum_ok,
        data,
        data_size,
    }))
}
