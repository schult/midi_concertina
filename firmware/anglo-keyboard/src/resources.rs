use assign_resources::assign_resources;
use embassy_stm32::{Peri, adc, bind_interrupts, dma, i2c, peripherals};

bind_interrupts!(pub struct Irqs {
    ADC1_COMP => adc::InterruptHandler<peripherals::ADC1>;
    I2C1 => i2c::EventInterruptHandler<peripherals::I2C1>, i2c::ErrorInterruptHandler<peripherals::I2C1>;
    DMA1_CHANNEL2_3 => dma::InterruptHandler<peripherals::DMA1_CH2>, dma::InterruptHandler<peripherals::DMA1_CH3>;
});

assign_resources! {
    dfu: DfuResources {
        flash: FLASH,
    }
    // TODO: Buttons? Chirality? Adc? Mux?
    i2c: I2cResources {
        i2c: I2C1,
        scl: PB8,
        sda: PB9,
        tx_dma: DMA1_CH2,
        rx_dma: DMA1_CH3,
    }
}
