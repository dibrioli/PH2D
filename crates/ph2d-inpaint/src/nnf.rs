//! The nearest-neighbour field (NNF) and its PatchMatch search.
//!
//! `off[idx]` is the offset from a target centre to its best-matching source
//! centre (`source = target + off`); `cost[idx]` is that match's SSD. The search
//! is the **jump-flooding** PatchMatch variant (Barnes 2009 + jump-flood
//! propagation): each pass reads the previous NNF and writes an improved one, so
//! it is order-independent — the CPU reference and the W2 GPU compute run the
//! *same* passes and reconcile within float ε (the GPU only differs in the last
//! bits of the f32 SSD sums).
//!
//! One E-step = recompute costs against the current content, a sequence of
//! jump-flood propagation passes (steps `n/2 … 2, 1`), then one random-search
//! pass with an exponentially shrinking radius. All transcendental-free: SSD
//! compares squared distances (no `sqrt`) and the radius halves by integer
//! shift (no `pow`).

use crate::hash::{rand_range, rand_u32, seed32};
use crate::mask::Regions;
use crate::plane::{Plane, clampi};

/// The 8 propagation directions (unit); scaled by the jump-flood step.
const DIRS: [(i32, i32); 8] = [
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
];

/// A nearest-neighbour field over an `w×h` grid.
#[derive(Clone, Debug)]
pub struct Nnf {
    pub w: usize,
    pub h: usize,
    pub off: Vec<[i32; 2]>,
    pub cost: Vec<f32>,
    /// The global seed — every per-pixel RNG stream is hashed from it (+ index +
    /// pass), so the field is reproducible and GPU-matchable.
    pub seed: u64,
}

/// Sum of squared RGB differences between the target patch (read from the
/// evolving `content`, centred at `t`) and the source patch (read from the fixed
/// `src` image, centred at `s`), over the `(2r+1)²` window. Clamp-to-edge.
#[inline]
pub fn patch_ssd(content: &Plane, src: &Plane, t: [i32; 2], s: [i32; 2], r: i32) -> f32 {
    let mut acc = 0.0f32;
    for dy in -r..=r {
        for dx in -r..=r {
            let a = content.get(t[0] + dx, t[1] + dy);
            let b = src.get(s[0] + dx, s[1] + dy);
            let d0 = a[0] - b[0];
            let d1 = a[1] - b[1];
            let d2 = a[2] - b[2];
            acc += d0 * d0 + d1 * d1 + d2 * d2;
        }
    }
    acc
}

#[inline]
fn xy(idx: u32, w: usize) -> [i32; 2] {
    [(idx as usize % w) as i32, (idx as usize / w) as i32]
}

/// Salt bases keeping the init draw and each pass's random-search draw on
/// distinct per-pixel counter streams (see [`crate::hash`]).
pub(crate) const SALT_INIT: u32 = 0x1117;
pub(crate) const SALT_SEARCH: u32 = 0x5EED_0000;

impl Nnf {
    /// Random NNF init: each target centre points at a uniformly-chosen source
    /// centre (drawn from its own per-pixel stream), cost evaluated against the
    /// current content. Deterministic for a given `seed`; GPU-matchable.
    pub fn init(content: &Plane, src: &Plane, reg: &Regions, r: i32, seed: u64) -> Nnf {
        let (w, h) = (reg.w, reg.h);
        let s32 = seed32(seed);
        let mut off = vec![[0i32; 2]; w * h];
        let mut cost = vec![f32::INFINITY; w * h];
        let ns = reg.sources.len() as u32;
        for &ti in &reg.targets {
            let t = xy(ti, w);
            let pick = rand_u32(s32, ti, SALT_INIT, 0) % ns;
            let si = reg.sources[pick as usize];
            let s = xy(si, w);
            off[ti as usize] = [s[0] - t[0], s[1] - t[1]];
            cost[ti as usize] = patch_ssd(content, src, t, s, r);
        }
        Nnf {
            w,
            h,
            off,
            cost,
            seed,
        }
    }

    /// One E-step: refresh costs against the (possibly updated) `content`, run
    /// jump-flood propagation passes, then a random-search pass. `pass` is the EM
    /// iteration index — it salts the per-pixel search streams so successive
    /// iterations explore fresh candidates.
    pub fn e_step(&mut self, content: &Plane, src: &Plane, reg: &Regions, r: i32, pass: u64) {
        // Costs are stale after an M-step changed the content — refresh them.
        for &ti in &reg.targets {
            let t = xy(ti, self.w);
            let o = self.off[ti as usize];
            self.cost[ti as usize] = patch_ssd(content, src, t, [t[0] + o[0], t[1] + o[1]], r);
        }
        // Jump-flood propagation: steps n/2, n/4, … 1.
        let mut step = (self.w.max(self.h).next_power_of_two() / 2).max(1) as i32;
        while step >= 1 {
            self.propagate(content, src, reg, r, step);
            if step == 1 {
                break;
            }
            step /= 2;
        }
        self.random_search(content, src, reg, r, pass);
    }

