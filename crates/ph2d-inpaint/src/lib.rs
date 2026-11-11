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
//! [`inpaint_cpu`] is the gold reference: counter-hash draws, radii that halve by
//! integer shift, the pyramid in sRGB float (no `pow`). Each level's EM loop runs
//! on premultiplied RGBA8 ([`Q8`]) with an integer SSD (ties to the lower source
//! index) and a Q16 vote, so the W2 GPU path (behind `feature = "gpu"`) gives the
//! SAME bytes (B38.3); the runtime picks GPU, falling back to CPU.
//!
//! # Contract
//! Input is straight sRGB `RGBA8` (`w*h*4`) plus an 8-bit mask (`w*h`, byte
//! `>= 128` ⇒ hole to fill). Output is `RGBA8` where the hole pixels are the
//! reconstruction (alpha carried, B38.2) and every known pixel is **byte-identical
//! to the input** — inpaint only ever changes the masked region.

#![forbid(unsafe_code)]

mod antigo;
#[cfg(feature = "gpu")]
mod gpu;
mod hash;
mod mask;
mod nnf;
mod plane;
mod q8;
mod rng;
mod vote;

#[cfg(feature = "gpu")]
pub use gpu::{GpuInpaintError, GpuInpainter};
pub use mask::{Mask, Regions};
pub use nnf::Nnf;
pub use plane::Plane;
pub use q8::{MAX_PATCH_RADIUS, Q8};
pub use rng::SplitMix64;

use mask::Mask as HoleMask;
use nnf::Nnf as Field;
use plane::Plane as Img;

