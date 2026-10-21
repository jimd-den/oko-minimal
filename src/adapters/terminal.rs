//! The terminal, as a device: raw mode, a read with a timeout, the size of
//! the window, and the way back to the settings that were there before.
//!
//! Everything here can fail for reasons the program cannot control, and
//! everything here touches a device. The pure half of the terminal
//! adapter, the bytes a frame is made of, is in `presenter`.

use std::io;
use std::os::fd::RawFd;
use std::time::Duration;

use libc::{self, c_int, termios};

/// The terminal, taken over.
pub struct Terminal {
    fd: RawFd,
    original: termios,
    raw: bool,
}

impl Terminal {
    /// Take the terminal on the given descriptor.
    pub fn open(fd: RawFd) -> io::Result<Terminal> {
        let mut original = unsafe { std::mem::zeroed::<termios>() };
        if unsafe { libc::tcgetattr(fd, &mut original) } != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Terminal {
            fd,
            original,
            raw: false,
        })
    }

    /// Take the terminal on standard input.
    pub fn stdin() -> io::Result<Terminal> {
        Terminal::open(libc::STDIN_FILENO)
    }

    /// Turn off the line discipline, so a key arrives when it is pressed.
    pub fn set_raw(&mut self) -> io::Result<()> {
        let mut raw = self.original;
        raw.c_lflag &= !(libc::ICANON | libc::ECHO);
        raw.c_iflag &= !(libc::IXON);
        raw.c_cc[libc::VMIN] = 1;
        raw.c_cc[libc::VTIME] = 0;
        if unsafe { libc::tcsetattr(self.fd, libc::TCSANOW, &raw) } != 0 {
            return Err(io::Error::last_os_error());
        }
        self.raw = true;
        Ok(())
    }

    /// Read the bytes a key press brought, waiting at most `timeout`.
    ///
    /// Zero bytes means the wait ran out, which is how the loop notices
    /// that the document may have changed.
    pub fn read(&mut self, buffer: &mut [u8], timeout: Duration) -> io::Result<usize> {
        let mut fds = libc::pollfd {
            fd: self.fd,
            events: libc::POLLIN,
            revents: 0,
        };
        let millis = timeout.as_millis().min(c_int::MAX as u128) as c_int;
        let ready = unsafe { libc::poll(&mut fds, 1, millis) };
        if ready == 0 {
            return Ok(0);
        }
        if ready < 0 {
            return Err(io::Error::last_os_error());
        }
        let count = unsafe {
            libc::read(
                self.fd,
                buffer.as_mut_ptr().cast::<libc::c_void>(),
                buffer.len(),
            )
        };
        if count < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(count as usize)
    }

    /// The size of the window, in character cells.
    pub fn size(&self) -> io::Result<(u32, u32)> {
        let mut window = unsafe { std::mem::zeroed::<libc::winsize>() };
        if unsafe { libc::ioctl(self.fd, libc::TIOCGWINSZ, &mut window) } != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok((window.ws_col as u32, window.ws_row as u32))
    }

    /// Put the terminal back the way it was found.
    pub fn restore(&mut self) -> io::Result<()> {
        if !self.raw {
            return Ok(());
        }
        self.raw = false;
        if unsafe { libc::tcsetattr(self.fd, libc::TCSANOW, &self.original) } != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}

impl Drop for Terminal {
    /// Put the terminal back, however the program ends.
    ///
    /// A panic unwinds through here, and a terminal left in raw mode is a
    /// broken session for the person who ran it.
    fn drop(&mut self) {
        let _ = self.restore();
    }
}
