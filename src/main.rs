#![feature(uniform_paths, const_fn)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

mod io;
mod util;

use bootloader_precompiled::{bootinfo::BootInfo, entry_point};
use core::{fmt::Write, ops::Deref, panic::PanicInfo};
use io::vga::{Color, SCREEN};
use util::halt;

fn bootloader_main(info: &'static BootInfo) -> ! {
    writeln!(SCREEN.lock(), "P4 = 0x{:x}", info.p4_table_addr);
    writeln!(SCREEN.lock(), "Memory Map:");
    for region in info.memory_map.iter() {
        writeln!(
            SCREEN.lock(),
            "    [0x{:012x}, 0x{:012x}) - {:?}",
            region.range.start_addr(),
            region.range.end_addr(),
            region.region_type,
        );
    }
    writeln!(SCREEN.lock(), "Package = {:?}", info.package.deref());
    writeln!(SCREEN.lock());

    SCREEN.lock().set_font_color(Color::Magenta);
    write!(SCREEN.lock(), "Hello ");
    SCREEN.lock().set_font_color(Color::LightBlue);
    write!(SCREEN.lock(), "World!");

    panic!("OOPS");
    halt()
}

#[cfg(not(test))]
entry_point!(bootloader_main);

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    SCREEN.lock().set_font_color(Color::LightRed);
    writeln!(SCREEN.lock(), "\n\nKERNEL PANIC\n  {}", info);
    halt()
}
