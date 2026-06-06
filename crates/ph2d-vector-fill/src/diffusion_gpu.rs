//! GPU diffusion path — Walk-on-Spheres (WoS) reference + dispatch data (ADR-0060
//! §2.5, file split off `diffusion_curve.rs`'s "GPU dispatch" role for cohesion).
//!
//! Where [`crate::poisson_cpu`] is the deterministic *multigrid* solver (the
//! Mobile-Core tier and golden reference), this module is the *Monte-Carlo* path
//! the Heavy/Standard/Lite GPU tiers run: each pixel estimates the harmonic field
//! value by averaging many **walk-on-spheres** random walks that terminate on the
//! nearest diffusion curve and return that curve's side colour.
//!
//! ## What lives here vs. the renderer (isolation)
//!
//! This module owns everything that needs **no GPU**: the WGSL shader source
//! ([`DIFFUSION_WGSL`] / [`BILATERAL_UPSAMPLE_WGSL`], naga-validated in tests),
//! the storage-buffer packing ([`pack_curves`]), the dispatch uniform
//! ([`DiffusionParams`]), the tier matrix ([`DiffusionTier::plan`]), and a **CPU
//! reference WoS** ([`walk_on_spheres_field`] / [`wos_estimate_point`]) that
//! mirrors the shader line-for-line. The actual wgpu pipeline / dispatch / texture
//! upload is renderer wiring (the Coordenador's `ph2d-render`).
//!
//! ## Why the CPU reference matters
//!
//! It lets us prove the WoS *algorithm* correct **without a GPU**: with the same
//! curves, the Monte-Carlo estimate converges (in expectation) to the multigrid
//! field — and the test [`tests::wos_converges_to_multigrid`] checks exactly that
//! at the symmetry point where the two agree independent of discretization. The
//! RNG is [`ph2d_expr::eval::noise1`] — the integer hash that is **bit-identical
//! CPU↔GPU** — so a deterministic-mode GPU run can match this reference bit for
//! bit (ADR-0060 §2.6).

use core::f32::consts::TAU;

use glam::Vec2;
use ph2d_color::OklabColor;

use crate::diffusion_curve::DiffusionCurveSet;
use crate::poisson_cpu::ColorField;

/// The WoS Monte-Carlo compute shader (storage-buffer output; mirrors
/// [`walk_on_spheres_field`]). Prepend [`ph2d_expr::wgsl_prelude`] before
/// validating/using — it provides the `ph2d_noise1` RNG the walks share with the
/// CPU reference.
pub const DIFFUSION_WGSL: &str = include_str!("../shaders/diffusion.wgsl");

/// The 2-pass joint-bilateral upsample shader (denoise low-res WoS, then guided
/// upscale) — ADR-0060 §2.5 JBU. Self-contained WGSL (no prelude needed).
pub const BILATERAL_UPSAMPLE_WGSL: &str = include_str!("../shaders/bilateral_upsample.wgsl");

// ───────────────────────────── tier matrix (§2.5) ──────────────────────────

/// Which solver a device tier runs for a diffusion-curve fill (ADR-0060 §2.5).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DiffusionAlgorithm {
    /// Walk-on-Spheres on the GPU (Heavy/Standard/Lite/Web).
    WosGpu,
    /// The [`crate::poisson_cpu`] multigrid fallback (Mobile-Core, no compute).
    MultigridCpu,
}

/// Device capability tier (ADR-0060 §2.5 — five rows of the tier matrix).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DiffusionTier {
    /// Desktop: 1080p, 64 spp, ≤5 ms.
    Heavy,
    /// iPad Pro: 540p (0.5×), 32 spp, ≤4 ms + JBU.
    Standard,
    /// Android top: 270p (0.25×), 16 spp, ≤3 ms + JBU.
    Lite,
    /// Entry mobile: CPU multigrid (no compute), ≤15 ms off-thread.
    MobileCore,
    /// WebGPU: matches Standard (or CPU fallback if compute is slow).
    Web,
}

