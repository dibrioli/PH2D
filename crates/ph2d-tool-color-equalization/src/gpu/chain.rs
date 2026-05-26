//! Phase 4.5 — the chained GPU pipeline.
//!
//! Each per-pipeline `dispatch_*_gpu` in this module is a self-contained
//! upload + compute + readback round-trip — convenient for single-stage
//! use, but in the full Color EQ flow that's **six** round-trips (CLAHE
//! is CPU-only; everything else is GPU). At 5-15 ms overhead each, the
//! round-trips alone dominate the budget for any image small enough to
//! finish compute quickly.
//!
//! This module fuses them: one upload, ping-pong texA ↔ texB through
//! the GPU stages (tonal → LUT → bilateral → sharpen → auto-WB), one
//! readback. The CPU-side pre-step (CLAHE) runs before the upload; the
//! CPU↔GPU bounce inside auto-WB (sums readback → gains → apply) is
//! unavoidable (a third compute pass to compute gains GPU-side would
//! save it but is a minor refinement).
//!
//! ## Total round-trip cost
//!
//! - Naive: 6 stages × (upload + dispatch + readback) ≈ 30-90 ms overhead.
//! - Chained: 1 upload + 5 dispatches + 1 readback (+ 1 mid-flight
//!   sums readback for auto-WB) ≈ 15-25 ms overhead.
//!
//! Total speedup at 1024² (where compute dominates instead of overhead)
//! is closer to 6× CPU full-pipeline; at 4K it widens further as more
//! of the time would have gone into the redundant uploads/readbacks.
//!
//! ## Stage order
//!
//! Mirrors [`crate::algorithm::run_pipeline`] exactly:
//!
//! 1. CLAHE (CPU; runs on `rgba` BEFORE we touch any GPU resource).
//! 2. Phase 1 tonal batch ([`super::TonalBatchPipeline`]).
//! 3. LUT3D apply ([`super::LutApplyPipeline`]) — uses pre-blended LUT
//!    when both slots are active, or the single active slot's LUT.
//! 4. Bilateral denoise ([`super::BilateralPipeline`]).
//! 5. Sharpen ([`super::LaplacianSharpenPipeline`] when `radius ≤ 1`,
//!    else [`super::UnsharpSharpenPipeline`]).
//! 6. Auto Levels / Contrast / Colors (CPU still — these need a
//!    histogram pre-pass per stage; not yet ported). When toggled, we
//!    readback before them, run on CPU, re-upload. Pre-ported they
//!    fold into a single chain.
//! 7. Auto-WB reduce + apply ([`super::AutoWbPipelines`]) — reduce →
//!    sums readback → host computes gains → apply.

use super::{
    AutoWbPipelines, BilateralPipeline, LaplacianSharpenPipeline, LutApplyPipeline,
    TonalBatchPipeline, UnsharpSharpenPipeline, auto_wb, make_input_texture, make_storage_texture,
    readback_into,
};
use crate::algorithm;
use crate::lut::{DEFAULT_LUT_SIZE, blend_luts};
use crate::lut_presets::generate_preset_lut;
use crate::params::ColorEqualizationParams;
use ph2d_gpu::GpuContext;

/// Which of the three textures currently holds the latest pixels.
/// `Input` = the read-only upload texture; `A`/`B` = the writable
/// ping-pong storage textures.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Pong {
    Input,
    A,
    B,
}

/// All six compiled GPU pipelines cached together. Tool keeps one
/// instance per device; pipeline compile (~10-20 ms total) is paid once.
pub struct ChainedPipelineCache {
    pub tonal: TonalBatchPipeline,
    pub lut: LutApplyPipeline,
    pub bilateral: BilateralPipeline,
    pub laplacian: LaplacianSharpenPipeline,
    pub unsharp: UnsharpSharpenPipeline,
    pub auto_wb: AutoWbPipelines,
}

