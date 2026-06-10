use assert_float_eq::*;
use i2c_proto::*;
use mockall::mock;
use mockall::predicate::*;
use version::FirmwareVersion;

mock! {
    ControllerIo {}
    impl ControllerIo for ControllerIo {
        type Error = std::io::Error;

        async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), std::io::Error>;
        async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), std::io::Error>;
        async fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), std::io::Error>;
    }
}

#[tokio::test]
async fn get_buttons_forwards_address() {
    let mut io = MockControllerIo::new();
    io.expect_read()
        .with(eq(0x22), always())
        .returning(|_, _| Ok(()))
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let _ = controller.get_buttons(address).await;
}

#[tokio::test]
async fn get_buttons_propogates_io_error() {
    let mut io = MockControllerIo::new();
    io.expect_read()
        .returning(|_, _| Err(std::io::Error::from(std::io::ErrorKind::Other)))
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let result = controller.get_buttons(address).await;

    assert!(matches!(result, Err(ControllerError::Communication(_))));
    if let Err(ControllerError::Communication(e)) = result {
        assert_eq!(e.kind(), std::io::ErrorKind::Other);
    }
}

#[tokio::test]
async fn get_buttons_parses_response() {
    let mut io = MockControllerIo::new();
    io.expect_read()
        .returning(|_, buffer| {
            let buttons: u16 = 0x1234;
            buffer.copy_from_slice(&buttons.to_be_bytes());
            Ok(())
        })
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let buttons = controller.get_buttons(address).await.unwrap();
    assert_eq!(buttons, 0x1234);
}

#[tokio::test]
async fn get_version_forwards_address() {
    let mut io = MockControllerIo::new();
    io.expect_write_read()
        .with(eq(0x22), always(), always())
        .returning(|_, _, _| Ok(()))
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let _ = controller.get_version(address).await;
}

#[tokio::test]
async fn get_version_sends_command() {
    let mut io = MockControllerIo::new();
    io.expect_write_read()
        .with(always(), eq([0x00, 0xFF]), always())
        .returning(|_, _, _| Ok(()))
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let _ = controller.get_version(address).await;
}

#[tokio::test]
async fn get_version_propogates_io_error() {
    let mut io = MockControllerIo::new();
    io.expect_write_read()
        .returning(|_, _, _| Err(std::io::Error::from(std::io::ErrorKind::Other)))
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let result = controller.get_version(address).await;

    assert!(matches!(result, Err(ControllerError::Communication(_))));
    if let Err(ControllerError::Communication(e)) = result {
        assert_eq!(e.kind(), std::io::ErrorKind::Other);
    }
}

#[tokio::test]
async fn get_version_parses_response() {
    let mut io = MockControllerIo::new();
    io.expect_write_read()
        .returning(|_, _, buffer| {
            buffer.copy_from_slice(&[1, 2, 3, 0, 6]);
            Ok(())
        })
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let version = controller.get_version(address).await.unwrap();
    assert_eq!(
        version,
        FirmwareVersion {
            major: 1,
            minor: 2,
            patch: 3,
            prerelease: None,
        }
    );
}

#[tokio::test]
async fn get_version_produces_generic_prerelease() {
    let mut io = MockControllerIo::new();
    io.expect_write_read()
        .returning(|_, _, buffer| {
            buffer.copy_from_slice(&[1, 2, 3, 1, 7]);
            Ok(())
        })
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let version = controller.get_version(address).await.unwrap();
    assert_eq!(
        version,
        FirmwareVersion {
            major: 1,
            minor: 2,
            patch: 3,
            prerelease: Some("*"),
        }
    );
}

#[tokio::test]
async fn get_version_reports_bad_checksum() {
    let mut io = MockControllerIo::new();
    io.expect_write_read()
        .returning(|_, _, buffer| {
            buffer.copy_from_slice(&[1, 2, 3, 1, 6]);
            Ok(())
        })
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let error = controller.get_version(address).await.unwrap_err();
    assert!(matches!(error, ControllerError::CorruptMessage));
}

