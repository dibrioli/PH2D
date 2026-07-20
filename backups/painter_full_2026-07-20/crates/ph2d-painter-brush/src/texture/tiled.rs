//! **Canvas-anchored (Tiled) samplers** — the paper/grain slot read as a fixed height-field over the
//! image, independent of any dab (the watercolor render-path granulates the wash through these). Split
//! from `texture.rs` for the workspace LOC cap; the entry points are re-exported there so callers keep
//! using `texture::{sample_tiled, sample_tiled_rot, sample_tiled_rot_wrapped, angle_basis}`.

use super::*;

/// Sample a **canvas-anchored** (Tiled) texture at canvas pixel `(px, py)` — the paper as a fixed
/// height-field over the image, independent of any dab (the watercolor render-path reads the Grain slot
/// through this to granulate the wash, `docs/Painter/10…`). Identity basis, no rotation/jitter; honours
/// the texture's Size + Offset. `image` backs [`TextureKind::Image`] (a tagged layer used as paper);
/// without it that kind is inert. Returns the coverage in `[0, 1]`; `1.0` when the slot is inactive.
#[must_use]
pub fn sample_tiled(s: &TextureSettings, px: i64, py: i64, image: Option<&ImageMask>) -> f32 {
    sample_tiled_rot(s, px, py, image, angle_basis(s.angle_deg))
}

/// The rotation basis `[cos, sin]` for a whole-degree angle (transcendental-free, HR-5) — precompute it
/// once, then feed it to [`sample_tiled_rot`] per pixel so the per-call cost is a few mults, not trig.
#[must_use]
pub fn angle_basis(deg: u16) -> [f32; 2] {
    rotate_by_degrees(deg)
}

/// [`sample_tiled`] with a caller-precomputed rotation basis `rot` (from [`angle_basis`]) — the hot-loop
/// form: the tile coords are rotated by `rot` before the pattern is sampled, so the paper honours its
/// **Angle** (fibre orientation). `rot = [1, 0]` is no rotation.
#[must_use]
pub fn sample_tiled_rot(
    s: &TextureSettings,
    px: i64,
    py: i64,
    image: Option<&ImageMask>,
    rot: [f32; 2],
) -> f32 {
    sample_tiled_rot_wrapped(s, px, py, image, rot, [0.0, 0.0])
}

/// [`sample_tiled_rot`] that also makes a **lattice** procedural ([`lattice_tileable`]) seam-free across
/// a SPRITE tiling period: `period_px` is the sprite's per-axis span (px; `0` = that axis doesn't tile).
/// The lattice hash wraps at `round(period_px · size / 256)` cells — an integer because the caller snaps
/// the Size so a whole number of cells spans the sprite (a fractional span would seam mid-cell). Gated to
/// **no rotation** (`rot = [1, 0]`): a rotated axis-lattice can't align with the axis-aligned sprite seam.
/// `period_px = [0, 0]` (or a non-lattice kind) ⇒ no wrap, byte-identical to [`sample_tiled_rot`].
#[must_use]
pub fn sample_tiled_rot_wrapped(
    s: &TextureSettings,
    px: i64,
    py: i64,
    image: Option<&ImageMask>,
    rot: [f32; 2],
    period_px: [f32; 2],
) -> f32 {
    if !s.is_active() {
        return 1.0;
    }
    let sx = s.size[0].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
    let sy = s.size[1].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
    let p = [px as f32 + 0.5, py as f32 + 0.5];
    let rel = [p[0] / TEX_TILE_BASE_PX, p[1] / TEX_TILE_BASE_PX];
    // Rotate the tile coordinates by the Angle basis, then apply the Offset.
    //
    // ⚠️ The TRANSPOSE form (`+`, i.e. `R(-angle)` on the coordinate), which is what every other rotation
    // in the engine uses — the footprint, the slot frames, the stencil rect and the pattern within it.
    // This one was the lone outlier in the FORWARD form until 2026-07-19, so the same Angle slider turned
    // a Tiled Grain and a Paper of the same kind in OPPOSITE directions. Rotating the lookup coordinate by
    // `-angle` is what makes the pattern appear rotated by `+angle`; it is also Blender's convention
    // (`BKE_brush_sample_tex_3d` negates `mtex->rot` for exactly this reason).
    let tex = [
        (rel[0] * rot[0] + rel[1] * rot[1]) * sx + s.offset[0],
        (-rel[0] * rot[1] + rel[1] * rot[0]) * sy + s.offset[1],
    ];
    // Hash wrap: only with no rotation (else the axis grid can't meet the axis-aligned seam) and a real
    // sprite period. Applies to the LATTICE procedurals AND the hash-jittered analytic patterns (Dots /
    // Scales, `analytic_needs_hash_wrap`) — both hash their cells, so the seam needs the hash aliased at
    // the cell period. `rel`-span across the sprite = `period_px · size / 256` cells (integer here because
    // the caller snapped the Size); `[0, 0]` ⇒ `sample_kind_t` no-ops the wrap (byte-identical).
    let period = if rot[0] == 1.0
        && rot[1] == 0.0
        && (lattice_tileable(s.kind) || analytic_needs_hash_wrap(s.kind))
    {
        [
            lattice_period(period_px[0], sx),
            lattice_period(period_px[1], sy),
        ]
    } else {
        [0, 0]
    };
    patterns::sample_kind_t(s.kind, tex, s.params, image, period).clamp(0.0, 1.0)
}

/// The lattice wrap period in whole cells for one axis: the sprite span `period_px` expressed in `rel`
/// cells (`period_px · size / 256`), rounded. `period_px <= 0` (untiled axis) ⇒ `0` = no wrap.
#[must_use]
fn lattice_period(period_px: f32, size: f32) -> i32 {
    if period_px > 0.0 {
        (period_px * size / TEX_TILE_BASE_PX).round().max(0.0) as i32
    } else {
        0
    }
}
