use hex_literal::hex;
use midi_tools::file_dump::*;
use midi_tools::messages::sysex::SysEx;
use midi_tools::messages::sysex;
use mockall::mock;
use mockall::predicate::*;
use mockall::Sequence;

mock! {
    FileWriter {}
    impl FileWriter for FileWriter {
        type ErrorType = i32;
        async fn open(&mut self) -> Result<(), i32>;
        async fn write(&mut self, data: &[u8]) -> Result<(), i32>;
        async fn close(&mut self) -> Result<(), i32>;
    }
}

mock! {
    SysExOutput {}
    impl SysExOutput for SysExOutput {
        async fn send(&mut self, sysex: midi_tools::messages::sysex::SysEx);
    }
}

#[tokio::test]
async fn receiver_ignores_header_with_wrong_device_ids() {
    const SOURCE_ID: u8 = 0;
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    file.expect_open().times(0);
    sysex.expect_send().times(0);

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::file_dump_header(0x00, SOURCE_ID, 0, FILE_TYPE)).await;
    receiver.process(SysEx::file_dump_header(0x33, SOURCE_ID, 0, FILE_TYPE)).await;
    receiver.process(SysEx::file_dump_header(0x7E, SOURCE_ID, 0, FILE_TYPE)).await;
}

#[tokio::test]
async fn receiver_cancels_header_with_wrong_file_type() {
    const SOURCE_ID: u8 = 0;
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    file.expect_open().times(0);
    sysex.expect_send().with(eq(SysEx::cancel(SOURCE_ID, 0))).return_const(()).times(3);

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::file_dump_header(DEVICE_ID, SOURCE_ID, 0, "TEXT")).await;
    receiver.process(SysEx::file_dump_header(DEVICE_ID, SOURCE_ID, 0, "MIDI")).await;
    receiver.process(SysEx::file_dump_header(DEVICE_ID, SOURCE_ID, 0, "BINN")).await;
}

#[tokio::test]
async fn receiver_opens_file_on_accepted_header() {
    const SOURCE_ID: u8 = 0;
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    file.expect_open().return_const(Ok(())).times(1);
    sysex.expect_send().return_const(());

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::file_dump_header(DEVICE_ID, SOURCE_ID, 0, FILE_TYPE)).await;
}

