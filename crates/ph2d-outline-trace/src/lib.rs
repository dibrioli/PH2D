//! Picture alpha → collision outline: integer rings in Q4 (1/16 px) that COVER every opaque pixel.
//! Pipeline, laws and measurements: `README.md`. Every step is integer and fixed-order: the same bytes
//! on any machine.

mod geom;
mod march;
mod pieces;
mod simplify;

pub use pieces::{convex_pieces, hertel_mehlhorn, triangulate};

/// Sub-pixel units per pixel.
pub const Q4: i32 = 16;

/// A closed ring of Q4 points (x right, y down). Outer rings have POSITIVE shoelace area in these
/// coordinates, holes negative; each ring starts at its smallest point (`[x, y]` order).
pub type Ring = Vec<[i32; 2]>;

/// One bit per pixel (stored as a byte, `0` or `1`), row-major.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mask {
    pub w: u32,
    pub h: u32,
    pub bits: Vec<u8>,
}

impl Mask {
    /// Opaque = alpha ≥ `cutoff`. `rgba.len()` must be `w * h * 4`.
    #[must_use]
    pub fn from_rgba8(w: u32, h: u32, rgba: &[u8], cutoff: u8) -> Self {
        let bits = rgba
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| u8::from(p[3] >= cutoff))
            .collect();
        Self { w, h, bits }
    }

    /// Byte OR of same-sized masks (the Animated Sprite law: one outline for every frame).
    /// `None` when empty or the sizes differ.
    #[must_use]
    pub fn union(masks: &[Mask]) -> Option<Mask> {
        let first = masks.first()?;
        if masks.iter().any(|m| m.w != first.w || m.h != first.h) {
            return None;
        }
        let mut out = first.clone();
        for m in &masks[1..] {
            for (o, b) in out.bits.iter_mut().zip(&m.bits) {
                *o |= *b;
            }
        }
        Some(out)
    }

    /// Out of range is empty.
    #[must_use]
    pub fn get(&self, x: i64, y: i64) -> bool {
        if x < 0 || y < 0 || x >= i64::from(self.w) || y >= i64::from(self.h) {
            return false;
        }
        let i = usize::try_from(y * i64::from(self.w) + x).unwrap_or(usize::MAX);
        self.bits.get(i).is_some_and(|b| *b != 0)
    }

    #[must_use]
    pub fn count(&self) -> usize {
        self.bits.iter().filter(|b| **b != 0).count()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    /// Largest distance (Q4) the outline may move OUT from the art when it simplifies. Never in.
    pub detail_q4: u16,
    /// Rings (islands and holes) whose pixel area is below this are dropped: an island goes, a hole
    /// fills.
    pub min_island_px: u16,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Outline {
    pub rings: Vec<Ring>,
}

impl Outline {
    /// Even-odd containment of a Q4 point that lies on no edge (pixel centres never do).
    #[must_use]
    pub fn contains_q4(&self, p: [i32; 2]) -> bool {
        self.rings
            .iter()
            .filter(|r| geom::ring_crosses(r, p))
            .count()
            % 2
            == 1
    }

    /// Sum of signed doubled areas (Q4²): outer minus holes, times two.
    #[must_use]
    pub fn area2(&self) -> i64 {
        self.rings.iter().map(|r| geom::area2(r)).sum()
    }

    /// Canonical bytes: per ring `u32` length then `i32` x, y, all little-endian.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for r in &self.rings {
            out.extend_from_slice(&u32::try_from(r.len()).unwrap_or(u32::MAX).to_le_bytes());
            for p in r {
                out.extend_from_slice(&p[0].to_le_bytes());
                out.extend_from_slice(&p[1].to_le_bytes());
            }
        }
        out
    }

    #[must_use]
    pub fn vertex_count(&self) -> usize {
        self.rings.iter().map(Vec::len).sum()
    }
}

/// Most convex pieces one Outline shape may have. Resource: the solver tick — measured (W0, table in
/// `README.md`) 0.30 ms per tick for 257 pieces all touching the floor, 0.61 ms for two stacked.
pub const MAX_PIECES: usize = 256;
/// Most outline points before the pieces are cut. Resource: the retrace, synchronous in the frame of
/// the gesture (dragging Detail) and of keyed Detail in Play — measured 3.6 ms for 516 points (W0),
/// under a quarter of a 60 Hz frame. The cost is O(points²): whoever lowers that order re-measures this.
pub const MAX_POINTS: usize = 512;

/// A traced outline that fits [`MAX_POINTS`] and [`MAX_PIECES`], and the parameters that made it fit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fitted {
    pub outline: Outline,
    pub pieces: Vec<Vec<[i32; 2]>>,
    /// The parameters used: equal to the asked ones unless the ceiling raised them.
    pub params: Params,
}

