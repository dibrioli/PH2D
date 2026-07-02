//! `ph2d-inpaint` — exemplar-based image inpainting (defect correction).
//!
//! Fill a masked hole by copying real pixels from the rest of the image, using
//! **multi-scale PatchMatch** (Barnes et al. 2009) with a **jump-flooding**
//! nearest-neighbour field and **EM voting** reconstruction (Wexler/Simakov).
//! This is the classical, non-ML formulation behind Photoshop's Content-Aware
//! Fill; it beats AI inpainting on high-res / repetitive textures because it
//! copies instead of hallucinating (see ADR-0102).
//!
//! # Determinism
//! [`inpaint_cpu`] is the gold reference: seeded [`rng::SplitMix64`], SSD that
//! never takes a `sqrt`, radii that halve by integer shift, sRGB-float averaging
//! (no `pow`). Same seed ⇒ byte-identical output on every platform (HR-5). The
//! W2 GPU compute path (behind `feature = "gpu"`) runs the same jump-flood passes
//! and reconciles within float ε; the runtime picks GPU, falling back to CPU.
//!
//! # Contract
//! Input is straight sRGB `RGBA8` (`w*h*4`) plus an 8-bit mask (`w*h`, byte
//! `>= 128` ⇒ hole to fill). Output is `RGBA8` where the hole pixels are the
//! reconstruction (alpha forced opaque) and every known pixel is **byte-identical
//! to the input** — inpaint only ever changes the masked region.

#![forbid(unsafe_code)]

mod mask;
mod nnf;
mod plane;
mod rng;
mod vote;

pub use mask::{Mask, Regions};
pub use nnf::Nnf;
pub use plane::Plane;
pub use rng::SplitMix64;

use mask::Mask as HoleMask;
use nnf::Nnf as Field;
use plane::Plane as Img;

/// Tunables for a single inpaint. [`Default`] is a sensible balance of quality
/// and speed for interactive use.
#[derive(Clone, Copy, Debug)]
pub struct InpaintParams {
    /// Patch half-size; the patch is `(2*patch_radius+1)²`. Default 3 (7×7).
    pub patch_radius: u32,
    /// EM iterations per pyramid level (E = NNF search, M = voting). Default 6.
    pub em_iters: u32,
    /// Longest side of the coarsest pyramid level; the pyramid stops halving
    /// once it reaches this. Default 16.
    pub pyramid_min: u32,
    /// RNG seed — fix it for reproducible output. Default `0x1234_5678`.
    pub seed: u64,
}

impl Default for InpaintParams {
    fn default() -> Self {
        Self {
            patch_radius: 3,
            em_iters: 6,
            pyramid_min: 16,
            seed: 0x1234_5678,
        }
    }
}

/// A single inpaint request. `rgba` is `w*h*4` straight sRGB; `mask` is `w*h`
/// (byte `>= 128` ⇒ hole).
#[derive(Clone, Copy, Debug)]
pub struct InpaintRequest<'a> {
    pub width: u32,
    pub height: u32,
    pub rgba: &'a [u8],
    pub mask: &'a [u8],
    pub params: InpaintParams,
}

/// The reconstructed image (`w*h*4` sRGB `RGBA8`).
#[derive(Clone, Debug)]
pub struct InpaintResult {
    pub rgba: Vec<u8>,
}

