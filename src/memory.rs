use core::mem::size_of;

const PAGE_SIZE: usize = 4096;
const NUM_HEAP_PAGES: usize = 1000;

#[derive(Clone, Copy)]
#[repr(align(4096))]
pub struct Page([u8; PAGE_SIZE]);

impl Page {
    pub const fn new() -> Page {
        Page([0; PAGE_SIZE])
    }
}

struct Heap([Page; NUM_HEAP_PAGES]);

impl Heap {
    const fn new() -> Heap {
        Heap([Page::new(); NUM_HEAP_PAGES])
    }
}

static mut HEAP: Heap = Heap::new();

pub fn init_heap() {
    unsafe { crate::ALLOCATOR.init(&HEAP as *const _ as usize, size_of::<Heap>()) };
}

#[allow(dead_code)]
pub unsafe fn bad_write() {
    *(0 as *mut u8) = 42
}

#[allow(dead_code, unconditional_recursion)]
pub unsafe fn stack_overflow() {
    stack_overflow()
}
