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

/// Mask → covering outline. See `README.md` for the steps.
#[must_use]
pub fn trace(mask: &Mask, params: Params) -> Outline {
    let min2 = 2 * i64::from(params.min_island_px);
    let rings: Vec<Ring> = march::march(mask)
        .into_iter()
        .filter(|r| geom::area2(r).abs() >= min2)
        .map(|r| r.into_iter().map(|[x, y]| [x * Q4, y * Q4]).collect())
        .collect();
    let mut rings = simplify::simplify(&rings, i64::from(params.detail_q4));
    for r in &mut rings {
        geom::rotate_to_smallest(r);
    }
    rings.sort();
    Outline { rings }
}
