#![no_std]
#![no_main]

use defmt::panic;
use defmt_rtt as _;
// TODO use embassy_futures::join;
use embassy_stm32::{
    bind_interrupts,
    gpio,
    peripherals,
    rcc,
    usb,
};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::{ Channel, Receiver, Sender };
use embassy_time::Timer;
use embassy_usb::class::midi;
use embassy_usb::driver::EndpointError;
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USB => usb::InterruptHandler<peripherals::USB>;
});

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    let mut config = embassy_stm32::Config::default();
    config.rcc.hsi = true;
    config.rcc.sys = rcc::Sysclk::HSI;
    config.rcc.hsi48 = Some(rcc::Hsi48Config { sync_from_usb: true });
    config.rcc.mux.clk48sel = rcc::mux::Clk48sel::HSI48;
    let p = embassy_stm32::init(config);

    type EventPacketChannel = Channel<ThreadModeRawMutex, [u8; 4], 32>;
    let channel: &'static mut EventPacketChannel = {
        static CHANNEL: StaticCell<EventPacketChannel> = StaticCell::new();
        CHANNEL.init_with(|| EventPacketChannel::new())
    };

    let led = gpio::Output::new(p.PB4, gpio::Level::High, gpio::Speed::Low);
    spawner.spawn(control_panel_task(led, channel.sender())).unwrap();

    let usb_driver = usb::Driver::new(p.USB, Irqs, p.PA12, p.PA11);
    spawner.spawn(usb_task(usb_driver, channel.receiver())).unwrap();

    loop {
        // TODO: Leave some work in the main thread
        embassy_futures::yield_now().await;
    }
}

#[embassy_executor::task]
async fn control_panel_task(mut led: gpio::Output<'static>, pipe: Sender<'static, ThreadModeRawMutex, [u8; 4], 32>) {
    let mut playing = false;
    let note_on: [u8; 4] = [0x09, 0x90, 69, 127];
    let note_off: [u8; 4] = [0x08, 0x80, 69, 127];

    loop {
        playing = !playing;

        let packet = match playing {
            false => &note_off,
            true => &note_on,
        };
        pipe.send(packet.clone()).await;

        led.set_level(if playing { gpio::Level::High } else { gpio::Level::Low });
        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn usb_task(usb_driver: usb::Driver<'static, peripherals::USB>, pipe: Receiver<'static, ThreadModeRawMutex, [u8; 4], 32>) {
    // TODO: Get IDs from https://pid.codes/howto/
    const USB_VID: u16 = 0xCAFE;  // Default VID in TinyUSB
    const USB_PID: u16 = 0x4000 | (1 << 3);  // PID for MIDI-only device in TinyUSB
    let mut usb_config = embassy_usb::Config::new(USB_VID, USB_PID);
    usb_config.manufacturer = Some("Bushel Basket");
    usb_config.product = Some("Anglo M");
    usb_config.serial_number = Some(embassy_stm32::uid::uid_hex()); // TODO: This might require increasing the USB control buffer length

    const MAX_MIDI_PACKET_SIZE: usize = 64;
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut usb_builder = embassy_usb::Builder::new(
        usb_driver,
        usb_config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );
    let midi_class = midi::MidiClass::new(&mut usb_builder, 1, 1, MAX_MIDI_PACKET_SIZE as u16);
    let (mut midi_sender, mut midi_reciever) = midi_class.split();
    let mut usb_device = usb_builder.build();
    let usb_fut = usb_device.run();

    let sender_fut = async {
        let mut data = [0; MAX_MIDI_PACKET_SIZE];
        loop {
            midi_sender.wait_connection().await;

            loop {
                // TODO: Accumulate multiple packets if available
                data[..4].copy_from_slice(&pipe.receive().await);
                match midi_sender.write_packet(&data).await {
                    Ok(_) => (),
                    Err(EndpointError::BufferOverflow) => panic!("Buffer overflow"),
                    Err(EndpointError::Disabled) => break,
                }
            }
        }
    };

    let receiver_fut = async {
        let mut data = [0; MAX_MIDI_PACKET_SIZE];
        loop {
            midi_reciever.wait_connection().await;

            loop {
                midi_reciever.read_packet(&mut data).await.unwrap();
            }
        }
    };

    embassy_futures::join::join3(usb_fut, sender_fut, receiver_fut).await;
}