/// The resolved plan for a tier: where/how dense to solve, and with what.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DiffusionPlan {
    /// Fraction of display resolution to solve at (`1.0` / `0.5` / `0.25`).
    pub solve_scale: f32,
    /// WoS samples-per-pixel (`0` for the multigrid path).
    pub spp: u32,
    pub algorithm: DiffusionAlgorithm,
    /// Whether the result is joint-bilateral upsampled to display resolution.
    pub use_jbu: bool,
    /// The tier's per-frame budget in milliseconds (the renderer bench gate
    /// `vector_diffusion_curve_tier_budget` asserts against this).
    pub budget_ms: f32,
}

impl DiffusionTier {
    /// The tier's resolved [`DiffusionPlan`], straight from the ADR-0060 §2.5
    /// matrix.
    pub fn plan(self) -> DiffusionPlan {
        match self {
            DiffusionTier::Heavy => DiffusionPlan {
                solve_scale: 1.0,
                spp: 64,
                algorithm: DiffusionAlgorithm::WosGpu,
                use_jbu: false,
                budget_ms: 5.0,
            },
            DiffusionTier::Standard => DiffusionPlan {
                solve_scale: 0.5,
                spp: 32,
                algorithm: DiffusionAlgorithm::WosGpu,
                use_jbu: true,
                budget_ms: 4.0,
            },
            DiffusionTier::Lite => DiffusionPlan {
                solve_scale: 0.25,
                spp: 16,
                algorithm: DiffusionAlgorithm::WosGpu,
                use_jbu: true,
                budget_ms: 3.0,
            },
            DiffusionTier::MobileCore => DiffusionPlan {
                solve_scale: 1.0,
                spp: 0,
                algorithm: DiffusionAlgorithm::MultigridCpu,
                use_jbu: false,
                budget_ms: 15.0,
            },
            DiffusionTier::Web => DiffusionPlan {
                solve_scale: 0.5,
                spp: 32,
                algorithm: DiffusionAlgorithm::WosGpu,
                use_jbu: true,
                budget_ms: 4.0,
            },
        }
    }
}

// ─────────────────────── dispatch data (Pod, std140) ───────────────────────

/// Per-dispatch uniform the WoS shader binds (`@group(0) @binding(1)`). `#[repr(C,
/// align(16))]`, 32 bytes — a clean two-`vec4` std140 block, no padding holes.
#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DiffusionParams {
    pub width: u32,
    pub height: u32,
    /// Number of [`GpuSegment`]s in the storage buffer.
    pub segment_count: u32,
    /// WoS samples per pixel.
    pub spp: u32,
    /// Walk-length cap before falling back to the nearest curve colour.
    pub max_steps: u32,
    /// Global RNG seed (folded into the per-walk `ph2d_noise1` key).
    pub seed: u32,
    /// Termination radius: a walk within `epsilon` of a curve is absorbed.
    pub epsilon: f32,
    pub _pad: f32,
}

impl DiffusionParams {
    /// A sensible default for a `width × height` solve (32 spp, 64-step cap,
    /// `epsilon` ≈ one texel).
    pub fn new(width: u32, height: u32, segment_count: u32, spp: u32, seed: u32) -> Self {
        let epsilon = 1.0 / (width.max(height).max(2) - 1) as f32;
        Self {
            width,
            height,
            segment_count,
            spp,
            max_steps: 64,
            seed,
            epsilon,
            _pad: 0.0,
        }
    }

    /// Zero-copy bytes for `write_buffer`.
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

/// One flattened curve segment as the WoS storage buffer sees it
/// (`@group(0) @binding(0)`). `#[repr(C, align(16))]`, 48 bytes = three `vec4`s:
/// the endpoints, then the OKLab colour on each side (evaluated at the segment's
/// arc-length midpoint).
#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuSegment {
    /// `[ax, ay, bx, by]` in normalized `[0,1]²` fill space.
    pub endpoints: [f32; 4],
    /// `+normal` ("left") side OKLab `[L, a, b, alpha]`.
    pub left: [f32; 4],
    /// `-normal` ("right") side OKLab `[L, a, b, alpha]`.
    pub right: [f32; 4],
}

