//! PTY creation, raw-mode configuration, and symlink management.

use anyhow::{Context, Result};
use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::pty::openpty;
use nix::sys::termios::{cfmakeraw, tcgetattr, tcsetattr, SetArg};
use nix::unistd::ttyname;
use std::os::unix::fs::symlink;
use std::os::unix::io::{AsFd, AsRawFd, OwnedFd};
use std::path::{Path, PathBuf};

/// RAII guard that removes an optional symlink when dropped.
pub struct PtyCleanup(
    /// Path of the symlink to remove on drop, or `None` if no cleanup is needed.
    pub Option<PathBuf>,
);

impl Drop for PtyCleanup {
    fn drop(&mut self) {
        if let Some(p) = &self.0 {
            let _ = std::fs::remove_file(p);
        }
    }
}

/// An open PTY pair (master + slave) ready for async I/O.
pub struct PtyPair {
    /// Owned file descriptor for the master side of the PTY.
    pub master: OwnedFd,
    /// Path to the slave device (e.g. `/dev/pts/3`).
    pub slave_path: PathBuf,
    /// Cleans up the optional symlink when this `PtyPair` is dropped.
    pub cleanup: PtyCleanup,
    _slave_keepalive: OwnedFd,
}

impl PtyPair {
    /// Opens a new PTY pair, sets the master to raw non-blocking mode, and optionally
    /// creates a symlink at `symlink_path` pointing to the slave device.
    pub fn open(symlink_path: Option<&Path>) -> Result<Self> {
        let result = openpty(None, None).context("openpty failed")?;
        let slave_path = ttyname(result.slave.as_fd()).context("ttyname failed")?;
        make_raw(result.master.as_raw_fd())?;
        set_nonblocking(result.master.as_raw_fd())?;

        let cleanup = if let Some(link) = symlink_path {
            let _ = std::fs::remove_file(link);
            symlink(&slave_path, link).context("Failed to create symlink")?;
            PtyCleanup(Some(link.to_owned()))
        } else {
            PtyCleanup(None)
        };

        Ok(Self {
            master: result.master,
            slave_path,
            cleanup,
            _slave_keepalive: result.slave,
        })
    }
}

fn make_raw(fd: i32) -> Result<()> {
    use std::os::unix::io::BorrowedFd;
    let borrowed = unsafe { BorrowedFd::borrow_raw(fd) };
    let mut termios = tcgetattr(borrowed).context("tcgetattr failed")?;
    cfmakeraw(&mut termios);
    tcsetattr(borrowed, SetArg::TCSANOW, &termios).context("tcsetattr failed")?;
    Ok(())
}

fn set_nonblocking(fd: i32) -> Result<()> {
    let flags = fcntl(fd, FcntlArg::F_GETFL).context("fcntl F_GETFL failed")?;
    let new_flags = OFlag::from_bits_truncate(flags) | OFlag::O_NONBLOCK;
    fcntl(fd, FcntlArg::F_SETFL(new_flags)).context("fcntl F_SETFL failed")?;
    Ok(())
}
