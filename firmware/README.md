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
- [midi](midi/): Library that provides the MIDI features required by
  `anglo-controller` and `bin2syx`.

## Environment Setup

    rustup update
    rustup target add thumbv6m-none-eabi
    rustup component add llvm-tools
    cargo install cargo-binutils
    cargo install --locked probe-rs-tools

## Build SYX Firmware Package

    cd bin2syx
    cargo build
    cd ..

    cd anglo-controller
    cargo objcopy --release -- -O binary ../anglo-firmware.bin
    cd ..

    ./bin2syx/target/debug/bin2syx anglo-firmware.bin

## License

[GNU General Public License v3.0](LICENSE.txt)
