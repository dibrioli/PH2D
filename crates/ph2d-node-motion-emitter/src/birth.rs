//! **ONDE uma partícula nasce, e para que lado ela aponta** — a `Shape` do berço, o
//! `DirMode` da direção, e as duas funções puras que os leem.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 700 para `crates/`), e o corte
//! é por RESPONSABILIDADE: o `lib.rs` responde *quantas nascem e quando*, este responde
//! *onde e para onde*. ⚠️ O `gpu.rs` já nomeia estas duas funções como a lei que ele
//! reafirma à mão em WGSL — o doc dele aponta para cá, e continua a apontar.

use super::{LANE_SHAPE_U, LANE_SHAPE_V, rand01};
use crate::trig::cos_sin_cycles;

/// Where a particle is born, relative to the emitter's origin.
///
/// The **integer** the `shape_mode` param carries; anything else reads as [`Self::Point`], which
/// is what every graph that predates the param means.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(super) enum Shape {
    /// All from one spot — the emitter that always shipped.
    Point,
    /// Anywhere inside the ellipse `shape_w × shape_h`.
    Disc,
    /// On its outline only.
    Ring,
    /// Anywhere inside the rectangle of half-extents `shape_w × shape_h`.
    Rect,
}

impl Shape {
    pub(super) fn from_param(v: f32) -> Self {
        match v.round() as i32 {
            1 => Self::Disc,
            2 => Self::Ring,
            3 => Self::Rect,
            _ => Self::Point,
        }
    }
}

/// The birth offset of particle `id` inside `shape`, in the emitter's own frame — or `None`
/// when the shape is a point and there is **no offset at all**.
///
/// ⚠️ **The `None` is what makes the default byte-identical rather than merely zero.** Returning
/// `[0.0, 0.0]` would have the caller add it, and `-0.0 + 0.0` is `+0.0`: an origin the artist
/// typed as `-0` would come back with its sign flipped. The type says *there is nothing to add*,
/// so there is one place that knows it and no unreachable arm anywhere.
///
/// The draws are **uniform over the AREA**, not over the parameter: a disc sampled with a raw
/// radius piles up in the middle, which reads as a bright core the artist did not author — hence
/// the `sqrt`. Both lanes are the particle's own identity, so a birth place is as scrub-exact as
/// its velocity. The kernel mirrors this term for term.
pub(super) fn birth_offset(shape: Shape, w: f32, h: f32, seed: u32, id: u32) -> Option<[f32; 2]> {
    if shape == Shape::Point {
        return None;
    }
    let u = rand01(seed, id, LANE_SHAPE_U);
    let v = rand01(seed, id, LANE_SHAPE_V);
    Some(match shape {
        Shape::Rect => [(u - 0.5) * 2.0 * w, (v - 0.5) * 2.0 * h],
        // The outline: one draw is enough, and the second lane stays unread so the ring does
        // not silently depend on a number it has no use for.
        Shape::Ring => {
            let (c, s) = cos_sin_cycles(u);
            [c * w, s * h]
        }
        // `sqrt(u)` is the area-uniform radius; the affine `w`/`h` carries it to an ellipse.
        Shape::Disc | Shape::Point => {
            let r = u.sqrt();
            let (c, s) = cos_sin_cycles(v);
            [c * w * r, s * h * r]
        }
    })
}

/// **Which way a particle leaves** — along the artist's `angle`, or along its own radius.
///
/// `Outwards`/`Inwards` are only a question once a particle is born somewhere OTHER than the
/// origin, so this param arrived with the shape and not before it (the sheet marked it
/// *"not expressible without a shape"*, and the shape is what moved that number).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(super) enum DirMode {
    /// The cone `angle ± spread/2` — what always shipped.
    Angle,
    /// Away from the emitter's centre, through the birth place.
    Outwards,
    /// Towards it.
    Inwards,
}

impl DirMode {
    pub(super) fn from_param(v: f32) -> Self {
        match v.round() as i32 {
            1 => Self::Outwards,
            2 => Self::Inwards,
            _ => Self::Angle,
        }
    }
}

/// The unit axis a particle leaves along, BEFORE the cone's jitter — or `None` when there is
/// no radius to leave along and the artist's `angle` is the only direction there is.
///
/// ⚠️ **`None` is the whole reason `Angle` stays byte-identical.** The caller keeps its single
/// `cos_sin_cycles(angle + jitter·spread)` on that arm — the same expression, not an equivalent
/// one — and only the radial arms ROTATE. Two `cos_sin` calls composed are not the same `f32` as
/// one call on the summed angle, so folding the two arms together would have moved every existing
/// emitter by an ulp and taken the GPU parity with it.
///
/// ⚠️ **And the axis is ROTATED, never re-derived.** The birth offset is already a vector; asking
/// `atan2` for its angle, adding the jitter and calling `cos_sin` back would spend two
/// approximations to learn what a 2×2 rotation composes exactly — the lesson the radial array's
/// `align` paid a wave earlier.
///
/// A particle born exactly at the centre has no radial direction (a `Point` emitter, a zero-sized
/// shape, or a `Disc` draw that lands on the middle), so it falls back to the cone. That is the
/// honest answer, not a special case: "outwards" from a zero-length vector is not a direction.
pub(super) fn radial_axis(dir: DirMode, offset: Option<[f32; 2]>) -> Option<[f32; 2]> {
    let sign = match dir {
        DirMode::Angle => return None,
        DirMode::Outwards => 1.0,
        DirMode::Inwards => -1.0,
    };
    let d = offset?;
    let len2 = d[0] * d[0] + d[1] * d[1];
    if len2 <= 0.0 {
        return None;
    }
    let inv = sign / len2.sqrt();
    Some([d[0] * inv, d[1] * inv])
}
