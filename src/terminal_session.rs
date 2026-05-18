//! PTY-backed terminal session lifecycle and sizing helpers.

use std::{
    io::{Read, Write},
    thread,
};

use anyhow::{Context, Result};
use crossbeam_channel::{Receiver, Sender, unbounded};
use portable_pty::{Child, CommandBuilder, NativePtySystem, PtySize, PtySystem};

use crate::renderer::{CELL_HEIGHT, CELL_WIDTH, PADDING};

/// Terminal dimensions in character cells and backing pixels.
#[derive(Clone, Copy)]
pub(crate) struct TerminalSize {
    /// Number of terminal rows visible to the shell.
    pub(crate) rows: u16,
    /// Number of terminal columns visible to the shell.
    pub(crate) cols: u16,
    /// Window width reported to the PTY, capped to the PTY API's `u16` range.
    pub(crate) pixel_width: u16,
    /// Window height reported to the PTY, capped to the PTY API's `u16` range.
    pub(crate) pixel_height: u16,
}

/// Channels and child process handle for one running terminal session.
pub(crate) struct TerminalHandle {
    /// Sends raw bytes from window input into the PTY writer thread.
    pub(crate) input_tx: Sender<Vec<u8>>,
    /// Receives raw bytes read from the PTY reader thread.
    pub(crate) output_rx: Receiver<Vec<u8>>,
    /// Sends resize requests from the window event loop into the PTY resizer thread.
    pub(crate) resize_tx: Sender<TerminalSize>,
    /// Keeps the spawned shell alive for as long as the handle is retained.
    _child: Box<dyn Child + Send + Sync>,
}

/// Starts a shell attached to a PTY and returns channel handles for app I/O.
///
/// The session uses `$SHELL` when present and falls back to `/bin/fish`. Three
/// background threads bridge app input, PTY output, and resize messages because
/// the native window loop must stay responsive.
pub(crate) fn spawn_terminal(size: TerminalSize) -> Result<TerminalHandle> {
    let pty_system = NativePtySystem::default();
    let pair = pty_system
        .openpty(PtySize {
            rows: size.rows,
            cols: size.cols,
            pixel_width: size.pixel_width,
            pixel_height: size.pixel_height,
        })
        .context("failed to open PTY")?;

    let shell = std::env::var("SHELL").unwrap_or_else(|_| String::from("/bin/fish"));
    let command = CommandBuilder::new(shell);
    let child = pair
        .slave
        .spawn_command(command)
        .context("failed to spawn shell")?;
    drop(pair.slave);

    let mut reader = pair
        .master
        .try_clone_reader()
        .context("failed to clone PTY reader")?;
    let mut writer = pair
        .master
        .take_writer()
        .context("failed to take PTY writer")?;
    let (input_tx, input_rx) = unbounded::<Vec<u8>>();
    let (output_tx, output_rx) = unbounded::<Vec<u8>>();
    let (resize_tx, resize_rx) = unbounded::<TerminalSize>();

    thread::spawn(move || {
        while let Ok(bytes) = input_rx.recv() {
            if writer.write_all(&bytes).is_err() {
                break;
            }
            let _ = writer.flush();
        }
    });

    thread::spawn(move || {
        let mut buffer = [0; 8192];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => {
                    if output_tx.send(buffer[..count].to_vec()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    thread::spawn(move || {
        while let Ok(size) = resize_rx.recv() {
            let _ = pair.master.resize(PtySize {
                rows: size.rows,
                cols: size.cols,
                pixel_width: size.pixel_width,
                pixel_height: size.pixel_height,
            });
        }
    });

    Ok(TerminalHandle {
        input_tx,
        output_rx,
        resize_tx,
        _child: child,
    })
}

/// Converts a window size into terminal cells and PTY pixel dimensions.
///
/// The returned row and column counts always have at least one cell, even when
/// the window is smaller than the configured padding and cell size.
pub(crate) fn terminal_size_for_window(width: u32, height: u32) -> TerminalSize {
    let usable_width = width.saturating_sub(PADDING * 2);
    let usable_height = height.saturating_sub(PADDING * 2);
    let cols = (usable_width / CELL_WIDTH).max(1).min(u32::from(u16::MAX)) as u16;
    let rows = (usable_height / CELL_HEIGHT)
        .max(1)
        .min(u32::from(u16::MAX)) as u16;

    TerminalSize {
        rows,
        cols,
        pixel_width: width.min(u32::from(u16::MAX)) as u16,
        pixel_height: height.min(u32::from(u16::MAX)) as u16,
    }
}
