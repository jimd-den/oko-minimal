//! The terminal as a device: raw mode, and the way back.
//!
//! These tests use a pseudo-terminal, because a terminal is a device and the
//! only honest way to test one is against a device that behaves like one.
//! A pipe would not do: the settings of a pipe are not settings.

use std::panic::{AssertUnwindSafe, catch_unwind};

use super::terminal::Terminal;

/// A pseudo-terminal: a master side and a slave side, and the slave behaves
/// like a terminal.
///
/// Shared with `shell_tests`, which needs a real terminal to run the shell
/// against: the bug it pins is an order of bytes, and only a device shows it.
pub(crate) struct Pty {
    pub(crate) master: libc::c_int,
    pub(crate) slave: libc::c_int,
}

impl Pty {
    /// Open a pseudo-terminal.
    pub(crate) fn open() -> Pty {
        let master = unsafe { libc::open(c"/dev/ptmx".as_ptr(), libc::O_RDWR | libc::O_NOCTTY) };
        assert!(master >= 0, "the master side of a pty");

        let mut number: libc::c_uint = 0;
        unsafe { libc::ioctl(master, libc::TIOCGPTN, &mut number) };
        let mut unlock: libc::c_int = 0;
        unsafe { libc::ioctl(master, libc::TIOCSPTLCK, &mut unlock) };

        let path = std::ffi::CString::new(format!("/dev/pts/{number}")).expect("a path");
        let slave = unsafe { libc::open(path.as_ptr(), libc::O_RDWR | libc::O_NOCTTY) };
        assert!(slave >= 0, "the slave side of a pty");

        Pty { master, slave }
    }
}

impl Drop for Pty {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.slave);
            libc::close(self.master);
        }
    }
}

/// The settings of a terminal.
fn settings(fd: libc::c_int) -> libc::termios {
    let mut settings = unsafe { std::mem::zeroed::<libc::termios>() };
    unsafe { libc::tcgetattr(fd, &mut settings) };
    settings
}

#[test]
fn a_terminal_goes_raw_and_comes_back() {
    let pty = Pty::open();
    let before = settings(pty.slave);

    let mut terminal = Terminal::open(pty.slave).expect("a terminal");
    terminal.set_raw().expect("raw mode");

    let raw = settings(pty.slave);
    assert_eq!(raw.c_lflag & libc::ECHO, 0, "the echo is off in raw mode");

    terminal.restore().expect("the settings back");

    let after = settings(pty.slave);
    assert_eq!(after.c_lflag, before.c_lflag, "the settings are back");
}

#[test]
fn a_panic_puts_the_terminal_back() {
    let pty = Pty::open();
    let before = settings(pty.slave);

    let fell_over = catch_unwind(AssertUnwindSafe(|| {
        let mut terminal = Terminal::open(pty.slave).expect("a terminal");
        terminal.set_raw().expect("raw mode");
        panic!("the program fell over");
    }));

    assert!(fell_over.is_err(), "the panic happened");
    let after = settings(pty.slave);
    assert_eq!(
        after.c_lflag, before.c_lflag,
        "a terminal left in raw mode is a broken session"
    );
}

#[test]
fn a_descriptor_that_is_not_a_terminal_is_an_error_and_not_a_panic() {
    let error = match Terminal::open(-1) {
        Ok(_) => panic!("a descriptor that is not a terminal must not open"),
        Err(error) => error,
    };

    assert!(error.raw_os_error().is_some(), "{error}");
}
