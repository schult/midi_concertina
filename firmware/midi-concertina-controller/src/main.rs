#![no_std]
#![no_main]

use defmt::panic;
use defmt_rtt as _;
use embassy_futures::join;
use embassy_stm32::{
    bind_interrupts,
    gpio,
    peripherals,
    usb,
};
use embassy_time::Timer;
use embassy_usb::class::midi;
use embassy_usb::driver::EndpointError;
use panic_probe as _;

const MAX_MIDI_PACKET_SIZE: usize = 64;

bind_interrupts!(struct Irqs {
    USB => usb::InterruptHandler<peripherals::USB>;
});

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;

        config.rcc.hsi = true;
        config.rcc.pll = Some(Pll {
            source: PllSource::HSI,
            mul: PllMul::MUL3,  // PLLVCO = 16*3 = 48Mhz
            div: PllDiv::DIV2,  // 24Mhz clock (16 * 3 / 2)
        });
        config.rcc.sys = Sysclk::PLL1_R;

        config.rcc.hsi48 = Some(Hsi48Config { sync_from_usb: true });
        config.rcc.mux.clk48sel = mux::Clk48sel::HSI48;
    }
    let p = embassy_stm32::init(config);

    let usb_driver = embassy_stm32::usb::Driver::new(p.USB, Irqs, p.PA12, p.PA11);

    // TODO: Get IDs from https://pid.codes/howto/
    const USB_VID: u16 = 0xCAFE;  // Default VID in TinyUSB
    const USB_PID: u16 = 0x4000 | (1 << 3);  // PID for MIDI-only device in TinyUSB
    let mut usb_config = embassy_usb::Config::new(USB_VID, USB_PID);
    usb_config.manufacturer = Some("Bushel Basket");
    usb_config.product = Some("Anglo M");
    usb_config.serial_number = Some("0001");  // TODO

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

    let mut usb = usb_builder.build();
    let usb_task = usb.run();

    let midi_receive_task = async {
        let mut data = [0; MAX_MIDI_PACKET_SIZE];

        loop {
            midi_reciever.wait_connection().await;

            loop {
                match midi_reciever.read_packet(&mut data).await {
                    Ok(_) => (),
                    Err(EndpointError::BufferOverflow) => panic!("Buffer overflow"),
                    Err(EndpointError::Disabled) => break,
                }
            }
        }
    };

    let midi_send_task = async {
        let mut led = gpio::Output::new(p.PB4, gpio::Level::High, gpio::Speed::Low);
        let mut playing = false;
        let note_on: [u8; 4] = [0x09, 0x90, 69, 127];
        let note_off: [u8; 4] = [0x08, 0x80, 69, 127];

        loop {
            midi_sender.wait_connection().await;

            loop {
                playing = !playing;
                let packet = match playing {
                    false => &note_off,
                    true => &note_on,
                };
                match midi_sender.write_packet(packet).await {
                    Ok(_) => (),
                    Err(EndpointError::BufferOverflow) => panic!("Buffer overflow"),
                    Err(EndpointError::Disabled) => break,
                }
                led.set_level(if playing { gpio::Level::High } else { gpio::Level::Low });
                Timer::after_secs(1).await;
            }
        }
    };

    join::join3(usb_task, midi_receive_task, midi_send_task).await;
}
