//! Keyboard translation from winit events into terminal control bytes.

use winit::event::{KeyboardInput, ModifiersState, VirtualKeyCode};

/// Converts a winit keyboard event into terminal input bytes.
///
/// Printable text is handled separately through `ReceivedCharacter`; this helper
/// only maps control combinations and non-printable keys such as arrows.
pub(crate) fn key_bytes(input: KeyboardInput, modifiers: ModifiersState) -> Option<Vec<u8>> {
    let key = input.virtual_keycode?;

    if modifiers.ctrl() {
        return ctrl_key_bytes(key);
    }

    let bytes: &[u8] = match key {
        VirtualKeyCode::Return => b"\r",
        VirtualKeyCode::Back => b"\x7f",
        VirtualKeyCode::Tab => b"\t",
        VirtualKeyCode::Escape => b"\x1b",
        VirtualKeyCode::Left => b"\x1b[D",
        VirtualKeyCode::Right => b"\x1b[C",
        VirtualKeyCode::Up => b"\x1b[A",
        VirtualKeyCode::Down => b"\x1b[B",
        _ => return None,
    };

    Some(bytes.to_vec())
}

/// Maps Ctrl+A through Ctrl+Z into ASCII control bytes.
fn ctrl_key_bytes(key: VirtualKeyCode) -> Option<Vec<u8>> {
    let byte = match key {
        VirtualKeyCode::A => 0x01,
        VirtualKeyCode::B => 0x02,
        VirtualKeyCode::C => 0x03,
        VirtualKeyCode::D => 0x04,
        VirtualKeyCode::E => 0x05,
        VirtualKeyCode::F => 0x06,
        VirtualKeyCode::G => 0x07,
        VirtualKeyCode::H => 0x08,
        VirtualKeyCode::I => 0x09,
        VirtualKeyCode::J => 0x0a,
        VirtualKeyCode::K => 0x0b,
        VirtualKeyCode::L => 0x0c,
        VirtualKeyCode::M => 0x0d,
        VirtualKeyCode::N => 0x0e,
        VirtualKeyCode::O => 0x0f,
        VirtualKeyCode::P => 0x10,
        VirtualKeyCode::Q => 0x11,
        VirtualKeyCode::R => 0x12,
        VirtualKeyCode::S => 0x13,
        VirtualKeyCode::T => 0x14,
        VirtualKeyCode::U => 0x15,
        VirtualKeyCode::V => 0x16,
        VirtualKeyCode::W => 0x17,
        VirtualKeyCode::X => 0x18,
        VirtualKeyCode::Y => 0x19,
        VirtualKeyCode::Z => 0x1a,
        _ => return None,
    };

    Some(vec![byte])
}
