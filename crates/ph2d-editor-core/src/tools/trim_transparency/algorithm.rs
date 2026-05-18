//! Trim Transparency — edge-scanning algorithm.
//!
//! Port of the legacy engine's `trimTransparency` (TS/Canvas2D) to
//! pure Rust over RGBA8 buffers. The scan strategy was upgraded:
//! legacy did a single O(W*H) full sweep; here we sweep edges
//! inward, which is O(W + H) for the common case (mostly-transparent
//! borders — exactly the case where Trim is useful) and falls back
//! to O(W*H) only when the image is entirely transparent.
//!
//! No external deps: `std` only. The module is intentionally
//! self-contained so the integration test under `tests/` can pull
//! it in via `#[path]` without depending on the rest of `ph2d-editor`
//! (the surrounding island contract per `INTEGRATION.md`).

/// Non-transparent region of the input image, in original pixel
/// coordinates. `x`/`y` are the top-left corner; `width`/`height`
/// describe its extent. For an entirely-transparent input the
/// returned `bounds` is `{x: 0, y: 0, width: 1, height: 1}` and
/// `pixels` is a single fully-transparent RGBA pixel (legacy parity).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Output of [`trim_transparency`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrimResult {
    /// RGBA8 buffer of the trimmed region. `pixels.len() == width * height * 4`.
    pub pixels: Vec<u8>,
    /// Width of the trimmed region in pixels.
    pub width: u32,
    /// Height of the trimmed region in pixels.
    pub height: u32,
    /// Bounds of the non-transparent region inside the original
    /// image. The caller uses this to reproject metadata (e.g. the
    /// sprite pivot) so the world position stays put after the trim.
    pub bounds: Bounds,
    /// `false` when the bounds equal the full input (i.e. nothing
    /// to trim, the buffer was returned unchanged). The caller
    /// should skip the asset replacement + undo entry in that case.
    pub trimmed: bool,
}

/// Find the tight bounding box of pixels whose alpha exceeds
/// `alpha_threshold` and return a new RGBA8 buffer cropped to it.
///
/// `rgba` must be `width * height * 4` bytes long; mismatch panics.
/// `alpha_threshold = 0` matches the legacy engine behaviour
/// (trim anything that is not fully transparent).
///
/// ## Edge cases
///
/// - **Entirely transparent input** — returns a 1×1 fully-transparent
///   RGBA pixel with `bounds = (0, 0, 1, 1)` and `trimmed = true`.
///   This sentinel matches the legacy port so downstream code does
///   not need a separate "empty" branch when piping results through
///   asset stores that reject zero-sized images.
/// - **`width == 0` or `height == 0`** — degenerate input is treated
///   the same as entirely transparent (the loops do not execute,
///   `found` stays false, the sentinel 1×1 is returned).
/// - **Already-tight input** — `bounds` matches the full image and
///   `trimmed = false`. The returned `pixels` is a fresh copy of the
///   input (we still allocate; callers expecting zero-copy should
///   check `!trimmed` before consuming `pixels`).
///
/// ## Determinism
///
/// Pure integer comparison + byte copy. Bit-identical across all
/// platforms; safe for inclusion in any deterministic asset cooking
/// pipeline (HR-5 implication — this never reads back from GPU).
pub fn trim_transparency(rgba: &[u8], width: u32, height: u32, alpha_threshold: u8) -> TrimResult {
    let w = width as usize;
    let h = height as usize;

    if w == 0 || h == 0 {
        return all_transparent_sentinel();
    }
    assert_eq!(
        rgba.len(),
        w.checked_mul(h)
            .and_then(|n| n.checked_mul(4))
            .expect("rgba dimensions overflow usize"),
        "rgba buffer length must equal width * height * 4",
    );

    // ---- Edge scans ----------------------------------------------------
    // Scan inwards from each edge. The first edge scan also detects the
    // "everything is transparent" case (top scan completes with no hit).

    let mut min_y = h;
    'top: for y in 0..h {
        let row = y * w * 4;
        for x in 0..w {
            if rgba[row + x * 4 + 3] > alpha_threshold {
                min_y = y;
                break 'top;
            }
        }
    }
    if min_y == h {
        return all_transparent_sentinel();
    }

    let mut max_y = min_y;
    'bot: for y in (min_y..h).rev() {
        let row = y * w * 4;
        for x in 0..w {
            if rgba[row + x * 4 + 3] > alpha_threshold {
                max_y = y;
                break 'bot;
            }
        }
    }

    // Horizontal scans only need to look at rows in [min_y, max_y]
    // because we already know rows outside that band are entirely
    // below threshold.
    let mut min_x = w;
    'left: for x in 0..w {
        for y in min_y..=max_y {
            if rgba[y * w * 4 + x * 4 + 3] > alpha_threshold {
                min_x = x;
                break 'left;
            }
        }
    }
    // `min_x` is guaranteed to be set because the top scan found at
    // least one above-threshold pixel inside [min_y, max_y].
    debug_assert!(min_x < w);

    let mut max_x = min_x;
    'right: for x in (min_x..w).rev() {
        for y in min_y..=max_y {
            if rgba[y * w * 4 + x * 4 + 3] > alpha_threshold {
                max_x = x;
                break 'right;
            }
        }
    }

    let bw = max_x - min_x + 1;
    let bh = max_y - min_y + 1;
    let trimmed = !(min_x == 0 && min_y == 0 && bw == w && bh == h);

    // ---- Copy out ------------------------------------------------------
    let mut out = Vec::with_capacity(bw * bh * 4);
    let row_bytes = bw * 4;
    for y in min_y..=max_y {
        let src_start = y * w * 4 + min_x * 4;
        out.extend_from_slice(&rgba[src_start..src_start + row_bytes]);
    }

    TrimResult {
        pixels: out,
        width: bw as u32,
        height: bh as u32,
        bounds: Bounds {
            x: min_x as u32,
            y: min_y as u32,
            width: bw as u32,
            height: bh as u32,
        },
        trimmed,
    }
}

