use core::mem::size_of;
use rak::tables::{GDT64_PTR, L4_PTS};
use x86_64::registers::control::{Cr0Flags, Cr4Flags, EferFlags};

const STACK_SIZE: usize = 1024 * 1024; // 1 MiB
#[repr(C, align(4096))]
struct Stack([u8; STACK_SIZE]);
static mut STACK: Stack = Stack([0; STACK_SIZE]);

#[naked]
#[link_section = ".text32"]
#[cfg(target_os = "none")]
unsafe extern "C" fn start32() -> ! {
    asm!(
        ".code32",
        // Setup the stack, and pass the PVH structure (ebx) as
        // the first sysv64 param (edi) to the Rust code.
        "lea esp, [{esp_init}]",
        "mov edi, ebx",
        // Setup our 64-bit GDT (not used until the long jump below)
        "lgdt {gdt_init}",
        // Load our static page tables
        "lea eax, [{cr3_init}]",
        "mov cr3, eax",
        // Set CR4.PAE (Physical Address Extension)
        "mov eax, cr4",
        "or eax, {pae}",
        "mov cr4, eax",
        // Set EFER.LME (Long Mode Enable)
        "mov ecx, {efer}",
        "rdmsr",
        "or eax, {lme}",
        "wrmsr",
        // Set CRO.PG (Paging), must happen after the above 3 steps
        "mov eax, cr0",
        "or eax, {pg}",
        "mov cr0, eax", // must be followed by a branch
        // Far return to 64-bit Rust code
        "ljmp {code_sel}, OFFSET {start64}",
        ".code64",
        esp_init = sym STACK,
        gdt_init = sym GDT64_PTR,
        cr3_init = sym L4_PTS,
        start64 = sym crate::rust_start,
        code_sel = const 0x08u16,
        efer = const 0xC0000080u32,
        pae = const Cr4Flags::PHYSICAL_ADDRESS_EXTENSION.bits(),
        lme = const EferFlags::LONG_MODE_ENABLE.bits(),
        pg = const Cr0Flags::PAGING.bits(),
        options(noreturn),
    )
}

// The kind/name/desc of the PHV ELF Note are from xen/include/public/elfnote.h.
// This is the "Physical entry point into the kernel".
const XEN_ELFNOTE_PHYS32_ENTRY: u32 = 18;
type Name = [u8; 4];
type Desc = unsafe extern "C" fn() -> !;

// We make sure our ELF Note has an alignment of 4 for maximum compatibility.
// Some software (QEMU) calculates padding incorectly if alignment != 4.
#[repr(C, packed(4))]
struct Note {
    name_size: u32,
    desc_size: u32,
    kind: u32,
    name: Name,
    desc: Desc,
}

// This is: ELFNOTE(Xen, XEN_ELFNOTE_PHYS32_ENTRY, .quad rust32_start)
#[used]
#[link_section = ".note"]
#[cfg(target_os = "none")]
static PVH_NOTE: Note = Note {
    name_size: size_of::<Name>() as u32,
    desc_size: size_of::<Desc>() as u32,
    kind: XEN_ELFNOTE_PHYS32_ENTRY,
    name: *b"Xen\0",
    desc: start32,
};
