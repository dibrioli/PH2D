//! Padding — pure canvas-resize logic.
//!
//! `std`-only, no editor/ECS coupling: consumes straight-alpha RGBA8 +
//! a signed [`PaddingSpec`] and returns a fresh RGBA8 buffer plus the
//! pixel offset of the original content inside the new canvas. Port of
//! the legacy `addPadding` (Game-Engine-Legada/.../image/padding.ts),
//! which also subsumes `directionalExpand` (single-edge case).

/// Signed per-edge padding, in pixels. Positive = expand that edge with
/// fully-transparent pixels; negative = crop that many pixels off the
/// edge. A directional-expand drag edits exactly one field.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct PaddingSpec {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

impl PaddingSpec {
    /// Uniform padding on all four edges.
    pub fn uniform(all: i32) -> Self {
        Self {
            top: all,
            right: all,
            bottom: all,
            left: all,
        }
    }

    /// True when every edge is zero (caller can skip the bake + undo entry).
    pub fn is_noop(self) -> bool {
        self.top == 0 && self.right == 0 && self.bottom == 0 && self.left == 0
    }
}

/// Output of [`add_padding`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaddingResult {
    /// RGBA8 buffer of the resized canvas. `pixels.len() == width * height * 4`.
    pub pixels: Vec<u8>,
    /// New canvas dimensions (each clamped to ≥ 1).
    pub width: u32,
    pub height: u32,
    /// Pixel shift of the original content's top-left inside the new
    /// canvas (`padLeft - cropLeft`, `padTop - cropTop`). The caller uses
    /// this to reproject the sprite pivot so the world position holds.
    pub pivot_delta_x: i32,
    pub pivot_delta_y: i32,
    /// `false` when the spec was a no-op / produced an identical canvas
    /// (caller skips the asset swap + undo entry).
    pub changed: bool,
}