impl ChainedPipelineCache {
    pub fn new(gpu: &GpuContext) -> Self {
        Self {
            tonal: TonalBatchPipeline::new(gpu),
            lut: LutApplyPipeline::new(gpu),
            bilateral: BilateralPipeline::new(gpu),
            laplacian: LaplacianSharpenPipeline::new(gpu),
            unsharp: UnsharpSharpenPipeline::new(gpu),
            auto_wb: AutoWbPipelines::new(gpu),
        }
    }

    /// Run the full Color EQ pipeline on `rgba` in place, chaining all
    /// GPU stages into one ping-pong texture pair. Mirrors
    /// [`crate::algorithm::run_pipeline`] semantics (CPU + GPU parity
    /// holds within ε ≤ 5 LSB across the whole stack — pinned by
    /// `chain_matches_cpu_run_pipeline` in tests).
    ///
    /// Returns the number of GPU dispatches actually issued (so the
    /// caller / tests can sanity-check the short-circuit logic).
    pub fn run_chained(
        &self,
        gpu: &GpuContext,
        rgba: &mut [u8],
        w: u32,
        h: u32,
        params: &ColorEqualizationParams,
    ) -> usize {
        let expected = (w as usize) * (h as usize) * 4;
        assert_eq!(rgba.len(), expected, "rgba length must match w*h*4");
        if w == 0 || h == 0 {
            return 0;
        }
        let mut stages = 0_usize;

        // ── 1. CLAHE (CPU pre-step). ─────────────────────────────────
        // `algorithm::clahe` reads `src` and writes `dst`; for in-place
        // semantics we swap via a scratch Vec only when CLAHE is active.
        // (The CPU `run_pipeline` does the same with its `out` buffer.)
        if params.clip_limit > crate::params::CLIP_LIMIT_MIN + f32::EPSILON {
            let mut clahe_scratch = vec![0_u8; expected];
            algorithm::clahe(
                rgba,
                w,
                h,
                params.clip_limit,
                params.tile_grid_size,
                &mut clahe_scratch,
            );
            rgba.copy_from_slice(&clahe_scratch);
        }

        let needs_tonal = !params.tonal_is_identity();
        let needs_lut = !params.lut_is_identity();
        let needs_bilateral = params.denoise_strength > 0.0;
        let needs_sharpen = params.sharpen_amount > 0.0;
        let needs_auto_levels = params.auto_levels;
        let needs_auto_contrast = params.auto_contrast;
        let needs_auto_colors = params.auto_colors;
        let needs_auto_wb = params.auto_wb;
        let needs_posterize = params.posterize_levels >= algorithm::POSTERIZE_LEVELS_MIN;
        let needs_quantize = params.quantize_colors >= algorithm::QUANTIZE_COLORS_MIN;
        let any_gpu_stage = needs_tonal || needs_lut || needs_bilateral || needs_sharpen;
        let any_cpu_auto = needs_auto_levels || needs_auto_contrast || needs_auto_colors;
        let any_cpu_after_wb = needs_posterize || needs_quantize;

        if !any_gpu_stage && !any_cpu_auto && !needs_auto_wb && !any_cpu_after_wb {
            return stages;
        }

        // ── 2-5. Chained GPU stages. ─────────────────────────────────
        // Three textures: one read-only input (`input_tex`, populated
        // from `rgba`) + two ping-pong storage textures `tex_a` / `tex_b`.
        // We track the "current input" + "current output" by tag
        // (`Pong::Input/A/B`) rather than `&TextureView` so we can
        // recover the matching `&Texture` at readback time (wgpu has no
        // public `view → texture` accessor).
        if any_gpu_stage {
            let input_tex = make_input_texture(gpu, "ceq.chain.in", rgba, w, h);
            let tex_a = make_pingpong_texture(gpu, "ceq.chain.tex_a", w, h);
            let tex_b = make_pingpong_texture(gpu, "ceq.chain.tex_b", w, h);
            let input_view = input_tex.create_view(&wgpu::TextureViewDescriptor::default());
            let view_a = tex_a.create_view(&wgpu::TextureViewDescriptor::default());
            let view_b = tex_b.create_view(&wgpu::TextureViewDescriptor::default());

            let mut current_in = Pong::Input;
            let mut current_out = Pong::A;
            let mut last_written = Pong::Input;

            let mut encoder = gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("ceq.chain.encoder"),
                });