/// Pack a curve set into the WoS storage buffer: one [`GpuSegment`] per polyline
/// edge, each carrying the side colours at the edge's midpoint parameter.
pub fn pack_curves(set: &DiffusionCurveSet) -> Vec<GpuSegment> {
    let mut out = Vec::new();
    for curve in &set.curves {
        if !curve.is_valid() {
            continue;
        }
        let total = curve.arc_length().max(f32::EPSILON);
        let mut acc = 0.0_f32;
        for seg in curve.points.windows(2) {
            let (a, b) = (seg[0], seg[1]);
            let seg_len = (b - a).length();
            if seg_len < f32::EPSILON {
                continue;
            }
            let t_mid = (acc + 0.5 * seg_len) / total;
            let left = curve.left_color_at(t_mid);
            let right = curve.right_color_at(t_mid);
            out.push(GpuSegment {
                endpoints: [a.x, a.y, b.x, b.y],
                left: [left.l, left.a, left.b, left.alpha],
                right: [right.l, right.a, right.b, right.alpha],
            });
            acc += seg_len;
        }
    }
    out
}

// ─────────────────────── CPU Walk-on-Spheres reference ──────────────────────

/// The nearest-curve query result: distance to the curve and the OKLab colour of
/// the side the query point is on.
#[derive(Copy, Clone, Debug)]
struct Hit {
    dist: f32,
    lab: [f32; 3],
}

/// The WoS knobs that ride together (`spp`, walk-length cap, absorption radius,
/// RNG seed) — grouped so the estimator signatures stay tidy and so callers
/// configure a solve as one value.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WosConfig {
    pub spp: u32,
    pub max_steps: u32,
    pub epsilon: f32,
    pub seed: u32,
}

impl WosConfig {
    pub fn new(spp: u32, max_steps: u32, epsilon: f32, seed: u32) -> Self {
        Self {
            spp,
            max_steps,
            epsilon,
            seed,
        }
    }
}

/// Solve a full WoS field on the CPU (the GPU shader's exact mirror). Slow for
/// large grids × high `spp` — for point queries use [`wos_estimate_point`].
/// Empty input → a transparent field.
pub fn walk_on_spheres_field(
    set: &DiffusionCurveSet,
    width: usize,
    height: usize,
    cfg: WosConfig,
) -> ColorField {
    let segs = pack_curves(set);
    if segs.is_empty() {
        return ColorField::transparent(width, height);
    }
    let mut texel = vec![[0.0_f32; 4]; width * height];
    for y in 0..height {
        for x in 0..width {
            let uv = Vec2::new(
                x as f32 / (width - 1).max(1) as f32,
                y as f32 / (height - 1).max(1) as f32,
            );
            let lab = wos_estimate(&segs, uv, cfg, x as u32, y as u32);
            texel[y * width + x] = OklabColor::new(lab[0], lab[1], lab[2], 1.0)
                .to_linear()
                .as_array();
        }
    }
    ColorField {
        w: width,
        h: height,
        texel,
    }
}

/// Estimate the diffused **linear-RGBA** colour at a single normalized point —
/// the cheap path for validation / sparse queries. `px`/`py` decorrelate the
/// per-pixel RNG stream (pass the pixel index, or any stable tag).
pub fn wos_estimate_point(
    set: &DiffusionCurveSet,
    uv: Vec2,
    cfg: WosConfig,
    px: u32,
    py: u32,
) -> [f32; 4] {
    let segs = pack_curves(set);
    if segs.is_empty() {
        return [0.0; 4];
    }
    let lab = wos_estimate(&segs, uv, cfg, px, py);
    OklabColor::new(lab[0], lab[1], lab[2], 1.0)
        .to_linear()
        .as_array()
}

/// Average `cfg.spp` walk-on-spheres estimates of the OKLab field value at `p`.
fn wos_estimate(segs: &[GpuSegment], p: Vec2, cfg: WosConfig, px: u32, py: u32) -> [f32; 3] {
    let mut acc = [0.0_f32; 3];
    let samples = cfg.spp.max(1);
    for s in 0..samples {
        let mut x = p;
        let mut color = nearest(segs, x).lab; // fallback if the walk never absorbs
        for step in 0..cfg.max_steps {
            let hit = nearest(segs, x);
            if hit.dist <= cfg.epsilon {
                color = hit.lab;
                break;
            }
            let theta = rand01(px, py, s, step, cfg.seed) * TAU;
            x += Vec2::new(theta.cos(), theta.sin()) * hit.dist;
            color = hit.lab; // most-recent nearest, used if max_steps is hit
        }
        for (a, c) in acc.iter_mut().zip(color) {
            *a += c;
        }
    }
    let inv = 1.0 / samples as f32;
    [acc[0] * inv, acc[1] * inv, acc[2] * inv]
}

