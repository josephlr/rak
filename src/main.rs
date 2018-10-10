#![feature(
    alloc,
    alloc_error_handler,
    uniform_paths,
    const_fn,
    int_to_from_bytes,
    range_contains,
    abi_x86_interrupt,
    panic_info_message
)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

#[macro_use]
mod util;
mod interrupts;
mod io;
mod memory;

use bootloader::{bootinfo::BootInfo, entry_point};
use core::{fmt::Write, ops::Deref};
use io::pic8259::PICS;
use io::vga::{Color, SCREEN};
use slab_allocator::LockedHeap;

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

    memory::init_heap();
    interrupts::init_gdt();
    interrupts::init_idt();

    PICS.lock().set_imr(0b1111_1111_1111_1100);
    PICS.lock()
        .remap(interrupts::PIC_START..interrupts::PIC_END);
    x86_64::instructions::interrupts::enable();

    util::halt()
}

#[cfg(not(test))]
entry_point!(bootloader_main);

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let mut screen = unsafe { util::force_unwrap(&SCREEN) };
    screen.set_font_color(Color::LightRed);
    write!(screen, "\n\nPANIC");
    if let Some(location) = info.location() {
        write!(screen, "({})", location);
    }
    writeln!(screen, ":");

    if let Some(message) = info.message() {
        writeln!(screen, "{}", message);
    }
    util::halt()
}

#[cfg(not(test))]
#[alloc_error_handler]
fn oom(_: core::alloc::Layout) -> ! {
    panic!("OUT OF MEMORY")
}

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
