use embassy_sync::channel;

pub use util::DfuWriter;

pub struct MessageChannelAdapter {
    sender: channel::DynamicSender<'static, midi::Message>,
}

impl MessageChannelAdapter {
    pub fn new(sender: channel::DynamicSender<'static, midi::Message>) -> Self {
        MessageChannelAdapter { sender }
    }
}

impl midi::util::MessageSender for MessageChannelAdapter {
    async fn send(&mut self, message: midi::Message) {
        self.sender.send(message).await;
    }
}
