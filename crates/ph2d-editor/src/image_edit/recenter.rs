//! Pivot-preserving recenter after a crop-style image edit.
//!
//! PH2D sprites are center-anchored: a sprite's
//! [`ph2d_ecs::Transform::translation`] sits at the visual center of
//! the rendered quad. When an image edit crops to a tight bounding box
//! (Trim Transparency today; future Crop, Auto-Trim, Padding-strip),
//! the surviving opaque content may have lived off-center inside the
//! original frame. Re-spawning the cropped sprite at the same
//! translation would visibly shift the content. This module computes
//! the corrective translation delta so the post-crop sprite center
//! coincides with the pre-crop visual position of the surviving pixels.
//!
//! ## Coordinate spaces
//!
//! - **World space**: Y-up, meters. `Transform.translation`.
//! - **Pixel space**: Y-down, origin at the texture's top-left.
//!   [`PixelBounds`] lives here (mirrors how `image` and Vello deliver
//!   raster data — see SKILL §11.1 conventions table).
//!
//! The Y-flip happens at the boundary between the two spaces inside
//! [`recenter_after_crop`] so callers stay in their natural space.
//!
//! ## Formula
//!
//! Let `T` be the old translation, `S` the old world size, `P` the old
//! pixel size, `b` the bounds. The crop's center in pixel space is
//! `(b.x + b.w/2, b.y + b.h/2)`. Converting to a signed offset relative
//! to the sprite center and flipping Y on the way out of pixel space:
//!
//! ```text
//! Δx = S.x * ((b.x + b.w/2) / P.x - 0.5)
//! Δy = S.y * (0.5 - (b.y + b.h/2) / P.y)    // Y-up flip
//! ```
//!
//! New translation = `T + (Δx, Δy)`.
//!
//! HR-5: pure f32 arithmetic in a fixed evaluation order — no FMA, no
//! SIMD reordering — so the result is bit-deterministic across
//! platforms even though background removal isn't simulation state.

/// Bounding box of the surviving content in pixel space (Y-down,
/// origin at the texture's top-left). Equivalent in shape to the
/// `Bounds` returned by
/// [`crate::tools::trim_transparency`](crate::tools::trim_transparency),
/// kept as its own type here so callers from other image-edit tools
/// (Background Removal post-trim, future Crop) share this geometry
/// without depending on `tools::trim_transparency`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PixelBounds {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl PixelBounds {
    /// Convenience for callers holding the `tools::trim_transparency::Bounds`
    /// type. The two shapes are identical so the cast is a copy.
    pub fn from_trim(b: crate::tools::trim_transparency::Bounds) -> Self {
        Self {
            x: b.x,
            y: b.y,
            width: b.width,
            height: b.height,
        }
    }
}

