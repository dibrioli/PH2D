//! The M-step: reconstruct every hole pixel as the weighted average of all
//! overlapping matched source patches (Wexler/Simakov voting). Each target
//! centre `t` maps to a source centre `s = t + off`; for every offset `(dx,dy)`
//! in the patch, the source colour `src(s+dx, s+dy)` votes for the pixel
//! `t+(dx,dy)` — but only if that pixel is a HOLE pixel (known pixels stay
//! pinned to the real image). The weight is `1/(1+cost)` so sharper matches
//! count more; it is division-only (transcendental-free), so the GPU voting
//! kernel reproduces it within ε.
//!
//! Covered pixels that fall outside the image are **skipped** (not clamped): the
//! source read still clamps, but a target patch never scatters onto a clamped
//! border pixel. This makes the scatter here identical to the equivalent GPU
//! *gather* (for each hole pixel `p`, sum over target centres `c ∈ [p-r, p+r]²`),
//! which has no out-of-bounds coverage to clamp — so CPU and GPU voting agree.

use super::nnf::Nnf;
use crate::mask::{Mask, Regions};
use crate::plane::Plane;

/// Vote hole pixels of `content` in place. `src` is the fixed level image, `nnf`
/// the just-searched field, `r` the patch radius.
pub fn vote(content: &mut Plane, src: &Plane, mask: &Mask, reg: &Regions, nnf: &Nnf, r: i32) {
    let (w, h) = (content.w, content.h);
    let mut sum = vec![[0.0f32; 4]; w * h];
    let mut wsum = vec![0.0f32; w * h];

    for &ti in &reg.targets {
        let idx = ti as usize;
        let tx = (idx % w) as i32;
        let ty = (idx / w) as i32;
        let o = nnf.off[idx];
        let weight = 1.0 / (1.0 + nnf.cost[idx]);
        for dy in -r..=r {
            for dx in -r..=r {
                let px = tx + dx;
                let py = ty + dy;
                if px < 0 || px >= w as i32 || py < 0 || py >= h as i32 {
                    continue; // covered pixel off-image ⇒ skip (matches GPU gather)
                }
                let vi = py as usize * w + px as usize;
                if !mask.hole[vi] {
                    continue; // never overwrite a known pixel
                }
                let col = src.get(tx + o[0] + dx, ty + o[1] + dy);
                for k in 0..4 {
                    sum[vi][k] += weight * col[k];
                }
                wsum[vi] += weight;
            }
        }
    }

    for y in 0..h {
        for x in 0..w {
            let vi = y * w + x;
            if mask.hole[vi] && wsum[vi] > 0.0 {
                let iw = 1.0 / wsum[vi];
                // Alpha divides (not × iw): an all-opaque vote is then exactly 1, the colour unchanged.
                let s = sum[vi];
                content.set(x, y, [s[0] * iw, s[1] * iw, s[2] * iw, s[3] / wsum[vi]]);
            }
        }
    }
}
