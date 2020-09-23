#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]
#![cfg_attr(not(target_os = "none"), allow(unused_imports, dead_code))]
#![feature(llvm_asm, asm, naked_functions)]

use core::{fmt::Write, panic::PanicInfo};
use rak::Plat;
use uart_16550::SerialPort;

/// Workaround for https://github.com/rust-lang/cargo/issues/6784
/// Running "cargo test" or "cargo build" for the host target will try to build
/// this binary for the host target. For that, we need a main function.
#[cfg(not(target_os = "none"))]
pub fn main() {}

mod pvh;

#[cfg_attr(target_os = "none", panic_handler)]
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

pub extern "sysv64" fn rust_start(rdi: *const ()) -> ! {
    let mut plat = BareMetal::new();
    writeln!(plat.logger(), "{:p}", rdi).unwrap();
    rak::paging::log_tables(&mut plat).unwrap();

    #[no_mangle]
    static mut WRITE: usize = 22;
    unsafe { core::ptr::write_volatile(&mut WRITE, 1) };
    rak::paging::log_tables(&mut plat).unwrap();

    for _ in 0..10 {
        rak::print_random_val(&mut plat, &[1, 2, 3, 4, 5, 6, 7]).unwrap();
    }
    panic!("STOP")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bin_unit_test() {
        assert_ne!(2, 3);
    }
}