fn all_transparent_sentinel() -> TrimResult {
    TrimResult {
        pixels: vec![0, 0, 0, 0],
        width: 1,
        height: 1,
        bounds: Bounds {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        },
        trimmed: true,
    }
}

// ---------------------------------------------------------------------
// Unit tests — exhaustive coverage of edge cases.
// ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Build an RGBA buffer of `w*h*4` bytes, filled with `(0,0,0,alpha)`.
    fn solid_alpha(w: usize, h: usize, alpha: u8) -> Vec<u8> {
        let mut v = vec![0u8; w * h * 4];
        for i in (3..v.len()).step_by(4) {
            v[i] = alpha;
        }
        v
    }

    /// Put an opaque pixel (`alpha=255`) at `(x, y)` inside an
    /// otherwise-transparent `w*h` buffer.
    fn pixel(w: usize, h: usize, x: usize, y: usize) -> Vec<u8> {
        let mut v = vec![0u8; w * h * 4];
        let i = y * w * 4 + x * 4;
        v[i] = 255;
        v[i + 1] = 255;
        v[i + 2] = 255;
        v[i + 3] = 255;
        v
    }

    #[test]
    fn fully_transparent_returns_1x1_sentinel() {
        let rgba = solid_alpha(8, 8, 0);
        let r = trim_transparency(&rgba, 8, 8, 0);
        assert_eq!(r.width, 1);
        assert_eq!(r.height, 1);
        assert_eq!(
            r.bounds,
            Bounds {
                x: 0,
                y: 0,
                width: 1,
                height: 1
            }
        );
        assert_eq!(r.pixels, vec![0, 0, 0, 0]);
        assert!(r.trimmed);
    }

    #[test]
    fn fully_opaque_is_a_noop() {
        let rgba = solid_alpha(4, 3, 255);
        let r = trim_transparency(&rgba, 4, 3, 0);
        assert_eq!(r.width, 4);
        assert_eq!(r.height, 3);
        assert_eq!(
            r.bounds,
            Bounds {
                x: 0,
                y: 0,
                width: 4,
                height: 3
            }
        );
        assert!(!r.trimmed);
        assert_eq!(r.pixels, rgba);
    }

    #[test]
    fn single_pixel_at_origin_trims_to_1x1_at_0_0() {
        let rgba = pixel(8, 8, 0, 0);
        let r = trim_transparency(&rgba, 8, 8, 0);
        assert_eq!(
            r.bounds,
            Bounds {
                x: 0,
                y: 0,
                width: 1,
                height: 1
            }
        );
        assert_eq!(r.pixels, vec![255, 255, 255, 255]);
        assert!(r.trimmed);
    }

    #[test]
    fn single_pixel_at_corner_trims_to_1x1_at_corner() {
        let rgba = pixel(8, 8, 7, 7);
        let r = trim_transparency(&rgba, 8, 8, 0);
        assert_eq!(
            r.bounds,
            Bounds {
                x: 7,
                y: 7,
                width: 1,
                height: 1
            }
        );
        assert!(r.trimmed);
    }

    #[test]
    fn single_pixel_at_center_trims_to_1x1_at_center() {
        let rgba = pixel(8, 8, 3, 5);
        let r = trim_transparency(&rgba, 8, 8, 0);
        assert_eq!(
            r.bounds,
            Bounds {
                x: 3,
                y: 5,
                width: 1,
                height: 1
            }
        );
        assert!(r.trimmed);
    }

    #[test]
    fn rectangle_off_center_finds_exact_bounds() {
        // 16x16 image with an opaque 4x3 rect at (5,2)..(8,4) inclusive.
        let w = 16;
        let h = 16;
        let mut rgba = vec![0u8; w * h * 4];
        for y in 2..=4 {
            for x in 5..=8 {
                let i = y * w * 4 + x * 4;
                rgba[i] = 200;
                rgba[i + 1] = 100;
                rgba[i + 2] = 50;
                rgba[i + 3] = 255;
            }
        }
        let r = trim_transparency(&rgba, w as u32, h as u32, 0);
        assert_eq!(
            r.bounds,
            Bounds {
                x: 5,
                y: 2,
                width: 4,
                height: 3
            }
        );
        assert_eq!(r.width, 4);
        assert_eq!(r.height, 3);
        assert_eq!(r.pixels.len(), 4 * 3 * 4);
        // Spot-check a known pixel inside the trimmed buffer.
        // Top-left of trimmed = src (5,2) = (200,100,50,255).
        assert_eq!(&r.pixels[0..4], &[200, 100, 50, 255]);
        // Bottom-right of trimmed = src (8,4) — last row last pixel.
        let last = r.pixels.len() - 4;
        assert_eq!(&r.pixels[last..], &[200, 100, 50, 255]);
        assert!(r.trimmed);
    }

    #[test]
    fn threshold_excludes_alpha_at_or_below() {
        // Single pixel with alpha = 10. Threshold 10 → below cut.
        let mut rgba = vec![0u8; 4 * 4 * 4];
        let i = 2 * 4 * 4 + 2 * 4; // pixel (2,2)
        rgba[i + 3] = 10;
        let r = trim_transparency(&rgba, 4, 4, 10);
        // No pixel exceeds threshold → sentinel.
        assert_eq!(
            r.bounds,
            Bounds {
                x: 0,
                y: 0,
                width: 1,
                height: 1
            }
        );
        // Same pixel with threshold 9 → it exceeds → bounds find it.
        let r2 = trim_transparency(&rgba, 4, 4, 9);
        assert_eq!(
            r2.bounds,
            Bounds {
                x: 2,
                y: 2,
                width: 1,
                height: 1
            }
        );
    }

    #[test]
    fn pixels_outside_threshold_are_preserved_inside_bounds() {
        // 4x4. (0,0) has alpha=200 (opaque-ish). (3,3) has alpha=5.
        // With threshold=10, only (0,0) defines bounds → result is 1x1
        // at (0,0). The alpha=5 pixel is not in the trimmed output.
        let mut rgba = vec![0u8; 4 * 4 * 4];
        rgba[3] = 200; // pixel (0,0) alpha
        rgba[(3 * 4 * 4 + 3 * 4) + 3] = 5; // pixel (3,3) alpha
        let r = trim_transparency(&rgba, 4, 4, 10);
        assert_eq!(
            r.bounds,
            Bounds {
                x: 0,
                y: 0,
                width: 1,
                height: 1
            }
        );
        assert_eq!(r.pixels[3], 200);
    }

    /// Byte offset of pixel (x, y) in a row-major RGBA8 buffer of width `w`.
    fn idx(x: usize, y: usize, w: usize) -> usize {
        (y * w + x) * 4
    }

    #[test]
    fn pixels_with_low_alpha_inside_bounds_are_kept_as_is() {
        // Bounds defined by two opaque pixels at (0,0) and (3,3).
        // The pixel at (1,1) has alpha=5. Since (1,1) is inside the
        // bounding box, its bytes must appear unchanged in the result.
        let mut rgba = vec![0u8; 4 * 4 * 4];
        rgba[idx(0, 0, 4) + 3] = 255; // (0,0) alpha
        rgba[idx(3, 3, 4) + 3] = 255; // (3,3) alpha
        let i = idx(1, 1, 4);
        rgba[i] = 99;
        rgba[i + 3] = 5;
        let r = trim_transparency(&rgba, 4, 4, 10);
        assert_eq!(
            r.bounds,
            Bounds {
                x: 0,
                y: 0,
                width: 4,
                height: 4
            }
        );
        assert!(!r.trimmed);
        // (1,1) in result = same offset (1,1) since bounds == full image.
        let ri = idx(1, 1, 4);
        assert_eq!(r.pixels[ri], 99);
        assert_eq!(r.pixels[ri + 3], 5);
    }

    #[test]
    fn zero_width_returns_sentinel() {
        let r = trim_transparency(&[], 0, 5, 0);
        assert_eq!(
            r.bounds,
            Bounds {
                x: 0,
                y: 0,
                width: 1,
                height: 1
            }
        );
        assert!(r.trimmed);
    }

    #[test]
    fn zero_height_returns_sentinel() {
        let r = trim_transparency(&[], 5, 0, 0);
        assert_eq!(
            r.bounds,
            Bounds {
                x: 0,
                y: 0,
                width: 1,
                height: 1
            }
        );
        assert!(r.trimmed);
    }

    #[test]
    #[should_panic(expected = "rgba buffer length must equal")]
    fn buffer_length_mismatch_panics() {
        let rgba = vec![0u8; 3]; // claims 4x4 but only 3 bytes
        let _ = trim_transparency(&rgba, 4, 4, 0);
    }

    #[test]
    fn one_pixel_wide_column_finds_correct_horizontal_bounds() {
        // 8x4. Opaque column at x=5.
        let w = 8;
        let h = 4;
        let mut rgba = vec![0u8; w * h * 4];
        for y in 0..h {
            let i = y * w * 4 + 5 * 4;
            rgba[i + 3] = 255;
        }
        let r = trim_transparency(&rgba, w as u32, h as u32, 0);
        assert_eq!(
            r.bounds,
            Bounds {
                x: 5,
                y: 0,
                width: 1,
                height: 4
            }
        );
    }

    #[test]
    fn one_pixel_tall_row_finds_correct_vertical_bounds() {
        // 6x10. Opaque row at y=7.
        let w = 6;
        let h = 10;
        let mut rgba = vec![0u8; w * h * 4];
        for x in 0..w {
            let i = 7 * w * 4 + x * 4;
            rgba[i + 3] = 255;
        }
        let r = trim_transparency(&rgba, w as u32, h as u32, 0);
        assert_eq!(
            r.bounds,
            Bounds {
                x: 0,
                y: 7,
                width: 6,
                height: 1
            }
        );
    }

    #[test]
    fn pixels_buffer_length_matches_reported_dimensions() {
        // Property-style sanity: any non-trivial input where some
        // pixels are opaque must yield pixels.len() == w*h*4.
        let cases = [
            (8, 8, 3, 3),
            (1, 1, 0, 0),
            (32, 16, 30, 14),
            (16, 32, 1, 25),
        ];
        for (w, h, px, py) in cases {
            let rgba = pixel(w, h, px, py);
            let r = trim_transparency(&rgba, w as u32, h as u32, 0);
            assert_eq!(
                r.pixels.len() as u32,
                r.width * r.height * 4,
                "case w={w} h={h} px={px} py={py}",
            );
        }
    }
}
