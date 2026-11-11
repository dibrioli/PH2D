//! The M-step: reconstruct every hole pixel as the weighted average of all
//! overlapping matched source patches (Wexler/Simakov voting). Each target
//! centre `t` maps to a source centre `s = t + off`; for every offset `(dx,dy)`
//! in the patch, the source colour `src(s+dx, s+dy)` votes for the pixel
//! `t+(dx,dy)` — but only if that pixel is a HOLE pixel (known pixels stay
//! pinned to the real image). The weight is `1/(1+cost)` in Q16 fixed point and
//! the mean rounds half up, all `u32`, so the GPU vote is the same byte (B38.3).
//!
//! Covered pixels that fall outside the image are **skipped** (not clamped): the
//! source read still clamps, but a target patch never scatters onto a clamped
//! border pixel. This makes the scatter here identical to the equivalent GPU
//! *gather* (for each hole pixel `p`, sum over target centres `c ∈ [p-r, p+r]²`),
//! which has no out-of-bounds coverage to clamp — so CPU and GPU voting agree.

use crate::mask::{Mask, Regions};
use crate::nnf::Nnf;
use crate::q8::{Q8, media, peso};

/// Vote hole pixels of `content` in place. `src` is the fixed level image, `nnf`
/// the just-searched field, `r` the patch radius.
pub fn vote(content: &mut Q8, src: &Q8, mask: &Mask, reg: &Regions, nnf: &Nnf, r: i32) {
    let (w, h) = (content.w, content.h);
    let mut sum = vec![[0u32; 4]; w * h];
    let mut wsum = vec![0u32; w * h];

    for &ti in &reg.targets {
        let idx = ti as usize;
        let tx = (idx % w) as i32;
        let ty = (idx / w) as i32;
        let o = nnf.off[idx];
        let weight = peso(nnf.cost[idx]);
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
                    sum[vi][k] += weight * u32::from(col[k]);
                }
                wsum[vi] += weight;
            }
        }
    }

    for y in 0..h {
        for x in 0..w {
            let vi = y * w + x;
            if mask.hole[vi] && wsum[vi] > 0 {
                content.px[vi] = sum[vi].map(|s| media(s, wsum[vi]));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voting_a_constant_image_reproduces_the_constant() {
        let (w, h) = (24, 24);
        let img = Q8 {
            w,
            h,
            px: vec![[102, 179, 51, 255]; w * h],
        };
        let mut bytes = vec![0u8; w * h];
        for y in 10..14 {
            for x in 10..14 {
                bytes[y * w + x] = 255;
            }
        }
        let mask = Mask::from_bytes(w, h, &bytes);
        let reg = Regions::build(&mask, 3);
        // Content starts with the hole zeroed.
        let mut content = img.clone();
        for y in 10..14 {
            for x in 10..14 {
                content.px[y * w + x] = [0; 4];
            }
        }
        let nnf = Nnf::init(&content, &img, &reg, 3, 3);
        vote(&mut content, &img, &mask, &reg, &nnf, 3);
        // Any source patch is the constant colour, so the hole fills to it.
        for y in 10..14 {
            for x in 10..14 {
                assert_eq!(content.get(x, y), [102, 179, 51, 255], "at ({x},{y})");
            }
        }
    }
}