/// Compute the new world-space translation a sprite needs after a
/// crop-style image edit so the visual position of the surviving
/// content is preserved.
///
/// - `old_translation` — the entity's current `Transform.translation`
///   in world meters (Y-up).
/// - `old_size_world` — the entity's `Sprite.size` before the edit
///   (world meters, `[width, height]`).
/// - `old_size_px` — pixel dimensions of the source image that was
///   cropped (the texture the sprite was reading from, before the
///   trim ran).
/// - `bounds` — pixel-space bounding box of the surviving content
///   inside `old_size_px`.
///
/// Returns the new translation. Degenerate inputs (`old_size_px`
/// with a zero component) short-circuit to `old_translation` —
/// nothing to recenter against.
///
/// HR-5: deterministic f32 — no FMA, no reordering. HR-3: zero alloc.
pub fn recenter_after_crop(
    old_translation: [f32; 2],
    old_size_world: [f32; 2],
    old_size_px: [u32; 2],
    bounds: PixelBounds,
) -> [f32; 2] {
    if old_size_px[0] == 0 || old_size_px[1] == 0 {
        return old_translation;
    }
    let px_w = old_size_px[0] as f32;
    let px_h = old_size_px[1] as f32;
    let center_px_x = bounds.x as f32 + bounds.width as f32 * 0.5;
    let center_px_y = bounds.y as f32 + bounds.height as f32 * 0.5;
    let dx = old_size_world[0] * (center_px_x / px_w - 0.5);
    // Y-up flip: a positive `center_px_y` (toward bottom of texture)
    // corresponds to a NEGATIVE world Y offset.
    let dy = old_size_world[1] * (0.5 - center_px_y / px_h);
    [old_translation[0] + dx, old_translation[1] + dy]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: [f32; 2], b: [f32; 2], eps: f32) -> bool {
        (a[0] - b[0]).abs() < eps && (a[1] - b[1]).abs() < eps
    }

    #[test]
    fn full_image_bounds_is_noop() {
        // Bounds match the whole sprite → center is the same as
        // sprite center → no delta.
        let out = recenter_after_crop(
            [10.0, 5.0],
            [4.0, 2.0],
            [200, 100],
            PixelBounds {
                x: 0,
                y: 0,
                width: 200,
                height: 100,
            },
        );
        assert!(approx(out, [10.0, 5.0], 1e-6));
    }

    #[test]
    fn centered_bounds_is_noop() {
        // 50×50 box at the exact center of a 200×200 sprite stays at
        // the sprite's translation.
        let out = recenter_after_crop(
            [0.0, 0.0],
            [2.0, 2.0],
            [200, 200],
            PixelBounds {
                x: 75,
                y: 75,
                width: 50,
                height: 50,
            },
        );
        assert!(approx(out, [0.0, 0.0], 1e-6));
    }

    #[test]
    fn top_left_bounds_shifts_left_and_up() {
        // 100×100 box at the top-left quadrant of a 200×200 / 4×4m sprite.
        // Center of bounds in px = (50, 50); sprite center = (100, 100).
        // Δpx = (-50, -50). Δworld = (4 * (50/200 - 0.5), 4 * (0.5 - 50/200))
        //                          = (4 * -0.25, 4 * 0.25)
        //                          = (-1.0, +1.0).  (+Y world = up).
        let out = recenter_after_crop(
            [10.0, 20.0],
            [4.0, 4.0],
            [200, 200],
            PixelBounds {
                x: 0,
                y: 0,
                width: 100,
                height: 100,
            },
        );
        assert!(approx(out, [9.0, 21.0], 1e-5));
    }

    #[test]
    fn bottom_right_bounds_shifts_right_and_down() {
        // 100×100 box at the bottom-right quadrant.
        // Δworld = (+1.0, -1.0).
        let out = recenter_after_crop(
            [10.0, 20.0],
            [4.0, 4.0],
            [200, 200],
            PixelBounds {
                x: 100,
                y: 100,
                width: 100,
                height: 100,
            },
        );
        assert!(approx(out, [11.0, 19.0], 1e-5));
    }

    #[test]
    fn right_half_only_x_changes() {
        // Bounds covers the full height but only the right half — Y
        // delta must stay zero, X delta = +size.x / 4.
        let out = recenter_after_crop(
            [0.0, 0.0],
            [4.0, 4.0],
            [200, 200],
            PixelBounds {
                x: 100,
                y: 0,
                width: 100,
                height: 200,
            },
        );
        assert!(approx(out, [1.0, 0.0], 1e-5));
    }

    #[test]
    fn asymmetric_size_to_px_ratio() {
        // Sprite is 10m wide × 4m tall but the underlying texture is
        // 50×100 px (very non-square pixel-to-world ratio). Bounds
        // 25..50 × 0..100 (right half by width, full height). Δworld
        // should be (10 * (37.5/50 - 0.5), 0) = (2.5, 0).
        let out = recenter_after_crop(
            [0.0, 0.0],
            [10.0, 4.0],
            [50, 100],
            PixelBounds {
                x: 25,
                y: 0,
                width: 25,
                height: 100,
            },
        );
        assert!(approx(out, [2.5, 0.0], 1e-5));
    }

    #[test]
    fn zero_px_dimension_returns_old_translation() {
        let out = recenter_after_crop(
            [7.0, 3.0],
            [4.0, 4.0],
            [0, 100],
            PixelBounds {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            },
        );
        assert_eq!(out, [7.0, 3.0]);
    }

    #[test]
    fn from_trim_bounds_round_trips() {
        let trim = crate::tools::trim_transparency::Bounds {
            x: 4,
            y: 8,
            width: 12,
            height: 16,
        };
        let bounds = PixelBounds::from_trim(trim);
        assert_eq!(bounds.x, 4);
        assert_eq!(bounds.y, 8);
        assert_eq!(bounds.width, 12);
        assert_eq!(bounds.height, 16);
    }
}