    /// One double-buffered jump-flood pass at `step`: for every target centre,
    /// adopt a neighbour's source offset (aligned to this centre) if it scores
    /// better. Reads the old field, writes the new — order-independent.
    fn propagate(&mut self, content: &Plane, src: &Plane, reg: &Regions, r: i32, step: i32) {
        let old = self.off.clone();
        let old_cost = self.cost.clone();
        for &ti in &reg.targets {
            let idx = ti as usize;
            let t = xy(ti, self.w);
            let mut best_off = old[idx];
            let mut best = old_cost[idx];
            for (dx, dy) in DIRS {
                let qx = clampi(t[0] + dx * step, self.w) as i32;
                let qy = clampi(t[1] + dy * step, self.h) as i32;
                let qi = qy as usize * self.w + qx as usize;
                let cand = old[qi]; // neighbour's target→source offset
                let sx = t[0] + cand[0];
                let sy = t[1] + cand[1];
                if !reg.is_source[clampi(sy, self.h) * self.w + clampi(sx, self.w)] {
                    continue;
                }
                let c = patch_ssd(content, src, t, [sx, sy], r);
                if c < best {
                    best = c;
                    best_off = cand;
                }
            }
            self.off[idx] = best_off;
            self.cost[idx] = best;
        }
    }

    /// Random-search pass: around each target's current source, sample at an
    /// exponentially shrinking radius (`R, R/2, … 1`), keeping any improvement.
    /// Each target draws from its own per-pixel stream (salted by `pass`), so the
    /// pass is order-independent and GPU-matchable.
    fn random_search(&mut self, content: &Plane, src: &Plane, reg: &Regions, r: i32, pass: u64) {
        let s32 = seed32(self.seed);
        let salt = SALT_SEARCH.wrapping_add(pass as u32);
        let max_r = self.w.max(self.h) as i32;
        for &ti in &reg.targets {
            let idx = ti as usize;
            let t = xy(ti, self.w);
            let mut best_off = self.off[idx];
            let mut best = self.cost[idx];
            let mut radius = max_r;
            let mut k = 0u32;
            while radius >= 1 {
                let jx = rand_range(s32, ti, salt, 2 * k, -radius, radius);
                let jy = rand_range(s32, ti, salt, 2 * k + 1, -radius, radius);
                let cx = t[0] + best_off[0] + jx;
                let cy = t[1] + best_off[1] + jy;
                let sx = clampi(cx, self.w) as i32;
                let sy = clampi(cy, self.h) as i32;
                if reg.is_source[sy as usize * self.w + sx as usize] {
                    let c = patch_ssd(content, src, t, [sx, sy], r);
                    if c < best {
                        best = c;
                        best_off = [sx - t[0], sy - t[1]];
                    }
                }
                radius /= 2;
                k += 1;
            }
            self.off[idx] = best_off;
            self.cost[idx] = best;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mask::Mask;

    #[test]
    fn ssd_of_identical_patches_is_zero() {
        let mut p = Plane::new(8, 8);
        for y in 0..8 {
            for x in 0..8 {
                p.set(x, y, [x as f32 / 8.0, y as f32 / 8.0, 0.5]);
            }
        }
        assert_eq!(patch_ssd(&p, &p, [4, 4], [4, 4], 2), 0.0);
        assert!(patch_ssd(&p, &p, [4, 4], [2, 2], 2) > 0.0);
    }

    #[test]
    fn search_finds_the_matching_stripe_source() {
        // Vertical 2-px stripes: a hole patch should match a same-phase source.
        let (w, h) = (32, 32);
        let mut img = Plane::new(w, h);
        for y in 0..h {
            for x in 0..w {
                let v = if (x / 2) % 2 == 0 { 0.9 } else { 0.1 };
                img.set(x, y, [v, v, v]);
            }
        }
        let mut bytes = vec![0u8; w * h];
        for y in 13..19 {
            for x in 13..19 {
                bytes[y * w + x] = 255;
            }
        }
        let mask = Mask::from_bytes(w, h, &bytes);
        let reg = Regions::build(&mask, 3);
        let mut nnf = Nnf::init(&img, &img, &reg, 3, 1);
        for it in 0..6 {
            nnf.e_step(&img, &img, &reg, 3, it);
        }
        // A hole centre should match a source at the same stripe phase ⇒ near-zero cost.
        let ti = 16 * w + 16;
        assert!(
            nnf.cost[ti] < 0.05,
            "stripe match cost too high: {}",
            nnf.cost[ti]
        );
        let o = nnf.off[ti];
        assert_eq!(
            (16 + o[0]).rem_euclid(2),
            16 % 2,
            "matched a phase-shifted stripe"
        );
    }
}
