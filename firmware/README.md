# MIDI Concertina - Firmware

- [anglo-bootloader](anglo-bootloader/): Bootloader for both controller and
  keyboards that handles power-fail-safe firmware updates.
- [anglo-controller](anglo-controller/): Application firmware for the
  controller. Handles most of the instrument logic.
- [anglo-keyboard](anglo-keyboard/): Application firmware for the keyboards.
- [bin2syx](bin2syx/): Utility for converting firmware (.bin) files to SysEx
  (.syx) files that can be used to update the controller firmware over MIDI with
  software such as [SysEx Librarian](https://www.snoize.com/SysExLibrarian/) or
  [MIDI-OX](http://www.midiox.com/).
- [lib](lib/): Custom utility libraries

## Environment Setup

Install [rustup](https://rustup.rs/) and
[just](https://github.com/casey/just?tab=readme-ov-file#packages). Then run:

    just provision

## Initial Flashing

Build all firmware images:

    just build

Flash the controller first. This is necessary for the keyboards to receive
power.

    (cd anglo-controller && cargo embed --release)
    (cd anglo-controller-bootloader && cargo embed --release)

Then flash each keyboard:

    (cd anglo-keyboard && cargo embed --release)
    (cd anglo-keyboard-bootloader && cargo embed --release)

## Update Over MIDI

Create the SYX firmware package:

    just build

The update package will be written to `build/anglo-firmware.bin.syx`.

To update the firmware, press and hold the control panel button until the LED
begins flashing. Then transmit the SYX file to the instrument with a ~90ms pause
between messages using [SysEx Librarian](https://www.snoize.com/SysExLibrarian/),
[MIDI-OX](http://www.midiox.com/), or similar software. When the LED returns to
the breathing state, the update is complete.

## License

[GNU General Public License v3.0](LICENSE.txt)
