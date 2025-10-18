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
use embassy_usb::class::cdc_acm;
use embassy_usb::driver::EndpointError;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USB => usb::InterruptHandler<peripherals::USB>;
});

const MAX_PACKET_SIZE: usize = 128;

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

    const USB_VID: u16 = 0xC0DE;  // TODO
    const USB_PID: u16 = 0xCAFF;  // TODO
    // const USB_VID: u16 = 0xCAFE;  // TODO?
    // const USB_PID: u16 = 0x4008;  // TODO
    let mut usb_config = embassy_usb::Config::new(USB_VID, USB_PID);
    usb_config.manufacturer = Some("Bushel Basket Musical Instruments");
    usb_config.product = Some("Anglo Concertina M");
    usb_config.serial_number = Some("0001");  // TODO

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; MAX_PACKET_SIZE];

    let mut usb_state = cdc_acm::State::new();
    let mut usb_builder = embassy_usb::Builder::new(
        usb_driver,
        usb_config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );
    let mut usb_class = cdc_acm::CdcAcmClass::new(&mut usb_builder, &mut usb_state, MAX_PACKET_SIZE as u16);

    let mut usb = usb_builder.build();
    let usb_fut = usb.run();

    let echo_fut = async {
        loop {
            usb_class.wait_connection().await;
            let _ = echo(&mut usb_class).await;
        }
    };

    let led_fut = async {
        let mut led = gpio::Output::new(p.PB4, gpio::Level::High, gpio::Speed::Low);
        let switch = gpio::Input::new(p.PB5, gpio::Pull::Up);
        loop {
            led.set_level(
                match switch.get_level() {
                    gpio::Level::Low => gpio::Level::High,
                    gpio::Level::High => gpio::Level::Low,
                }
            );
            embassy_futures::yield_now().await;
        }
    };

    join::join3(usb_fut, echo_fut, led_fut).await;
}

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

async fn echo<'d, T: embassy_stm32::usb::Instance + 'd>(class: &mut cdc_acm::CdcAcmClass<'d, embassy_stm32::usb::Driver<'d, T>>) -> Result<(), Disconnected> {
    let mut buf = [0; MAX_PACKET_SIZE];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        class.write_packet(data).await?;
    }
}
