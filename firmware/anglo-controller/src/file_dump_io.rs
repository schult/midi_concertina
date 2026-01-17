use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::Sender;

pub use util::DfuWriter;

type MessageSender = Sender<'static, NoopRawMutex, midi::Message, 8>;

pub struct MessageChannelAdapter {
    sender: MessageSender,
}

impl MessageChannelAdapter {
    pub fn new(sender: MessageSender) -> Self {
        MessageChannelAdapter { sender }
    }
}

impl midi::util::MessageSender for MessageChannelAdapter {
    async fn send(&mut self, message: midi::Message) {
        self.sender.send(message).await;
    }
}
