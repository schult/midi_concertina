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

pub struct EventPacket<'a> {
    pub raw: &'a [u8],
}

impl<'a> EventPacket<'a> {
    pub fn parse(buffer: &'a [u8]) -> impl Iterator<Item = Self> {
        const EVENT_PACKET_SIZE: usize = 4;
        buffer.chunks_exact(EVENT_PACKET_SIZE).map(|x| EventPacket {
            raw: x.try_into().unwrap(),
        })
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
