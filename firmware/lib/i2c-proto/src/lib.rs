#![no_std]
#![allow(async_fn_in_trait)]

use core::error::Error;
use core::fmt::{Display, Formatter};
use version::FirmwareVersion;

// Limit i2c message size to 255 due to observed misbehavior with larger messages. It's possible
// this is the result of a bug in embassy-stm32, but I haven't investigated enough to be sure.
const PACKET_MAX_SIZE: usize = 255;
pub const PACKET_MAX_PAYLOAD_SIZE: usize = PACKET_MAX_SIZE - 3;

#[derive(Debug, PartialEq)]
pub enum Command {
    GetVersion,
    GetWriteStatus,
    WriteBegin,
    WritePacket {
        data: [u8; PACKET_MAX_PAYLOAD_SIZE],
        length: usize,
    },
    WriteEnd,
}

#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum WriteStatus {
    Ready,
    Busy,
    Cancel,
}

/*******************/

mod command_code {
    pub const GET_VERSION: u8 = 0x00;
    pub const GET_WRITE_STATUS: u8 = 0x01;
    pub const WRITE_BEGIN: u8 = 0x02;
    pub const WRITE_PACKET: u8 = 0x03;
    pub const WRITE_END: u8 = 0x04;
}

/*******************/

#[derive(Debug, PartialEq)]
pub enum ControllerError<CommError: Error> {
    CorruptMessage,
    Communication(CommError),
}

impl<E: Error> Display for ControllerError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            ControllerError::CorruptMessage => "Corrupt Message",
            ControllerError::Communication(e) => return Display::fmt(&e, f),
        };
        write!(f, "{}", message)
    }
}

impl<E: Error> Error for ControllerError<E> {}

impl<E: Error> From<E> for ControllerError<E> {
    fn from(value: E) -> Self {
        ControllerError::Communication(value)
    }
}

/*******************/

pub trait ControllerIo {
    type Error: core::error::Error;

    async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error>;
    async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error>;
    async fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error>;
}

/*******************/

pub struct Controller<IO: ControllerIo> {
    io: IO,
}

impl<IO: ControllerIo> Controller<IO> {
    pub fn new(io: IO) -> Self {
        Self { io }
    }

    pub async fn get_buttons(&mut self, address: u8) -> Result<u16, ControllerError<IO::Error>> {
        let mut buffer = [0; 2];
        self.io.read(address, &mut buffer).await?;
        Ok(u16::from_be_bytes(buffer))
    }

    pub async fn get_version(
        &mut self,
        address: u8,
    ) -> Result<FirmwareVersion<'static>, ControllerError<IO::Error>> {
        let command = command_code::GET_VERSION;
        let message = [command, !command];
        let mut buffer = [0; 5];
        self.io.write_read(address, &message, &mut buffer).await?;

        let (check, version) = buffer.split_last().unwrap();
        if *check != version.iter().fold(0, |acc, e| acc.wrapping_add(*e)) {
            return Err(ControllerError::CorruptMessage);
        }

        Ok(version::FirmwareVersion {
            major: version[0],
            minor: version[1],
            patch: version[2],
            prerelease: match version[3] {
                0 => None,
                _ => Some("*"),
            },
        })
    }

    pub async fn get_write_status(
        &mut self,
        address: u8,
    ) -> Result<WriteStatus, ControllerError<IO::Error>> {
        let command = command_code::GET_WRITE_STATUS;
        let message = [command, !command];
        let mut status = [0, 2];
        self.io.write_read(address, &message, &mut status).await?;

        if status[0] != !status[1] {
            return Err(ControllerError::CorruptMessage);
        }

        match status[0] {
            0x00 => Ok(WriteStatus::Ready),
            0x01 => Ok(WriteStatus::Busy),
            0x02 => Ok(WriteStatus::Cancel),
            _ => Ok(WriteStatus::Cancel),
        }
    }

    pub async fn write_begin(&mut self, address: u8) -> Result<(), ControllerError<IO::Error>> {
        let command = command_code::WRITE_BEGIN;
        let message = [command, !command];
        self.io
            .write(address, &message)
            .await
            .map_err(ControllerError::from)
    }

    pub async fn write_packet(
        &mut self,
        address: u8,
        data: &[u8],
    ) -> Result<(), ControllerError<IO::Error>> {
        assert!(data.len() > 0);
        assert!(data.len() <= PACKET_MAX_PAYLOAD_SIZE);

        let mut message = [0u8; PACKET_MAX_SIZE];
        let command = command_code::WRITE_PACKET;
        let data_idx = 2;
        let check_idx = data_idx + data.len();
        let message_length = check_idx + 1;
        message[0] = command;
        message[1] = !command;
        message[data_idx..check_idx].copy_from_slice(&data);
        message[check_idx] = message[data_idx..check_idx]
            .iter()
            .fold(0, |acc, e| acc.wrapping_add(*e));

        self.io
            .write(address, &message[..message_length])
            .await
            .map_err(ControllerError::from)
    }

    pub async fn write_end(&mut self, address: u8) -> Result<(), ControllerError<IO::Error>> {
        let command = command_code::WRITE_END;
        let message = [command, !command];
        self.io
            .write(address, &message)
            .await
            .map_err(ControllerError::from)
    }
}

