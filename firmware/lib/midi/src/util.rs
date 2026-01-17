#![allow(async_fn_in_trait)]

use crate::{ALL_CALL_DEVICE_ID, Message};

pub trait FileWriter {
    type ErrorType;

    async fn open(&mut self) -> Result<(), Self::ErrorType>;
    async fn write(&mut self, data: &[u8]) -> Result<(), Self::ErrorType>;
    async fn close(&mut self) -> Result<(), Self::ErrorType>;
}

pub trait MessageSender {
    async fn send(&mut self, message: Message);
}

struct FileDumpProgress {
    source_id: u8,
    packet_num: u8,
}

impl FileDumpProgress {
    fn next_packet(&self) -> u8 {
        (self.packet_num + 1) % 0x80
    }
}

pub struct FileDumpReceiver<'a, T: FileWriter, U: MessageSender> {
    file: &'a mut T,
    message: &'a mut U,
    device_id: u8,
    raw_file_type: [u8; 4],
    progress: Option<FileDumpProgress>,
}

impl<'a, T: FileWriter, U: MessageSender> FileDumpReceiver<'a, T, U> {
    pub fn new(file: &'a mut T, message: &'a mut U, device_id: u8, file_type: &str) -> Self {
        assert!(device_id < 0x80);

        let mut raw_file_type = [b' '; 4];
        raw_file_type.copy_from_slice(file_type.as_bytes());
        assert!(raw_file_type.iter().all(|x| *x < 0x80));

        FileDumpReceiver {
            file,
            message,
            device_id,
            raw_file_type,
            progress: None,
        }
    }

    async fn cancel(&mut self) {
        if let Some(progress) = &mut self.progress {
            self.message
                .send(Message::cancel(progress.source_id, progress.packet_num))
                .await;
            self.progress = None;
        }
    }

    pub async fn process(&mut self, message: Message) {
        if let Message::FileDumpHeader(header) = message {
            if ![self.device_id, ALL_CALL_DEVICE_ID].contains(&header.device_id) {
                return;
            }

            if header.raw_file_type != self.raw_file_type {
                self.message
                    .send(Message::cancel(header.source_id, 0))
                    .await;
                return;
            }

            self.message.send(Message::wait(header.source_id, 0)).await;
            let result = self.file.open().await;
            if result.is_err() {
                self.message
                    .send(Message::cancel(header.source_id, 0))
                    .await;
                return;
            }
            self.message.send(Message::ack(header.source_id, 0)).await;
            self.progress = Some(FileDumpProgress {
                source_id: header.source_id,
                packet_num: 0,
            });
        } else if let Some(progress) = &mut self.progress {
            match message {
                Message::FileDumpPacket(packet) => {
                    if ![self.device_id, ALL_CALL_DEVICE_ID].contains(&packet.device_id) {
                        return;
                    }

                    if packet.packet_num != progress.packet_num {
                        self.cancel().await;
                        return;
                    }

                    if !packet.checksum_ok {
                        self.message
                            .send(Message::nak(progress.source_id, progress.packet_num))
                            .await;
                        return;
                    }

                    self.message
                        .send(Message::wait(progress.source_id, progress.packet_num))
                        .await;
                    let result = self.file.write(packet.data()).await;
                    if result.is_err() {
                        self.cancel().await;
                        return;
                    }
                    self.message
                        .send(Message::ack(progress.source_id, progress.packet_num))
                        .await;
                    progress.packet_num = progress.next_packet();
                }
                Message::Eof(handshake) => {
                    if ![self.device_id, ALL_CALL_DEVICE_ID].contains(&handshake.device_id) {
                        return;
                    }

                    let _ = self.file.close().await;
                    self.progress = None;
                }
                _ => (),
            }
        }
    }
}