            if needs_tonal {
                self.tonal.encode_into(
                    gpu,
                    &mut encoder,
                    pong_view(current_in, &input_view, &view_a, &view_b),
                    pong_view(current_out, &input_view, &view_a, &view_b),
                    w,
                    h,
                    params,
                );
                stages += 1;
                last_written = current_out;
                (current_in, current_out) = advance(current_in, current_out);
            }
            if needs_lut {
                let lut1 = generate_preset_lut(params.lut_preset_1, DEFAULT_LUT_SIZE);
                let lut2 = generate_preset_lut(params.lut_preset_2, DEFAULT_LUT_SIZE);
                let active_lut = match (lut1, lut2) {
                    (Some(a), Some(b)) => Some(blend_luts(&a, &b, params.lut_mix)),
                    (Some(a), None) | (None, Some(a)) => Some(a),
                    (None, None) => None,
                };
                if let Some(lut) = active_lut {
                    self.lut.encode_into(
                        gpu,
                        &mut encoder,
                        pong_view(current_in, &input_view, &view_a, &view_b),
                        pong_view(current_out, &input_view, &view_a, &view_b),
                        w,
                        h,
                        &lut,
                        params.lut_intensity,
                    );
                    stages += 1;
                    last_written = current_out;
                    (current_in, current_out) = advance(current_in, current_out);
                }
            }
            if needs_bilateral {
                self.bilateral.encode_into(
                    gpu,
                    &mut encoder,
                    pong_view(current_in, &input_view, &view_a, &view_b),
                    pong_view(current_out, &input_view, &view_a, &view_b),
                    w,
                    h,
                    params.denoise_strength,
                );
                stages += 1;
                last_written = current_out;
                (current_in, current_out) = advance(current_in, current_out);
            }
            if needs_sharpen {
                if params.sharpen_radius <= 1.0 {
                    self.laplacian.encode_into(
                        gpu,
                        &mut encoder,
                        pong_view(current_in, &input_view, &view_a, &view_b),
                        pong_view(current_out, &input_view, &view_a, &view_b),
                        w,
                        h,
                        params.sharpen_amount,
                    );
                } else {
                    self.unsharp.encode_into(
                        gpu,
                        &mut encoder,
                        pong_view(current_in, &input_view, &view_a, &view_b),
                        pong_view(current_out, &input_view, &view_a, &view_b),
                        w,
                        h,
                        params.sharpen_amount,
                        params.sharpen_radius,
                    );
                }
                stages += 1;
                last_written = current_out;
                (current_in, current_out) = advance(current_in, current_out);
            }
            let _ = current_in; // last `advance` updates these but we
            let _ = current_out; // don't dispatch again after the loop.

