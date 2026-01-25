use i2c_proto::*;
use tokio::{sync, task};
use version::FirmwareVersion;

struct Pipe {
    controller: HalfPipe,
    device: HalfPipe,
}

struct HalfPipe {
    output: sync::mpsc::UnboundedSender<Vec<u8>>,
    input: sync::mpsc::UnboundedReceiver<Vec<u8>>,
}

impl Pipe {
    fn new() -> Self {
        let (controller_out, device_in) = sync::mpsc::unbounded_channel();
        let (device_out, controller_in) = sync::mpsc::unbounded_channel();
        Pipe {
            controller: HalfPipe {
                output: controller_out,
                input: controller_in,
            },
            device: HalfPipe {
                output: device_out,
                input: device_in,
            },
        }
    }
}

fn broken_pipe() -> std::io::Error {
    std::io::Error::from(std::io::ErrorKind::BrokenPipe)
}

impl ControllerIo for HalfPipe {
    type Error = std::io::Error;

    async fn read(&mut self, _address: u8, read: &mut [u8]) -> Result<(), std::io::Error> {
        let result = self.input.recv().await;
        if let Some(d) = result {
            let length = std::cmp::min(d.len(), read.len());
            read[..length].copy_from_slice(&d[..length]);
            read[length..].fill(0xFF);
            return Ok(());
        }
        Err(broken_pipe())
    }

    async fn write(&mut self, _address: u8, write: &[u8]) -> Result<(), std::io::Error> {
        self.output
            .send(Vec::from(write))
            .map_err(|_| broken_pipe())
    }

    async fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), std::io::Error> {
        self.write(address, write).await?;
        self.read(address, read).await
    }
}

impl DeviceIo for HalfPipe {
    type Command = ();
    type SendStatus = ();
    type Error = std::io::Error;

    async fn listen(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }

    async fn respond_to_read(&mut self, write: &[u8]) -> Result<(), std::io::Error> {
        self.output
            .send(Vec::from(write))
            .map_err(|_| broken_pipe())
    }

    async fn respond_to_write(&mut self, read: &mut [u8]) -> Result<usize, std::io::Error> {
        let result = self.input.recv().await;
        if let Some(d) = result {
            if d.len() > read.len() {
                return Err(std::io::Error::from(std::io::ErrorKind::Other));
            }
            read[..d.len()].copy_from_slice(d.as_slice());
            return Ok(d.len());
        }
        Err(broken_pipe())
    }
}

#[tokio::test]
async fn get_buttons() {
    let pipe = Pipe::new();
    let mut controller = Controller::new(pipe.controller);
    let mut device = Device::new(pipe.device);

    let mut tasks = task::JoinSet::new();

    tasks.spawn(async move {
        let addr = 0x22;
        let buttons = controller.get_buttons(addr).await.unwrap();
        assert_eq!(buttons, 0x1234);
    });

    tasks.spawn(async move {
        let buttons = 0x1234;
        device.send_buttons(buttons).await.unwrap();
    });

    tasks.join_all().await;
}

#[tokio::test]
async fn get_version_release() {
    let pipe = Pipe::new();
    let mut controller = Controller::new(pipe.controller);
    let mut device = Device::new(pipe.device);

    let mut tasks = task::JoinSet::new();

    tasks.spawn(async move {
        let addr = 0x22;
        let version = controller.get_version(addr).await.unwrap();
        assert_eq!(
            version,
            FirmwareVersion {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: None,
            }
        );
    });

    tasks.spawn(async move {
        let command = device.receive_command().await.unwrap();
        assert_eq!(command, Command::GetVersion);

        let version = FirmwareVersion {
            major: 1,
            minor: 2,
            patch: 3,
            prerelease: None,
        };
        device.send_version(&version).await.unwrap();
    });

    tasks.join_all().await;
}

#[tokio::test]
async fn get_version_prerelease() {
    let pipe = Pipe::new();
    let mut controller = Controller::new(pipe.controller);
    let mut device = Device::new(pipe.device);

    let mut tasks = task::JoinSet::new();

    tasks.spawn(async move {
        let addr = 0x22;
        let version = controller.get_version(addr).await.unwrap();
        assert_eq!(
            version,
            FirmwareVersion {
                major: 3,
                minor: 2,
                patch: 1,
                prerelease: Some("*"),
            }
        );
    });

    tasks.spawn(async move {
        let command = device.receive_command().await.unwrap();
        assert_eq!(command, Command::GetVersion);

        let version = FirmwareVersion {
            major: 3,
            minor: 2,
            patch: 1,
            prerelease: Some("nonsense"),
        };
        device.send_version(&version).await.unwrap();
    });

    tasks.join_all().await;
}

