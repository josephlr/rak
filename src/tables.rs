use x86_64::{
    structures::paging::{PageSize, PageTable, PageTableFlags, Size2MiB},
    PhysAddr,
};

const ADDRESS_SPACE_GIB: usize = 4;

const RWP_FLAGS: PageTableFlags = PageTableFlags::from_bits_truncate(
    PageTableFlags::PRESENT.bits() | PageTableFlags::WRITABLE.bits(),
);
const L2_FLAGS: PageTableFlags =
    PageTableFlags::from_bits_truncate(RWP_FLAGS.bits() | PageTableFlags::HUGE_PAGE.bits());

const fn make_huge_l2_tables(start: u64) -> [PageTable; ADDRESS_SPACE_GIB] {
    let mut l2s = [PageTable::new(); ADDRESS_SPACE_GIB];
    let mut addr = start;

    let mut i = 0;
    while i < ADDRESS_SPACE_GIB {
        let l2 = &mut l2s[i];

        let mut j = 0;
        while j < l2.entries.len() {
            let entry = &mut l2.entries[j];

            entry.set_addr(PhysAddr::new_truncate(addr), L2_FLAGS);
            addr += Size2MiB::SIZE;
            j += 1;
        }
        i += 1;
    }
    l2s
}

const fn make_higher_level_table(tables: &[PageTable]) -> PageTable {
    let mut pt = PageTable::new();
    let mut j = 0;
    while j < tables.len() {
        let entry = &mut pt.entries[j];
        let addr: *const PageTable = &tables[j];
        entry.set_addr(unsafe { PhysAddr::new_unsafe(addr) }, RWP_FLAGS);
        j += 1;
    }
    pt
}

#[link_section = ".l2"]
static L2_PTS: [PageTable; ADDRESS_SPACE_GIB] = make_huge_l2_tables(0);
#[link_section = ".l3"]
static L3_PTS: [PageTable; 1] = [make_higher_level_table(&L2_PTS)];
#[link_section = ".l4"]
pub static L4_PTS: [PageTable; 1] = [make_higher_level_table(&L3_PTS)];

use core::mem::size_of;

bitflags::bitflags! {
    // An extension of x86_64::structures::gdt::DescriptorFlags
    struct Descriptor: u64 {
        const LIMIT_0_15 =   0xFFFF;
        const BASE_0_23 = 0xFF_FFFF << 16;
        const ACCESSED =          1 << 40;
        const WRITABLE =          1 << 41;  // Only for Data-Segments
        const READABLE =          1 << 41;  // Only for Code-Segments
        const EXPANSION =         1 << 42;  // Only for Data-Segments
        const CONFORMING =        1 << 42;  // Only for Code-Segments
        const EXECUTABLE =        1 << 43;
        const USER_SEGMENT =      1 << 44;
        const DPL_RING_3 =        3 << 45;
        const PRESENT =           1 << 47;
        const LIMIT_16_19 =     0xF << 48;
        const SOFTWARE =          1 << 52;
        const BIT64 =             1 << 53;
        const BIT32 =             1 << 54;
        const GRANULARITY =       1 << 55;
        const BASE_24_31 =     0xFF << 56;

        // All segments are nonconforming, non-system, ring-0 only, and present.
        // We set ACCESSED in advance to avoid writing to the descriptor.
        const COMMON = Self::ACCESSED.bits | Self::USER_SEGMENT.bits | Self::PRESENT.bits;
        // BIT32 must be 0, all other bits (not yet mentioned) are ignored.
        const CODE64 = Self::COMMON.bits | Self::EXECUTABLE.bits | Self::BIT64.bits;
    }
}

// An alternative to x86_64::structures::DescriptorTablePointer that avoids
// "pointer-to-integer cast" (which rust does not support in statics).
#[repr(C, packed)]
pub struct Pointer {
    limit: u16,
    base: &'static Descriptor,
}

impl Pointer {
    const fn new(gdt: &'static [Descriptor]) -> Self {
        let size = gdt.len() * size_of::<Descriptor>();
        Self {
            limit: size as u16 - 1,
            base: &gdt[0],
        }
    }
}

// Our 64-bit GDT lives in RAM, so it can be accessed like any other global.
static GDT64: [Descriptor; 2] = [Descriptor::empty(), Descriptor::CODE64];
pub static GDT64_PTR: Pointer = Pointer::new(&GDT64);