/// Resize `rgba` (straight-alpha RGBA8, length `w*h*4`) by a signed
/// per-edge `spec`. Positive edges add transparent pixels; negative edges
/// crop. Crops are clamped so the content stays ≥ 1px on each axis.
///
/// Faithful port of the legacy `addPadding` (`packages/tools/.../padding.ts`),
/// which also subsumes `directionalExpand` (drag a single edge → one field).
/// Pure and deterministic (HR-5): no allocation beyond the output buffer,
/// no floating point, in-bounds by construction.
///
/// # Algorithm (mirrors `padding.ts`)
/// 1. Split each signed edge into pad (≥0) vs crop (the negated negative).
/// 2. Clamp crops so `contentW/H = max(1, src − cropNear − cropFar)`.
/// 3. `newW = max(1, contentW + padLeft + padRight)` (same for H).
/// 4. Allocate `newW*newH*4` transparent (all-zero) RGBA8; copy the
///    cropped content rect into it at `(padLeft, padTop)`.
/// 5. `pivot_delta = (padLeft − cropLeft, padTop − cropTop)`.
/// 6. `changed = spec is not a no-op AND dims/content actually moved`.
///
/// Pin with tests: pure expand (transparent border), pure crop, mixed
/// expand+crop on opposite edges, over-crop clamp to 1px, no-op spec.
pub fn add_padding(rgba: &[u8], w: u32, h: u32, spec: PaddingSpec) -> PaddingResult {
    // i64 throughout the geometry math: source dims are u32 and the signed
    // pads can be large, so this stays clear of overflow before the final
    // clamp back to u32. (Mirrors the legacy `addPadding`, which works in
    // JS doubles.)
    let src_w = w as i64;
    let src_h = h as i64;

    // 1. Split each signed edge into pad (≥ 0) vs crop (the negated negative).
    let (pad_left, mut crop_left) = split_edge(spec.left);
    let (pad_right, mut crop_right) = split_edge(spec.right);
    let (pad_top, mut crop_top) = split_edge(spec.top);
    let (pad_bottom, mut crop_bottom) = split_edge(spec.bottom);

    // 2. Clamp crops so opposite edges leave at least 1px of content (legacy
    //    order: each clamp sees the already-updated near edge). The trailing
    //    `.clamp(0, max)` guards the pathological double-over-crop case the
    //    JS version left negative — keeping the copy rect provably in-bounds.
    let max_crop_x = (src_w - 1).max(0);
    let max_crop_y = (src_h - 1).max(0);
    crop_left = crop_left.min(src_w - 1 - crop_right).clamp(0, max_crop_x);
    crop_right = crop_right.min(src_w - 1 - crop_left).clamp(0, max_crop_x);
    crop_top = crop_top.min(src_h - 1 - crop_bottom).clamp(0, max_crop_y);
    crop_bottom = crop_bottom.min(src_h - 1 - crop_top).clamp(0, max_crop_y);

    // 3. Content rect (≥ 1px each axis) and the resized canvas dimensions.
    let content_w = (src_w - crop_left - crop_right).max(1);
    let content_h = (src_h - crop_top - crop_bottom).max(1);
    let new_w_i = (content_w + pad_left + pad_right).max(1);
    let new_h_i = (content_h + pad_top + pad_bottom).max(1);

    let new_w = new_w_i as u32;
    let new_h = new_h_i as u32;

    // 4. Allocate the new canvas as fully-transparent (all-zero) RGBA8 and
    //    blit the cropped source rect into it at (pad_left, pad_top). The
    //    copyable span is the overlap of the content rect with both the
    //    source and the destination — bounds-checked so any degenerate spec
    //    (or a 0×0 source) simply copies nothing instead of panicking.
    let mut pixels = vec![0u8; (new_w_i * new_h_i * 4) as usize];

    if src_w > 0 && src_h > 0 {
        let src_stride = (src_w * 4) as usize;
        let dst_stride = (new_w_i * 4) as usize;

        let span_w = content_w
            .min(src_w - crop_left)
            .min(new_w_i - pad_left)
            .max(0);
        let span_h = content_h
            .min(src_h - crop_top)
            .min(new_h_i - pad_top)
            .max(0);

        for row in 0..span_h {
            let sy = (crop_top + row) as usize;
            let dy = (pad_top + row) as usize;
            let src_off = sy * src_stride + (crop_left as usize) * 4;
            let dst_off = dy * dst_stride + (pad_left as usize) * 4;
            let bytes = (span_w * 4) as usize;
            pixels[dst_off..dst_off + bytes].copy_from_slice(&rgba[src_off..src_off + bytes]);
        }
    }

    // 5. Pixel shift of the original content's top-left inside the new canvas.
    let pivot_delta_x = (pad_left - crop_left) as i32;
    let pivot_delta_y = (pad_top - crop_top) as i32;

    // 6. `changed` iff the canvas resized or the content moved within it
    //    (a no-op spec lands here as same-dims + zero pivot delta).
    let changed = new_w != w || new_h != h || pivot_delta_x != 0 || pivot_delta_y != 0;

    PaddingResult {
        pixels,
        width: new_w,
        height: new_h,
        pivot_delta_x,
        pivot_delta_y,
        changed,
    }
}