#[tokio::test]
async fn get_write_status_forwards_address() {
    let mut io = MockControllerIo::new();
    io.expect_write_read()
        .with(eq(0x22), always(), always())
        .returning(|_, _, _| Ok(()))
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let _ = controller.get_write_status(address).await;
}

#[tokio::test]
async fn get_write_status_sends_command() {
    let mut io = MockControllerIo::new();
    io.expect_write_read()
        .with(always(), eq([0x01, 0xFE]), always())
        .returning(|_, _, _| Ok(()))
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let _ = controller.get_write_status(address).await;
}

#[tokio::test]
async fn get_write_status_propogates_io_error() {
    let mut io = MockControllerIo::new();
    io.expect_write_read()
        .returning(|_, _, _| Err(std::io::Error::from(std::io::ErrorKind::Other)))
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let result = controller.get_write_status(address).await;

    assert!(matches!(result, Err(ControllerError::Communication(_))));
    if let Err(ControllerError::Communication(e)) = result {
        assert_eq!(e.kind(), std::io::ErrorKind::Other);
    }
}

#[tokio::test]
async fn get_write_status_parses_ready_response() {
    let mut io = MockControllerIo::new();
    io.expect_write_read()
        .returning(|_, _, buffer| {
            buffer.copy_from_slice(&[0x00, 0xFF]);
            Ok(())
        })
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let status = controller.get_write_status(address).await.unwrap();
    assert_eq!(status, WriteStatus::Ready);
}

#[tokio::test]
async fn get_write_status_parses_busy_response() {
    let mut io = MockControllerIo::new();
    io.expect_write_read()
        .returning(|_, _, buffer| {
            buffer.copy_from_slice(&[0x01, 0xFE]);
            Ok(())
        })
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let status = controller.get_write_status(address).await.unwrap();
    assert_eq!(status, WriteStatus::Busy);
}

#[tokio::test]
async fn get_write_status_parses_cancel_response() {
    let mut io = MockControllerIo::new();
    io.expect_write_read()
        .returning(|_, _, buffer| {
            buffer.copy_from_slice(&[0x02, 0xFD]);
            Ok(())
        })
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let status = controller.get_write_status(address).await.unwrap();
    assert_eq!(status, WriteStatus::Cancel);
}

#[tokio::test]
async fn get_write_status_reports_bad_checksum() {
    let mut io = MockControllerIo::new();
    io.expect_write_read()
        .returning(|_, _, buffer| {
            buffer.copy_from_slice(&[0x01, 0xFD]);
            Ok(())
        })
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let error = controller.get_write_status(address).await.unwrap_err();
    assert!(matches!(error, ControllerError::CorruptMessage));
}

#[tokio::test]
async fn write_begin_forwards_address() {
    let mut io = MockControllerIo::new();
    io.expect_write()
        .with(eq(0x22), always())
        .returning(|_, _| Ok(()))
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let _ = controller.write_begin(address).await;
}

#[tokio::test]
async fn write_begin_sends_command() {
    let mut io = MockControllerIo::new();
    io.expect_write()
        .with(always(), eq([0x02, 0xFD]))
        .returning(|_, _| Ok(()))
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let _ = controller.write_begin(address).await;
}

#[tokio::test]
async fn write_begin_propogates_io_error() {
    let mut io = MockControllerIo::new();
    io.expect_write()
        .returning(|_, _| Err(std::io::Error::from(std::io::ErrorKind::Other)))
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let result = controller.write_begin(address).await;

    assert!(matches!(result, Err(ControllerError::Communication(_))));
    if let Err(ControllerError::Communication(e)) = result {
        assert_eq!(e.kind(), std::io::ErrorKind::Other);
    }
}