#[tokio::test]
async fn get_write_status_ready() {
    let pipe = Pipe::new();
    let mut controller = Controller::new(pipe.controller);
    let mut device = Device::new(pipe.device);

    let mut tasks = task::JoinSet::new();

    tasks.spawn(async move {
        let addr = 0x22;
        let status = controller.get_write_status(addr).await.unwrap();
        assert_eq!(status, WriteStatus::Ready);
    });

    tasks.spawn(async move {
        let command = device.receive_command().await.unwrap();
        assert_eq!(command, Command::GetWriteStatus);

        let status = WriteStatus::Ready;
        device.send_write_status(status).await.unwrap();
    });

    tasks.join_all().await;
}

#[tokio::test]
async fn get_write_status_busy() {
    let pipe = Pipe::new();
    let mut controller = Controller::new(pipe.controller);
    let mut device = Device::new(pipe.device);

    let mut tasks = task::JoinSet::new();

    tasks.spawn(async move {
        let addr = 0x22;
        let status = controller.get_write_status(addr).await.unwrap();
        assert_eq!(status, WriteStatus::Busy);
    });

    tasks.spawn(async move {
        let command = device.receive_command().await.unwrap();
        assert_eq!(command, Command::GetWriteStatus);

        let status = WriteStatus::Busy;
        device.send_write_status(status).await.unwrap();
    });

    tasks.join_all().await;
}

#[tokio::test]
async fn get_write_status_cancel() {
    let pipe = Pipe::new();
    let mut controller = Controller::new(pipe.controller);
    let mut device = Device::new(pipe.device);

    let mut tasks = task::JoinSet::new();

    tasks.spawn(async move {
        let addr = 0x22;
        let status = controller.get_write_status(addr).await.unwrap();
        assert_eq!(status, WriteStatus::Cancel);
    });

    tasks.spawn(async move {
        let command = device.receive_command().await.unwrap();
        assert_eq!(command, Command::GetWriteStatus);

        let status = WriteStatus::Cancel;
        device.send_write_status(status).await.unwrap();
    });

    tasks.join_all().await;
}

#[tokio::test]
async fn write_begin() {
    let pipe = Pipe::new();
    let mut controller = Controller::new(pipe.controller);
    let mut device = Device::new(pipe.device);

    let mut tasks = task::JoinSet::new();

    tasks.spawn(async move {
        let addr = 0x22;
        controller.write_begin(addr).await.unwrap();
    });

    tasks.spawn(async move {
        let command = device.receive_command().await.unwrap();
        assert_eq!(command, Command::WriteBegin);
    });

    tasks.join_all().await;
}

#[tokio::test]
async fn write_packet() {
    let pipe = Pipe::new();
    let mut controller = Controller::new(pipe.controller);
    let mut device = Device::new(pipe.device);

    let mut tasks = task::JoinSet::new();

    tasks.spawn(async move {
        let addr = 0x22;
        let data = [1, 1, 2, 3, 5, 8, 13, 21];
        controller.write_packet(addr, &data).await.unwrap();
    });

    tasks.spawn(async move {
        let length = 8;
        let mut data = [0; 256];
        data[..length].copy_from_slice(&[1, 1, 2, 3, 5, 8, 13, 21]);

        let command = device.receive_command().await.unwrap();
        assert_eq!(command, Command::WritePacket { data, length });
    });

    tasks.join_all().await;
}

#[tokio::test]
async fn write_end() {
    let pipe = Pipe::new();
    let mut controller = Controller::new(pipe.controller);
    let mut device = Device::new(pipe.device);

    let mut tasks = task::JoinSet::new();

    tasks.spawn(async move {
        let addr = 0x22;
        controller.write_end(addr).await.unwrap();
    });

    tasks.spawn(async move {
        let command = device.receive_command().await.unwrap();
        assert_eq!(command, Command::WriteEnd);
    });

    tasks.join_all().await;
}
