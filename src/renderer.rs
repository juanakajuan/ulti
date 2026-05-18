//! Pixel framebuffer renderer for the current terminal screen contents.

use fontdue::Font;
use vt100::Parser;

/// Fixed terminal cell width in pixels.
pub(crate) const CELL_WIDTH: u32 = 9;
/// Fixed terminal cell height in pixels.
pub(crate) const CELL_HEIGHT: u32 = 18;
/// Empty border around the terminal grid in pixels.
pub(crate) const PADDING: u32 = 12;

/// Font rasterization size used for each terminal cell.
const FONT_SIZE: f32 = 15.0;
/// RGBA background color for the terminal surface.
const BACKGROUND: [u8; 4] = [18, 18, 24, 255];
/// RGBA foreground color for rendered glyphs.
const FOREGROUND: [u8; 4] = [225, 229, 238, 255];
/// RGBA fill color for the terminal cursor cell.
const CURSOR: [u8; 4] = [120, 170, 255, 255];

/// Draws the parsed terminal screen into the RGBA framebuffer.
///
/// The frame is cleared first, then each non-empty terminal cell is rasterized
/// with a fixed-width pen advance. The cursor is drawn as a filled cell before
/// the glyph so text remains visible on top of it.
pub(crate) fn draw_terminal(
    frame: &mut [u8],
    width: u32,
    height: u32,
    font: &Font,
    parser: &Parser,
) {
    for pixel in frame.chunks_exact_mut(4) {
        pixel.copy_from_slice(&BACKGROUND);
    }

    let screen = parser.screen();
    let (rows, cols) = screen.size();
    let cursor = screen.cursor_position();
    for row in 0..rows {
        for col in 0..cols {
            if row == cursor.0 && col == cursor.1 {
                fill_rect(
                    frame,
                    width,
                    height,
                    PADDING + u32::from(col) * CELL_WIDTH,
                    PADDING + u32::from(row) * CELL_HEIGHT,
                    CELL_WIDTH,
                    CELL_HEIGHT,
                    CURSOR,
                );
            }

            let Some(cell) = screen.cell(row, col) else {
                continue;
            };
            let contents = cell.contents();
            if contents.trim().is_empty() {
                continue;
            }

            draw_text(
                frame,
                width,
                height,
                font,
                &contents,
                PADDING + u32::from(col) * CELL_WIDTH,
                PADDING + u32::from(row) * CELL_HEIGHT,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
/// Rasterizes UTF-8 text into the framebuffer at a terminal cell origin.
fn draw_text(frame: &mut [u8], width: u32, height: u32, font: &Font, text: &str, x: u32, y: u32) {
    let mut pen_x = x;
    for character in text.chars() {
        let (metrics, bitmap) = font.rasterize(character, FONT_SIZE);
        let glyph_x = i64::from(pen_x) + metrics.xmin as i64;
        let baseline_y = i64::from(y) + 14;
        let glyph_y = baseline_y - metrics.height as i64 - metrics.ymin as i64;

        for glyph_row in 0..metrics.height {
            for glyph_col in 0..metrics.width {
                let alpha = bitmap[glyph_row * metrics.width + glyph_col];
                if alpha == 0 {
                    continue;
                }

                let pixel_x = glyph_x + glyph_col as i64;
                let pixel_y = glyph_y + glyph_row as i64;
                if pixel_x < 0
                    || pixel_y < 0
                    || pixel_x >= i64::from(width)
                    || pixel_y >= i64::from(height)
                {
                    continue;
                }

                let offset = ((pixel_y as u32 * width + pixel_x as u32) * 4) as usize;
                blend_pixel(&mut frame[offset..offset + 4], FOREGROUND, alpha);
            }
        }

        pen_x += CELL_WIDTH;
    }
}

#[allow(clippy::too_many_arguments)]
/// Fills a clipped rectangle in the framebuffer.
fn fill_rect(
    frame: &mut [u8],
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    rect_width: u32,
    rect_height: u32,
    color: [u8; 4],
) {
    let max_y = (y + rect_height).min(height);
    let max_x = (x + rect_width).min(width);
    for pixel_y in y..max_y {
        for pixel_x in x..max_x {
            let offset = ((pixel_y * width + pixel_x) * 4) as usize;
            frame[offset..offset + 4].copy_from_slice(&color);
        }
    }
}

/// Alpha-blends one foreground color into an existing RGBA framebuffer pixel.
fn blend_pixel(pixel: &mut [u8], color: [u8; 4], alpha: u8) {
    let alpha = u16::from(alpha);
    let inverse_alpha = 255 - alpha;
    pixel[0] = ((u16::from(color[0]) * alpha + u16::from(pixel[0]) * inverse_alpha) / 255) as u8;
    pixel[1] = ((u16::from(color[1]) * alpha + u16::from(pixel[1]) * inverse_alpha) / 255) as u8;
    pixel[2] = ((u16::from(color[2]) * alpha + u16::from(pixel[2]) * inverse_alpha) / 255) as u8;
    pixel[3] = 255;
}