/// Split one signed edge value into `(pad, crop)`: a positive value is pad
/// with zero crop, a negative value is crop (the negated magnitude) with
/// zero pad.
fn split_edge(edge: i32) -> (i64, i64) {
    let e = edge as i64;
    if e < 0 { (0, -e) } else { (e, 0) }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `w`×`h` canvas where every pixel is the opaque solid `[r,g,b,255]`.
    fn solid(w: u32, h: u32, rgb: [u8; 3]) -> Vec<u8> {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..(w * h) {
            v.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
        v
    }

    /// Read the RGBA pixel at `(x, y)` from a `w`-wide RGBA8 buffer.
    fn px(buf: &[u8], w: u32, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * w + x) * 4) as usize;
        [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
    }

    /// Overwrite the RGBA pixel at `(x, y)` in a `w`-wide RGBA8 buffer.
    fn set_px(buf: &mut [u8], w: u32, x: u32, y: u32, rgba: [u8; 4]) {
        let i = ((y * w + x) * 4) as usize;
        buf[i..i + 4].copy_from_slice(&rgba);
    }

    #[test]
    fn noop_spec_returns_identical_canvas_unchanged() {
        let src = solid(4, 3, [10, 20, 30]);
        let r = add_padding(&src, 4, 3, PaddingSpec::default());
        assert_eq!((r.width, r.height), (4, 3));
        assert_eq!(r.pixels, src);
        assert_eq!((r.pivot_delta_x, r.pivot_delta_y), (0, 0));
        assert!(!r.changed);
    }

    #[test]
    fn pure_expand_adds_transparent_border_and_offsets_content() {
        // +2 left, +1 top → content shifts to (2,1); canvas grows by those.
        let src = solid(2, 2, [255, 0, 0]);
        let spec = PaddingSpec {
            top: 1,
            right: 0,
            bottom: 0,
            left: 2,
        };
        let r = add_padding(&src, 2, 2, spec);

        assert_eq!((r.width, r.height), (4, 3));
        assert_eq!((r.pivot_delta_x, r.pivot_delta_y), (2, 1));
        assert!(r.changed);

        // Content sits at (2,1)..(4,3); everything else is transparent zeros.
        assert_eq!(px(&r.pixels, r.width, 2, 1), [255, 0, 0, 255]);
        assert_eq!(px(&r.pixels, r.width, 3, 2), [255, 0, 0, 255]);
        assert_eq!(px(&r.pixels, r.width, 0, 0), [0, 0, 0, 0]);
        assert_eq!(px(&r.pixels, r.width, 1, 1), [0, 0, 0, 0]); // left border
        assert_eq!(px(&r.pixels, r.width, 2, 0), [0, 0, 0, 0]); // top border
    }

    #[test]
    fn pure_crop_trims_edges_and_reports_negative_pivot() {
        // 4×4 with a distinct top-left marker; crop 1px off left + top.
        let mut src = solid(4, 4, [0, 0, 255]);
        // mark pixel (1,1) — becomes the new top-left after the crop.
        set_px(&mut src, 4, 1, 1, [9, 8, 7, 255]);
        let spec = PaddingSpec {
            top: -1,
            right: 0,
            bottom: -1,
            left: -1,
        };
        let r = add_padding(&src, 4, 4, spec);

        // contentW = 4 − cropLeft(1) − cropRight(0) = 3
        // contentH = 4 − cropTop(1) − cropBottom(1) = 2
        assert_eq!((r.width, r.height), (3, 2));
        assert_eq!((r.pivot_delta_x, r.pivot_delta_y), (-1, -1));
        assert!(r.changed);
        // Old (1,1) marker is now at the new origin (0,0).
        assert_eq!(px(&r.pixels, r.width, 0, 0), [9, 8, 7, 255]);
    }

    #[test]
    fn expand_one_edge_crop_opposite_keeps_width_but_moves_content() {
        // left +1 (pad), right -1 (crop): width unchanged, content shifts +1.
        let src = solid(3, 1, [100, 100, 100]);
        let spec = PaddingSpec {
            top: 0,
            right: -1,
            bottom: 0,
            left: 1,
        };
        let r = add_padding(&src, 3, 1, spec);

        // contentW = 3 - 0(cropLeft) - 1(cropRight) = 2; newW = 2 + 1 + 0 = 3.
        assert_eq!((r.width, r.height), (3, 1));
        assert_eq!((r.pivot_delta_x, r.pivot_delta_y), (1, 0));
        // canvas same size but content moved → changed.
        assert!(r.changed);
        assert_eq!(px(&r.pixels, r.width, 0, 0), [0, 0, 0, 0]); // new transparent col
        assert_eq!(px(&r.pixels, r.width, 1, 0), [100, 100, 100, 255]);
        assert_eq!(px(&r.pixels, r.width, 2, 0), [100, 100, 100, 255]);
    }

    #[test]
    fn over_crop_clamps_content_to_one_pixel_per_axis() {
        // Crop far more than the source on every edge → content clamps to 1×1.
        let src = solid(5, 5, [1, 2, 3]);
        let spec = PaddingSpec {
            top: -100,
            right: -100,
            bottom: -100,
            left: -100,
        };
        let r = add_padding(&src, 5, 5, spec);

        assert_eq!((r.width, r.height), (1, 1));
        assert!(r.changed);
        // The single surviving pixel is a real source pixel (opaque), not a
        // transparent hole — the copy rect stayed in bounds.
        assert_eq!(px(&r.pixels, 1, 0, 0), [1, 2, 3, 255]);
    }

    #[test]
    fn output_buffer_length_matches_reported_dimensions() {
        let src = solid(6, 4, [0, 0, 0]);
        let spec = PaddingSpec {
            top: 3,
            right: -2,
            bottom: 1,
            left: 5,
        };
        let r = add_padding(&src, 6, 4, spec);
        assert_eq!(r.pixels.len(), (r.width * r.height * 4) as usize);
    }
}
