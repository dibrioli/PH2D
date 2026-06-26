//! Stencil mapping: the on-canvas rect you paint *through*. Its frame (centre / extent / rotation)
//! comes from the brush's DEDICATED `stencil_offset` / `stencil_size` / `stencil_angle_deg` — fully
//! independent of the texture's own `offset` / `size` / `angle_deg`, which tile the pattern for the
//! other mappings. Shared by [`super::dab_basis`], the tool's overlay, and [`super::sample`] so the
//! painted mask and its outline agree exactly.

use super::{TEX_SIZE_MAX, TEX_SIZE_MIN, TexDabBasis, TextureSettings, rotate_by_degrees};

/// Procedural tiles the **Stencil** rect spans per axis, so the pattern reads (one tile would be a
/// flat cell). Density is fixed; the rect's on-canvas size is the brush's [`TextureSettings::stencil_size`].
const STENCIL_TILES: f32 = 4.0;

/// The Stencil rect's centre + half-extent (canvas px) + rotation unit vector. [`TextureSettings::stencil_offset`]
/// maps `[-1, 1]` onto the canvas span (centre at `0`); [`TextureSettings::stencil_size`] is the
/// half-extent as a canvas fraction (`1.0` ≈ full, default `0.5` = 50 % of the sprite); rotation is
/// the baked [`TextureSettings::stencil_angle_deg`].
#[must_use]
pub fn stencil_frame(s: &TextureSettings, canvas: [f32; 2]) -> ([f32; 2], [f32; 2], [f32; 2]) {
    let center = [
        (0.5 + 0.5 * s.stencil_offset[0]) * canvas[0],
        (0.5 + 0.5 * s.stencil_offset[1]) * canvas[1],
    ];
    let half = [
        (s.stencil_size[0].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX) * 0.5 * canvas[0]).max(1e-3),
        (s.stencil_size[1].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX) * 0.5 * canvas[1]).max(1e-3),
    ];
    (center, half, rotate_by_degrees(s.stencil_angle_deg))
}

/// Map canvas pixel `p` into the stencil's tile coordinates, or `None` when `p` falls outside the rect
/// (the dab deposits nothing there). The rect's basis + centre + half-extent are pre-resolved on the
/// per-dab [`TexDabBasis`]; this projects, masks, and maps the rect's `[-1, 1]²` onto one tile window.
pub(super) fn stencil_tex_coord(b: &TexDabBasis, p: [f32; 2]) -> Option<[f32; 2]> {
    let rel = [p[0] - b.stencil_center[0], p[1] - b.stencil_center[1]];
    let lx = (rel[0] * b.u[0] + rel[1] * b.u[1]) / b.stencil_half[0];
    let ly = (rel[0] * b.v[0] + rel[1] * b.v[1]) / b.stencil_half[1];
    if lx.abs() > 1.0 || ly.abs() > 1.0 {
        return None; // outside the stencil → paint nothing
    }
    // Map [-1,1]² onto a fixed-density tile window so the procedural pattern reads in the rect.
    Some([
        (lx + 1.0) * 0.5 * STENCIL_TILES,
        (ly + 1.0) * 0.5 * STENCIL_TILES,
    ])
}
