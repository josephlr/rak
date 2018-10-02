use spin::{Mutex, MutexGuard};
use x86_64::instructions::hlt;

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

pub fn halt() -> ! {
    loop {
        hlt()
    }
}
