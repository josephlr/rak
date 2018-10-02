use x86_64::instructions::hlt;

#[macro_export]
macro_rules! lock_write {
    ($dst:expr, $($arg:tt)*) => (write!($dst.lock(), $($arg)*))
}

#[macro_export]
macro_rules! lock_writeln {
    ($dst:expr, $($arg:tt)*) => (writeln!($dst.lock(), $($arg)*))
}

pub fn halt() -> ! {
    loop {
        hlt()
    }
}
