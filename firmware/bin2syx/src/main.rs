use embedded_io_adapters::std::FromStd;
use midi::SystemExclusiveMessage;
use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read};
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: bin2syx <input-file>");
        return;
    }

    let input_path = Path::new(&args[1]);
    let input_file = match File::open(input_path) {
        Ok(file) => file,
        Err(e) => {
            println!("Couldn't open {}: {}", input_path.display(), e);
            return;
        }
    };
    let file_size = match input_file.metadata() {
        Ok(metadata) => metadata.len(),
        Err(e) => {
            println!("Couldn't determine input file size: {e}");
            return;
        }
    };
    let file_size: u32 = match file_size.try_into() {
        Ok(n) => n,
        Err(e) => {
            println!("Couldn't store file size: {e}");
            return;
        }
    };

    let output_path = input_path.with_added_extension("syx");
    let output_file = match File::create(output_path.as_path()) {
        Ok(file) => file,
        Err(e) => {
            println!("Couldn't open {}: {}", output_path.display(), e);
            return;
        }
    };

    let mut reader = BufReader::new(input_file);
    let mut writer = FromStd::new(BufWriter::new(output_file));

    const DEVICE_ID: u8 = 1;
    const SOURCE_ID: u8 = 0;

    let header = SystemExclusiveMessage::file_dump_header(DEVICE_ID, SOURCE_ID, file_size, "BIN ");
    if let Err(e) = header.write(&mut writer) {
        println!("Write error: {e}");
        return;
    }

    let mut packet_num = 0;
    loop {
        let mut data = [0; 112];
        let n = match reader.read(&mut data) {
            Ok(n) => n,
            Err(e) => {
                println!("Read error: {e}");
                return;
            }
        };

        if n > 0 {
            let packet =
                SystemExclusiveMessage::file_dump_packet(DEVICE_ID, packet_num, &data[..n]);
            if let Err(e) = packet.write(&mut writer) {
                println!("Write error: {e}");
                return;
            }
        } else {
            let eof = SystemExclusiveMessage::eof(DEVICE_ID, packet_num);
            if let Err(e) = eof.write(&mut writer) {
                println!("Write error: {e}");
                return;
            }
            break;
        }

        packet_num = (packet_num + 1) % 0x80;
    }
}
