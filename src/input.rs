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

    let sequence: &[u8] = match key {
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

    Some(sequence.to_vec())
}

/// Maps Ctrl+A through Ctrl+Z into ASCII control bytes.
fn ctrl_key_bytes(key: VirtualKeyCode) -> Option<Vec<u8>> {
    ctrl_letter_byte(key).map(|byte| vec![byte])
}

/// Converts letter keys to their Ctrl-modified ASCII control byte.
fn ctrl_letter_byte(key: VirtualKeyCode) -> Option<u8> {
    use VirtualKeyCode::{
        A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    };

    const CTRL_LETTERS: [VirtualKeyCode; 26] = [
        A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    ];

    CTRL_LETTERS
        .iter()
        .position(|&letter| letter == key)
        .and_then(|index| u8::try_from(index + 1).ok())
}
