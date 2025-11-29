#![no_std]

pub mod channel_voice_message;
pub mod system_exclusive_message;
pub mod usb;
pub mod util;

pub use channel_voice_message::{Channel, ChannelVoiceMessage, Note, cc};
pub use system_exclusive_message::{ALL_CALL_DEVICE_ID, SystemExclusiveMessage};

pub enum Message {
    ChannelVoice(ChannelVoiceMessage),
    SystemExclusive(SystemExclusiveMessage),
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
