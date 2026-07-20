//! **Seamless-tiling metadata** for the [`super`] patterns — which kinds can tile across a sprite seam
//! and the per-axis period a caller snaps the Size to. Split from `patterns.rs` for the workspace LOC cap.
//! Pure classification (no sampling): the samplers live in `patterns.rs`; the canvas-anchored snap+wrap
//! that consumes this lives in `texture/tiled.rs` + the tool's `watercolor_noise::snap_slot_size`.

use super::{freq_mul, knob};
use crate::texture::{MAX_TEX_PARAMS, TextureKind};

/// True when `kind` is a **lattice** procedural that [`super::sample_kind_t`] can wrap at an integer period
/// for seamless any-size tiling (the value-noise family + Voronoi). A caller that snaps a slot's Size for
/// sprite-seamless tiling ([`crate::texture::sample_tiled_rot_wrapped`]) must snap THESE to an integer
/// `rel`-span; analytic patterns snap to their own period instead ([`analytic_tile_period`]).
#[must_use]
pub fn lattice_tileable(kind: TextureKind) -> bool {
    matches!(
        kind,
        TextureKind::Noise
            | TextureKind::Clouds
            | TextureKind::DistortedNoise
            | TextureKind::Musgrave
            | TextureKind::Stucci
            | TextureKind::Grain
            | TextureKind::Voronoi
    )
}

/// The fundamental **period** (in `rel` units, per axis `[u, v]`) of an ANALYTIC pattern — `Some` when the
/// kind is exactly periodic with a RATIONAL period, so a caller can snap the slot Size to make the sprite
/// span an integer number of periods and the pattern tiles **seam-free**. The pure-periodic kinds need NO
/// sampler change (aligning the span is enough); the hash-jittered ones (Dots / Scales) ALSO need the cell
/// hash wrapped ([`analytic_needs_hash_wrap`]), like the lattice. A `0.0` on an axis = that axis is ignored
/// / constant (any Size is seamless there — don't snap it). `None` = NOT snap-tileable: the turbulence kinds
/// (Magic / Marble / Wood — noise, not periodic → they'd need the lattice hash-wrap) and the IRRATIONAL-
/// period kinds (Triangles `√3`, Hexagons `√3·g` — a pixel seam can never land exactly on an irrational
/// period). The frequency knob is slot `2` of `params[2..]` (the shared `Frequency` → coordinate multiplier
/// `freq_mul`), matching each sampler in `patterns.rs`; period-only kinds (Checker / Bricks / Dots / …)
/// ignore `params`.
#[must_use]
pub fn analytic_tile_period(kind: TextureKind, params: [f32; MAX_TEX_PARAMS]) -> Option<[f32; 2]> {
    let k = &params[2..];
    // A frequency-knob pattern (`f = frac(coord · g)`, `g = freq_mul(knob 1)`) repeats every `1/g`.
    let per = 1.0 / freq_mul(knob(k, 1));
    Some(match kind {
        // Cell-parity lattices — period 2, independent of the knobs (Softness only blurs the edge).
        TextureKind::Checker | TextureKind::Diamonds => [2.0, 2.0],
        // Frequency-driven directional / mesh patterns.
        TextureKind::Stripes => [per, 0.0], // v ignored → seamless on v at any Size
        TextureKind::Grid | TextureKind::Crosshatch => [per, per],
        TextureKind::Waves => [1.0, per], // ripple reads `wave01(u)` (period 1); bands run `v · g`
        TextureKind::Chevron => [per, 1.0], // zig runs `u · g`; bands `wave01(v)` (period 1)
        TextureKind::Weave => [2.0 * per, 2.0 * per], // over/under parity → period `2/g`
        // Cell-based, period 1 across, 2 down (alternating rows). Dots/Scales ALSO hash-wrap (below).
        TextureKind::Dots => [1.0, 1.0],
        TextureKind::Bricks | TextureKind::Scales => [1.0, 2.0],
        // Gradient (Blender Blend): `Repeat` = knob 1 → `1 + 5·knob` ramps per unit; v ignored.
        TextureKind::Gradient => [1.0 / (1.0 + knob(k, 1) * 5.0), 0.0],
        _ => return None,
    })
}

/// True when an analytic pattern jitters its cells with a per-cell hash (Dots / Scales `Randomness`): on
/// top of the Size snap ([`analytic_tile_period`]) the sampler ALSO needs that hash wrapped at the cell
/// period, exactly like the lattice ([`lattice_tileable`]) — [`crate::texture::sample_tiled_rot_wrapped`]
/// passes the period for these too. A cell-hash pattern that is only size-snapped still seams (each cell's
/// jitter is unique across the seam).
#[must_use]
pub fn analytic_needs_hash_wrap(kind: TextureKind) -> bool {
    matches!(kind, TextureKind::Dots | TextureKind::Scales)
}
