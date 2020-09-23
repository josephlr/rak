use x86_64::structures::{
    gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
    DescriptorTablePointer,
};

const fn make_gdt() -> (GlobalDescriptorTable, SegmentSelector) {
    let mut gdt = GlobalDescriptorTable::new();
    let cs = gdt.add_entry(Descriptor::kernel_code_segment());
    (gdt, cs)
}

const GDT: (GlobalDescriptorTable, SegmentSelector) = make_gdt();
pub static GDT_PTR: DescriptorTablePointer = GDT.0.pointer();
pub const CS: SegmentSelector = GDT.1;
