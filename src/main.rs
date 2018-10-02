#![feature(uniform_paths, const_fn)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

mod io;
#[macro_use]
mod util;

use bootloader_precompiled::{bootinfo::BootInfo, entry_point};
use core::{fmt::Write, ops::Deref, panic::PanicInfo};
use io::vga::{Color, SCREEN};

fn bootloader_main(info: &'static BootInfo) -> ! {
    lock_writeln!(SCREEN, "P4 = 0x{:x}", info.p4_table_addr);
    lock_writeln!(SCREEN, "Memory Map:");
    for region in info.memory_map.iter() {
        lock_writeln!(
            SCREEN,
            "    [0x{:012x}, 0x{:012x}) - {:?}",
            region.range.start_addr(),
            region.range.end_addr(),
            region.region_type,
        );
    }
    lock_writeln!(SCREEN, "Package = {:?}", info.package.deref());

    unsafe { util::qemu_shutdown() }
}

#[cfg(not(test))]
entry_point!(bootloader_main);

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut screen = unsafe { util::force_unwrap(&SCREEN) };
    screen.set_font_color(Color::LightRed);
    writeln!(screen, "\n\nKERNEL PANIC\n  {}", info);
    util::halt()
}
