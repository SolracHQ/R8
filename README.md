# R8: Yet Another CHIP-8 Emulator in Rust 😉

R8 is a **WIP** CHIP-8 emulator written in Rust, using sdl2 and egui. It is mostly functional, but it still lacks sound and debugging interface, and a better way to load the roms. For now, you can load the roms by dragging and dropping them to the emulator window. It also has some strange behaviors that I am checking.

## What is CHIP-8?

CHIP-8 is an interpreted programming language that was used to create games for some home computers in the 1970s and 1980s. It has a simple instruction set and graphics system, and it can run on various platforms with minimal changes.

## References

I used the following sources to learn about CHIP-8 and implement the emulator:

- [Wikipedia article on CHIP-8](https://en.wikipedia.org/wiki/CHIP-8)
- [CHIP-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
- [Mastering CHIP-8](https://github.com/mattmikolay/chip-8/wiki/Mastering-CHIP%E2%80%908)

## ROMS source
- [Public Domain Roms](https://www.zophar.net/pdroms/chip8.html)