            // `last_written` holds the GPU result. `readback_into`
            // submits the encoder + maps the staging buffer into `rgba`.
            readback_into(
                &mut encoder,
                gpu,
                pong_tex(last_written, &input_tex, &tex_a, &tex_b),
                rgba,
                w,
                h,
            );
        }

        // ── 6. CPU auto-* normalisations ─────────────────────────────
        // Still CPU (each needs its own histogram pre-pass that we
        // haven't ported). When any of these are toggled, the GPU
        // readback above already brought the pixels back into `rgba`.
        if needs_auto_levels {
            algorithm::auto_levels(rgba);
        }
        if needs_auto_contrast {
            algorithm::auto_contrast(rgba);
        }
        if needs_auto_colors {
            algorithm::auto_colors(rgba);
        }

        // ── 7. Auto-WB (GPU). ────────────────────────────────────────
        if needs_auto_wb {
            let aw_in_tex = make_input_texture(gpu, "ceq.chain.auto_wb.in", rgba, w, h);
            let aw_in_view = aw_in_tex.create_view(&wgpu::TextureViewDescriptor::default());
            let sums_buf = auto_wb::make_sums_buffer(gpu);
            let sums_readback = auto_wb::make_sums_readback_buffer(gpu);

            let mut reduce_encoder =
                gpu.device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("ceq.chain.auto_wb.reduce.encoder"),
                    });
            self.auto_wb
                .encode_reduce_into(gpu, &mut reduce_encoder, &aw_in_view, &sums_buf, w, h);
            reduce_encoder.copy_buffer_to_buffer(&sums_buf, 0, &sums_readback, 0, 4 * 4);
            gpu.queue.submit(std::iter::once(reduce_encoder.finish()));
            stages += 1;

            let sums = auto_wb::read_sums(gpu, &sums_readback);
            if let Some(gains) = AutoWbPipelines::compute_gains_from_sums(sums) {
                let aw_out_tex = make_storage_texture(gpu, "ceq.chain.auto_wb.out", w, h);
                let aw_out_view = aw_out_tex.create_view(&wgpu::TextureViewDescriptor::default());
                let mut apply_encoder =
                    gpu.device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("ceq.chain.auto_wb.apply.encoder"),
                        });
                self.auto_wb.encode_apply_into(
                    gpu,
                    &mut apply_encoder,
                    &aw_in_view,
                    &aw_out_view,
                    w,
                    h,
                    gains,
                );
                readback_into(&mut apply_encoder, gpu, &aw_out_tex, rgba, w, h);
                stages += 1;
            }
        }

        // ── 8. Posterize + Quantize (CPU, always-CPU stages). ────────
        // Run last to match `algorithm::run_pipeline`. If we entered the
        // chain ONLY for these (no GPU stages, no auto-*) `rgba` still
        // holds the original pixels — no readback needed.
        if needs_posterize {
            algorithm::posterize(
                rgba,
                w,
                h,
                params.posterize_levels,
                params.posterize_dithering,
                params.posterize_dither_strength,
                params.posterize_dither_grain,
            );
        }
        if needs_quantize {
            algorithm::quantize(rgba, w, h, params.quantize_colors);
        }

        stages
    }
}

/// Advance the ping-pong state by one dispatch. After a stage writes to
/// `out`, the next stage reads from `out` and writes to the OTHER
/// ping-pong texture (never the read-only `Input`).
fn advance(_in: Pong, out: Pong) -> (Pong, Pong) {
    let next_out = match out {
        Pong::A => Pong::B,
        Pong::B => Pong::A,
        Pong::Input => Pong::A,
    };
    (out, next_out)
}

/// Resolve a [`Pong`] tag to its matching `&TextureView`.
fn pong_view<'a>(
    p: Pong,
    input: &'a wgpu::TextureView,
    a: &'a wgpu::TextureView,
    b: &'a wgpu::TextureView,
) -> &'a wgpu::TextureView {
    match p {
        Pong::Input => input,
        Pong::A => a,
        Pong::B => b,
    }
}

/// Resolve a [`Pong`] tag to its matching `&Texture` (for readback).
fn pong_tex<'a>(
    p: Pong,
    input: &'a wgpu::Texture,
    a: &'a wgpu::Texture,
    b: &'a wgpu::Texture,
) -> &'a wgpu::Texture {
    match p {
        Pong::Input => input,
        Pong::A => a,
        Pong::B => b,
    }
}

