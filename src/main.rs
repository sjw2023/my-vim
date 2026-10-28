use std::io::{self, Read};

const NCCS: usize = 32;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Termios {
    c_iflag: u32,     // input flags
    c_oflag: u32,     // outout flags
    c_cflag: u32,     // control flags
    c_lflag: u32,     // local flags
    c_line: u8,       // line discipline ( cc_t = unsigned char)
    c_cc: [u8; NCCS], // control characters
    c_ispeed: u32,    // input baud rate (speed_t = unsigned int)
    c_ospeed: u32,    // output baud rate
}

// c_iflag /usr/include/bits/termios_*.h
const BRKINT: u32 = 0o2;
const INPCK: u32 = 0o20;
const ISTRIP: u32 = 0o40;
const ICRNL: u32 = 0o400;
const IXON: u32 = 0o2000;
// c_oflag (termios-c_oflag.h)
const OPOST: u32 = 0o1;
// c_cflag (termios-c_cflag.h)
const CS8: u32 = 0o60;
// c_lflag (termios-c_lflag.h)
const ISIG: u32 = 0o1;
const ICANON: u32 = 0o2;
const ECHO: u32 = 0o10;
const IEXTEN: u32 = 0o100000;
// c_cc indexes (termios-c_cc.h)
const VTIME: usize = 5;
const VMIN: usize = 6;
// tcsetattr action (termios-tcflow.h)
const TCSAFLUSH: i32 = 2;
// file descriptor 0 = standart input
const STDIN_FILENO: i32 = 0;

// C function from libc; Rust links libc already, so no crate is needed
unsafe extern "C" {
    fn tcgetattr(fd: i32, termios: *mut Termios) -> i32;
    fn tcsetattr(fd: i32, optional_actions: i32, termios: *const Termios) -> i32;
}

struct RawMode {
    original: Termios, // the settings to put back on exit
}

impl RawMode {
    fn enable() -> io::Result<RawMode> {
        let mut original = Termios::default();
        // read the current settings into `original`
        if unsafe { tcgetattr(STDIN_FILENO, &mut original) } == -1 {
            return Err(io::Error::last_os_error());
        }

        let mut raw = original; // Copy: change the copy, keep the original
        raw.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
        raw.c_oflag &= !OPOST;
        raw.c_cflag |= CS8;
        raw.c_lflag &= !(ECHO | ICANON | ISIG | IEXTEN);
        raw.c_cc[VMIN] = 1; // read() waits until at least 1 byte arrives
        raw.c_cc[VTIME] = 0; // no timeout

        if unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw) } == -1 {
            return Err(io::Error::last_os_error());
        }
        Ok(RawMode { original })
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        // runs on return, on `?`, and on panic: the terminal is always restored
        unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &self.original) };
    }
}

fn main() -> io::Result<()> {
    let _raw = RawMode::enable()?; // `_raw`, not `_`: `_` would drop it at once

    let mut stdin = io::stdin();
    let mut buf = [0u8; 1];
    loop {
        if stdin.read(&mut buf)? == 0 {
            break;
        }
        let b = buf[0];
        if b.is_ascii_graphic() {
            print!("{b} ('{}')\r\n", b as char); // OPOST is off: write \r\n ourselves
        } else {
            print!("{b}\r\n"); // control byte; number only
        }
        if b == b'q' {
            break;
        }
    }
    Ok(())
}
