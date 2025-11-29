#![no_std]

use embedded_io::Write;

pub mod channel_voice_message;
pub mod system_exclusive_message;
pub mod usb;
pub mod util;

pub use channel_voice_message::{ChannelVoiceMessage, Note, cc};
pub use system_exclusive_message::{ALL_CALL_DEVICE_ID, SystemExclusiveMessage};

pub enum Message {
    ChannelVoice(ChannelVoiceMessage),
    SystemExclusive(SystemExclusiveMessage),
}

impl Message {
    pub fn write<T: Write>(&self, writer: &mut T) -> Result<(), T::Error> {
        match self {
            Message::ChannelVoice(m) => m.write(writer),
            Message::SystemExclusive(m) => m.write(writer),
        }
    }
}

impl From<ChannelVoiceMessage> for Message {
    fn from(value: ChannelVoiceMessage) -> Self {
        Message::ChannelVoice(value)
    }
}

impl From<SystemExclusiveMessage> for Message {
    fn from(value: SystemExclusiveMessage) -> Self {
        Message::SystemExclusive(value)
    }
}
