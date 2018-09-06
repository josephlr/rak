use core::fmt;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::port::Port;

pub struct Serial {
    data: Port<u8>,
    line_status: Port<u8>,
}

#[allow(dead_code)]
#[repr(u16)]
enum Addr {
    COM1 = 0x3F8,
    COM2 = 0x2F8,
    COM3 = 0x3E8,
    COM4 = 0x2E8,
}

#[allow(dead_code)]
#[repr(u16)]
enum Baud {
    B115200 = 1,
    B57600 = 2,
    B38400 = 3,
    B19200 = 6,
    B9600 = 12,
    B4800 = 24,
    B2400 = 48,
    B1200 = 96,
    B300 = 384,
}

impl Serial {
    fn new(addr: Addr, baud: Baud) -> Self {
        let base = addr as u16;
        let mut data = Port::<u8>::new(base);
        let mut interrupt_ctrl = Port::<u8>::new(base + 1);
        let mut fifo_ctrl = Port::<u8>::new(base + 2);
        let mut line_ctrl = Port::<u8>::new(base + 3);
        let mut modem_ctrl = Port::<u8>::new(base + 4);
        let line_status = Port::<u8>::new(base + 5);

        let [low_baud, high_baud] = (baud as u16).to_le_bytes();
        unsafe {
            interrupt_ctrl.write(0x00); // Disable interrupts
            line_ctrl.write(0x80); // Enable DLAB
            data.write(low_baud);
            interrupt_ctrl.write(high_baud);
            line_ctrl.write(0x03); // Disable DLAB, 8 bits, stop bit, no parity
            fifo_ctrl.write(0xC7); // FIFO ??
            modem_ctrl.write(0x0B); // Modem ??
            interrupt_ctrl.write(0x01); // Enable interrupts
        }
        Serial { data, line_status }
    }

    fn send(&mut self, data: u8) {
        unsafe {
            while self.line_status.read() & 0x20 == 0 {}
            self.data.write(data);
        }
    }

    fn recv(&self) -> u8 {
        unsafe {
            while self.line_status.read() & 0x01 == 0 {}
            self.data.read()
        }
    }

    pub fn echo(&mut self) {
        loop {
            let data = self.recv();
            writeln!(self, "{:x}", data);
        }
    }
}

impl fmt::Write for Serial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() {
            match c {
                0x08 | 0x7F => {
                    self.send(0x08);
                    self.send(b' ');
                    self.send(0x08);
                }
                _ => self.send(c),
            }
        }
        Ok(())
    }
}

lazy_static! {
    pub static ref SERIAL1: Mutex<Serial> = Mutex::new(Serial::new(Addr::COM1, Baud::B115200));
}
