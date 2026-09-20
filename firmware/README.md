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

## Build SYX Firmware Package

    just build

This will create `build/anglo-firmware.bin` and `build/anglo-firmware.bin.syx`.

## License

[GNU General Public License v3.0](LICENSE.txt)
