use x86_64::instructions::{hlt, port::Port};

pub unsafe fn qemu_exit() -> ! {
    Port::<u32>::new(0xf4).write(0);
    stop()
}

pub fn stop() -> ! {
    loop {
        hlt()
    }
}
