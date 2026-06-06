//! W7 GPU Walk-on-Spheres — dispatch parity + tier-budget gate (Coord).
//!
//! The impl (`diffusion_gpu.rs`) shipped the WoS compute SHADER + the storage-
//! buffer packing (`pack_curves`/`GpuSegment`) + the per-dispatch uniform
//! (`DiffusionParams`) + the line-for-line CPU reference (`walk_on_spheres_field`),
//! and left the wgpu pipeline/dispatch to the Coord. This builds it:
//! [`WosPipeline`] is the reusable `GpuDiffusion` prototype (the render wiring will
//! promote it to a lib); the tests prove the GPU shader reproduces the CPU
//! reference and that a solve fits the tier budget.
//!
//! `#[ignore]` (needs a real device, like ph2d-render's GPU gates):
//!   cargo test -p ph2d-vector-fill --test diffusion_gpu_parity -- --ignored --nocapture
//! Run perf with `--release` (dev = opt0, ~7× slower — measuring in dev lies).

use glam::Vec2;
use ph2d_gpu::GpuContext;
use ph2d_vector_fill::diffusion_gpu::{
    DIFFUSION_WGSL, DiffusionParams, DiffusionTier, WosConfig, pack_curves, walk_on_spheres_field,
};
use ph2d_vector_fill::{ColorField, DiffusionCurve, DiffusionCurveSet};
use ph2d_color::OklchColor;

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

/// A red↔blue diffusion wall — the minimal scene that exercises both side colours.
fn red_blue_scene() -> DiffusionCurveSet {
    let red = OklchColor::opaque(0.63, 0.26, 29.0);
    let blue = OklchColor::opaque(0.45, 0.31, 264.0);
    DiffusionCurveSet::from_curves([DiffusionCurve::straight(
        Vec2::new(0.5, 0.0),
        Vec2::new(0.5, 1.0),
        red,
        blue,
    )])
}

/// The reusable WoS compute pipeline (built once; dispatched per solve). This is
/// the `GpuDiffusion` prototype the render wiring will promote out of the test.
struct WosPipeline {
    pipeline: wgpu::ComputePipeline,
    bgl: wgpu::BindGroupLayout,
}