impl Fitted {
    /// Whether the ceiling raised Detail (or, past the picture's size, Ignore specks under).
    #[must_use]
    pub fn raised(&self, asked: Params) -> bool {
        self.params != asked
    }
}

/// [`trace`] then [`convex_pieces`] at `p`, if the result fits both ceilings.
fn fits(mask: &Mask, p: Params) -> Option<Fitted> {
    let outline = trace_within(mask, p, MAX_POINTS)?;
    let pieces = convex_pieces(&outline, p.detail_q4);
    (pieces.len() <= MAX_PIECES).then_some(Fitted {
        outline,
        pieces,
        params: p,
    })
}

/// The smallest knob value above `lo` (known not to fit) and up to `cap` whose result fits: doubling,
/// then bisection. Deterministic integer steps.
fn smallest(mask: &Mask, lo: u16, cap: u16, at: impl Fn(u16) -> Params) -> Option<(u16, Fitted)> {
    if lo >= cap {
        return None;
    }
    let (mut lo, mut hi) = (lo, lo.max(1));
    let mut best = loop {
        hi = hi.saturating_mul(2).min(cap);
        if let Some(f) = fits(mask, at(hi)) {
            break f;
        }
        if hi >= cap {
            return None;
        }
        lo = hi;
    };
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        match fits(mask, at(mid)) {
            Some(f) => (hi, best) = (mid, f),
            None => lo = mid,
        }
    }
    Some((hi, best))
}

/// [`trace`] then [`convex_pieces`], raising Detail to the SMALLEST value whose outline fits both
/// ceilings (never cutting the art). Only when Detail already spans the picture and the islands alone
/// are too many does Ignore specks under rise the same way.
#[must_use]
pub fn trace_fitted(mask: &Mask, asked: Params) -> Fitted {
    if let Some(f) = fits(mask, asked) {
        return f;
    }
    let span =
        u16::try_from(mask.w.max(mask.h).saturating_mul(Q4.unsigned_abs())).unwrap_or(u16::MAX);
    let detail_q4 = asked.detail_q4.max(span);
    let at_detail = |d| Params {
        detail_q4: d,
        ..asked
    };
    if let Some((_, f)) = smallest(mask, asked.detail_q4, detail_q4, at_detail) {
        return f;
    }
    let at_specks = |s| Params {
        detail_q4,
        min_island_px: s,
    };
    smallest(mask, asked.min_island_px, u16::MAX, at_specks).map_or_else(
        // Unreachable in practice (at `u16::MAX` px every island goes): empty, never a panic.
        || Fitted {
            outline: Outline::default(),
            pieces: Vec::new(),
            params: at_specks(u16::MAX),
        },
        |(_, f)| f,
    )
}

/// Mask → covering outline. See `README.md` for the steps.
#[must_use]
pub fn trace(mask: &Mask, params: Params) -> Outline {
    trace_with(mask, params, false, usize::MAX).unwrap_or_default()
}

/// [`trace`], or `None` when the outline has more than `cap` points (then the rest of the work is skipped).
#[doc(hidden)]
#[must_use]
pub fn trace_within(mask: &Mask, params: Params, cap: usize) -> Option<Outline> {
    trace_with(mask, params, false, cap)
}

/// The simplifier's conflict rounds run on this thread so far (the cost [`trace_within`] skips).
#[doc(hidden)]
#[must_use]
pub fn sweeps_on_this_thread() -> u64 {
    simplify::sweeps()
}

/// [`trace`] with the pixel-centre guard asking every ring instead of the row buckets: the reference
/// that `tests/fora.rs` holds byte-equal to [`trace`] and times against it.
#[doc(hidden)]
#[must_use]
pub fn trace_reference_guard(mask: &Mask, params: Params) -> Outline {
    trace_with(mask, params, true, usize::MAX).unwrap_or_default()
}

fn trace_with(mask: &Mask, params: Params, brute: bool, cap: usize) -> Option<Outline> {
    let min2 = 2 * i64::from(params.min_island_px);
    let rings: Vec<Ring> = march::march(mask)
        .into_iter()
        .filter(|r| geom::area2(r).abs() >= min2)
        .map(|r| r.into_iter().map(|[x, y]| [x * Q4, y * Q4]).collect())
        .collect();
    let mut rings = simplify::simplify_capped(&rings, i64::from(params.detail_q4), brute, cap)?;
    for r in &mut rings {
        geom::rotate_to_smallest(r);
    }
    rings.sort();
    Some(Outline { rings })
}
