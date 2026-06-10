#![no_std]
#![no_main]

#[cfg(feature = "defmt")]
use defmt::panic;
#[cfg(feature = "defmt")]
use defmt_rtt as _;
#[cfg(feature = "defmt")]
use panic_probe as _;
#[cfg(not(feature = "defmt"))]
use panic_reset as _;

use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::{gpio, peripherals, rcc, time::khz};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::watch::{self, Watch};
use embassy_time::Timer;
use version::FirmwareVersion;

use crate::resources::*;

mod bellows;
mod dfu;
mod i2c;
mod i2c_transfer;
mod keymap;
mod resources;
mod usb;

type MessageChannel = Channel<ThreadModeRawMutex, midi::Message, 8>;

static MIDI_OUT_CHANNEL: MessageChannel = MessageChannel::new();
static MIDI_IN_CHANNEL: MessageChannel = MessageChannel::new();

static UPDATE_COMPLETE: Watch<ThreadModeRawMutex, bool, 1> = Watch::new_with(false);

static BELLOWS_STATE: Watch<ThreadModeRawMutex, bellows::State, 1> =
    Watch::new_with(bellows::State::default());

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    let mut config = embassy_stm32::Config::default();
    config.rcc.hsi = true;
    config.rcc.sys = rcc::Sysclk::HSI;
    config.rcc.hsi48 = Some(rcc::Hsi48Config {
        sync_from_usb: true,
    });
    config.rcc.mux.clk48sel = rcc::mux::Clk48sel::HSI48;
    let p = embassy_stm32::init(config);
    let r = split_resources!(p);

    let controller_version = FirmwareVersion {
        major: env!("FIRMWARE_MAJOR_VERSION").parse().unwrap(),
        minor: env!("FIRMWARE_MINOR_VERSION").parse().unwrap(),
        patch: env!("FIRMWARE_PATCH_VERSION").parse().unwrap(),
        prerelease: option_env!("FIRMWARE_PRERELEASE_VERSION"),
    };
    #[cfg(feature = "defmt")]
    defmt::info!("Controller version: {}", controller_version);

    spawner.spawn(
        dfu::task(
            r.dfu,
            MIDI_IN_CHANNEL.dyn_receiver(),
            MIDI_OUT_CHANNEL.dyn_sender(),
        )
        .unwrap(),
    );

    let led_pin = PwmPin::new(r.led.pin, gpio::OutputType::PushPull);
    let led_pwm = SimplePwm::new(
        r.led.timer,
        Some(led_pin),
        None,
        None,
        None,
        khz(30),
        Default::default(),
    );
    let button = gpio::Input::new(r.button.pin, gpio::Pull::Up);
    spawner
        .spawn(control_panel_task(UPDATE_COMPLETE.receiver().unwrap(), led_pwm, button).unwrap());

    spawner.spawn(
        usb::task(
            r.usb,
            MIDI_IN_CHANNEL.dyn_sender(),
            MIDI_OUT_CHANNEL.dyn_receiver(),
        )
        .unwrap(),
    );

    let i2c_mutex = i2c::init(r.i2c);

    let bellows_sender = BELLOWS_STATE.dyn_sender();
    let mut bellows_receiver = BELLOWS_STATE.dyn_receiver().unwrap();

    spawner.spawn(bellows::task(i2c_mutex, bellows_sender).unwrap());

    let i2c_wrapper = i2c::I2cWrapper::new(i2c_mutex);
    let mut i2c_controller = i2c_proto::Controller::new(i2c_wrapper);

    const LEFT_ADDR: u8 = 0x22;
    const RIGHT_ADDR: u8 = 0x23;
    let mut left_buttons: u16 = 0;
    let mut right_buttons: u16 = 0;

    const MIDI_CHANNEL: u8 = 0;

    // Remove keyboard upgrade code/data when defmt is enabled to make more room in flash
    // TODO: Check if this is still necessary on 192kb part
    // #[cfg(not(feature = "defmt"))]
    {
        // TODO: Bring keyboard_firmware back
        // let keyboard_firmware = include_bytes!("../../build/anglo-keyboard.bin");
        let mut transfer_sessions = [
            i2c_transfer::Session::new(LEFT_ADDR),
            i2c_transfer::Session::new(RIGHT_ADDR),
        ];
        for session in &mut transfer_sessions {
            const RETRY_COUNT: usize = 3;
            for _ in 0..RETRY_COUNT {
                if let Ok(keyboard_version) = i2c_controller.get_version(session.address()).await {
                    #[cfg(feature = "defmt")]
                    defmt::info!(
                        "Keyboard({}) version: {}",
                        session.address(),
                        keyboard_version
                    );

                    /*
                    if keyboard_version != controller_version
                        || keyboard_version.prerelease.is_some()
                    {
                        session.begin(keyboard_firmware);
                    }
                    */
                    break;
                }
            }
        }
        /*
        let mut transfer_in_progress = true;
        while transfer_in_progress {
            transfer_in_progress = false;
            for session in &mut transfer_sessions {
                transfer_in_progress |= match session.poll(&mut i2c_controller).await {
                    Ok(i2c_transfer::Status::InProgress) => true,
                    Err(_) => panic!("I2C communication error"),
                    _ => false,
                }
            }
        }
        */
    }

    UPDATE_COMPLETE.sender().send(true);

    let mut bellows_state = bellows::State::default();

    loop {
        let left_notes = match bellows_state.direction() {
            bellows::Direction::Push => &keymap::LEFT_PUSH[..],
            bellows::Direction::Pull => &keymap::LEFT_PULL[..],
            bellows::Direction::None => &[],
        };

        let right_notes = match bellows_state.direction() {
            bellows::Direction::Push => &keymap::RIGHT_PUSH[..],
            bellows::Direction::Pull => &keymap::RIGHT_PULL[..],
            bellows::Direction::None => &[],
        };

        if let Ok(new_state) = i2c_controller.get_buttons(LEFT_ADDR).await {
            let changes = left_buttons ^ new_state;
            for (i, note) in left_notes.iter().enumerate() {
                if (changes >> i) & 1 == 1 {
                    if (new_state >> i) & 1 == 1 {
                        let message = midi::Message::note_on(MIDI_CHANNEL, *note, 127);
                        MIDI_OUT_CHANNEL.send(message).await;
                    } else {
                        let message = midi::Message::note_off(MIDI_CHANNEL, *note, 127);
                        MIDI_OUT_CHANNEL.send(message).await;
                    }
                }
            }

            left_buttons = new_state;
        }

        if let Ok(new_state) = i2c_controller.get_buttons(RIGHT_ADDR).await {
            let changes = right_buttons ^ new_state;
            for (i, note) in right_notes.iter().enumerate() {
                if (changes >> i) & 1 == 1 {
                    if (new_state >> i) & 1 == 1 {
                        let message = midi::Message::note_on(MIDI_CHANNEL, *note, 127);
                        MIDI_OUT_CHANNEL.send(message).await;
                    } else {
                        let message = midi::Message::note_off(MIDI_CHANNEL, *note, 127);
                        MIDI_OUT_CHANNEL.send(message).await;
                    }
                }
            }

            right_buttons = new_state;
        }

        let new_bellows_state = bellows_receiver.get().await;
        if new_bellows_state != bellows_state {
            if new_bellows_state.direction() != bellows_state.direction() {
                for (i, note) in left_notes.iter().enumerate() {
                    if (left_buttons >> i) & 1 == 1 {
                        let message = midi::Message::note_off(MIDI_CHANNEL, *note, 127);
                        MIDI_OUT_CHANNEL.send(message).await;
                    }
                }

                for (i, note) in right_notes.iter().enumerate() {
                    if (right_buttons >> i) & 1 == 1 {
                        let message = midi::Message::note_off(MIDI_CHANNEL, *note, 127);
                        MIDI_OUT_CHANNEL.send(message).await;
                    }
                }

                left_buttons = 0;
                right_buttons = 0;
            }

            bellows_state = new_bellows_state;

            let msb_message = midi::Message::control_change(
                MIDI_CHANNEL,
                midi::cc::CHANNEL_VOLUME_MSB,
                bellows_state.magnitude_msb(),
            );
            let lsb_message = midi::Message::control_change(
                MIDI_CHANNEL,
                midi::cc::CHANNEL_VOLUME_LSB,
                bellows_state.magnitude_lsb(),
            );
            MIDI_OUT_CHANNEL.send(msb_message).await;
            MIDI_OUT_CHANNEL.send(lsb_message).await;
        }

        Timer::after_micros(100).await;
    }
}

#[embassy_executor::task]
async fn control_panel_task(
    mut update_complete: watch::Receiver<'static, ThreadModeRawMutex, bool, 1>,
    mut led_pwm: SimplePwm<'static, peripherals::TIM3>,
    button: gpio::Input<'static>,
) {
    let mut led = led_pwm.ch1();
    let led_max = led.max_duty_cycle() / 4;
    let led_step = led_max / 64;
    let mut led_duty = 0;
    let mut inc = true;
    led.enable();

    // TODO: Hold button to enter wait for firmware update
    // TODO: Indicate system state with LED

    while !update_complete.get().await {
        if inc && (led_max - led_duty < led_step) {
            led_duty = led_max;
            inc = !inc;
        } else if !inc && (led_duty < led_step) {
            led_duty = 0;
            inc = !inc;
        } else if inc {
            led_duty = led_duty + led_step
        } else {
            led_duty = led_duty - led_step
        }
        led.set_duty_cycle(led_duty);

        Timer::after_millis(10).await;
    }

    led.set_duty_cycle(led_max);
    loop {
        match button.get_level() {
            gpio::Level::High => led.enable(),
            gpio::Level::Low => led.disable(),
        }
        Timer::after_millis(10).await;
    }
}
