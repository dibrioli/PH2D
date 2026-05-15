//! Make Square — symmetric transparent-pad to square.
//!
//! Pads an RGBA8 image on its shorter axis with fully-transparent
//! pixels (RGBA = `(0, 0, 0, 0)`) so the resulting buffer is a perfect
//! square. The original pixels are copied verbatim into the centered
//! sub-region of the output; nothing is cropped, resampled, or
//! colour-shifted.
//!
//! The pad band is split as evenly as possible: for an odd-sized
//! difference the leading edge (top or left) gets `floor(diff / 2)`
//! and the trailing edge (bottom or right) gets `ceil(diff / 2)`.
//! This matches the ImageMagick `-gravity Center -extent` convention
//! and CSS centered-background, so callers can predict where the
//! original lands.
//!
//! No external deps: `std` only. The module is intentionally
//! self-contained so the integration test under `tests/` can pull it
//! in via `#[path]` without depending on the rest of `ph2d-editor`
//! (same island contract as `trim_transparency` — see
//! `INTEGRATION.md`).

/// Output of [`make_square`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MakeSquareResult {
    /// RGBA8 buffer of the square output. `pixels.len() == size * size * 4`.
    pub pixels: Vec<u8>,
    /// Side length of the square output in pixels (`max(width, height)`
    /// for normal inputs; `1` for the degenerate-input sentinel).
    pub size: u32,
    /// Column where the original image's left edge lands inside the
    /// new canvas. The caller uses this to reproject sprite metadata
    /// (e.g. the pivot) so the world position stays put after the
    /// canvas grows.
    pub offset_x: u32,
    /// Row where the original image's top edge lands inside the new
    /// canvas.
    pub offset_y: u32,
    /// `false` when the input was already square (the buffer was
    /// returned unchanged). The caller should skip the asset
    /// replacement + undo entry in that case.
    pub made_square: bool,
}

/// Pad an RGBA8 image with fully-transparent pixels on its shorter
/// axis so the result is a perfect square. Source pixels are copied
/// verbatim into the centered sub-region of the new canvas; nothing
/// is cropped, resampled, or colour-shifted.
///
/// `rgba` must be `width * height * 4` bytes long; mismatch panics.
///
/// ## Centering convention
///
/// For an odd-sized pad band, the leading edge (top or left) receives
/// `floor(diff / 2)` and the trailing edge (bottom or right) gets the
/// remainder. Example: a 5×2 input becomes 5×5 with `offset_y = 1`
/// (1 transparent row above the original, 2 below).
///
/// ## Edge cases
///
/// - **Already square** (`width == height`) — returns a fresh copy of
///   the input with `offset_x = offset_y = 0` and `made_square = false`.
///   Callers expecting zero-copy should check `!made_square` before
///   consuming `pixels`.
/// - **`width == 0` or `height == 0`** — degenerate input is treated as
///   transparent: returns a 1×1 fully-transparent RGBA pixel with
///   `made_square = true`, matching `trim_transparency`'s sentinel
///   contract so downstream asset stores don't need a separate
///   zero-area branch.
///
/// ## Determinism
///
/// Pure integer arithmetic + byte copy. Bit-identical across all
/// platforms; safe for inclusion in any deterministic asset cooking
/// pipeline (HR-5 implication — never reads back from GPU).
/// Allocation is fine here: this action is dispatched from a user
/// click, not from a HR-3 hot path (render / physics / audio /
/// editor_layout).
pub fn make_square(rgba: &[u8], width: u32, height: u32) -> MakeSquareResult {
    let w = width as usize;
    let h = height as usize;

    if w == 0 || h == 0 {
        return degenerate_sentinel();
    }
    assert_eq!(
        rgba.len(),
        w.checked_mul(h)
            .and_then(|n| n.checked_mul(4))
            .expect("rgba dimensions overflow usize"),
        "rgba buffer length must equal width * height * 4",
    );

    if w == h {
        return MakeSquareResult {
            pixels: rgba.to_vec(),
            size: width,
            offset_x: 0,
            offset_y: 0,
            made_square: false,
        };
    }

    let size = w.max(h);
    let offset_x = (size - w) / 2;
    let offset_y = (size - h) / 2;

    let mut out = vec![0u8; size * size * 4];
    let row_bytes_in = w * 4;
    let stride_out = size * 4;
    for y in 0..h {
        let src = y * row_bytes_in;
        let dst = (y + offset_y) * stride_out + offset_x * 4;
        out[dst..dst + row_bytes_in].copy_from_slice(&rgba[src..src + row_bytes_in]);
    }

    MakeSquareResult {
        pixels: out,
        size: size as u32,
        offset_x: offset_x as u32,
        offset_y: offset_y as u32,
        made_square: true,
    }
}

