extern crate alloc;

use alloc::boxed::Box;
use core::{fmt::Write, mem::size_of};
use crate::io::vga::{Color, SCREEN};
use crate::memory::Page;
use lazy_static::lazy_static;
use x86_64::{
    instructions::{segmentation::set_cs, tables::load_tss},
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable},
        idt::{ExceptionStackFrame, InterruptDescriptorTable},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

const DOUBLE_FAULT_IST_IDX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_IDX as usize] = {
            const STACK_SIZE: usize = 4096;
            let stack = Box::leak(Box::new([0; STACK_SIZE]));
            VirtAddr::from_ptr(stack) + STACK_SIZE
        };
        tss
    };
}

pub fn init_gdt() {
    let double_fault_stack = Box::leak(Box::new(Page::new()));

    let tss = Box::leak(Box::new(TaskStateSegment::new()));
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_IDX as usize] =
        VirtAddr::from_ptr(double_fault_stack) + size_of::<Page>();

    let gdt = Box::leak(Box::new(GlobalDescriptorTable::new()));
    let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
    let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));

    gdt.load();
    unsafe {
        set_cs(code_selector);
        load_tss(tss_selector);
    }
}

pub fn init_idt() {
    let idt = Box::leak(Box::new(InterruptDescriptorTable::new()));
    idt.breakpoint.set_handler_fn(breakpoint);
    let double_fault_entry = idt.double_fault.set_handler_fn(double_fault);
    unsafe { double_fault_entry.set_stack_index(DOUBLE_FAULT_IST_IDX) };

    idt.load();
}

extern "x86-interrupt" fn breakpoint(stack_frame: &mut ExceptionStackFrame) {
    SCREEN.lock().set_font_color(Color::Magenta);
    lock_writeln!(SCREEN, "BREAKPOINT: {:?}", stack_frame);
}

extern "x86-interrupt" fn double_fault(stack_frame: &mut ExceptionStackFrame, _: u64) {
    panic!("DOUBLE FAULT: {:?}", stack_frame)
}
