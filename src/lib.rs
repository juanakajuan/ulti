//! Ulti library modules for the terminal emulator runtime.

/// Application startup, window event handling, and render loop orchestration.
pub mod app;

/// Font discovery and loading for terminal glyph rasterization.
mod font;
/// Keyboard input translation from window events into terminal byte sequences.
mod input;
/// Runtime coordination for one pane's terminal session and parser state.
mod pane_runtime;
/// Framebuffer renderer for the parsed terminal screen grid.
mod renderer;
/// PTY-backed terminal session process and channel management.
mod terminal_session;
