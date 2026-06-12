#![no_std]
#![no_main]

mod bellows;
mod control_panel;
mod dfu;
mod i2c;
mod keyboard;
mod keymap;
mod mode;
mod resources;
mod usb;

#[cfg(feature = "defmt")]
use defmt_rtt as _;
#[cfg(feature = "defmt")]
use panic_probe as _;
#[cfg(not(feature = "defmt"))]
use panic_reset as _;

use crate::mode::Mode;
use crate::resources::*;
use embassy_futures::join::join;
use embassy_futures::yield_now;
use embassy_stm32::{gpio, rcc};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::watch::Watch;
use version::FirmwareVersion;

type MessageChannel = Channel<ThreadModeRawMutex, midi::Message, 8>;

static MIDI_OUT_CHANNEL: MessageChannel = MessageChannel::new();
static MIDI_IN_CHANNEL: MessageChannel = MessageChannel::new();

static MODE: Watch<ThreadModeRawMutex, Mode, 1> = Watch::new();
static MODE_REQUEST: Channel<ThreadModeRawMutex, Mode, 1> = Channel::new();

static LEFT_KEYBOARD_STATE: Watch<ThreadModeRawMutex, u16, 1> = Watch::new_with(0);
static RIGHT_KEYBOARD_STATE: Watch<ThreadModeRawMutex, u16, 1> = Watch::new_with(0);
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

    spawner.spawn(mode::task(MODE_REQUEST.dyn_receiver(), MODE.dyn_sender()).unwrap());

    spawner.spawn(control_panel::task(r.control_panel, MODE.dyn_receiver().unwrap()).unwrap());

    spawner.spawn(
        dfu::task(
            r.dfu,
            MIDI_IN_CHANNEL.dyn_receiver(),
            MIDI_OUT_CHANNEL.dyn_sender(),
        )
        .unwrap(),
    );

    spawner.spawn(
        usb::task(
            r.usb,
            MIDI_IN_CHANNEL.dyn_sender(),
            MIDI_OUT_CHANNEL.dyn_receiver(),
        )
        .unwrap(),
    );

    // Power on i2c-connected devices
    let _i2c_power = gpio::Output::new(r.power.i2c, gpio::Level::Low, gpio::Speed::Low);

    let i2c_mutex = i2c::init(r.i2c);

    const LEFT_KEYBOARD_ADDRESS: u8 = 0x22;
    const RIGHT_KEYBOARD_ADDRESS: u8 = 0x23;

    MODE_REQUEST.send(Mode::KeyboardInit).await;

    const KEYBOARD_FIRMWARE: &[u8] = include_bytes!("../../build/anglo-keyboard.bin");
    let left_keyboard_update = keyboard::update_firmware(
        i2c_mutex,
        LEFT_KEYBOARD_ADDRESS,
        &controller_version,
        KEYBOARD_FIRMWARE,
    );
    let right_keyboard_update = keyboard::update_firmware(
        i2c_mutex,
        RIGHT_KEYBOARD_ADDRESS,
        &controller_version,
        KEYBOARD_FIRMWARE,
    );
    join(left_keyboard_update, right_keyboard_update).await;

    MODE_REQUEST.send(Mode::Ready).await;

    spawner.spawn(
        keyboard::task(
            i2c_mutex,
            LEFT_KEYBOARD_ADDRESS,
            LEFT_KEYBOARD_STATE.dyn_sender(),
        )
        .unwrap(),
    );
    spawner.spawn(
        keyboard::task(
            i2c_mutex,
            RIGHT_KEYBOARD_ADDRESS,
            RIGHT_KEYBOARD_STATE.dyn_sender(),
        )
        .unwrap(),
    );

    spawner.spawn(bellows::task(i2c_mutex, BELLOWS_STATE.dyn_sender()).unwrap());

    const MIDI_CHANNEL: u8 = 0;

    let mut left_keyboard_receiver = LEFT_KEYBOARD_STATE.receiver().unwrap();
    let mut right_keyboard_receiver = RIGHT_KEYBOARD_STATE.receiver().unwrap();
    let mut bellows_receiver = BELLOWS_STATE.receiver().unwrap();

    let mut left_buttons: u16 = 0;
    let mut right_buttons: u16 = 0;
    let mut bellows_state = bellows::State::default();

    loop {
        let left_notes = match bellows_state.direction() {
            bellows::Direction::Push => keymap::LEFT_PUSH.as_slice(),
            bellows::Direction::Pull => keymap::LEFT_PULL.as_slice(),
            bellows::Direction::None => &[],
        };

        let right_notes = match bellows_state.direction() {
            bellows::Direction::Push => keymap::RIGHT_PUSH.as_slice(),
            bellows::Direction::Pull => keymap::RIGHT_PULL.as_slice(),
            bellows::Direction::None => &[],
        };

        let new_left_buttons = left_keyboard_receiver.get().await;
        let changes = left_buttons ^ new_left_buttons;
        for (i, note) in left_notes.iter().enumerate() {
            if (changes >> i) & 1 == 1 {
                if (new_left_buttons >> i) & 1 == 1 {
                    let message = midi::Message::note_on(MIDI_CHANNEL, *note, 127);
                    MIDI_OUT_CHANNEL.send(message).await;
                } else {
                    let message = midi::Message::note_off(MIDI_CHANNEL, *note, 127);
                    MIDI_OUT_CHANNEL.send(message).await;
                }
            }
        }
        left_buttons = new_left_buttons;

        let new_right_buttons = right_keyboard_receiver.get().await;
        let changes = right_buttons ^ new_right_buttons;
        for (i, note) in right_notes.iter().enumerate() {
            if (changes >> i) & 1 == 1 {
                if (new_right_buttons >> i) & 1 == 1 {
                    let message = midi::Message::note_on(MIDI_CHANNEL, *note, 127);
                    MIDI_OUT_CHANNEL.send(message).await;
                } else {
                    let message = midi::Message::note_off(MIDI_CHANNEL, *note, 127);
                    MIDI_OUT_CHANNEL.send(message).await;
                }
            }
        }
        right_buttons = new_right_buttons;

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

        yield_now().await;
    }
}
