use assign_resources::assign_resources;
use embassy_stm32::{Peri, bind_interrupts, dma, i2c, peripherals, usb};

bind_interrupts!(pub struct Irqs {
    USB => usb::InterruptHandler<peripherals::USB>;
    I2C1 => i2c::EventInterruptHandler<peripherals::I2C1>, i2c::ErrorInterruptHandler<peripherals::I2C1>;
    DMA1_CHANNEL2_3 => dma::InterruptHandler<peripherals::DMA1_CH2>, dma::InterruptHandler<peripherals::DMA1_CH3>;
});

assign_resources! {
    dfu: DfuResources {
        flash: FLASH,
    }
    led: LedResources {
        pin: PB4,
        timer: TIM3,
    }
    button: ButtonResources {
        pin: PB5,
    }
    usb: UsbResources {
        usb: USB,
        dp: PA12,
        dm: PA11,
    }
    i2c: I2cResources {
        power: PA3,
        i2c: I2C1,
        scl: PB8,
        sda: PB9,
        tx_dma: DMA1_CH2,
        rx_dma: DMA1_CH3,
    }
}
