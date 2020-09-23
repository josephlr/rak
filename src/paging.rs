use crate::Plat;
use core::{fmt, mem::align_of};
use x86_64::{
    structures::paging::{PageSize, PageTableFlags as Flags, Size2MiB},
    PhysAddr,
};

/// Alternative to x86_64::PageTableEntry that can be used with static pointers.
struct Entry(*const u8);
unsafe impl Send for Entry {}
unsafe impl Sync for Entry {}

impl Entry {
    const fn new() -> Self {
        Self(core::ptr::null())
    }
    const fn leaf_entry(addr: PhysAddr, flags: Flags) -> Self {
        assert!((addr.as_u64() as usize) % align_of::<Table>() == 0);
        Self((addr.as_u64() + flags.bits()) as *const _)
    }
    const fn ptr_entry(table: &Table, flags: Flags) -> Self {
        let ptr: *const u8 = table as *const Table as *const u8;
        Self(ptr.wrapping_add(flags.bits() as usize))
    }
    fn addr(&self) -> PhysAddr {
        PhysAddr::new((self.0 as u64) - self.flags().bits())
    }
    fn flags(&self) -> Flags {
        let bits = (self.0 as usize) % align_of::<Table>();
        Flags::from_bits_truncate(bits as u64)
    }
}

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("Entry");
        f.field("addr", &self.addr());
        f.field("flags", &self.flags());
        f.finish()
    }
}

const ENTRY_COUNT: usize = 512;

/// Alternative to x86_64::PageTable that uses alternative Entry
#[repr(C, align(4096))]
pub struct Table([Entry; ENTRY_COUNT]);

impl Table {
    const fn new() -> Self {
        Self([Entry::new(); ENTRY_COUNT])
    }
}

// Number of GiB to identity map (max 512 GiB)
const ADDRESS_SPACE_GIB: usize = 4;

// Common flags to all page table entries.
const FLAGS: Flags = Flags::from_bits_truncate(Flags::PRESENT.bits() | Flags::WRITABLE.bits());
// Set HUGE_PAGE in our L2 so we don't need to have a PML1
const L2_FLAGS: Flags = Flags::from_bits_truncate(FLAGS.bits() | Flags::HUGE_PAGE.bits());

// Make a PML2 that maps virtual addresses [0, ADDRESS_SPACE_GIB) to a
// contiguous range of physical addresses starting at `start`.
const fn make_pml2s(start: u64) -> [Table; ADDRESS_SPACE_GIB] {
    let mut pml2s = [Table::new(); ADDRESS_SPACE_GIB];
    let mut addr = start;

    let mut i = 0;
    while i < ADDRESS_SPACE_GIB {
        let pml2 = &mut pml2s[i];

        let mut j = 0;
        while j < pml2.0.len() {
            pml2.0[j] = Entry::leaf_entry(PhysAddr::new_truncate(addr), L2_FLAGS);
            addr += Size2MiB::SIZE;
            j += 1;
        }
        i += 1;
    }
    pml2s
}

// Make a page table pointing to the provided array of tables.
const fn make_ptr_table(tables: &[Table]) -> Table {
    let mut table = Table::new();
    let mut j = 0;
    while j < tables.len() {
        table.0[j] = Entry::ptr_entry(&tables[j], FLAGS);
        j += 1;
    }
    table
}

static PML2: [Table; ADDRESS_SPACE_GIB] = make_pml2s(0);
static PML3: [Table; 1] = [make_ptr_table(&PML2)];
pub static PML4: Table = make_ptr_table(&PML3);

pub fn log_tables(plat: &mut dyn Plat) -> fmt::Result {
    writeln!(plat.logger(), "L4 addr: {:p}", &PML4)?;
    writeln!(plat.logger(), "L4 entry: {:?}", PML4.0[0])?;
    writeln!(plat.logger(), "L3 addr: {:p}", &PML3)?;
    writeln!(plat.logger(), "L3 entry: {:?}", PML3[0].0[0])?;
    writeln!(plat.logger(), "L2 addr: {:p}", &PML2)?;
    for i in 0..10 {
        writeln!(plat.logger(), "L2 entry: {:?}", PML2[0].0[i])?;
    }
    Ok(())
}
