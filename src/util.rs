use spin::{Mutex, MutexGuard};
use x86_64::instructions::{hlt, port::Port};

#[macro_export]
macro_rules! lock_write {
    ($dst:expr, $($arg:tt)*) => (write!($dst.lock(), $($arg)*))
}

#[macro_export]
macro_rules! lock_writeln {
    ($dst:expr, $($arg:tt)*) => (writeln!($dst.lock(), $($arg)*))
}

pub unsafe fn force_unwrap<T>(m: &Mutex<T>) -> MutexGuard<T> {
    loop {
        m.force_unlock();
        if let Some(guard) = m.try_lock() {
            return guard;
        }
    }
}

#[allow(dead_code)]
pub fn port_wait(n: usize) {
    for _ in 0..n {
        unsafe { Port::new(0x80).write(0u8) }
    }
}

pub fn halt() -> ! {
    loop {
        hlt()
    }
}

// This is unsafe, because for this to work, QEMU must be run with:
//    -device isa-debug-exit,iobase=0xf4,iosize=0x04
#[allow(dead_code)]
pub unsafe fn qemu_shutdown() -> ! {
    Port::new(0xf4).write(0u32);
    halt()
}
