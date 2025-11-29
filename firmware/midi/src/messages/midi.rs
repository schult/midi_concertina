use embedded_io::Write;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum Channel {
    Ch1,
    Ch2,
    Ch3,
    Ch4,
    Ch5,
    Ch6,
    Ch7,
    Ch8,
    Ch9,
    Ch10,
    Ch11,
    Ch12,
    Ch13,
    Ch14,
    Ch15,
    Ch16,
}

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

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub struct NoteData {
    pub channel: Channel,
    pub note: Note,
    pub velocity: u8,
}

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub struct ControlData {
    pub channel: Channel,
    pub control: u8,
    pub value: u8,
}

#[derive(Debug, PartialEq)]
pub enum Midi {
    NoteOn(NoteData),
    NoteOff(NoteData),
    ControlChange(ControlData),
}

impl Midi {
    pub const MAX_LENGTH: usize = 3;

    pub fn note_on(channel: Channel, note: Note, velocity: u8) -> Self {
        assert!(velocity <= 0x7F);
        Self::NoteOn(NoteData {
            channel,
            note,
            velocity,
        })
    }

    pub fn note_off(channel: Channel, note: Note, velocity: u8) -> Self {
        assert!(velocity <= 0x7F);
        Self::NoteOff(NoteData {
            channel,
            note,
            velocity,
        })
    }

    pub fn control_change(channel: Channel, control: u8, value: u8) -> Self {
        assert!(control <= 0x7F);
        assert!(value <= 0x7F);
        Self::ControlChange(ControlData {
            channel,
            control,
            value,
        })
    }

    pub fn write<T: Write>(&self, writer: &mut T) -> Result<(), T::Error> {
        match self {
            Midi::NoteOn(d) => {
                writer.write_all(&[0x90 | d.channel as u8, d.note as u8, d.velocity])
            }
            Midi::NoteOff(d) => {
                writer.write_all(&[0x80 | d.channel as u8, d.note as u8, d.velocity])
            }
            Midi::ControlChange(d) => {
                writer.write_all(&[0xB0 | d.channel as u8, d.control, d.value])
            }
        }
    }
}