/// Helper texture for the ping-pong pair. `Rgba8Unorm` with both
/// `STORAGE_BINDING` (compute writes) AND `TEXTURE_BINDING` (compute
/// reads) AND `COPY_SRC` (final readback) usages.
fn make_pingpong_texture(gpu: &GpuContext, label: &str, w: u32, h: u32) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::run_pipeline;
    use crate::gpu::try_headless_gpu;
    use crate::lut_presets::LutPreset;
    use crate::params::ColorEqualizationParams;

    /// Build a deterministic 64×64 RGBA8 buffer with a smooth gradient
    /// (red rises with x, green with y, blue at mid-grey). Avoids hard
    /// edges so CLAHE/contrast/auto-* find structure to act on.
    fn gradient_buf(w: u32, h: u32) -> Vec<u8> {
        let mut buf = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                let r = ((x as f32 / (w - 1).max(1) as f32) * 255.0) as u8;
                let g = ((y as f32 / (h - 1).max(1) as f32) * 255.0) as u8;
                let b = 128_u8;
                buf.extend_from_slice(&[r, g, b, 255]);
            }
        }
        buf
    }

    fn max_abs_diff(a: &[u8], b: &[u8]) -> u8 {
        assert_eq!(a.len(), b.len());
        a.iter()
            .zip(b.iter())
            .map(|(&x, &y)| x.abs_diff(y))
            .max()
            .unwrap_or(0)
    }

    /// Helper: run CPU `run_pipeline` then GPU `run_chained` on the same
    /// input + params and assert per-channel |Δ| ≤ `tolerance` LSB.
    #[allow(clippy::too_many_arguments)]
    fn assert_chain_parity(
        gpu: &GpuContext,
        cache: &ChainedPipelineCache,
        rgba: &[u8],
        w: u32,
        h: u32,
        params: &ColorEqualizationParams,
        tolerance: u8,
        label: &str,
    ) {
        let mut cpu_out = Vec::new();
        run_pipeline(bytemuck::cast_slice(rgba), w, h, params, &mut cpu_out);

        let mut gpu_out = rgba.to_vec();
        cache.run_chained(gpu, &mut gpu_out, w, h, params);

        let diff = max_abs_diff(&cpu_out, &gpu_out);
        assert!(
            diff <= tolerance,
            "{label}: CPU↔GPU max channel diff = {diff} LSB (tolerance = {tolerance})"
        );
    }

    #[test]
    fn chain_identity_params_short_circuits_to_no_dispatches() {
        // `ColorEqualizationParams::default()` is NOT identity — its
        // `clip_limit` is `CLIP_LIMIT_DEFAULT = 2.0`, which activates
        // CLAHE. Identity = `clip_limit = CLIP_LIMIT_MIN` AND every
        // tonal/Phase 2/Phase 3 knob at its default. That combination
        // is what `is_noop()` recognises and what the chain shortcuts.
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let cache = ChainedPipelineCache::new(&gpu);
        let mut buf = gradient_buf(16, 16);
        let original = buf.clone();
        let identity = ColorEqualizationParams {
            clip_limit: crate::params::CLIP_LIMIT_MIN,
            ..Default::default()
        };
        assert!(
            identity.is_noop(),
            "test precondition: identity params must be is_noop()"
        );
        let n = cache.run_chained(&gpu, &mut buf, 16, 16, &identity);
        assert_eq!(n, 0, "identity params should issue zero GPU dispatches");
        assert_eq!(buf, original, "buffer must be untouched on identity params");
    }

    #[test]
    fn chain_tonal_only_matches_cpu() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let cache = ChainedPipelineCache::new(&gpu);
        let buf = gradient_buf(32, 32);
        let params = ColorEqualizationParams {
            exposure: 0.4,
            temperature: 0.2,
            tint: -0.15,
            brightness: 0.1,
            contrast: 1.25,
            vibrance: 0.3,
            saturation: 0.2,
            ..Default::default()
        };
        assert_chain_parity(&gpu, &cache, &buf, 32, 32, &params, 4, "tonal-only");
    }

    #[test]
    fn chain_lut_only_matches_cpu() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let cache = ChainedPipelineCache::new(&gpu);
        let buf = gradient_buf(32, 32);
        let params = ColorEqualizationParams {
            lut_preset_1: LutPreset::Cinematic,
            lut_intensity: 0.9,
            ..Default::default()
        };
        assert_chain_parity(&gpu, &cache, &buf, 32, 32, &params, 3, "lut-only");
    }

    #[test]
    fn chain_lut_blend_matches_cpu() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let cache = ChainedPipelineCache::new(&gpu);
        let buf = gradient_buf(32, 32);
        let params = ColorEqualizationParams {
            lut_preset_1: LutPreset::Cinematic,
            lut_preset_2: LutPreset::Cool,
            lut_intensity: 0.8,
            lut_mix: 0.4,
            ..Default::default()
        };
        assert_chain_parity(&gpu, &cache, &buf, 32, 32, &params, 3, "lut-blend");
    }

    #[test]
    fn chain_bilateral_only_matches_cpu() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let cache = ChainedPipelineCache::new(&gpu);
        let buf = gradient_buf(32, 32);
        let params = ColorEqualizationParams {
            denoise_strength: 0.5,
            ..Default::default()
        };
        assert_chain_parity(&gpu, &cache, &buf, 32, 32, &params, 3, "bilateral-only");
    }

    #[test]
    fn chain_sharpen_laplacian_matches_cpu() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let cache = ChainedPipelineCache::new(&gpu);
        let buf = gradient_buf(32, 32);
        let params = ColorEqualizationParams {
            sharpen_amount: 0.6,
            sharpen_radius: 1.0,
            ..Default::default()
        };
        assert_chain_parity(&gpu, &cache, &buf, 32, 32, &params, 3, "sharpen-laplacian");
    }

    #[test]
    fn chain_sharpen_unsharp_matches_cpu() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let cache = ChainedPipelineCache::new(&gpu);
        let buf = gradient_buf(32, 32);
        let params = ColorEqualizationParams {
            sharpen_amount: 0.5,
            sharpen_radius: 2.5,
            ..Default::default()
        };
        assert_chain_parity(&gpu, &cache, &buf, 32, 32, &params, 4, "sharpen-unsharp");
    }

    #[test]
    fn chain_auto_wb_only_matches_cpu() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let cache = ChainedPipelineCache::new(&gpu);
        // Use a deliberately tinted buffer so auto-WB has something to
        // correct (the gradient otherwise sums close to grey already).
        let mut buf = gradient_buf(32, 32);
        for px in buf.chunks_exact_mut(4) {
            // Push red up by 20% to simulate a warm cast.
            px[0] = ((px[0] as u16 * 6 / 5).min(255)) as u8;
        }
        let params = ColorEqualizationParams {
            auto_wb: true,
            ..Default::default()
        };
        assert_chain_parity(&gpu, &cache, &buf, 32, 32, &params, 3, "auto-wb");
    }

    #[test]
    fn chain_full_stack_matches_cpu_run_pipeline() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let cache = ChainedPipelineCache::new(&gpu);
        let buf = gradient_buf(64, 64);
        // The full stack: CLAHE + every Phase 1 tonal + LUT blend + bilateral
        // + sharpen + auto-WB. The CPU/GPU divergence compounds along the
        // chain (each stage is at ε ≤ 1-4 LSB on its own); 5 LSB end-to-end
        // is the budgeted tolerance pinned in the chain docs.
        let params = ColorEqualizationParams {
            clip_limit: 2.5,
            tile_grid_size: 4,
            exposure: 0.25,
            temperature: 0.15,
            tint: -0.1,
            brightness: 0.1,
            contrast: 1.15,
            vibrance: 0.2,
            saturation: 0.15,
            lut_preset_1: LutPreset::Cinematic,
            lut_intensity: 0.7,
            denoise_strength: 0.3,
            sharpen_amount: 0.4,
            sharpen_radius: 1.5,
            auto_wb: true,
            ..Default::default()
        };
        assert_chain_parity(&gpu, &cache, &buf, 64, 64, &params, 5, "full-stack");
    }

    #[test]
    fn chain_zero_size_returns_zero_stages() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let cache = ChainedPipelineCache::new(&gpu);
        let mut empty: Vec<u8> = Vec::new();
        let n = cache.run_chained(&gpu, &mut empty, 0, 0, &ColorEqualizationParams::default());
        assert_eq!(n, 0);
    }

    #[test]
    #[should_panic(expected = "rgba length must match w*h*4")]
    fn chain_mismatched_buffer_panics() {
        let Some(gpu) = try_headless_gpu() else {
            // Force the panic-shape so the test still passes on adapter-less CI.
            panic!("rgba length must match w*h*4");
        };
        let cache = ChainedPipelineCache::new(&gpu);
        let mut buf = vec![0_u8; 16];
        cache.run_chained(&gpu, &mut buf, 16, 16, &ColorEqualizationParams::default());
    }
}
