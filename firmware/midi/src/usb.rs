use crate::Message;
use circular_buffer::CircularBuffer;

#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum Cin {
    Misc = 0x0,
    CableEvent = 0x1,
    SysCommon2Byte = 0x2,
    SysCommon3Byte = 0x3,
    SysEx = 0x4,
    Sys1Byte = 0x5,
    SysExEnd2Byte = 0x6,
    SysExEnd3Byte = 0x7,
    NoteOff = 0x8,
    NoteOn = 0x9,
    PolyKeyPress = 0xA,
    ControlChange = 0xB,
    ProgramChange = 0xC,
    ChannelPressure = 0xD,
    PitchBendChange = 0xE,
    SingleByte = 0xF,
}

impl From<u8> for Cin {
    fn from(value: u8) -> Self {
        match value {
            0x0 => Cin::Misc,
            0x1 => Cin::CableEvent,
            0x2 => Cin::SysCommon2Byte,
            0x3 => Cin::SysCommon3Byte,
            0x4 => Cin::SysEx,
            0x5 => Cin::Sys1Byte,
            0x6 => Cin::SysExEnd2Byte,
            0x7 => Cin::SysExEnd3Byte,
            0x8 => Cin::NoteOff,
            0x9 => Cin::NoteOn,
            0xA => Cin::PolyKeyPress,
            0xB => Cin::ControlChange,
            0xC => Cin::ProgramChange,
            0xD => Cin::ChannelPressure,
            0xE => Cin::PitchBendChange,
            0xF => Cin::SingleByte,
            _ => core::panic!(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct EventPacket {
    pub raw: [u8; 4],
}

impl EventPacket {
    pub fn parse(buffer: &[u8]) -> impl Iterator<Item = Self> {
        const EVENT_PACKET_SIZE: usize = 4;
        buffer.chunks_exact(EVENT_PACKET_SIZE).map(|x| EventPacket {
            raw: x.try_into().unwrap(),
        })
    }

    pub fn encode(cable: u8, message: &Message) -> Encode {
        assert!(cable <= 0x0F);

        let mut buffer = CircularBuffer::<{ Message::MAX_LENGTH }, u8>::new();
        message.write(&mut buffer).unwrap();
        let data = buffer.make_contiguous();

        match message {
            Message::NoteOn(_) => Encode::new(cable, Cin::NoteOn, data),
            Message::NoteOff(_) => Encode::new(cable, Cin::NoteOff, data),
            Message::ControlChange(_) => Encode::new(cable, Cin::ControlChange, data),
            Message::Ack(_)
            | Message::Nak(_)
            | Message::Wait(_)
            | Message::Cancel(_)
            | Message::Eof(_)
            | Message::IdentityRequest(_)
            | Message::IdentityReply(_)
            | Message::FileDumpHeader(_)
            | Message::FileDumpPacket(_) => Encode::from_sysex(cable, data),
        }
    }

    pub fn cable(&self) -> u8 {
        (self.raw[0] & 0xF0) >> 4
    }

    pub fn cin(&self) -> Cin {
        Cin::from(self.raw[0] & 0x0F)
    }

    pub fn payload(&self) -> &[u8] {
        let size = match self.cin() {
            Cin::Sys1Byte | Cin::SingleByte => 1,
            Cin::SysCommon2Byte
            | Cin::SysExEnd2Byte
            | Cin::ProgramChange
            | Cin::ChannelPressure => 2,
            _ => 3,
        };
        let end = size + 1;
        &self.raw[1..end]
    }
}

pub struct Encode {
    packets: CircularBuffer<{ (Message::MAX_LENGTH + 2) / 3 }, EventPacket>,
}

impl Iterator for Encode {
    type Item = EventPacket;

    fn next(&mut self) -> Option<Self::Item> {
        self.packets.pop_front()
    }
}

impl Encode {
    fn new(cable: u8, cin: Cin, data: &[u8]) -> Self {
        let mut packet = EventPacket{ raw: [0; 4] };
        packet.raw[0] = (cable << 4) | cin as u8;
        packet.raw[1..(1 + data.len())].copy_from_slice(data);

        let mut result = Self { packets: CircularBuffer::new() };
        result.packets.push_back(packet);
        result
    }

    fn from_sysex(cable: u8, data: &[u8]) -> Self {
        let mut result = Self { packets: CircularBuffer::new() };

        let mut iter = data.chunks(3).peekable();
        while let Some(chunk) = iter.next() {
            let mut raw = [0; 4];
            if iter.peek().is_some() {
                raw[0] = 0x04;
                raw[1..].copy_from_slice(chunk);
            } else {
                raw[0] = 0x04 + chunk.len() as u8;
                raw[1..(1 + chunk.len())].copy_from_slice(chunk);
            }
            raw[0] |= cable << 4;
            result.packets.push_back(EventPacket { raw });
        }

        result
    }
}