/// Nearest segment to `p`: its distance and the OKLab colour of the side `p` is
/// on (`+normal` → `left`, else `right`). Mirrors the WGSL `nearest`.
fn nearest(segs: &[GpuSegment], p: Vec2) -> Hit {
    let mut best_d2 = f32::INFINITY;
    let mut best = [0.0_f32; 3];
    for seg in segs {
        let a = Vec2::new(seg.endpoints[0], seg.endpoints[1]);
        let b = Vec2::new(seg.endpoints[2], seg.endpoints[3]);
        let ab = b - a;
        let len2 = ab.length_squared().max(f32::EPSILON);
        let t = ((p - a).dot(ab) / len2).clamp(0.0, 1.0);
        let cp = a + ab * t;
        let d2 = (p - cp).length_squared();
        if d2 < best_d2 {
            best_d2 = d2;
            // normal = rotate90(tangent); side = sign of dot(p - cp, normal).
            let normal = Vec2::new(-ab.y, ab.x);
            let side = (p - cp).dot(normal);
            let col = if side >= 0.0 { seg.left } else { seg.right };
            best = [col[0], col[1], col[2]];
        }
    }
    Hit {
        dist: best_d2.sqrt(),
        lab: best,
    }
}

/// One uniform `[0,1)` sample from the shared `ph2d_noise1` integer hash, keyed
/// so `(pixel, sample, step, seed)` each get a decorrelated stream. The WGSL
/// computes the identical key, so deterministic-mode GPU == this reference.
#[inline]
fn rand01(px: u32, py: u32, s: u32, step: u32, seed: u32) -> f32 {
    let key = px as f32 * 12.9898
        + py as f32 * 78.233
        + s as f32 * 37.719
        + step as f32 * 0.618_034
        + seed as f32 * 0.314_159;
    ph2d_expr::eval::noise1(key)
}