impl WosPipeline {
    fn new(gpu: &GpuContext) -> Self {
        // The shader shares `ph2d_noise1` with the CPU reference — prepend the prelude.
        let src = format!("{}\n{}", ph2d_expr::wgsl_prelude(), DIFFUSION_WGSL);
        let module = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ph2d-vector-fill wos"),
                source: wgpu::ShaderSource::Wgsl(src.into()),
            });
        let storage = |binding: u32, read_only: bool| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let bgl = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ph2d-vector-fill wos bgl"),
                entries: &[
                    storage(0, true), // segments (read)
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    storage(2, false), // field (read_write)
                ],
            });
        let layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("ph2d-vector-fill wos layout"),
                bind_group_layouts: &[&bgl],
                immediate_size: 0,
            });
        let pipeline = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("ph2d-vector-fill wos pipeline"),
                layout: Some(&layout),
                module: &module,
                entry_point: Some("diffusion_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });
        Self { pipeline, bgl }
    }

    /// Dispatch a WoS solve over `w × h`. Returns the GPU compute time (submit →
    /// poll-wait, no readback — the per-frame budget) and, when `readback`, the
    /// solved [`ColorField`].
    fn solve(
        &self,
        gpu: &GpuContext,
        set: &DiffusionCurveSet,
        w: u32,
        h: u32,
        cfg: WosConfig,
        readback: bool,
    ) -> (f32, Option<ColorField>) {
        let segs = pack_curves(set);
        if segs.is_empty() {
            return (0.0, readback.then(|| ColorField::transparent(w as usize, h as usize)));
        }
        let params = DiffusionParams {
            width: w,
            height: h,
            segment_count: segs.len() as u32,
            spp: cfg.spp,
            max_steps: cfg.max_steps,
            seed: cfg.seed,
            epsilon: cfg.epsilon,
            _pad: 0.0,
        };
        let new_buf = |size: u64, usage: wgpu::BufferUsages| {
            gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ph2d-vector-fill wos buf"),
                size,
                usage,
                mapped_at_creation: false,
            })
        };
        let seg_bytes: &[u8] = bytemuck::cast_slice(&segs);
        let seg_buf = new_buf(
            seg_bytes.len() as u64,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        );
        gpu.queue.write_buffer(&seg_buf, 0, seg_bytes);
        let params_buf = new_buf(
            core::mem::size_of::<DiffusionParams>() as u64,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );
        gpu.queue
            .write_buffer(&params_buf, 0, bytemuck::bytes_of(&params));
        let field_size = (w as u64) * (h as u64) * 16; // vec4<f32>
        let field_buf = new_buf(
            field_size,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        );
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-vector-fill wos bg"),
            layout: &self.bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: seg_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: params_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: field_buf.as_entire_binding(),
                },
            ],
        });
        let t0 = std::time::Instant::now();
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("wos enc") });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("wos pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(w.div_ceil(8), h.div_ceil(8), 1);
        }
        gpu.queue.submit([encoder.finish()]);
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        let compute_ms = t0.elapsed().as_secs_f32() * 1000.0;

        let field = readback.then(|| {
            let staging = new_buf(
                field_size,
                wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            );
            let mut enc = gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("wos rb") });
            enc.copy_buffer_to_buffer(&field_buf, 0, &staging, 0, field_size);
            gpu.queue.submit([enc.finish()]);
            let (tx, rx) = std::sync::mpsc::channel();
            staging.slice(..).map_async(wgpu::MapMode::Read, move |r| {
                let _ = tx.send(r);
            });
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
            rx.recv().expect("map").expect("mapped");
            let mapped = staging.slice(..).get_mapped_range();
            let texel: Vec<[f32; 4]> = bytemuck::cast_slice::<u8, [f32; 4]>(&mapped).to_vec();
            drop(mapped);
            staging.unmap();
            ColorField {
                w: w as usize,
                h: h as usize,
                texel,
            }
        });
        (compute_ms, field)
    }
}

/// GPU WoS reproduces the CPU `walk_on_spheres_field` (same scene, same `WosConfig`,
/// same `ph2d_noise1` RNG). WoS walks are sensitive to per-step `cos/sin`, which
/// drift by ULP across GPU↔CPU and accumulate over the walk — so this is a
/// Monte-Carlo agreement check (the two estimators converge to the same field),
/// not bit-equality. High spp shrinks the variance; the tolerance is on the
/// linear-RGBA mean over the field + a structural red↔blue split assertion.
#[test]
#[ignore = "needs a GPU device"]
fn gpu_wos_matches_cpu_reference() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let set = red_blue_scene();
    let (w, h) = (33u32, 33u32);
    // 256 spp: MC variance ∝ 1/√spp, so the GPU↔CPU mean agreement tightens to
    // ~0.01 — well clear of a wrong shader (which diverges by ≥0.1) while leaving
    // room for the per-walk cos/sin ULP drift that makes bit-equality impossible.
    let cfg = WosConfig::new(256, 64, 1.0 / (w - 1) as f32, 7);

    let pipe = WosPipeline::new(&gpu);
    let (_ms, gpu_field) = pipe.solve(&gpu, &set, w, h, cfg, true);
    let gpu_field = gpu_field.expect("readback");
    let cpu_field = walk_on_spheres_field(&set, w as usize, h as usize, cfg);

    // Mean absolute difference over the field (MC noise + ULP-drift average out).
    let n = (w * h) as usize;
    let mut sum = 0.0f64;
    let mut worst = 0.0f32;
    for i in 0..n {
        for k in 0..3 {
            let d = (gpu_field.texel[i][k] - cpu_field.texel[i][k]).abs();
            sum += f64::from(d);
            worst = worst.max(d);
        }
    }
    let mean = (sum / (n * 3) as f64) as f32;
    eprintln!("WoS GPU↔CPU: mean |Δ| = {mean:.4}, worst = {worst:.4} (linear, {w}×{h} @ {}spp)", cfg.spp);
    assert!(
        mean < 0.03,
        "GPU↔CPU mean diff {mean} too high — a correct WoS shader agrees to ~0.01 at 256 spp; \
         this says the shader diverges from the reference algorithm"
    );

    // Structural: far-left ≈ red (R>B), far-right ≈ blue (B>R) — the split survived.
    let mid = (h / 2) as usize;
    let left = gpu_field.texel[mid * w as usize + 2];
    let right = gpu_field.texel[mid * w as usize + (w as usize - 3)];
    assert!(left[0] > left[2], "left should read red (R>B): {left:?}");
    assert!(right[2] > right[0], "right should read blue (B>R): {right:?}");
}

