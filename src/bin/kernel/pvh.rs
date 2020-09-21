use core::mem::size_of;

#[naked]
unsafe extern "C" fn rust32_start() -> ! {
    // TODO insert code to go from 32-bit mode to 64-bit mode
    loop {}
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

// This is: ELFNOTE(Xen, XEN_ELFNOTE_PHYS32_ENTRY, .quad rust_start)
#[cfg(not(test))]
#[link_section = ".note"]
#[used]
static PVH_NOTE: Note = Note {
    name_size: size_of::<Name>() as u32,
    desc_size: size_of::<Desc>() as u32,
    kind: XEN_ELFNOTE_PHYS32_ENTRY,
    name: *b"Xen\0",
    desc: rust32_start,
};