/*******************/

#[derive(Debug)]
pub enum DeviceError<CommError: Error> {
    ShortMessage(usize),
    LongMessage(usize),
    UnknownCommand(u8),
    CorruptCommand,
    CorruptMessage,
    EmptyPacket,
    Communication(CommError),
}

impl<E: Error> Display for DeviceError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            DeviceError::ShortMessage(length) => write!(f, "Short Message: {length} bytes"),
            DeviceError::LongMessage(length) => write!(f, "Long Message: {length} bytes"),
            DeviceError::UnknownCommand(command) => write!(f, "Unknown Command: {command:#04x}"),
            DeviceError::CorruptCommand => write!(f, "Corrupt Command"),
            DeviceError::CorruptMessage => write!(f, "Corrupt Message"),
            DeviceError::EmptyPacket => write!(f, "Empty Packet"),
            DeviceError::Communication(e) => Display::fmt(&e, f),
        }
    }
}

impl<E: Error> Error for DeviceError<E> {}

impl<E: Error> From<E> for DeviceError<E> {
    fn from(value: E) -> Self {
        DeviceError::Communication(value)
    }
}

/*******************/

pub trait DeviceIo {
    type Command;
    type SendStatus;
    type Error: Error;

    async fn listen(&mut self) -> Result<Self::Command, Self::Error>;
    async fn respond_to_read(&mut self, write: &[u8]) -> Result<Self::SendStatus, Self::Error>;
    async fn respond_to_write(&mut self, read: &mut [u8]) -> Result<usize, Self::Error>;
}

/*******************/

pub struct Device<IO: DeviceIo> {
    io: IO,
}

impl<IO: DeviceIo> Device<IO> {
    pub fn new(io: IO) -> Self {
        Self { io }
    }

    pub async fn listen(&mut self) -> Result<IO::Command, IO::Error> {
        self.io.listen().await
    }

    pub async fn send_buttons(
        &mut self,
        buttons: u16,
    ) -> Result<IO::SendStatus, DeviceError<IO::Error>> {
        let buffer = buttons.to_be_bytes();
        self.io
            .respond_to_read(&buffer)
            .await
            .map_err(DeviceError::from)
    }

    pub async fn send_version<'b>(
        &mut self,
        version: &FirmwareVersion<'b>,
    ) -> Result<IO::SendStatus, DeviceError<IO::Error>> {
        let prerelease = match version.prerelease {
            Some(_) => '*' as u8,
            None => 0,
        };

        let mut message = [version.major, version.minor, version.patch, prerelease, 0];

        let check = message.iter().fold(0u8, |acc, e| acc.wrapping_add(*e));
        *message.last_mut().unwrap() = check;

        self.io
            .respond_to_read(&message)
            .await
            .map_err(DeviceError::from)
    }

    pub async fn send_write_status(
        &mut self,
        status: WriteStatus,
    ) -> Result<IO::SendStatus, DeviceError<IO::Error>> {
        let byte = status as u8;
        let message = [byte, !byte];
        self.io
            .respond_to_read(&message)
            .await
            .map_err(DeviceError::from)
    }

    pub async fn receive_command(&mut self) -> Result<Command, DeviceError<IO::Error>> {
        let mut buffer = [0u8; PACKET_MAX_SIZE];
        match self.io.respond_to_write(&mut buffer).await {
            Ok(len) => {
                if len < 2 {
                    return Err(DeviceError::ShortMessage(len));
                }

                if len > buffer.len() {
                    return Err(DeviceError::LongMessage(len));
                }

                let message = &buffer[..len];

                if message[0] != !message[1] {
                    return Err(DeviceError::CorruptCommand);
                }

                match message[0] {
                    command_code::GET_VERSION => return Ok(Command::GetVersion),
                    command_code::GET_WRITE_STATUS => return Ok(Command::GetWriteStatus),
                    command_code::WRITE_BEGIN => return Ok(Command::WriteBegin),
                    command_code::WRITE_PACKET => {
                        if message.len() < 4 {
                            return Err(DeviceError::EmptyPacket);
                        }

                        let (check, payload) = message[2..].split_last().unwrap();
                        if *check != payload.iter().fold(0, |acc, e| acc.wrapping_add(*e)) {
                            return Err(DeviceError::CorruptMessage);
                        }

                        let length = payload.len();
                        let mut data = [0; PACKET_MAX_PAYLOAD_SIZE];
                        data[..length].copy_from_slice(payload);
                        return Ok(Command::WritePacket { data, length });
                    }
                    command_code::WRITE_END => return Ok(Command::WriteEnd),
                    unknown_code => return Err(DeviceError::UnknownCommand(unknown_code)),
                }
            }
            Err(e) => {
                return Err(DeviceError::Communication(e));
            }
        }
    }
}
