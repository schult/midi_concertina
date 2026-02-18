use i2c_proto::*;
use mockall::mock;
use mockall::predicate::*;
use version::FirmwareVersion;

mock! {
    DeviceIo {}
    impl DeviceIo for DeviceIo {
        type Command = ();
        type SendStatus = i32;
        type Error = std::io::Error;

        async fn listen(&mut self) -> Result<(), std::io::Error>;
        async fn respond_to_read(&mut self, write: &[u8]) -> Result<i32, std::io::Error>;
        async fn respond_to_write(&mut self, read: &mut [u8]) -> Result<usize, std::io::Error>;
    }
}

#[tokio::test]
async fn send_buttons_propogates_io_error() {
    let mut io = MockDeviceIo::new();
    io.expect_respond_to_read()
        .returning(|_| Err(std::io::Error::from(std::io::ErrorKind::Other)))
        .times(1);

    let buttons = 0;
    let mut device = Device::new(io);
    let result = device.send_buttons(buttons).await;

    assert!(matches!(result, Err(DeviceError::Communication(_))));
    if let Err(DeviceError::Communication(e)) = result {
        assert_eq!(e.kind(), std::io::ErrorKind::Other);
    }
}

#[tokio::test]
async fn send_buttons_propogates_send_status() {
    let mut io = MockDeviceIo::new();
    io.expect_respond_to_read().returning(|_| Ok(42)).times(1);

    let buttons = 0;
    let mut device = Device::new(io);
    let send_status = device.send_buttons(buttons).await.unwrap();

    assert_eq!(send_status, 42);
}

#[tokio::test]
async fn send_buttons_encodes_payload() {
    let mut io = MockDeviceIo::new();
    io.expect_respond_to_read()
        .with(eq([0x12, 0x34]))
        .returning(|_| Ok(42))
        .times(1);

    let buttons = 0x1234;
    let mut device = Device::new(io);
    device.send_buttons(buttons).await.unwrap();
}

#[tokio::test]
async fn send_version_propogates_io_error() {
    let mut io = MockDeviceIo::new();
    io.expect_respond_to_read()
        .returning(|_| Err(std::io::Error::from(std::io::ErrorKind::Other)))
        .times(1);

    let version = FirmwareVersion {
        major: 1,
        minor: 2,
        patch: 3,
        prerelease: None,
    };
    let mut device = Device::new(io);
    let result = device.send_version(&version).await;

    assert!(matches!(result, Err(DeviceError::Communication(_))));
    if let Err(DeviceError::Communication(e)) = result {
        assert_eq!(e.kind(), std::io::ErrorKind::Other);
    }
}

#[tokio::test]
async fn send_version_propogates_send_status() {
    let mut io = MockDeviceIo::new();
    io.expect_respond_to_read().returning(|_| Ok(42)).times(1);

    let version = FirmwareVersion {
        major: 1,
        minor: 2,
        patch: 3,
        prerelease: None,
    };
    let mut device = Device::new(io);
    let send_status = device.send_version(&version).await.unwrap();

    assert_eq!(send_status, 42);
}

#[tokio::test]
async fn send_version_encodes_release_payload() {
    let mut io = MockDeviceIo::new();
    io.expect_respond_to_read()
        .withf(|d| d.starts_with(&[1, 2, 3, 0]))
        .returning(|_| Ok(0))
        .times(1);

    let version = FirmwareVersion {
        major: 1,
        minor: 2,
        patch: 3,
        prerelease: None,
    };
    let mut device = Device::new(io);
    device.send_version(&version).await.unwrap();
}

#[tokio::test]
async fn send_version_encodes_prerelease_payload() {
    let mut io = MockDeviceIo::new();
    io.expect_respond_to_read()
        .withf(|d| d.starts_with(&[1, 2, 3, '*' as u8]))
        .returning(|_| Ok(0))
        .times(1);

    let version = FirmwareVersion {
        major: 1,
        minor: 2,
        patch: 3,
        prerelease: Some("arbitrary"),
    };
    let mut device = Device::new(io);
    device.send_version(&version).await.unwrap();
}