#[inline]
fn to_u8(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

/// Decode straight sRGB `RGBA8` into an RGB float [`Plane`] (alpha dropped — the
/// algorithm reconstructs colour; alpha is restored from the source at the end).
fn plane_from_rgba(w: usize, h: usize, rgba: &[u8]) -> Img {
    let mut p = Img::new(w, h);
    for (dst, src) in p.px.chunks_exact_mut(3).zip(rgba.chunks_exact(4)) {
        dst[0] = f32::from(src[0]) / 255.0;
        dst[1] = f32::from(src[1]) / 255.0;
        dst[2] = f32::from(src[2]) / 255.0;
    }
    p
}

/// Mean RGB over the KNOWN pixels of `img` (seeds the coarsest hole). Falls back
/// to mid-grey if — degenerately — every pixel is a hole.
fn mean_known(img: &Img, mask: &HoleMask) -> [f32; 3] {
    let mut s = [0.0f32; 3];
    let mut n = 0u32;
    for y in 0..img.h {
        for x in 0..img.w {
            if !mask.hole[y * img.w + x] {
                let c = img.get(x as i32, y as i32);
                s[0] += c[0];
                s[1] += c[1];
                s[2] += c[2];
                n += 1;
            }
        }
    }
    if n == 0 {
        return [0.5, 0.5, 0.5];
    }
    let inv = 1.0 / n as f32;
    [s[0] * inv, s[1] * inv, s[2] * inv]
}

/// Re-encode: hole pixels take the reconstruction (alpha opaque); known pixels
/// are copied byte-for-byte from the source so the untouched region is exact.
fn rgba_from_plane(content: &Img, mask: &HoleMask, src_rgba: &[u8]) -> Vec<u8> {
    let (w, h) = (content.w, content.h);
    let mut out = src_rgba.to_vec();
    for y in 0..h {
        for x in 0..w {
            if mask.hole[y * w + x] {
                let c = content.get(x as i32, y as i32);
                let o = (y * w + x) * 4;
                out[o] = to_u8(c[0]);
                out[o + 1] = to_u8(c[1]);
                out[o + 2] = to_u8(c[2]);
                out[o + 3] = 255;
            }
        }
    }
    out
}

/// Inpaint on the CPU — the deterministic gold reference.
///
/// # Panics
/// If `rgba.len() != width*height*4` or `mask.len() != width*height`.
pub fn inpaint_cpu(req: &InpaintRequest<'_>) -> InpaintResult {
    let w = req.width as usize;
    let h = req.height as usize;
    let n = w * h;
    assert_eq!(req.rgba.len(), n * 4, "rgba must be width*height*4");
    assert_eq!(req.mask.len(), n, "mask must be width*height");
    let r = req.params.patch_radius as i32;

    let base_img = plane_from_rgba(w, h, req.rgba);
    let base_mask = HoleMask::from_bytes(w, h, req.mask);
    if !base_mask.has_hole() {
        return InpaintResult {
            rgba: req.rgba.to_vec(),
        };
    }

    // Coarse-to-fine pyramid, index 0 = finest. Halve until the longest side
    // reaches `pyramid_min` (or a 1-px dimension).
    let stop = req.params.pyramid_min.max(2) as usize;
    let mut planes = vec![base_img.clone()];
    let mut masks = vec![base_mask.clone()];
    loop {
        let last = planes.last().expect("non-empty");
        if last.w.max(last.h) <= stop || last.w <= 1 || last.h <= 1 {
            break;
        }
        planes.push(last.downsample());
        masks.push(masks.last().expect("non-empty").downsample());
    }

    // Start at the coarsest level that still has a usable source patch. Coarser
    // levels grow the hole (`max`-downsample), so a level with no source is
    // unfillable — descend until one has sources, else bail (all hole).
    let mut start = planes.len() - 1;
    loop {
        if !Regions::build(&masks[start], r).sources.is_empty() {
            break;
        }
        if start == 0 {
            return InpaintResult {
                rgba: req.rgba.to_vec(),
            };
        }
        start -= 1;
    }

    let mut rng = SplitMix64::new(req.params.seed);
    let mut prev: Option<Img> = None;
    for lvl in (0..=start).rev() {
        let img = &planes[lvl];
        let mask = &masks[lvl];
        let reg = Regions::build(mask, r);

        // Seed the hole: upsample the coarser result, or mean-fill at the top.
        let mut content = img.clone();
        match &prev {
            Some(p) => {
                let up = p.upsample_to(img.w, img.h);
                for y in 0..img.h {
                    for x in 0..img.w {
                        if mask.hole[y * img.w + x] {
                            content.set(x, y, up.get(x as i32, y as i32));
                        }
                    }
                }
            }
            None => {
                let m = mean_known(img, mask);
                for y in 0..img.h {
                    for x in 0..img.w {
                        if mask.hole[y * img.w + x] {
                            content.set(x, y, m);
                        }
                    }
                }
            }
        }

        if reg.sources.is_empty() {
            prev = Some(content); // defensive; finer levels always have sources
            continue;
        }
        let mut nnf = Field::init(&content, img, &reg, r, &mut rng);
        for _ in 0..req.params.em_iters {
            nnf.e_step(&content, img, &reg, r, &mut rng);
            vote::vote(&mut content, img, mask, &reg, &nnf, r);
        }
        prev = Some(content);
    }

    let result = prev.expect("the start level always runs");
    InpaintResult {
        rgba: rgba_from_plane(&result, &base_mask, req.rgba),
    }
}

#[cfg(test)]
mod tests;
