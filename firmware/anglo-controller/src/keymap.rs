use midi::Note;

pub const NUM_LEFT: usize = 15;
pub const NUM_RIGHT: usize = 15;

#[rustfmt::skip]
pub const LEFT_PUSH: [Note; NUM_LEFT] = [
    Note::E3, Note::A3, Note::Db4, Note::A4, Note::Ab4,
    Note::C3, Note::G3, Note::C4, Note::E4, Note::G4,
    Note::B3, Note::D4, Note::G4, Note::B4, Note::D5,
];

#[rustfmt::skip]
pub const LEFT_PULL: [Note; NUM_LEFT] = [
    Note::F3, Note::Bb3, Note::Eb4, Note::G4, Note::Bb4,
    Note::G3, Note::B3, Note::D4, Note::F4, Note::A4,
    Note::A3, Note::Gb4, Note::A4, Note::C5, Note::E5,
];

#[rustfmt::skip]
pub const RIGHT_PUSH: [Note; NUM_RIGHT] = [
    Note::Db5, Note::A5, Note::Ab5, Note::Db6, Note::A6,
    Note::C5, Note::E5, Note::G5, Note::C6, Note::E6,
    Note::G5, Note::B5, Note::D6, Note::G6, Note::B6,
];

#[rustfmt::skip]
pub const RIGHT_PULL: [Note; NUM_RIGHT] = [
    Note::Eb5, Note::G5, Note::Bb5, Note::Eb6, Note::F6,
    Note::B4, Note::D5, Note::F5, Note::A5, Note::B5,
    Note::Gb5, Note::A5, Note::C6, Note::E6, Note::Gb6,
];