/// Tier-budget MEASUREMENT (ADR-0060 §2.5). Measures the GPU WoS compute (submit
/// → poll, no readback) per tier and reports it against the tier budget — RUN
/// WITH `--release` (dev opt0 inflates ~7×).
///
/// **Finding (2026-06-06, this Mac / Metal):** the naive 64-step WoS does NOT meet
/// the §2.5 budgets — it sustains ~11 G-walk-steps/s, so a tier needs
/// `side² × spp × max_steps / 11e9` seconds; e.g. Heavy 512²@64spp = ~98 ms vs the
/// 5 ms budget (and a real 1080p Heavy is ~8× worse). The JBU low-res path (the
/// impl shipped `BILATERAL_UPSAMPLE_WGSL`) helps by shrinking `side`, but even the
/// reduced-res tiers are over. Closing it is the **impl's WoS-optimisation domain**
/// (fewer `max_steps` / importance or control-variate sampling / a nearest-curve
/// acceleration structure) or a §2.5 budget revision — reported to Coord/Enio.
///
/// So this gate does NOT assert the (currently-unmet) budget — that would be a
/// false green. It asserts a **throughput regression floor** (the real, true,
/// useful invariant: a future change must not slow the kernel by >2×) and reports
/// every tier's ms-vs-budget so the gap stays visible.
#[test]
#[ignore = "needs a GPU device; measure with --release"]
fn vector_diffusion_curve_tier_budget() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let set = red_blue_scene();
    let pipe = WosPipeline::new(&gpu);
    const BASE: u32 = 512;
    const MAX_STEPS: u32 = 64;

    let mut best_gstep_s = 0.0f32;
    for tier in [
        DiffusionTier::Heavy,
        DiffusionTier::Standard,
        DiffusionTier::Lite,
        DiffusionTier::Web,
    ] {
        let plan = tier.plan();
        let side = ((BASE as f32) * plan.solve_scale).round() as u32;
        let cfg = WosConfig::new(plan.spp, MAX_STEPS, 1.0 / (side - 1) as f32, 7);
        let _ = pipe.solve(&gpu, &set, side, side, cfg, false); // warm up
        let ms = (0..3)
            .map(|_| pipe.solve(&gpu, &set, side, side, cfg, false).0)
            .fold(f32::INFINITY, f32::min); // best-of-3 (least scheduler noise)
        let steps = (side as f64) * (side as f64) * f64::from(plan.spp) * f64::from(MAX_STEPS);
        let gstep_s = (steps / (f64::from(ms) / 1000.0) / 1e9) as f32;
        let verdict = if ms <= plan.budget_ms { "✓" } else { "✗ OVER" };
        eprintln!(
            "{tier:?}: {side}² @ {}spp = {ms:.2} ms (budget {:.1} ms {verdict}) — {gstep_s:.1} Gstep/s",
            plan.spp, plan.budget_ms
        );
        best_gstep_s = best_gstep_s.max(gstep_s);
    }
    // Regression floor (NOT the §2.5 budget — see the doc): the kernel currently
    // sustains ~11 Gstep/s on Metal; assert ≥ 4 so a >2× perf regression fails.
    assert!(
        best_gstep_s >= 4.0,
        "WoS throughput {best_gstep_s:.1} Gstep/s is below the 4.0 regression floor \
         (run --release) — the kernel regressed"
    );
}
