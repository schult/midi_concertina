use core::slice::Chunks;
use embassy_stm32::i2c;
use i2c_proto;

#[derive(Clone, PartialEq)]
pub enum Status {
    InProgress,
    Finished,
    Failed,
}

pub struct Session<'a> {
    address: u8,
    beginning: Option<Chunks<'a, u8>>,
    iter: Option<Chunks<'a, u8>>,
    retries_remaining: u8,
    status: Status,
}

impl<'a> Session<'a> {
    pub fn new(address: u8) -> Self {
        Self {
            address,
            beginning: None,
            iter: None,
            retries_remaining: 0,
            status: Status::Finished,
        }
    }

    pub fn begin(&mut self, data: &'a [u8]) {
        self.beginning = Some(data.chunks(i2c_proto::PACKET_MAX_PAYLOAD_SIZE));
        self.retries_remaining = 5;
        self.status = Status::InProgress;
    }

    pub async fn poll(
        &mut self,
        i2c: &mut i2c_proto::Controller<crate::i2c::I2cWrapper<'a>>,
    ) -> Result<Status, i2c_proto::ControllerError<i2c::Error>> {
        if self.status != Status::InProgress {
            match i2c.get_write_status(self.address).await {
                Ok(i2c_proto::WriteStatus::Ready) => {
                    if let Some(chunk) = self.iter.as_mut().unwrap().next() {
                        i2c.write_packet(self.address, chunk).await?;
                    } else {
                        i2c.write_end(self.address).await?;
                        self.status = Status::Finished;
                    }
                }
                Ok(i2c_proto::WriteStatus::Cancel) => {
                    if self.retries_remaining > 0 {
                        self.iter = self.beginning.clone();
                        self.retries_remaining -= 1;
                        i2c.write_begin(self.address).await?;
                    } else {
                        self.status = Status::Failed;
                    }
                }
                _ => (),
            }
        }
        return Ok(self.status.clone());
    }
}