#[tokio::test]
async fn receiver_requests_wait_before_open() {
    const SOURCE_ID: u8 = 0;
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    let mut seq = Sequence::new();
    sysex.expect_send().with(eq(SysEx::wait(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    file.expect_open().return_const(Ok(())).times(1).in_sequence(&mut seq);
    sysex.expect_send().return_const(());

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::file_dump_header(DEVICE_ID, SOURCE_ID, 0, FILE_TYPE)).await;
}

#[tokio::test]
async fn receiver_cancels_on_open_error() {
    const SOURCE_ID: u8 = 0;
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    let mut seq = Sequence::new();
    sysex.expect_send().with(eq(SysEx::wait(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    file.expect_open().return_const(Err(0)).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::cancel(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::file_dump_header(DEVICE_ID, SOURCE_ID, 0, FILE_TYPE)).await;
}

#[tokio::test]
async fn receiver_acks_on_open_ok() {
    const SOURCE_ID: u8 = 0;
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    let mut seq = Sequence::new();
    sysex.expect_send().with(eq(SysEx::wait(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    file.expect_open().return_const(Ok(())).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::ack(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::file_dump_header(DEVICE_ID, SOURCE_ID, 0, FILE_TYPE)).await;
}

#[tokio::test]
async fn receiver_ignores_eof_before_header() {
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    file.expect_open().times(0);
    sysex.expect_send().times(0);

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::eof(DEVICE_ID, 2)).await;
}

#[tokio::test]
async fn receiver_ignores_eof_with_wrong_device_id() {
    const SOURCE_ID: u8 = 0;
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    file.expect_open().return_const(Ok(())).times(1);
    sysex.expect_send().return_const(());
    file.expect_close().times(0);

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::file_dump_header(DEVICE_ID, SOURCE_ID, 0, FILE_TYPE)).await;
    receiver.process(SysEx::eof(0x00, 0)).await;
    receiver.process(SysEx::eof(0x33, 0)).await;
    receiver.process(SysEx::eof(0x7E, 0)).await;
}

#[tokio::test]
async fn receiver_closes_file_on_eof() {
    const SOURCE_ID: u8 = 0;
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    file.expect_open().return_const(Ok(())).times(1);
    sysex.expect_send().return_const(());
    file.expect_close().return_const(Ok(())).times(1);

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::file_dump_header(DEVICE_ID, SOURCE_ID, 0, FILE_TYPE)).await;
    receiver.process(SysEx::eof(DEVICE_ID, 0)).await;
}

#[tokio::test]
async fn receiver_accepts_any_eof_packet_num() {
    const SOURCE_ID: u8 = 0;
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    file.expect_open().return_const(Ok(())).times(1);
    sysex.expect_send().return_const(());
    file.expect_close().return_const(Ok(())).times(1);

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::file_dump_header(DEVICE_ID, SOURCE_ID, 0, FILE_TYPE)).await;
    receiver.process(SysEx::eof(DEVICE_ID, 43)).await;
}

#[tokio::test]
async fn receiver_ignores_packets_before_header() {
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    file.expect_open().times(0);
    sysex.expect_send().times(0);

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::file_dump_packet(DEVICE_ID, 0, &hex!("01 02 03 04"))).await;
}

#[tokio::test]
async fn receiver_writes_file_on_accepted_packet() {
    const SOURCE_ID: u8 = 0;
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    file.expect_open().return_const(Ok(()));
    file.expect_write().with(eq(hex!("01 02 03 04"))).return_const(Ok(())).times(1);
    sysex.expect_send().return_const(());

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::file_dump_header(DEVICE_ID, SOURCE_ID, 0, FILE_TYPE)).await;
    receiver.process(SysEx::file_dump_packet(DEVICE_ID, 0, &hex!("01 02 03 04"))).await;
}

#[tokio::test]
async fn receiver_requests_wait_before_write() {
    const SOURCE_ID: u8 = 0;
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    let mut seq = Sequence::new();
    sysex.expect_send().with(eq(SysEx::wait(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    file.expect_open().return_const(Ok(())).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::ack(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::wait(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    file.expect_write().return_const(Ok(())).times(1).in_sequence(&mut seq);
    sysex.expect_send().return_const(());

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::file_dump_header(DEVICE_ID, SOURCE_ID, 0, FILE_TYPE)).await;
    receiver.process(SysEx::file_dump_packet(DEVICE_ID, 0, &hex!("01 02 03 04"))).await;
}

#[tokio::test]
async fn receiver_cancels_on_write_error() {
    const SOURCE_ID: u8 = 0;
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    let mut seq = Sequence::new();
    sysex.expect_send().with(eq(SysEx::wait(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    file.expect_open().return_const(Ok(())).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::ack(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::wait(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    file.expect_write().return_const(Err(0)).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::cancel(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::file_dump_header(DEVICE_ID, SOURCE_ID, 0, FILE_TYPE)).await;
    receiver.process(SysEx::file_dump_packet(DEVICE_ID, 0, &hex!("01 02 03 04"))).await;
}

#[tokio::test]
async fn receiver_acks_on_valid_packet() {
    const SOURCE_ID: u8 = 0;
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    let mut seq = Sequence::new();
    sysex.expect_send().with(eq(SysEx::wait(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    file.expect_open().return_const(Ok(())).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::ack(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::wait(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::ack(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::wait(SOURCE_ID, 1))).return_const(()).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::ack(SOURCE_ID, 1))).return_const(()).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::wait(SOURCE_ID, 2))).return_const(()).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::ack(SOURCE_ID, 2))).return_const(()).times(1).in_sequence(&mut seq);
    file.expect_write().return_const(Ok(()));

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::file_dump_header(DEVICE_ID, SOURCE_ID, 0, FILE_TYPE)).await;
    receiver.process(SysEx::file_dump_packet(DEVICE_ID, 0, &hex!("01 02 03 04"))).await;
    receiver.process(SysEx::file_dump_packet(DEVICE_ID, 1, &hex!("F0 E0 D0 C0"))).await;
    receiver.process(SysEx::file_dump_packet(DEVICE_ID, 2, &hex!("55 66 77 88"))).await;
}

#[tokio::test]
async fn receiver_naks_on_bad_checksum() {
    const SOURCE_ID: u8 = 0;
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    let mut seq = Sequence::new();
    sysex.expect_send().with(eq(SysEx::wait(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    file.expect_open().return_const(Ok(())).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::ack(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::nak(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::file_dump_header(DEVICE_ID, SOURCE_ID, 0, FILE_TYPE)).await;

    let mut corrupt_packet = SysEx::file_dump_packet(DEVICE_ID, 0, &hex!("01 02 03 04"));
    if let SysEx::FileDumpPacket(data) = &mut corrupt_packet {
        data.checksum_ok = false;
    }
    receiver.process(corrupt_packet).await;
}

#[tokio::test]
async fn receiver_ignores_packets_with_wrong_device_id() {
    const SOURCE_ID: u8 = 0;
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    let mut seq = Sequence::new();
    sysex.expect_send().with(eq(SysEx::wait(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    file.expect_open().return_const(Ok(())).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::ack(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::wait(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    file.expect_write().with(eq(hex!("01 02 03 04"))).return_const(Ok(())).times(1);
    sysex.expect_send().with(eq(SysEx::ack(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::file_dump_header(DEVICE_ID, SOURCE_ID, 0, FILE_TYPE)).await;
    receiver.process(SysEx::file_dump_packet(DEVICE_ID, 0, &hex!("01 02 03 04"))).await;
    receiver.process(SysEx::file_dump_packet(0x00, 1, &hex!("F0 E0 D0 C0"))).await;
    receiver.process(SysEx::file_dump_packet(0x33, 1, &hex!("F0 E0 D0 C0"))).await;
    receiver.process(SysEx::file_dump_packet(0x7E, 1, &hex!("F0 E0 D0 C0"))).await;
}

#[tokio::test]
async fn receiver_cancels_on_out_of_order_packet() {
    const SOURCE_ID: u8 = 0;
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    let mut seq = Sequence::new();
    sysex.expect_send().with(eq(SysEx::wait(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    file.expect_open().return_const(Ok(())).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::ack(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::wait(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::ack(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::wait(SOURCE_ID, 1))).return_const(()).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::ack(SOURCE_ID, 1))).return_const(()).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::cancel(SOURCE_ID, 2))).return_const(()).times(1).in_sequence(&mut seq);
    file.expect_write().return_const(Ok(()));

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::file_dump_header(DEVICE_ID, SOURCE_ID, 0, FILE_TYPE)).await;
    receiver.process(SysEx::file_dump_packet(DEVICE_ID, 0, &hex!("01 02 03 04"))).await;
    receiver.process(SysEx::file_dump_packet(DEVICE_ID, 1, &hex!("F0 E0 D0 C0"))).await;
    receiver.process(SysEx::file_dump_packet(DEVICE_ID, 3, &hex!("55 66 77 88"))).await;
}

#[tokio::test]
async fn receiver_accepts_all_call_header() {
    const SOURCE_ID: u8 = 0;
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    file.expect_open().return_const(Ok(())).times(1);
    sysex.expect_send().return_const(());

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::file_dump_header(sysex::ALL_CALL_DEVICE_ID, SOURCE_ID, 0, FILE_TYPE)).await;
}

#[tokio::test]
async fn receiver_accepts_all_call_packet() {
    const SOURCE_ID: u8 = 0;
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    file.expect_open().return_const(Ok(()));
    file.expect_write().with(eq(hex!("01 02 03 04"))).return_const(Ok(())).times(1);
    sysex.expect_send().return_const(());

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::file_dump_header(DEVICE_ID, SOURCE_ID, 0, FILE_TYPE)).await;
    receiver.process(SysEx::file_dump_packet(sysex::ALL_CALL_DEVICE_ID, 0, &hex!("01 02 03 04"))).await;
}

#[tokio::test]
async fn receiver_ignores_packets_after_header_with_wrong_file_type() {
    const SOURCE_ID: u8 = 0;
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    file.expect_open().times(0);
    file.expect_write().times(0);
    sysex.expect_send().return_const(());

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::file_dump_header(DEVICE_ID, SOURCE_ID, 0, "TEXT")).await;
    receiver.process(SysEx::file_dump_packet(DEVICE_ID, 0, &hex!("01 02 03 04"))).await;
}

#[tokio::test]
async fn receiver_ignores_packets_after_open_error() {
    const SOURCE_ID: u8 = 0;
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    file.expect_open().return_const(Err(0));
    file.expect_write().times(0);
    sysex.expect_send().return_const(());

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::file_dump_header(DEVICE_ID, SOURCE_ID, 0, FILE_TYPE)).await;
    receiver.process(SysEx::file_dump_packet(DEVICE_ID, 0, &hex!("01 02 03 04"))).await;
}

#[tokio::test]
async fn receiver_ignores_packets_after_write_error() {
    const SOURCE_ID: u8 = 0;
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    let mut seq = Sequence::new();
    sysex.expect_send().with(eq(SysEx::wait(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    file.expect_open().return_const(Ok(())).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::ack(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::wait(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    file.expect_write().return_const(Err(0)).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::cancel(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::file_dump_header(DEVICE_ID, SOURCE_ID, 0, FILE_TYPE)).await;
    receiver.process(SysEx::file_dump_packet(DEVICE_ID, 0, &hex!("01 02 03 04"))).await;
    receiver.process(SysEx::file_dump_packet(DEVICE_ID, 1, &hex!("F0 E0 D0 C0"))).await;
    receiver.process(SysEx::file_dump_packet(DEVICE_ID, 2, &hex!("55 66 77 88"))).await;
}

#[tokio::test]
async fn receiver_ignores_packets_after_out_of_order_packet() {
    const SOURCE_ID: u8 = 0;
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    let mut seq = Sequence::new();
    sysex.expect_send().with(eq(SysEx::wait(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    file.expect_open().return_const(Ok(())).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::ack(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::cancel(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::file_dump_header(DEVICE_ID, SOURCE_ID, 0, FILE_TYPE)).await;
    receiver.process(SysEx::file_dump_packet(DEVICE_ID, 5, &hex!("01 02 03 04"))).await;
    receiver.process(SysEx::file_dump_packet(DEVICE_ID, 0, &hex!("F0 E0 D0 C0"))).await;
    receiver.process(SysEx::file_dump_packet(DEVICE_ID, 1, &hex!("55 66 77 88"))).await;
}

#[tokio::test]
async fn receiver_ignores_packets_after_eof() {
    const SOURCE_ID: u8 = 0;
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    file.expect_open().return_const(Ok(())).times(1);
    sysex.expect_send().return_const(());
    file.expect_close().return_const(Ok(())).times(1);

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::file_dump_header(DEVICE_ID, SOURCE_ID, 0, FILE_TYPE)).await;
    receiver.process(SysEx::eof(DEVICE_ID, 0)).await;
    receiver.process(SysEx::file_dump_packet(DEVICE_ID, 0, &hex!("01 02 03 04"))).await;
    receiver.process(SysEx::file_dump_packet(DEVICE_ID, 1, &hex!("F0 E0 D0 C0"))).await;
    receiver.process(SysEx::file_dump_packet(DEVICE_ID, 2, &hex!("55 66 77 88"))).await;
}

#[tokio::test]
async fn receiver_eof_after_eof() {
    const SOURCE_ID: u8 = 0;
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    file.expect_open().return_const(Ok(())).times(1);
    sysex.expect_send().return_const(());
    file.expect_close().return_const(Ok(())).times(1);

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::file_dump_header(DEVICE_ID, SOURCE_ID, 0, FILE_TYPE)).await;
    receiver.process(SysEx::eof(DEVICE_ID, 0)).await;
    receiver.process(SysEx::eof(DEVICE_ID, 0)).await;
}

#[tokio::test]
async fn receiver_accepts_packets_after_bad_checksum() {
    const SOURCE_ID: u8 = 0;
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    let mut seq = Sequence::new();
    sysex.expect_send().with(eq(SysEx::wait(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    file.expect_open().return_const(Ok(())).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::ack(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::nak(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    sysex.expect_send().with(eq(SysEx::wait(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);
    file.expect_write().with(eq(hex!("01 02 03 04"))).return_const(Ok(())).times(1);
    sysex.expect_send().with(eq(SysEx::ack(SOURCE_ID, 0))).return_const(()).times(1).in_sequence(&mut seq);

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::file_dump_header(DEVICE_ID, SOURCE_ID, 0, FILE_TYPE)).await;

    let mut corrupt_packet = SysEx::file_dump_packet(DEVICE_ID, 0, &hex!("01 02 03 04"));
    if let SysEx::FileDumpPacket(data) = &mut corrupt_packet {
        data.checksum_ok = false;
    }
    receiver.process(corrupt_packet).await;

    receiver.process(SysEx::file_dump_packet(DEVICE_ID, 0, &hex!("01 02 03 04"))).await;
}

#[tokio::test]
async fn receiver_reopens_file_on_new_header() {
    const SOURCE_ID: u8 = 0;
    const DEVICE_ID: u8 = 1;
    const FILE_TYPE: &str = "BIN ";

    let mut file = MockFileWriter::new();
    let mut sysex = MockSysExOutput::new();

    file.expect_open().return_const(Ok(())).times(2);
    sysex.expect_send().return_const(());

    let mut receiver = FileDumpReceiver::new(&mut file, &mut sysex, DEVICE_ID, FILE_TYPE);
    receiver.process(SysEx::file_dump_header(DEVICE_ID, SOURCE_ID, 0, FILE_TYPE)).await;
    receiver.process(SysEx::file_dump_header(DEVICE_ID, SOURCE_ID, 0, FILE_TYPE)).await;
}
