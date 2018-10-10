// Driver for the 8259A Programable Interrupt Controller (PIC)
// December 1988 Datasheet downloaded from:
// https://pdos.csail.mit.edu/6.828/2014/readings/hardware/8259A.pdf

use bit_field::BitField;
use bitflags::bitflags;
use core::ops::Range;
use spin::Mutex;
use x86_64::instructions::port::Port;

const INTERRUPTS_PER_PIC: u8 = 8;

bitflags! {
    struct ICW1: u8 {
        const ICW4_NEEDED      = 1 << 0;
        const SINGLE_MODE      = 1 << 1; // Cleared: cascade mode
        const CAll_ADDR_IVAL_8 = 1 << 2; // Cleared: CALL address interval of 4
        const LEVEL_TRIGGERED  = 1 << 3; // Cleared: edge triggered
        const MAND             = 1 << 4; // Mandatory
    }
}

// The input to the master that the second PIC is slaved to (used for ICW3)
const SLAVE_INPUT: u8 = 2;

bitflags! {
    struct ICW4: u8 {
        const INTEL_8086      = 1 << 0; // Cleared: MCS-80/85 mode
        const AUTO_EOI        = 1 << 1; // Cleared: normal End of Interrupt
        const BUFFERED_MASTER = 1 << 2; // Cleared: buffered mode - slave
        const BUFFERED        = 1 << 3; // Cleared: non buffred mode
        const FULLY_NESTED    = 1 << 4;
    }
}

const OCW2_EOI: u8 = 0x20;

bitflags! {
    struct OCW3: u8 {
        const READ_ISR    = 1 << 0; // Cleared: read IRR
        const ENABLE_READ = 1 << 1;
        const POLL        = 1 << 2;
        const MAND        = 1 << 3; // Mandatory
    }
}

pub static PICS: Mutex<ChainedPics> = Mutex::new(ChainedPics::new());

struct Pic {
    interrupt_ids: Range<u8>,
    command: Port<u8>,
    data: Port<u8>,
}

impl Pic {
    fn irq(&self, interrupt_id: u8) -> Option<u8> {
        if !self.interrupt_ids.contains(&interrupt_id) {
            return None;
        }
        Some(interrupt_id - self.interrupt_ids.start)
    }
}

pub struct ChainedPics {
    master: Pic,
    slave: Pic,
}

impl ChainedPics {
    const fn new() -> Self {
        ChainedPics {
            master: Pic {
                interrupt_ids: 0x08..0x10,
                command: Port::new(0x20),
                data: Port::new(0x21),
            },
            slave: Pic {
                interrupt_ids: 0x70..0x78,
                command: Port::new(0xA0),
                data: Port::new(0xA1),
            },
        }
    }

    pub fn remap(&mut self, interrupt_ids: Range<u8>) {
        assert!(interrupt_ids.len() == (2 * INTERRUPTS_PER_PIC).into());
        assert!(interrupt_ids.start % INTERRUPTS_PER_PIC == 0);
        let Range { start, end } = interrupt_ids;
        let slave_start = start + 8;

        self.master.interrupt_ids = start..slave_start;
        self.slave.interrupt_ids = slave_start..end;

        let imr = self.read_imr();
        unsafe {
            // ICW1
            self.write_command((ICW1::ICW4_NEEDED | ICW1::MAND).bits);

            // ICW2
            self.master.data.write(self.master.interrupt_ids.start);
            self.slave.data.write(self.slave.interrupt_ids.start);

            // ICW3
            self.master.data.write(1 << SLAVE_INPUT);
            self.slave.data.write(SLAVE_INPUT);

            // ICW4
            self.master.data.write(ICW4::INTEL_8086.bits);
            self.slave.data.write(ICW4::INTEL_8086.bits);
        }
        self.set_imr(imr);
    }

    fn read_command(&self) -> u16 {
        unsafe { u16::from_le_bytes([self.master.command.read(), self.slave.command.read()]) }
    }

    unsafe fn write_command(&mut self, command: u8) {
        self.master.command.write(command);
        self.slave.command.write(command);
    }

    #[allow(dead_code)]
    pub fn read_irr(&mut self) -> u16 {
        unsafe { self.write_command((OCW3::ENABLE_READ | OCW3::MAND).bits) };
        self.read_command()
    }

    #[allow(dead_code)]
    pub fn read_isr(&mut self) -> u16 {
        unsafe { self.write_command((OCW3::READ_ISR | OCW3::ENABLE_READ | OCW3::MAND).bits) };
        self.read_command()
    }

    #[allow(dead_code)]
    pub fn read_imr(&self) -> u16 {
        unsafe { u16::from_le_bytes([self.master.data.read(), self.slave.data.read()]) }
    }

    pub fn set_imr(&mut self, mask: u16) {
        let [master_mask, slave_mask] = mask.to_le_bytes();
        unsafe {
            self.master.data.write(master_mask);
            self.slave.data.write(slave_mask);
        }
    }

    fn irq(&self, interrupt_id: u8) -> Option<u8> {
        match (self.master.irq(interrupt_id), self.slave.irq(interrupt_id)) {
            (Some(irq1), _) => Some(irq1),
            (None, Some(irq2)) => Some(irq2 + INTERRUPTS_PER_PIC),
            (None, None) => None,
        }
    }

    #[allow(dead_code)]
    pub fn handles_interrupt(&self, interrupt_id: u8) -> bool {
        self.irq(interrupt_id).is_some()
    }

    pub fn servicing_interrupt(&mut self, interrupt_id: u8) -> bool {
        match self.irq(interrupt_id) {
            Some(irq) => self.read_isr().get_bit(irq.into()),
            None => false,
        }
    }

    pub fn cleanup_interrupt(&mut self, interrupt_id: u8) {
        // Handle spurious interrupts
        let is_serviced = self.servicing_interrupt(interrupt_id);
        let is_slave = self.slave.irq(interrupt_id).is_some();

        if is_slave || is_serviced {
            unsafe { self.master.command.write(OCW2_EOI) };
        }
        if is_slave && is_serviced {
            unsafe { self.slave.command.write(OCW2_EOI) };
        }
    }
}
