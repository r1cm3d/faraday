/*!
PTY-based OBD-II ECU emulator for faraday.

Creates a virtual serial port (PTY) that speaks the ELM327/STN text protocol,
allowing faraday to connect as if it were a real OBD-II adapter.
*/

mod ecu;
mod handler;
mod pty;

use anyhow::{Context, Result};
use clap::Parser;
use handler::EcuHandler;
use pty::PtyPair;
use std::os::unix::io::{AsRawFd, OwnedFd};
use std::path::PathBuf;
use tokio::io::unix::AsyncFd;
use tokio::signal;
use tracing::info;

#[derive(Parser)]
#[command(name = "faraday-emu", about = "PTY-based OBD-II ECU emulator")]
struct Args {
    #[arg(long, help = "Symlink from PATH to PTY slave (e.g. /tmp/faraday-dev)")]
    link: Option<PathBuf>,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    setup_logging(args.verbose);

    let pair = PtyPair::open(args.link.as_deref()).context("Failed to open PTY")?;
    println!("{}", pair.slave_path.display());
    info!("PTY slave: {}", pair.slave_path.display());

    let _cleanup = pair.cleanup;
    let async_fd = AsyncFd::new(pair.master).context("Failed to register PTY with tokio")?;

    let mut handler = EcuHandler::new();
    let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
        .context("Failed to install SIGTERM handler")?;

    tokio::select! {
        _ = signal::ctrl_c() => { info!("Ctrl-C received, shutting down"); }
        _ = sigterm.recv()   => { info!("SIGTERM received, shutting down"); }
        r = io_loop(&async_fd, &mut handler) => {
            r.context("IO loop error")?;
        }
    }

    Ok(())
}

async fn io_loop(
    async_fd: &AsyncFd<OwnedFd>,
    handler: &mut EcuHandler,
) -> Result<()> {
    let fd = async_fd.as_raw_fd();
    let mut line_buf: Vec<u8> = Vec::with_capacity(128);

    loop {
        let mut guard = async_fd.readable().await?;
        let mut buf = [0u8; 256];

        match guard.try_io(|_| {
            nix::unistd::read(fd, &mut buf).map_err(std::io::Error::from)
        }) {
            Ok(Ok(0)) => return Ok(()),
            Ok(Ok(n)) => {
                for &byte in &buf[..n] {
                    if byte == b'\r' {
                        let line =
                            String::from_utf8_lossy(&line_buf).trim().to_uppercase();
                        let cmd = EcuHandler::parse_command(&line);
                        let response = handler.handle(cmd);
                        write_response(fd, &response)?;
                        line_buf.clear();
                    } else if byte != b'\n' {
                        line_buf.push(byte);
                    }
                }
            }
            Ok(Err(e)) => return Err(e.into()),
            Err(_) => {}
        }
    }
}

fn write_response(fd: i32, data: &[u8]) -> Result<()> {
    let mut written = 0;
    while written < data.len() {
        let n = unsafe {
            libc::write(
                fd,
                data[written..].as_ptr() as *const libc::c_void,
                data.len() - written,
            )
        };
        if n < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        written += n as usize;
    }
    Ok(())
}

fn setup_logging(verbose: u8) {
    let level = match verbose {
        0 => tracing::Level::WARN,
        1 => tracing::Level::INFO,
        2 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };
    let _ = tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .try_init();
}
