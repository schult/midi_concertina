use circular_buffer::CircularBuffer;
use hex_literal::hex;
use midi_tools::messages::midi::*;

#[test]
fn midi_note_on_constructor() {
    let value = Midi::note_on(Channel::Ch3, Note::Ab4, 64);

    assert!(matches!(value, Midi::NoteOn(_)));
    if let Midi::NoteOn(d) = value {
        assert_eq!(d.channel, Channel::Ch3);
        assert_eq!(d.note, Note::Ab4);
        assert_eq!(d.velocity, 64);
    }
}

#[test]
#[should_panic]
fn midi_note_on_constructor_panics_if_velocity_over_7_bits() {
    let _ = Midi::note_on(Channel::Ch3, Note::Ab4, 1 << 7);
}

#[test]
fn midi_note_off_constructor() {
    let value = Midi::note_off(Channel::Ch3, Note::Ab4, 64);

    assert!(matches!(value, Midi::NoteOff(_)));
    if let Midi::NoteOff(d) = value {
        assert_eq!(d.channel, Channel::Ch3);
        assert_eq!(d.note, Note::Ab4);
        assert_eq!(d.velocity, 64);
    }
}

#[test]
#[should_panic]
fn midi_note_off_constructor_panics_if_velocity_over_7_bits() {
    let _ = Midi::note_off(Channel::Ch3, Note::Ab4, 1 << 7);
}

#[test]
fn midi_control_change_constructor() {
    let value = Midi::control_change(Channel::Ch3, 31, 64);

    assert!(matches!(value, Midi::ControlChange(_)));
    if let Midi::ControlChange(d) = value {
        assert_eq!(d.channel, Channel::Ch3);
        assert_eq!(d.control, 31);
        assert_eq!(d.value, 64);
    }
}

#[test]
#[should_panic]
fn midi_control_change_constructor_panics_if_control_over_7_bits() {
    let _ = Midi::control_change(Channel::Ch3, 1 << 7, 64);
}

#[test]
#[should_panic]
fn midi_control_change_constructor_panics_if_value_over_7_bits() {
    let _ = Midi::control_change(Channel::Ch3, 31, 1 << 7);
}

#[test]
fn midi_write_note_on() {
    let mut output = CircularBuffer::<{ Midi::MAX_LENGTH }, u8>::new();
    let message = Midi::note_on(Channel::Ch2, Note::C4, 64);
    let result = message.write(&mut output);
    assert_eq!(result, Ok(()));
    assert_eq!(output, hex!("91 3c 40"));
}

#[test]
fn midi_write_note_off() {
    let mut output = CircularBuffer::<{ Midi::MAX_LENGTH }, u8>::new();
    let message = Midi::note_off(Channel::Ch2, Note::C4, 64);
    let result = message.write(&mut output);
    assert_eq!(result, Ok(()));
    assert_eq!(output, hex!("81 3c 40"));
}

#[test]
fn midi_write_control_change() {
    let mut output = CircularBuffer::<{ Midi::MAX_LENGTH }, u8>::new();
    let message = Midi::control_change(Channel::Ch2, 7, 32);
    let result = message.write(&mut output);
    assert_eq!(result, Ok(()));
    assert_eq!(output, hex!("B1 07 20"));
}
