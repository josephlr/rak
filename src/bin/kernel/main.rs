#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(unused_imports, dead_code))]
#![feature(asm, naked_functions)]

use core::{fmt::Write, panic::PanicInfo};
use uart_16550::SerialPort;
use rak::Plat;

/// Workaround for https://github.com/rust-lang/cargo/issues/6784
/// Running "cargo test" or "cargo build" for the host target will try to build
/// this binary for the host target. For that, we need a main function.
#[cfg(not(any(test, target_os = "none")))]
#[export_name = "main"]
pub extern "C" fn fake_main() -> i32 { 0 }

mod pvh;

#[cfg_attr(not(test), panic_handler)]
fn panic(_: &PanicInfo) -> ! {
    static FOO: usize = 0;
    loop {
        unsafe { core::ptr::read_volatile(&FOO) };
    }
}

struct BareMetal(SerialPort);

impl BareMetal {
    fn new() -> Self {
        // SAFETY: Standard COM port
        let mut port = unsafe { SerialPort::new(0x3f8) };
        port.init();
        Self(port)
    }
}

impl Plat for BareMetal {
    fn logger(&mut self) -> &mut dyn Write {
        &mut self.0
    }
}

#[no_mangle]
pub extern "C" fn rust64_start() {
    let mut plat = BareMetal::new(); 
    plat.print_random_val(&[1, 2, 3, 4, 5, 6, 7]);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bin_unit_test() {
        assert_ne!(2, 3);
    }
}