/// Tunables for a single inpaint. [`Default`] is a sensible balance of quality
/// and speed for interactive use.
#[derive(Clone, Copy, Debug)]
pub struct InpaintParams {
    /// Patch half-size; the patch is `(2*patch_radius+1)²`. Default 3 (7×7); capped at
    /// [`MAX_PATCH_RADIUS`].
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

thread_local! {
    static ALFA_DE_ANTES: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static NIVEL_F32_DE_ANTES: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Prancha B38.3 only: `true` runs each level in the f32 arithmetic of before (float SSD, float vote,
/// CPU), so the before/after board is drawn in one process. Never set in the product.
#[doc(hidden)]
pub fn prancha_b38_3_nivel_f32_de_antes(on: bool) {
    NIVEL_F32_DE_ANTES.set(on);
}

/// Prancha B38 only: `true` puts back the alpha-blind heal of before (alpha read as opaque, the hole
/// written opaque), so the before/after board is drawn in one process. Never set in the product.
#[doc(hidden)]
pub fn prancha_b38_alfa_de_antes(on: bool) {
    ALFA_DE_ANTES.set(on);
}

#[inline]
fn to_u8(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

/// Decode straight sRGB `RGBA8` into a premultiplied RGBA float [`Plane`]: alpha is matched and
/// voted with the colour, so a heal at the edge of paint on a transparent layer carries the alpha
/// (B38.2). Opaque (`α = 255` ⇒ `1.0`) is the straight colour bit for bit.
fn plane_from_rgba(w: usize, h: usize, rgba: &[u8]) -> Img {
    let antes = ALFA_DE_ANTES.get();
    let mut p = Img::new(w, h);
    for (dst, src) in
        p.px.as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(rgba.as_chunks::<4>().0.iter())
    {
        let a = if antes {
            1.0
        } else {
            f32::from(src[3]) / 255.0
        };
        for k in 0..3 {
            dst[k] = f32::from(src[k]) / 255.0 * a;
        }
        dst[3] = a;
    }
    p
}

/// Mean premultiplied RGBA over the KNOWN pixels of `img` (seeds the coarsest hole). Falls back
/// to mid-grey if — degenerately — every pixel is a hole.
fn mean_known(img: &Img, mask: &HoleMask) -> [f32; 4] {
    let mut s = [0.0f32; 4];
    let mut n = 0u32;
    for y in 0..img.h {
        for x in 0..img.w {
            if !mask.hole[y * img.w + x] {
                let c = img.get(x as i32, y as i32);
                s[0] += c[0];
                s[1] += c[1];
                s[2] += c[2];
                s[3] += c[3];
                n += 1;
            }
        }
    }
    if n == 0 {
        return [0.5, 0.5, 0.5, 1.0];
    }
    let inv = 1.0 / n as f32;
    [s[0] * inv, s[1] * inv, s[2] * inv, s[3] / n as f32]
}

/// Re-encode: hole pixels take the reconstruction, back to straight alpha; known pixels
/// are copied byte-for-byte from the source so the untouched region is exact.
fn rgba_from_plane(content: &Img, mask: &HoleMask, src_rgba: &[u8]) -> Vec<u8> {
    let (w, h) = (content.w, content.h);
    let mut out = src_rgba.to_vec();
    for y in 0..h {
        for x in 0..w {
            if mask.hole[y * w + x] {
                let c = content.get(x as i32, y as i32);
                let o = (y * w + x) * 4;
                let a = if ALFA_DE_ANTES.get() {
                    255
                } else {
                    to_u8(c[3])
                };
                for k in 0..3 {
                    out[o + k] = if a == 0 { 0 } else { to_u8(c[k] / c[3]) };
                }
                out[o + 3] = a;
            }
        }
    }
    out
}

/// Drive the coarse-to-fine pyramid, delegating each level's EM loop to
/// `run_level` (the CPU or GPU backend). `run_level(content, src, mask, reg, r,
/// em_iters, seed)` fills the hole pixels of `content` in place. Returns the
/// finest-level reconstruction, or `None` when the image is unfillable (no
/// source patch survives even at the coarsest level, i.e. it is all hole).
/// The first level the backend refuses stops the pyramid with its error.
fn run_pyramid<F, E>(
    base_img: &Img,
    base_mask: &HoleMask,
    params: &InpaintParams,
    mut run_level: F,
) -> Result<Option<Img>, E>
where
    F: FnMut(&mut Q8, &Q8, &HoleMask, &Regions, i32, u32, u64) -> Result<(), E>,
{
    let r = params.patch_radius.min(MAX_PATCH_RADIUS) as i32;

    // Coarse-to-fine pyramid, index 0 = finest. Halve until the longest side
    // reaches `pyramid_min` (or a 1-px dimension).
    let stop = params.pyramid_min.max(2) as usize;
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
            return Ok(None);
        }
        start -= 1;
    }

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
        if NIVEL_F32_DE_ANTES.get() {
            antigo::nivel(
                &mut content,
                img,
                mask,
                &reg,
                r,
                params.em_iters,
                params.seed,
            );
        } else {
            let mut q = Q8::from_plane(&content);
            run_level(
                &mut q,
                &Q8::from_plane(img),
                mask,
                &reg,
                r,
                params.em_iters,
                params.seed,
            )?;
            content = q.to_plane();
        }
        prev = Some(content);
    }

    Ok(Some(prev.expect("the start level always runs")))
}

/// Decode, run the pyramid on `level`, re-encode — the one path every backend takes.
///
/// # Panics
/// If `rgba.len() != width*height*4` or `mask.len() != width*height`.
fn inpaint_with<F, E>(req: &InpaintRequest<'_>, level: F) -> Result<InpaintResult, E>
where
    F: FnMut(&mut Q8, &Q8, &HoleMask, &Regions, i32, u32, u64) -> Result<(), E>,
{
    let Some((base_img, base_mask)) = decode_request(req) else {
        return Ok(InpaintResult {
            rgba: req.rgba.to_vec(),
        });
    };
    Ok(
        match run_pyramid(&base_img, &base_mask, &req.params, level)? {
            Some(result) => InpaintResult {
                rgba: rgba_from_plane(&result, &base_mask, req.rgba),
            },
            None => InpaintResult {
                rgba: req.rgba.to_vec(),
            },
        },
    )
}

/// A backend that refused a level hands the WHOLE request to [`inpaint_cpu`]: the answer is then the
/// reference byte for byte, never a mix of the two backends' levels.
fn or_the_cpu<E: std::fmt::Display>(
    req: &InpaintRequest<'_>,
    attempt: Result<InpaintResult, E>,
) -> InpaintResult {
    attempt.unwrap_or_else(|e| {
        eprintln!("[ph2d-inpaint] {e}; falling back to the CPU reference");
        inpaint_cpu(req)
    })
}

/// The CPU backend for one level: EM loop (E = jump-flood NNF search, M = vote).
fn cpu_level(
    content: &mut Q8,
    src: &Q8,
    mask: &HoleMask,
    reg: &Regions,
    r: i32,
    em_iters: u32,
    seed: u64,
) {
    let mut nnf = Field::init(content, src, reg, r, seed);
    for it in 0..em_iters {
        nnf.e_step(content, src, reg, r, u64::from(it));
        vote::vote(content, src, mask, reg, &nnf, r);
    }
}

/// Inpaint on the CPU — the deterministic gold reference.
///
/// # Panics
/// If `rgba.len() != width*height*4` or `mask.len() != width*height`.
pub fn inpaint_cpu(req: &InpaintRequest<'_>) -> InpaintResult {
    let Ok(out) = inpaint_with(req, |c, s, m, reg, r, it, seed| {
        cpu_level(c, s, m, reg, r, it, seed);
        Ok::<(), std::convert::Infallible>(())
    });
    out
}

/// Validate + decode a request into `(image, mask)`. Returns `None` when there
/// is no hole (the caller should return the input unchanged).
///
/// # Panics
/// If `rgba.len() != width*height*4` or `mask.len() != width*height`.
fn decode_request(req: &InpaintRequest<'_>) -> Option<(Img, HoleMask)> {
    let w = req.width as usize;
    let h = req.height as usize;
    let n = w * h;
    assert_eq!(req.rgba.len(), n * 4, "rgba must be width*height*4");
    assert_eq!(req.mask.len(), n, "mask must be width*height");
    let base_mask = HoleMask::from_bytes(w, h, req.mask);
    if !base_mask.has_hole() {
        return None;
    }
    Some((plane_from_rgba(w, h, req.rgba), base_mask))
}

/// Inpaint on the GPU (jump-flood NNF + voting compute), the same bytes as the CPU
/// reference (B38.3), on pipelines compiled once by the caller ([`GpuInpainter::new`]).
/// Prefer [`inpaint`] / [`inpaint_on`] for the runtime GPU-or-CPU choice.
///
/// # Errors
/// The first level the device cannot finish ([`GpuInpaintError`]).
#[cfg(feature = "gpu")]
pub fn inpaint_gpu_with(
    gpu: &ph2d_gpu::GpuContext,
    painter: &GpuInpainter,
    req: &InpaintRequest<'_>,
) -> Result<InpaintResult, GpuInpaintError> {
    inpaint_with(req, |content, src, mask, reg, r, em_iters, seed| {
        painter.run_level(
            gpu,
            content,
            src,
            mask,
            reg,
            r,
            em_iters,
            hash::seed32(seed),
        )
    })
}

/// [`inpaint_gpu_with`] compiling the pipelines for this one call.
///
/// # Errors
/// The first level the device cannot finish ([`GpuInpaintError`]).
#[cfg(feature = "gpu")]
pub fn inpaint_gpu(
    gpu: &ph2d_gpu::GpuContext,
    req: &InpaintRequest<'_>,
) -> Result<InpaintResult, GpuInpaintError> {
    inpaint_gpu_with(gpu, &GpuInpainter::new(gpu), req)
}

/// GPU with CPU fallback on pipelines the session keeps: a level the device refuses
/// hands the whole request to [`inpaint_cpu`].
#[cfg(feature = "gpu")]
pub fn inpaint_on(
    gpu: &ph2d_gpu::GpuContext,
    painter: &GpuInpainter,
    req: &InpaintRequest<'_>,
) -> InpaintResult {
    or_the_cpu(req, inpaint_gpu_with(gpu, painter, req))
}

/// Inpaint with the GPU when a [`GpuContext`](ph2d_gpu::GpuContext) is available,
/// otherwise the CPU reference — the runtime "GPU with CPU fallback" entry point
/// (`gpu = None` ⇒ pure CPU; a level the device refuses ⇒ [`inpaint_cpu`]).
#[cfg(feature = "gpu")]
pub fn inpaint(gpu: Option<&ph2d_gpu::GpuContext>, req: &InpaintRequest<'_>) -> InpaintResult {
    match gpu {
        Some(g) => or_the_cpu(req, inpaint_gpu(g, req)),
        None => inpaint_cpu(req),
    }
}

#[cfg(test)]
mod tests;