fn degenerate_sentinel() -> MakeSquareResult {
    MakeSquareResult {
        pixels: vec![0, 0, 0, 0],
        size: 1,
        offset_x: 0,
        offset_y: 0,
        made_square: true,
    }
}

// ---------------------------------------------------------------------
// Unit tests — exhaustive coverage of edge cases.
// ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Byte offset of pixel (x, y) in a row-major RGBA8 buffer of width `w`.
    fn idx(x: usize, y: usize, w: usize) -> usize {
        (y * w + x) * 4
    }

    /// Build an RGBA buffer of `w*h*4` bytes with one opaque pixel
    /// `(255, 255, 255, 255)` at `(x, y)`.
    fn pixel(w: usize, h: usize, x: usize, y: usize) -> Vec<u8> {
        let mut v = vec![0u8; w * h * 4];
        let i = idx(x, y, w);
        v[i] = 255;
        v[i + 1] = 255;
        v[i + 2] = 255;
        v[i + 3] = 255;
        v
    }

    /// Fill every pixel's alpha to 255 to mark an entire buffer as opaque.
    fn opaque(w: usize, h: usize) -> Vec<u8> {
        let mut v = vec![0u8; w * h * 4];
        for i in (3..v.len()).step_by(4) {
            v[i] = 255;
        }
        v
    }

    #[test]
    fn already_square_returns_unchanged_copy_with_made_square_false() {
        let rgba = pixel(4, 4, 1, 2);
        let r = make_square(&rgba, 4, 4);
        assert_eq!(r.size, 4);
        assert_eq!(r.offset_x, 0);
        assert_eq!(r.offset_y, 0);
        assert!(!r.made_square);
        assert_eq!(r.pixels, rgba);
    }

    #[test]
    fn wider_than_tall_pads_top_and_bottom_evenly() {
        // 4×2 → 4×4, diff_y = 2 → 1 row above, 1 row below.
        let rgba = opaque(4, 2);
        let r = make_square(&rgba, 4, 2);
        assert_eq!(r.size, 4);
        assert_eq!(r.offset_x, 0);
        assert_eq!(r.offset_y, 1);
        assert!(r.made_square);
        assert_eq!(r.pixels.len(), 4 * 4 * 4);

        for x in 0..4 {
            assert_eq!(r.pixels[idx(x, 0, 4) + 3], 0, "pad row 0 x={x}");
            assert_eq!(r.pixels[idx(x, 3, 4) + 3], 0, "pad row 3 x={x}");
        }
        for y in 1..=2 {
            for x in 0..4 {
                assert_eq!(r.pixels[idx(x, y, 4) + 3], 255, "src row y={y} x={x}");
            }
        }
    }

    #[test]
    fn taller_than_wide_pads_left_and_right_evenly() {
        let rgba = opaque(2, 4);
        let r = make_square(&rgba, 2, 4);
        assert_eq!(r.size, 4);
        assert_eq!(r.offset_x, 1);
        assert_eq!(r.offset_y, 0);
        assert!(r.made_square);

        for y in 0..4 {
            assert_eq!(r.pixels[idx(0, y, 4) + 3], 0, "pad col 0 y={y}");
            assert_eq!(r.pixels[idx(3, y, 4) + 3], 0, "pad col 3 y={y}");
            for x in 1..=2 {
                assert_eq!(r.pixels[idx(x, y, 4) + 3], 255, "src col x={x} y={y}");
            }
        }
    }

    #[test]
    fn odd_vertical_diff_floors_offset_on_top_edge() {
        // 5×2 → 5×5, diff_y = 3. Leading edge floors → offset_y = 1.
        // Row 0 transparent, rows 1..=2 source, rows 3..=4 transparent.
        let rgba = opaque(5, 2);
        let r = make_square(&rgba, 5, 2);
        assert_eq!(r.size, 5);
        assert_eq!(r.offset_x, 0);
        assert_eq!(r.offset_y, 1, "floor(3/2) = 1 on the leading (top) edge");
        for x in 0..5 {
            assert_eq!(r.pixels[idx(x, 0, 5) + 3], 0);
        }
        for y in 1..=2 {
            for x in 0..5 {
                assert_eq!(r.pixels[idx(x, y, 5) + 3], 255);
            }
        }
        for y in 3..=4 {
            for x in 0..5 {
                assert_eq!(r.pixels[idx(x, y, 5) + 3], 0);
            }
        }
    }

    #[test]
    fn odd_horizontal_diff_floors_offset_on_left_edge() {
        let rgba = opaque(2, 5);
        let r = make_square(&rgba, 2, 5);
        assert_eq!(r.size, 5);
        assert_eq!(r.offset_x, 1, "floor(3/2) = 1 on the leading (left) edge");
        assert_eq!(r.offset_y, 0);
        for y in 0..5 {
            assert_eq!(r.pixels[idx(0, y, 5) + 3], 0);
            for x in 1..=2 {
                assert_eq!(r.pixels[idx(x, y, 5) + 3], 255);
            }
            for x in 3..=4 {
                assert_eq!(r.pixels[idx(x, y, 5) + 3], 0);
            }
        }
    }

    #[test]
    fn pixel_colors_are_preserved_byte_for_byte() {
        // Unique colour per pixel; verify every source pixel lands at
        // (x + offset_x, y + offset_y) with identical RGBA bytes.
        let w = 2usize;
        let h = 4usize;
        let mut rgba = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let i = idx(x, y, w);
                rgba[i] = 10 + (x as u8) * 20;
                rgba[i + 1] = 30 + (y as u8) * 10;
                rgba[i + 2] = 70;
                rgba[i + 3] = 200;
            }
        }
        let r = make_square(&rgba, w as u32, h as u32);
        assert_eq!(r.size, 4);
        assert_eq!(r.offset_x, 1);
        assert_eq!(r.offset_y, 0);
        for y in 0..h {
            for x in 0..w {
                let s = idx(x, y, w);
                let d = idx(
                    x + r.offset_x as usize,
                    y + r.offset_y as usize,
                    r.size as usize,
                );
                assert_eq!(
                    &r.pixels[d..d + 4],
                    &rgba[s..s + 4],
                    "mismatch at src ({x},{y})",
                );
            }
        }
    }

    #[test]
    fn padded_region_bytes_are_fully_zero_not_just_alpha() {
        // Source: opaque red 3×5. After pad to 5×5, columns 0 and 4
        // must be `(0, 0, 0, 0)` — every byte, not just alpha.
        let w = 3;
        let h = 5;
        let mut rgba = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let i = idx(x, y, w);
                rgba[i] = 255;
                rgba[i + 3] = 255;
            }
        }
        let r = make_square(&rgba, w as u32, h as u32);
        let s = r.size as usize;
        for y in 0..s {
            for x in [0usize, 4] {
                let i = idx(x, y, s);
                assert_eq!(&r.pixels[i..i + 4], &[0, 0, 0, 0]);
            }
        }
    }

    #[test]
    fn one_by_one_is_already_square_noop() {
        let rgba = pixel(1, 1, 0, 0);
        let r = make_square(&rgba, 1, 1);
        assert_eq!(r.size, 1);
        assert!(!r.made_square);
        assert_eq!(r.pixels, rgba);
    }

    #[test]
    fn thin_tall_strip_pads_to_square_along_short_axis() {
        // 1×4 → 4×4 with offset_x = floor(3/2) = 1.
        let rgba = pixel(1, 4, 0, 2);
        let r = make_square(&rgba, 1, 4);
        assert_eq!(r.size, 4);
        assert_eq!(r.offset_x, 1);
        assert_eq!(r.offset_y, 0);
        let i = idx(1, 2, 4);
        assert_eq!(&r.pixels[i..i + 4], &[255, 255, 255, 255]);
    }

    #[test]
    fn idempotent_after_first_application() {
        // Running make_square on its own output is a no-op (the second
        // call sees a square buffer and short-circuits).
        let rgba = opaque(7, 3);
        let r1 = make_square(&rgba, 7, 3);
        assert!(r1.made_square);
        let r2 = make_square(&r1.pixels, r1.size, r1.size);
        assert!(!r2.made_square);
        assert_eq!(r2.pixels, r1.pixels);
    }

    #[test]
    fn zero_width_returns_sentinel() {
        let r = make_square(&[], 0, 5);
        assert_eq!(r.size, 1);
        assert_eq!(r.pixels, vec![0, 0, 0, 0]);
        assert!(r.made_square);
    }

    #[test]
    fn zero_height_returns_sentinel() {
        let r = make_square(&[], 5, 0);
        assert_eq!(r.size, 1);
        assert_eq!(r.pixels, vec![0, 0, 0, 0]);
        assert!(r.made_square);
    }

    #[test]
    fn zero_by_zero_returns_sentinel() {
        let r = make_square(&[], 0, 0);
        assert_eq!(r.size, 1);
        assert!(r.made_square);
    }

    #[test]
    #[should_panic(expected = "rgba buffer length must equal")]
    fn buffer_length_mismatch_panics() {
        let rgba = vec![0u8; 3];
        let _ = make_square(&rgba, 4, 4);
    }

    #[test]
    fn output_pixels_buffer_length_always_matches_size_squared() {
        let cases = [(8, 8), (1, 1), (32, 16), (16, 32), (5, 1), (1, 5), (7, 3)];
        for (w, h) in cases {
            let rgba = vec![0u8; w * h * 4];
            let r = make_square(&rgba, w as u32, h as u32);
            assert_eq!(
                r.pixels.len() as u32,
                r.size * r.size * 4,
                "case w={w} h={h}",
            );
        }
    }
}
