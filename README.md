# jgenesis

WIP multi-console Sega emulator. Currently mostly supports the Sega Master System, the Game Gear, and the Sega Genesis / Mega Drive.

Major TODOs:
* Implement a GUI
* Implement a few remaining YM2612 features (CSM and SSG-EG, they're obscure but some games did use them)
  * Volume levels also sound slightly off in some games
* Halt the 68000 for the appropriate amount of time whenever a memory-to-VRAM DMA runs; not doing this causes graphical glitches in some games (e.g. Sonic 2 in split-screen 2-player mode)
* Support PAL for Genesis
* Support 6-button Genesis controllers
* Support the SMS optional YM2413 FM sound chip
* Support for specific Genesis games that do weird things with cartridge hardware (e.g. Phantasy Star 4 and Super Street Fighter 2)
* Support player 2 inputs
* Support the SMS reset button
* Support persistent save files for Genesis games with persistent cartridge RAM
* Improve Genesis performance (there are some low-hanging fruit in the VDP and YM2612 implementations)

## Dependencies

### Rust

This project requires the latest stable version of the [Rust toolchain](https://doc.rust-lang.org/book/ch01-01-installation.html) to build.

### SDL2

This project requires [SDL2](https://www.libsdl.org/) core headers to build.

Linux (Debian-based):
```
sudo apt install libsdl2-dev
```

macOS:
```
brew install sdl2
```

### GTK3 (Linux GUI only)

On Linux only, the GUI requires [GTK3](https://www.gtk.org/) headers to build.

Linux (Debian-based):
```
sudo apt install libgtk-3-dev
```

## Build & Run

```
cargo run --release --bin jgenesis-cli -- -f <path_to_rom_file>
```

## Screenshots

![Screenshot from 2023-08-27 22-45-06](https://github.com/jsgroth/jgenesis/assets/1137683/7d1567ce-39ba-4645-9aff-3c6d6e0afb80)

![Screenshot from 2023-08-27 22-45-32](https://github.com/jsgroth/jgenesis/assets/1137683/90d96e18-57a8-4327-8d9d-385f55a718b3)

![Screenshot from 2023-08-27 22-47-13](https://github.com/jsgroth/jgenesis/assets/1137683/d2ec2bc6-de7d-4ff1-98c5-10a0c4db7391)

![Screenshot from 2023-08-27 22-53-09](https://github.com/jsgroth/jgenesis/assets/1137683/05a7c309-0706-4627-9b45-313f259cc494)
