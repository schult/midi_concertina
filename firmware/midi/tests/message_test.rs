use circular_buffer::CircularBuffer;
use hex_literal::hex;
use midi::*;

#[test]
fn message_note_on_constructor() {
    let value = Message::note_on(2, Note::Ab4, 64);

    assert!(matches!(value, Message::NoteOn(_)));
    if let Message::NoteOn(d) = value {
        assert_eq!(d.channel, 2);
        assert_eq!(d.note, Note::Ab4);
        assert_eq!(d.velocity, 64);
    }
}

#[test]
#[should_panic]
fn message_note_on_constructor_panics_if_channel_over_4_bits() {
    let _ = Message::note_on(1 << 4, Note::Ab4, 0);
}

#[test]
#[should_panic]
fn message_note_on_constructor_panics_if_velocity_over_7_bits() {
    let _ = Message::note_on(2, Note::Ab4, 1 << 7);
}

#[test]
fn message_note_off_constructor() {
    let value = Message::note_off(2, Note::Ab4, 64);

    assert!(matches!(value, Message::NoteOff(_)));
    if let Message::NoteOff(d) = value {
        assert_eq!(d.channel, 2);
        assert_eq!(d.note, Note::Ab4);
        assert_eq!(d.velocity, 64);
    }
}

#[test]
#[should_panic]
fn message_note_off_constructor_panics_if_channel_over_4_bits() {
    let _ = Message::note_off(1 << 4, Note::Ab4, 0);
}

#[test]
#[should_panic]
fn message_note_off_constructor_panics_if_velocity_over_7_bits() {
    let _ = Message::note_off(2, Note::Ab4, 1 << 7);
}

#[test]
fn message_control_change_constructor() {
    let value = Message::control_change(2, 31, 64);

    assert!(matches!(value, Message::ControlChange(_)));
    if let Message::ControlChange(d) = value {
        assert_eq!(d.channel, 2);
        assert_eq!(d.control, 31);
        assert_eq!(d.value, 64);
    }
}

#[test]
#[should_panic]
fn message_control_change_constructor_panics_if_channel_over_4_bits() {
    let _ = Message::control_change(1 << 4, 31, 64);
}

#[test]
#[should_panic]
fn message_control_change_constructor_panics_if_control_over_7_bits() {
    let _ = Message::control_change(2, 1 << 7, 64);
}

#[test]
#[should_panic]
fn message_control_change_constructor_panics_if_value_over_7_bits() {
    let _ = Message::control_change(2, 31, 1 << 7);
}

#[test]
fn message_write_note_on() {
    let mut output = CircularBuffer::<{ Message::MAX_LENGTH }, u8>::new();
    let message = Message::note_on(1, Note::C4, 64);
    let result = message.write(&mut output);
    assert_eq!(result, Ok(()));
    assert_eq!(output, hex!("91 3c 40"));
}

#[test]
fn message_write_note_off() {
    let mut output = CircularBuffer::<{ Message::MAX_LENGTH }, u8>::new();
    let message = Message::note_off(1, Note::C4, 64);
    let result = message.write(&mut output);
    assert_eq!(result, Ok(()));
    assert_eq!(output, hex!("81 3c 40"));
}

#[test]
fn message_write_control_change() {
    let mut output = CircularBuffer::<{ Message::MAX_LENGTH }, u8>::new();
    let message = Message::control_change(1, 7, 32);
    let result = message.write(&mut output);
    assert_eq!(result, Ok(()));
    assert_eq!(output, hex!("B1 07 20"));
}
