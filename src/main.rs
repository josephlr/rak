#![feature(uniform_paths, panic_handler, const_fn, int_to_from_bytes)]
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]
#![cfg_attr(not(test), no_main)]
#![no_std]

mod serial;
mod util;
mod vga;

use bootloader::{bootinfo, entry_point};
use core::fmt::Write;
use core::panic::PanicInfo;
use serial::SERIAL1;
use ux::u9;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{PageTable, PageTableEntry, PageTableFlags, RecursivePageTable};
use x86_64::VirtAddr;

fn print_used_entries(pt: &PageTable, level: usize) {
    for i in u16::from(u9::MIN)..u16::from(u9::MAX) {
        let i = usize::from(i);
        let pte = &pt[i];
        if !pte.is_unused() {
            writeln!(SERIAL1.lock(), "Level = {}", level);
            writeln!(SERIAL1.lock(), "{:o} - {:#?}", i, pte);
            writeln!(SERIAL1.lock());

            if !pte.flags().contains(PageTableFlags::HUGE_PAGE) && level > 1 {
                let addr = (pt as *const _ as usize) << 9;
                let addr = (addr + (i << 12)) as *const PageTable;
                print_used_entries(unsafe { &*addr }, level - 1)
            }
        }
    }
}

entry_point!(start);

fn start(boot_info: &'static bootinfo::BootInfo) -> ! {
    writeln!(
        SERIAL1.lock(),
        "P4: {:#?}",
        VirtAddr::new(boot_info.p4_table_addr)
    );
    writeln!(SERIAL1.lock(), "MemoryMap: {:#?}", boot_info.memory_map);
    writeln!(SERIAL1.lock(), "CR3 is {:#?}", Cr3::read());
    writeln!(SERIAL1.lock());

    let p4_addr = VirtAddr::new(0o177777_777_777_777_777_0000);

    let p4: &'static mut PageTable = unsafe { &mut *p4_addr.as_mut_ptr() };
    print_used_entries(p4, 4);
    let rpt = RecursivePageTable::new(p4);

    unsafe { util::qemu_exit() }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    unsafe { SERIAL1.force_unlock() }
    let mut w = SERIAL1.lock();
    write!(w, "\nKERNEL PANIC: {}", panic_info);
    util::stop()
}
