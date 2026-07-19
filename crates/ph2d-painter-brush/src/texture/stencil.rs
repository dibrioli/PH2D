//! Stencil mapping: the on-canvas rect you paint *through*. Its frame (centre / extent / rotation)
//! comes from the brush's DEDICATED `stencil_offset` / `stencil_size` / `stencil_angle_deg` — fully
//! independent of the texture's own `offset` / `size` / `angle_deg`, which tile the pattern for the
//! other mappings. Shared by [`super::dab_basis`], the tool's overlay, and [`super::sample`] so the
//! painted mask and its outline agree exactly.

use super::{
    TEX_SIZE_MAX, TEX_SIZE_MIN, TexDabBasis, TextureKind, TextureSettings, rotate_by_degrees,
};

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

/// The per-dab [`TexDabBasis`] for a Stencil texture (no per-dab Rake / Random / jitter — the rect is
/// fixed in image space). The rect's own rotation (`stencil_u`/`v`, from `stencil_angle_deg`) projects
/// a pixel for the mask, while `u`/`v` carry the TEXTURE's [`TextureSettings::angle_deg`] so the
/// pattern can rotate WITHIN the rect independently of the gizmo (Enio 2026-06-26).
pub(super) fn stencil_basis(s: &TextureSettings, canvas: [f32; 2]) -> TexDabBasis {
    let (center, half, su) = stencil_frame(s, canvas);
    let tu = rotate_by_degrees(s.angle_deg);
    TexDabBasis {
        u: tu,
        v: super::perp(tu),
        jitter: [0.0, 0.0],
        stencil_center: center,
        stencil_half: half,
        stencil_u: su,
        stencil_v: super::perp(su),
        // Stencil is canvas-fixed — the dab flatten/rotate does not deform it.
        footprint: crate::footprint::FootprintDeform::identity(),
        // Stencil is a Grain mapping; it never flows (Flow is Shape-only).
        arc_len: 0.0,
    }
}

/// Map canvas pixel `p` into the stencil's tile coordinates, or `None` when `p` falls outside the rect
/// (the dab deposits nothing there). The rect's rotation basis + centre + half-extent are pre-resolved
/// on the per-dab [`TexDabBasis`]; this projects `p` into the rect's local `[-1, 1]²` (the mask), then
/// maps it onto the procedural's tile window with the TEXTURE's own Size (density), Angle (`b.u`/`b.v`,
/// rotation WITHIN the rect) and Offset applied — so the pattern inside the gizmo is tunable
/// independently of the gizmo placement. At the texture defaults (size 1, angle 0, offset 0) this is
/// byte-identical to the plain `(l + 1)·½·STENCIL_TILES` window.
pub(super) fn stencil_tex_coord(
    s: &TextureSettings,
    b: &TexDabBasis,
    p: [f32; 2],
) -> Option<[f32; 2]> {
    // Project into the rect's local frame (the rect's OWN rotation basis) for the mask.
    let rel = [p[0] - b.stencil_center[0], p[1] - b.stencil_center[1]];
    let lx = (rel[0] * b.stencil_u[0] + rel[1] * b.stencil_u[1]) / b.stencil_half[0];
    let ly = (rel[0] * b.stencil_v[0] + rel[1] * b.stencil_v[1]) / b.stencil_half[1];
    if lx.abs() > 1.0 || ly.abs() > 1.0 {
        return None; // outside the stencil → paint nothing
    }
    // Texture transform WITHIN the rect: scale the local coord by the texture Size (denser pattern),
    // rotate it by the texture Angle basis (`b.u`/`b.v`) about the rect centre, then map onto the tile
    // window and shift by the texture Offset (in tiles). Identity at the defaults.
    let sx = s.size[0].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
    let sy = s.size[1].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
    let cu = lx * sx;
    let cv = ly * sy;
    let ru = cu * b.u[0] + cv * b.u[1];
    let rv = cu * b.v[0] + cv * b.v[1];
    Some(stencil_window(s, [ru, rv]))
}

