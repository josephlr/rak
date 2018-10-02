# Rust Asynchronous Kernel (RAK)

An attempt to write a microkernel in [Rust](https://www.rust-lang.org) that
makes heavy use of Rust's
[async/await](https://github.com/rust-lang/rfcs/blob/master/text/2394-async_await.md)
functionality.

## Resources

- [OSDev wiki](https://wiki.osdev.org) (as always)
- Philipp Oppermann's blog [Writing an OS in Rust](https://os.phil-opp.com/)

## Important Dependencies

- [`x86_64`](https://docs.rs/crate/x86_64): low-level processor bindings
- [`bootloader`](https://docs.rs/crate/bootloader): pure Rust bootloader

## Legal

Copyright © 2018 The RAK Authors (licensed under [Apache 2.0](LICENSE)).

This product is not affliated with (or endorsed by) any company or organization.

Author: Joe Richey <joerichey94@gmail.com>