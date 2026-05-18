//! Native window setup and event loop for the bare bones terminal surface.

use anyhow::{Context, Result};
use pixels::{Pixels, SurfaceTexture};
use vt100::Parser;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, ModifiersState, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::{
    font::load_monospace_font,
    input::key_bytes,
    renderer::{CELL_HEIGHT, CELL_WIDTH, PADDING, draw_terminal},
    terminal_session::{spawn_terminal, terminal_size_for_window},
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
    let mut parser = Parser::new(terminal_size.rows, terminal_size.cols, 0);
    let terminal = spawn_terminal(terminal_size)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        while let Ok(bytes) = terminal.output_rx.try_recv() {
            parser.process(&bytes);
            window.request_redraw();
        }

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(size) => {
                    framebuffer_width = size.width;
                    framebuffer_height = size.height;

                    if let Err(error) = pixels.resize_surface(size.width, size.height) {
                        eprintln!("resize surface failed: {error}");
                        *control_flow = ControlFlow::Exit;
                        return;
                    }

                    if let Err(error) = pixels.resize_buffer(size.width, size.height) {
                        eprintln!("resize buffer failed: {error}");
                        *control_flow = ControlFlow::Exit;
                        return;
                    }

                    let new_terminal_size = terminal_size_for_window(size.width, size.height);
                    parser.set_size(new_terminal_size.rows, new_terminal_size.cols);
                    let _ = terminal.resize_tx.send(new_terminal_size);
                    window.request_redraw();
                }
                WindowEvent::ModifiersChanged(next_modifiers) => modifiers = next_modifiers,
                WindowEvent::ReceivedCharacter(character)
                    if !modifiers.ctrl() && !character.is_control() =>
                {
                    let mut buffer = [0; 4];
                    let text = character.encode_utf8(&mut buffer);
                    let _ = terminal.input_tx.send(text.as_bytes().to_vec());
                }
                WindowEvent::KeyboardInput { input, .. }
                    if input.state == ElementState::Pressed =>
                {
                    if let Some(bytes) = key_bytes(input, modifiers) {
                        let _ = terminal.input_tx.send(bytes);
                    }
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                draw_terminal(
                    pixels.frame_mut(),
                    framebuffer_width,
                    framebuffer_height,
                    &font,
                    &parser,
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
