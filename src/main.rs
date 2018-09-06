#![feature(uniform_paths, panic_handler, const_fn, int_to_from_bytes)]
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]
#![cfg_attr(not(test), no_main)]
#![no_std]

mod serial;
mod util;
mod vga;

use bootloader_precompiled::{bootinfo, entry_point};
use core::fmt::Write;
use core::panic::PanicInfo;
use serial::SERIAL1;
use vga::SCREEN;

entry_point!(start);

fn start(boot_info: &'static bootinfo::BootInfo) -> ! {
    let mut w = SERIAL1.lock();
    writeln!(w, "P4 Table is at {:x}", boot_info.p4_table_addr);
    writeln!(w, "There are {} memory regions", boot_info.memory_map.len());
    w.echo();
    unsafe { util::qemu_exit() }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    unsafe { SCREEN.force_unlock() }
    let mut w = SCREEN.lock();
    w.set_font_color(vga::Color::LightRed);
    write!(w, "\nKERNEL PANIC: {}", panic_info);
    util::stop()
}
