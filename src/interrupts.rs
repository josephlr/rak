extern crate alloc;

use crate::io::pic8259::PICS;
use crate::io::vga::{Color, SCREEN};
use crate::memory::Page;
use alloc::boxed::Box;
use core::{fmt::Write, mem::size_of};
use lazy_static::lazy_static;
use x86_64::{
    instructions::{port::Port, segmentation::set_cs, tables::load_tss},
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable},
        idt::{ExceptionStackFrame, InterruptDescriptorTable},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

const DOUBLE_FAULT_IST_IDX: u16 = 0;
pub const PIC_START: u8 = 0x20;
pub const PIC_END: u8 = 0x30;

const TIMER_IRQ: u8 = 0;
const KEYBOARD_IRQ: u8 = 1;

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

    idt[(PIC_START + TIMER_IRQ) as usize].set_handler_fn(timer);
    idt[(PIC_START + KEYBOARD_IRQ) as usize].set_handler_fn(keyboard);

    idt.load();
}

extern "x86-interrupt" fn breakpoint(stack_frame: &mut ExceptionStackFrame) {
    SCREEN.lock().set_font_color(Color::Magenta);
    lock_writeln!(SCREEN, "BREAKPOINT: {:?}", stack_frame);
}

extern "x86-interrupt" fn double_fault(stack_frame: &mut ExceptionStackFrame, _: u64) {
    panic!("DOUBLE FAULT: {:?}", stack_frame)
}

extern "x86-interrupt" fn timer(_: &mut ExceptionStackFrame) {
    SCREEN.lock().set_font_color(Color::LightBlue);
    lock_write!(SCREEN, ".");
    PICS.lock().cleanup_interrupt(PIC_START + TIMER_IRQ);
}

extern "x86-interrupt" fn keyboard(_: &mut ExceptionStackFrame) {
    SCREEN.lock().set_font_color(Color::LightGreen);
    let scan: u8 = unsafe { Port::new(0x60).read() };
    lock_write!(SCREEN, "{:x}", scan);
    PICS.lock().cleanup_interrupt(PIC_START + KEYBOARD_IRQ);
}

#[allow(dead_code)]
fn log_pic_regs() {
    let mut pics = PICS.lock();
    lock_writeln!(
        SCREEN,
        "IRR={:x}, ISR={:x}, IMR={:x}",
        pics.read_irr(),
        pics.read_isr(),
        pics.read_imr()
    );
}
