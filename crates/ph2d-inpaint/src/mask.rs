//! [`Mask`] (which pixels are the hole to fill) and [`Regions`] (the per-level
//! classification of patch centres into *source* and *target*).
//!
//! For a patch radius `r`, a centre `(x,y)` is:
//! * a **source** if its whole `(2r+1)²` patch is known (no hole pixel) — these
//!   are the only patches we copy FROM;
//! * a **target** if its patch overlaps the hole — these are the patches we fill
//!   IN (every hole pixel is the centre of a target patch, so every hole pixel
//!   is voted on).
//!
//! Patch reads clamp to the edge, so any centre in `[0,w)×[0,h)` is usable and
//! classification is likewise clamp-aware.

use crate::plane::clampi;

/// A binary hole mask. `hole[y*w+x] == true` ⇒ the pixel must be reconstructed.
#[derive(Clone, Debug)]
pub struct Mask {
    pub w: usize,
    pub h: usize,
    pub hole: Vec<bool>,
}

impl Mask {
    /// Build from a caller byte mask (`>= 128` ⇒ hole), length `w*h`.
    pub fn from_bytes(w: usize, h: usize, bytes: &[u8]) -> Self {
        let hole = bytes.iter().map(|&b| b >= 128).collect();
        Self { w, h, hole }
    }

    /// Is `(x,y)` (clamped) a hole pixel?
    #[inline]
    pub fn is_hole(&self, x: i32, y: i32) -> bool {
        let cx = clampi(x, self.w);
        let cy = clampi(y, self.h);
        self.hole[cy * self.w + cx]
    }

    /// Any hole at all? (No hole ⇒ nothing to do.)
    pub fn has_hole(&self) -> bool {
        self.hole.iter().any(|&b| b)
    }

    /// Halve resolution. A coarse pixel is a hole if ANY of its 2×2 children is
    /// a hole (`max`): coarse holes must COVER the fine hole so the coarse level
    /// never treats defect pixels as a valid source. Rounds dimensions up to
    /// match [`Plane::downsample`](crate::plane::Plane::downsample).
    pub fn downsample(&self) -> Mask {
        let w2 = self.w.div_ceil(2).max(1);
        let h2 = self.h.div_ceil(2).max(1);
        let mut hole = vec![false; w2 * h2];
        for y in 0..h2 {
            for x in 0..w2 {
                let (sx, sy) = (x as i32 * 2, y as i32 * 2);
                let any = [(0, 0), (1, 0), (0, 1), (1, 1)]
                    .iter()
                    .any(|&(dx, dy)| self.is_hole(sx + dx, sy + dy));
                hole[y * w2 + x] = any;
            }
        }
        Mask { w: w2, h: h2, hole }
    }
}

/// Per-level, per-radius patch-centre classification.
#[derive(Clone, Debug)]
pub struct Regions {
    pub w: usize,
    pub h: usize,
    /// `is_source[idx]` ⇒ the patch centred there is entirely known.
    pub is_source: Vec<bool>,
    /// Flat indices of every source centre (for random NNF init / search).
    pub sources: Vec<u32>,
    /// Flat indices of every target centre (patch overlaps the hole).
    pub targets: Vec<u32>,
}

impl Regions {
    /// Classify every centre of `mask` for patch radius `r` (clamp-aware).
    pub fn build(mask: &Mask, r: i32) -> Self {
        let (w, h) = (mask.w, mask.h);
        let mut is_source = vec![false; w * h];
        let mut sources = Vec::new();
        let mut targets = Vec::new();
        for y in 0..h as i32 {
            for x in 0..w as i32 {
                let mut overlaps = false;
                let mut all_known = true;
                'patch: for dy in -r..=r {
                    for dx in -r..=r {
                        if mask.is_hole(x + dx, y + dy) {
                            overlaps = true;
                            all_known = false;
                            break 'patch;
                        }
                    }
                }
                let idx = (y as usize * w + x as usize) as u32;
                if all_known {
                    is_source[idx as usize] = true;
                    sources.push(idx);
                }
                if overlaps {
                    targets.push(idx);
                }
            }
        }
        Self {
            w,
            h,
            is_source,
            sources,
            targets,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn central_hole(w: usize, h: usize, hx: usize, hy: usize, hw: usize, hh: usize) -> Mask {
        let mut bytes = vec![0u8; w * h];
        for y in hy..hy + hh {
            for x in hx..hx + hw {
                bytes[y * w + x] = 255;
            }
        }
        Mask::from_bytes(w, h, &bytes)
    }

    #[test]
    fn source_excludes_and_target_covers_the_hole() {
        let m = central_hole(16, 16, 6, 6, 4, 4);
        let reg = Regions::build(&m, 2);
        // A far corner centre (0,0): its clamped patch never reaches the hole.
        assert!(reg.is_source[0]);
        // Every hole pixel is a target centre.
        for y in 6..10 {
            for x in 6..10 {
                assert!(
                    reg.targets.contains(&((y * 16 + x) as u32)),
                    "hole pixel ({x},{y}) must be a target centre"
                );
            }
        }
        // A hole-centre is never a source.
        assert!(!reg.is_source[7 * 16 + 7]);
    }

    #[test]
    fn downsample_grows_the_hole_by_covering() {
        let m = central_hole(8, 8, 3, 3, 2, 2);
        let d = m.downsample();
        assert_eq!((d.w, d.h), (4, 4));
        assert!(d.has_hole());
        // The fine 2×2 hole at (3..5) covers coarse pixels (1..3)×(1..3).
        assert!(d.is_hole(1, 1));
        assert!(d.is_hole(2, 2));
    }
}