#[tokio::test]
#[should_panic]
async fn write_packet_panics_if_data_empty() {
    let io = MockControllerIo::new();
    let mut controller = Controller::new(io);
    let address = 0x22;
    let data = [];
    let _ = controller.write_packet(address, &data).await;
}

#[tokio::test]
#[should_panic]
async fn write_packet_panics_if_data_too_large() {
    let io = MockControllerIo::new();
    let mut controller = Controller::new(io);
    let address = 0x22;
    let data = [0; PACKET_MAX_PAYLOAD_SIZE + 1];
    let _ = controller.write_packet(address, &data).await;
}

#[tokio::test]
async fn write_packet_forwards_address() {
    let mut io = MockControllerIo::new();
    io.expect_write()
        .with(eq(0x22), always())
        .returning(|_, _| Ok(()))
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let data = [0];
    let _ = controller.write_packet(address, &data).await;
}

#[tokio::test]
async fn write_packet_sends_command() {
    let mut io = MockControllerIo::new();
    io.expect_write()
        .with(always(), function(|d: &[u8]| d.starts_with(&[0x03, 0xFC])))
        .returning(|_, _| Ok(()))
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let data = [0];
    let _ = controller.write_packet(address, &data).await;
}

#[tokio::test]
async fn write_packet_sends_data() {
    let mut io = MockControllerIo::new();
    io.expect_write()
        .with(always(), function(|d: &[u8]| d[2..6] == [1, 2, 3, 4]))
        .returning(|_, _| Ok(()))
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let data = [1, 2, 3, 4];
    let _ = controller.write_packet(address, &data).await;
}

#[tokio::test]
async fn write_packet_calculates_correct_checksum() {
    let mut io = MockControllerIo::new();
    io.expect_write()
        .with(always(), function(|d: &[u8]| *d.last().unwrap() == 10))
        .returning(|_, _| Ok(()))
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let data = [1, 2, 3, 4];
    let _ = controller.write_packet(address, &data).await;
}

#[tokio::test]
async fn write_packet_message_is_correct_length() {
    let mut io = MockControllerIo::new();
    io.expect_write()
        .with(always(), function(|d: &[u8]| d.len() == 7))
        .returning(|_, _| Ok(()))
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let data = [1, 2, 3, 4];
    let _ = controller.write_packet(address, &data).await;
}

#[tokio::test]
async fn write_packet_propogates_io_error() {
    let mut io = MockControllerIo::new();
    io.expect_write()
        .returning(|_, _| Err(std::io::Error::from(std::io::ErrorKind::Other)))
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let data = [0];
    let result = controller.write_packet(address, &data).await;

    assert!(matches!(result, Err(ControllerError::Communication(_))));
    if let Err(ControllerError::Communication(e)) = result {
        assert_eq!(e.kind(), std::io::ErrorKind::Other);
    }
}

#[tokio::test]
async fn write_end_forwards_address() {
    let mut io = MockControllerIo::new();
    io.expect_write()
        .with(eq(0x22), always())
        .returning(|_, _| Ok(()))
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let _ = controller.write_end(address).await;
}

#[tokio::test]
async fn write_end_sends_command() {
    let mut io = MockControllerIo::new();
    io.expect_write()
        .with(always(), eq([0x04, 0xFB]))
        .returning(|_, _| Ok(()))
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let _ = controller.write_end(address).await;
}

#[tokio::test]
async fn write_end_propogates_io_error() {
    let mut io = MockControllerIo::new();
    io.expect_write()
        .returning(|_, _| Err(std::io::Error::from(std::io::ErrorKind::Other)))
        .times(1);

    let mut controller = Controller::new(io);
    let address = 0x22;
    let result = controller.write_end(address).await;

    assert!(matches!(result, Err(ControllerError::Communication(_))));
    if let Err(ControllerError::Communication(e)) = result {
        assert_eq!(e.kind(), std::io::ErrorKind::Other);
    }
}
