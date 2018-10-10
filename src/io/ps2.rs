// Driver for PS/2 keyboards and mice.
// Currently only keyboards are supported.

use bitflags::bitflags;
use x86_64::instructions::port::Port;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Command {
    ReadCCB = 0x20,
    WriteCCB = 0x60,
    Led = 0xed,
    Echo = 0xee,
    ScanCode = 0xf0,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Response {
    Error1 = 0x00,
    SelfTestPassed = 0xaa,
    Echo = 0xee,
    Ack = 0xfa,
    SelfTestFailed1 = 0xfc,
    SelfTestFailed2 = 0xfd,
    Resend = 0xfe,
    Error2 = 0xff,
}

bitflags! {
    struct Status: u8 {
        const OUTPUT_BUFFER_FULL = 1 << 0; // Cleared: Output buffer empty
        const INPUT_BUFFER_FULL  = 1 << 1; // Cleared: Input buffer empty
        const SYSTEM_FLAG        = 1 << 2;
        const INPUT_IS_COMMAND   = 1 << 3; // Cleared: Input is data to device
        const TIMEOUT_ERROR      = 1 << 6;
        const PARITY_ERROR       = 1 << 7;
    }
}

struct Ps2 {
    data: Port<u8>,
    command: Port<u8>,
}

impl Ps2 {
    // PS2 controller must exist, or this command is unsafe.
    const unsafe fn new() -> Self {
        Ps2 {
            data: Port::new(0x60),
            command: Port::new(0x64),
        }
    }
}
