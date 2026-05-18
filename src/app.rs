//! Native window setup and event loop for the bare bones terminal surface.

use anyhow::{Context, Result};
use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, ModifiersState, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::{
    font::load_monospace_font,
    input::key_bytes,
    pane_runtime::PaneRuntime,
    renderer::{CELL_HEIGHT, CELL_WIDTH, PADDING, draw_terminal},
    terminal_session::{
        TerminalSession, TerminalSessionError, spawn_terminal, terminal_size_for_window,
    },
};

/// Initial terminal width, in character cells, before the window is resized.
const INITIAL_COLS: u16 = 100;
/// Initial terminal height, in character cells, before the window is resized.
const INITIAL_ROWS: u16 = 32;

/// Starts the Ulti application window, terminal session, and render loop.
///
/// Returns an error if startup cannot create the native window, framebuffer, font,
/// or PTY-backed shell. Runtime render and resize failures are reported to stderr
/// before the event loop exits.
pub fn run() -> Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Ulti")
        .with_inner_size(LogicalSize::new(
            f64::from(INITIAL_COLS) * f64::from(CELL_WIDTH) + f64::from(PADDING * 2),
            f64::from(INITIAL_ROWS) * f64::from(CELL_HEIGHT) + f64::from(PADDING * 2),
        ))
        .build(&event_loop)
        .context("failed to create window")?;

    let mut modifiers = ModifiersState::empty();
    let font = load_monospace_font()?;
    let size = window.inner_size();
    let surface = SurfaceTexture::new(size.width, size.height, &window);
    let mut pixels =
        Pixels::new(size.width, size.height, surface).context("failed to create renderer")?;
    let mut framebuffer_width = size.width;
    let mut framebuffer_height = size.height;
    let terminal_size = terminal_size_for_window(size.width, size.height);
    let terminal = spawn_terminal(terminal_size)?;
    let mut pane_runtime = PaneRuntime::new(terminal_size, terminal);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        if let Err(error) = drain_terminal_output(&mut pane_runtime, &window) {
            report_terminal_error(error, control_flow);
            return;
        }

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                framebuffer_width = size.width;
                framebuffer_height = size.height;

                if !resize_framebuffer(&mut pixels, size.width, size.height) {
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                if let Err(error) =
                    pane_runtime.resize(terminal_size_for_window(size.width, size.height))
                {
                    report_terminal_error(error, control_flow);
                    return;
                }
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::ModifiersChanged(next_modifiers),
                ..
            } => modifiers = next_modifiers,
            Event::WindowEvent {
                event: WindowEvent::ReceivedCharacter(character),
                ..
            } if !modifiers.ctrl() && !character.is_control() => {
                if let Err(error) = send_character_input(&mut pane_runtime, character) {
                    report_terminal_error(error, control_flow);
                    return;
                }
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } if input.state == ElementState::Pressed => {
                if let Some(bytes) = key_bytes(input, modifiers) {
                    if let Err(error) = pane_runtime.write_input(bytes) {
                        report_terminal_error(error, control_flow);
                        return;
                    }
                }
            }
            Event::RedrawRequested(_) => {
                draw_terminal(
                    pixels.frame_mut(),
                    framebuffer_width,
                    framebuffer_height,
                    &font,
                    pane_runtime.parser(),
                );
                if let Err(error) = pixels.render() {
                    eprintln!("render failed: {error}");
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
    });
}

/// Processes all pending PTY output and schedules a redraw when output arrives.
fn drain_terminal_output<S>(
    pane_runtime: &mut PaneRuntime<S>,
    window: &Window,
) -> std::result::Result<(), TerminalSessionError>
where
    S: TerminalSession,
{
    if pane_runtime.drain_output()? {
        window.request_redraw();
    }

    Ok(())
}

/// Resizes both pixel surfaces, reporting failures before asking the loop to exit.
fn resize_framebuffer(pixels: &mut Pixels, width: u32, height: u32) -> bool {
    if let Err(error) = pixels.resize_surface(width, height) {
        eprintln!("resize surface failed: {error}");
        return false;
    }

    if let Err(error) = pixels.resize_buffer(width, height) {
        eprintln!("resize buffer failed: {error}");
        return false;
    }

    true
}

/// Encodes one printable character and sends it to the terminal input stream.
fn send_character_input<S>(
    pane_runtime: &mut PaneRuntime<S>,
    character: char,
) -> std::result::Result<(), TerminalSessionError>
where
    S: TerminalSession,
{
    let mut buffer = [0; 4];
    let text = character.encode_utf8(&mut buffer);
    pane_runtime.write_input(text.as_bytes().to_vec())
}

/// Reports terminal session failures and asks the native event loop to exit.
fn report_terminal_error(error: TerminalSessionError, control_flow: &mut ControlFlow) {
    eprintln!("terminal session failed: {error}");
    *control_flow = ControlFlow::Exit;
}