// ──────────────────────────────── tests ────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diffusion_curve::DiffusionCurve;
    use crate::poisson_cpu::{Resolution, solve_color_field_cycles};
    use ph2d_color::OklchColor;

    /// Two full-height walls (a channel): the field between them is a smooth
    /// red→blue ramp, so WoS has *real* Monte-Carlo variance — unlike a single
    /// wall, which is a variance-free step. Orientation comes out of the same
    /// `DiffusionCurveSet` both solvers consume, so it is consistent by
    /// construction.
    fn channel_set() -> DiffusionCurveSet {
        let red = OklchColor::opaque(0.63, 0.26, 29.0);
        let blue = OklchColor::opaque(0.45, 0.31, 264.0);
        DiffusionCurveSet::from_curves([
            DiffusionCurve::straight(Vec2::new(0.25, 0.0), Vec2::new(0.25, 1.0), red, blue),
            DiffusionCurve::straight(Vec2::new(0.75, 0.0), Vec2::new(0.75, 1.0), red, blue),
        ])
    }

    #[test]
    fn wos_converges_to_multigrid() {
        // At the channel centre (x=0.5) the harmonic field is the symmetric
        // midpoint regardless of either solver's discretization, so WoS's
        // Monte-Carlo estimate must match the multigrid golden there to within
        // the Monte-Carlo standard error (spp=1024 → ≈0.01 in OKLab).
        let set = channel_set();
        let res = Resolution::square(129).unwrap();
        let mg = solve_color_field_cycles(&set, res, 24);
        let mg_center = mg.sample(Vec2::new(0.5, 0.5));

        let eps = 1.0 / 128.0;
        let cfg = WosConfig::new(1024, 96, eps, 1);
        let wos_center = wos_estimate_point(&set, Vec2::new(0.5, 0.5), cfg, 64, 64);

        for k in 0..3 {
            assert!(
                (wos_center[k] - mg_center[k]).abs() < 0.03,
                "channel k={k}: WoS {} vs multigrid {} (centre)",
                wos_center[k],
                mg_center[k]
            );
        }
    }

    #[test]
    fn wos_produces_a_gradient() {
        // Left of centre leans one colour, right the other — a real ramp exists.
        let set = channel_set();
        let cfg = WosConfig::new(512, 96, 1.0 / 128.0, 1);
        let left = wos_estimate_point(&set, Vec2::new(0.30, 0.5), cfg, 30, 64);
        let right = wos_estimate_point(&set, Vec2::new(0.70, 0.5), cfg, 70, 64);
        // The two ends differ markedly in at least one linear channel.
        let spread: f32 = (0..3)
            .map(|k| (left[k] - right[k]).abs())
            .fold(0.0, f32::max);
        assert!(spread > 0.05, "expected a visible ramp, spread = {spread}");
    }

    #[test]
    fn wos_is_bit_deterministic() {
        let set = channel_set();
        let cfg = WosConfig::new(16, 64, 1.0 / 16.0, 7);
        let a = walk_on_spheres_field(&set, 17, 17, cfg);
        let b = walk_on_spheres_field(&set, 17, 17, cfg);
        assert_eq!(
            a.texel, b.texel,
            "WoS must be bit-identical for a fixed seed"
        );
    }

    #[test]
    fn empty_set_is_transparent() {
        let set = DiffusionCurveSet::new();
        let f = walk_on_spheres_field(&set, 9, 9, WosConfig::new(8, 32, 0.1, 0));
        assert!(f.texel.iter().all(|t| *t == [0.0; 4]));
    }

    #[test]
    fn tier_matrix_matches_adr_2_5() {
        assert_eq!(DiffusionTier::Heavy.plan().spp, 64);
        assert_eq!(DiffusionTier::Heavy.plan().budget_ms, 5.0);
        assert!(!DiffusionTier::Heavy.plan().use_jbu);
        assert_eq!(DiffusionTier::Standard.plan().solve_scale, 0.5);
        assert_eq!(DiffusionTier::Lite.plan().spp, 16);
        assert!(DiffusionTier::Lite.plan().use_jbu);
        assert_eq!(
            DiffusionTier::MobileCore.plan().algorithm,
            DiffusionAlgorithm::MultigridCpu
        );
        assert_eq!(DiffusionTier::MobileCore.plan().spp, 0);
    }

    #[test]
    fn dispatch_data_is_pod_std140() {
        assert_eq!(core::mem::size_of::<DiffusionParams>(), 32);
        assert_eq!(core::mem::size_of::<DiffusionParams>() % 16, 0);
        assert_eq!(core::mem::size_of::<GpuSegment>(), 48);
        assert_eq!(core::mem::size_of::<GpuSegment>() % 16, 0);
        let p = DiffusionParams::new(256, 256, 4, 32, 1);
        assert_eq!(p.as_bytes().len(), 32);
    }

    #[test]
    fn pack_curves_one_segment_per_edge() {
        let set = channel_set(); // 2 straight curves = 2 single-edge polylines
        let segs = pack_curves(&set);
        assert_eq!(segs.len(), 2);
        // endpoints round-trip the authored geometry.
        assert_eq!(segs[0].endpoints, [0.25, 0.0, 0.25, 1.0]);
    }

    #[test]
    fn diffusion_wgsl_validates() {
        // The WoS shader uses `ph2d_noise1`; the renderer prepends the prelude,
        // so validation does too (mirrors `cache::compile_fill`).
        let src = format!("{}{}", ph2d_expr::wgsl_prelude(), DIFFUSION_WGSL);
        let module = naga::front::wgsl::parse_str(&src)
            .unwrap_or_else(|e| panic!("diffusion.wgsl parse:\n{}", e.emit_to_string(&src)));
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::empty(),
        )
        .validate(&module)
        .unwrap_or_else(|e| panic!("diffusion.wgsl validate:\n{}", e.emit_to_string(&src)));
    }

    #[test]
    fn bilateral_upsample_wgsl_validates() {
        let src = BILATERAL_UPSAMPLE_WGSL;
        let module = naga::front::wgsl::parse_str(src).unwrap_or_else(|e| {
            panic!("bilateral_upsample.wgsl parse:\n{}", e.emit_to_string(src))
        });
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::empty(),
        )
        .validate(&module)
        .unwrap_or_else(|e| {
            panic!(
                "bilateral_upsample.wgsl validate:\n{}",
                e.emit_to_string(src)
            )
        });
    }
}