#[tokio::test]
async fn send_version_calculates_checksum() {
    let mut io = MockDeviceIo::new();
    io.expect_respond_to_read()
        .withf(|d| d.ends_with(&[48]))
        .returning(|_| Ok(0))
        .times(1);

    let version = FirmwareVersion {
        major: 1,
        minor: 2,
        patch: 3,
        prerelease: Some("arbitrary"),
    };
    let mut device = Device::new(io);
    device.send_version(&version).await.unwrap();
}

#[tokio::test]
async fn send_version_sends_correct_length() {
    let mut io = MockDeviceIo::new();
    io.expect_respond_to_read()
        .withf(|d| d.len() == 5)
        .returning(|_| Ok(0))
        .times(1);

    let version = FirmwareVersion {
        major: 1,
        minor: 2,
        patch: 3,
        prerelease: None,
    };
    let mut device = Device::new(io);
    device.send_version(&version).await.unwrap();
}

#[tokio::test]
async fn send_write_status_propogates_io_error() {
    let mut io = MockDeviceIo::new();
    io.expect_respond_to_read()
        .returning(|_| Err(std::io::Error::from(std::io::ErrorKind::Other)))
        .times(1);

    let status = WriteStatus::Ready;
    let mut device = Device::new(io);
    let result = device.send_write_status(status).await;

    assert!(matches!(result, Err(DeviceError::Communication(_))));
    if let Err(DeviceError::Communication(e)) = result {
        assert_eq!(e.kind(), std::io::ErrorKind::Other);
    }
}

#[tokio::test]
async fn send_write_status_propogates_send_status() {
    let mut io = MockDeviceIo::new();
    io.expect_respond_to_read().returning(|_| Ok(42)).times(1);

    let status = WriteStatus::Ready;
    let mut device = Device::new(io);
    let send_status = device.send_write_status(status).await.unwrap();

    assert_eq!(send_status, 42);
}

#[tokio::test]
async fn send_write_status_encodes_ready() {
    let mut io = MockDeviceIo::new();
    io.expect_respond_to_read()
        .with(eq([0x00, 0xFF]))
        .returning(|_| Ok(0))
        .times(1);

    let status = WriteStatus::Ready;
    let mut device = Device::new(io);
    device.send_write_status(status).await.unwrap();
}

#[tokio::test]
async fn send_write_status_encodes_busy() {
    let mut io = MockDeviceIo::new();
    io.expect_respond_to_read()
        .with(eq([0x01, 0xFE]))
        .returning(|_| Ok(0))
        .times(1);

    let status = WriteStatus::Busy;
    let mut device = Device::new(io);
    device.send_write_status(status).await.unwrap();
}

#[tokio::test]
async fn send_write_status_encodes_cancel() {
    let mut io = MockDeviceIo::new();
    io.expect_respond_to_read()
        .with(eq([0x02, 0xFD]))
        .returning(|_| Ok(0))
        .times(1);

    let status = WriteStatus::Cancel;
    let mut device = Device::new(io);
    device.send_write_status(status).await.unwrap();
}

#[tokio::test]
async fn receive_command_propogates_io_error() {
    let mut io = MockDeviceIo::new();
    io.expect_respond_to_write()
        .returning(|_| Err(std::io::Error::from(std::io::ErrorKind::Other)))
        .times(1);

    let mut device = Device::new(io);
    let result = device.receive_command().await;

    assert!(matches!(result, Err(DeviceError::Communication(_))));
    if let Err(DeviceError::Communication(e)) = result {
        assert_eq!(e.kind(), std::io::ErrorKind::Other);
    }
}

#[tokio::test]
async fn receive_command_reports_messages_that_are_too_short() {
    let mut io = MockDeviceIo::new();
    io.expect_respond_to_write()
        .returning(|d| {
            d[0] = 0x02;
            Ok(1)
        })
        .times(1);

    let mut device = Device::new(io);
    let result = device.receive_command().await;

    assert!(matches!(result, Err(DeviceError::ShortMessage(1))));
}

#[tokio::test]
async fn receive_command_reports_messages_that_are_too_long() {
    let mut io = MockDeviceIo::new();
    io.expect_respond_to_write()
        .returning(|d| {
            d[0] = 0x03;
            d[1] = 0xFC;
            Ok(10_000)
        })
        .times(1);

    let mut device = Device::new(io);
    let result = device.receive_command().await;

    assert!(matches!(result, Err(DeviceError::LongMessage(10_000))));
}