/// Map the rect-local rotated coord `r ∈ [-1, 1]²` onto the pattern's sample window + Offset. An **Image**
/// fills the rect ONCE at Size `(1, 1)` — its coord goes STRAIGHT to [`super::patterns::sample_image`]
/// (which does its own `½ + ½` → `[0, 1]` fill), so the painted stencil matches the Grain preview (also
/// once at size 1). Procedurals map onto the multi-tile [`STENCIL_TILES`] window so their cells read
/// (Enio 2026-06-28).
fn stencil_window(s: &TextureSettings, r: [f32; 2]) -> [f32; 2] {
    if matches!(s.kind, TextureKind::Image) {
        [r[0] + s.offset[0], r[1] + s.offset[1]]
    } else {
        [
            (r[0] + 1.0) * 0.5 * STENCIL_TILES + s.offset[0],
            (r[1] + 1.0) * 0.5 * STENCIL_TILES + s.offset[1],
        ]
    }
}

/// `0.0` when canvas pixel `(px, py)` falls OUTSIDE a Stencil-mapped texture's rect (deposit nothing),
/// else `1.0`. Unlike [`super::sample`] this is a pure rect MASK independent of the grain VALUE — so a
/// Grain Color Ramp can't paint `ramp[0]` outside the rect (`sample` returns `0` there, which a ramp
/// reads as a colour, not as "no paint" — Enio 2026-06-28). `1.0` for every non-Stencil mapping.
#[must_use]
pub fn stencil_gate(s: &TextureSettings, b: &TexDabBasis, px: i64, py: i64) -> f32 {
    if !s.mapping.is_stencil() {
        return 1.0;
    }
    let p = [px as f32 + 0.5, py as f32 + 0.5];
    f32::from(stencil_tex_coord(s, b, p).is_some())
}

/// Render the Grain pattern as it tiles **inside** the Stencil rect into an RGBA8 `w`×`h` buffer
/// (row-major), in the rect's LOCAL axis-aligned frame — the shell rotates + places it onto the gizmo.
/// Each texel's local coord `[-1, 1]²` runs through the SAME within-rect transform as
/// [`stencil_tex_coord`] (texture Size / Angle / Offset onto the [`STENCIL_TILES`] window) and is
/// sampled via [`super::patterns::sample_kind`], so the preview matches what a stroke would deposit
/// there. Grayscale (the raw coverage scalar), fully opaque. Drives the transient on-canvas preview
/// shown while the user transforms the gizmo or its params (Enio 2026-06-26). No-op if `out` is short.
pub fn render_stencil_preview(
    s: &TextureSettings,
    image: Option<&super::ImageMask>,
    out: &mut [u8],
    w: u32,
    h: u32,
) {
    if w == 0 || h == 0 || out.len() < (w as usize) * (h as usize) * 4 {
        return;
    }
    let sx = s.size[0].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
    let sy = s.size[1].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
    let tu = rotate_by_degrees(s.angle_deg); // the TEXTURE angle basis (`b.u` in `stencil_tex_coord`)
    let tv = super::perp(tu);
    let enc = |c: f32| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    for py in 0..h {
        let ly = (py as f32 + 0.5) / h as f32 * 2.0 - 1.0;
        for px in 0..w {
            let lx = (px as f32 + 0.5) / w as f32 * 2.0 - 1.0;
            let cu = lx * sx;
            let cv = ly * sy;
            let ru = cu * tu[0] + cv * tu[1];
            let rv = cu * tv[0] + cv * tv[1];
            let tex = stencil_window(s, [ru, rv]); // same window as the painted stencil (image fills 1:1)
            let g = enc(super::patterns::sample_kind(s.kind, tex, s.params, image));
            let i = ((py * w + px) * 4) as usize;
            out[i] = g;
            out[i + 1] = g;
            out[i + 2] = g;
            out[i + 3] = 255;
        }
    }
}