#[tokio::test]
async fn receive_command_reports_corrupt_command() {
    let mut io = MockDeviceIo::new();
    io.expect_respond_to_write()
        .returning(|d| {
            d[0] = 0x02;
            d[1] = 0xFE;
            Ok(2)
        })
        .times(1);

    let mut device = Device::new(io);
    let result = device.receive_command().await;

    assert!(matches!(result, Err(DeviceError::CorruptCommand)));
}

#[tokio::test]
async fn receive_command_reports_unknown_command() {
    let mut io = MockDeviceIo::new();
    io.expect_respond_to_write()
        .returning(|d| {
            d[0] = 0x33;
            d[1] = 0xCC;
            Ok(2)
        })
        .times(1);

    let mut device = Device::new(io);
    let result = device.receive_command().await;

    assert!(matches!(result, Err(DeviceError::UnknownCommand(0x33))));
}

#[tokio::test]
async fn receive_command_parses_get_version() {
    let mut io = MockDeviceIo::new();
    io.expect_respond_to_write()
        .returning(|d| {
            d[0] = 0x00;
            d[1] = 0xFF;
            Ok(2)
        })
        .times(1);

    let mut device = Device::new(io);
    let command = device.receive_command().await.unwrap();

    assert_eq!(command, Command::GetVersion);
}

#[tokio::test]
async fn receive_command_parses_get_write_status() {
    let mut io = MockDeviceIo::new();
    io.expect_respond_to_write()
        .returning(|d| {
            d[0] = 0x01;
            d[1] = 0xFE;
            Ok(2)
        })
        .times(1);

    let mut device = Device::new(io);
    let command = device.receive_command().await.unwrap();

    assert_eq!(command, Command::GetWriteStatus);
}

#[tokio::test]
async fn receive_command_parses_write_begin() {
    let mut io = MockDeviceIo::new();
    io.expect_respond_to_write()
        .returning(|d| {
            d[0] = 0x02;
            d[1] = 0xFD;
            Ok(2)
        })
        .times(1);

    let mut device = Device::new(io);
    let command = device.receive_command().await.unwrap();

    assert_eq!(command, Command::WriteBegin);
}

#[tokio::test]
async fn receive_command_parses_write_packet() {
    let mut io = MockDeviceIo::new();
    io.expect_respond_to_write()
        .returning(|d| {
            d[0] = 0x03;
            d[1] = 0xFC;
            d[2] = 0x12;
            d[3] = 0x34;
            d[4] = 0x46;
            Ok(5)
        })
        .times(1);

    let mut device = Device::new(io);
    let command = device.receive_command().await.unwrap();

    assert!(matches!(
        command,
        Command::WritePacket { data: _, length: _ }
    ));
    if let Command::WritePacket { data, length } = command {
        assert!(data.starts_with(&[0x12, 0x34]));
        assert_eq!(length, 2);
    }
}

#[tokio::test]
async fn receive_command_reports_write_packet_that_is_empty() {
    let mut io = MockDeviceIo::new();
    io.expect_respond_to_write()
        .returning(|d| {
            d[0] = 0x03;
            d[1] = 0xFC;
            d[2] = 0x00;
            Ok(3)
        })
        .times(1);

    let mut device = Device::new(io);
    let result = device.receive_command().await;

    assert!(matches!(result, Err(DeviceError::EmptyPacket)));
}

#[tokio::test]
async fn receive_command_reports_write_packet_with_bad_checksum() {
    let mut io = MockDeviceIo::new();
    io.expect_respond_to_write()
        .returning(|d| {
            d[0] = 0x03;
            d[1] = 0xFC;
            d[2] = 0x67;
            d[3] = 0x68;
            Ok(4)
        })
        .times(1);

    let mut device = Device::new(io);
    let result = device.receive_command().await;

    assert!(matches!(result, Err(DeviceError::CorruptMessage)));
}

#[tokio::test]
async fn receive_command_parses_write_end() {
    let mut io = MockDeviceIo::new();
    io.expect_respond_to_write()
        .returning(|d| {
            d[0] = 0x04;
            d[1] = 0xFB;
            Ok(2)
        })
        .times(1);

    let mut device = Device::new(io);
    let command = device.receive_command().await.unwrap();

    assert_eq!(command, Command::WriteEnd);
}
