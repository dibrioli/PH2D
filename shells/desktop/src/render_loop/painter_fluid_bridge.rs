//! W15.3 GPU fluid drive (ADR-0049, feature `fluid`) — steps the painter's live
//! wet field on the GPU AND composites the Kubelka–Munk glaze on the GPU each
//! frame, removing the per-frame pigment readback + the CPU composite (the stalls
//! that capped large-canvas perf).
//!
//! Feature-gated: the default build excludes this module (and `ph2d-painter-fluid`),
//! so the painter runs the CPU diffusion+composite path. With `--features fluid`
//! the render loop calls [`drive_fluid_gpu`] right after the active tool's `on_tick`.
//!
//! ## The resident-composite flow (per frame, no pigment readback)
//! 1. The pigment stays GPU-resident in the solver's `pig_a`; this frame's dabs
//!    (the tool grid's pigment) are uploaded as an additive `deposit` and the field
//!    diffuses/advects ON the GPU. Water is the CPU mirror (uploaded for the gate;
//!    the CPU owns evaporation + the dry-check) — so no water readback either.
//! 2. The GPU compositor reads `pig_a` directly + the pre-stroke backdrop → the
//!    canvas RGBA, and we read back ONLY the wet row band into `canvas_rgba` (the
//!    canonical layer the existing preview upload + Apply/undo consume).
//!
//! Downcasts to the concrete `PainterTool` (allowlisted bridge, same exception
//! class as `painter_bridge.rs`): the GPU drive needs the concrete fluid hooks.

use ph2d_editor::ToolRegistry;
use ph2d_gpu::GpuContext;
use ph2d_painter_brush::wet_composite::prepare_wet_composite_from_stroke;
use ph2d_painter_fluid::{DabGpu, FluidCompositor, FluidSolver};
use std::cell::RefCell;
use std::time::Instant;

/// Opt-in per-phase profiler for the fluid drive (`PH2D_FLUID_PROFILE=1`). Confirms
/// where the per-frame wall-clock goes — sim step vs the composite (whose `device.poll`
/// readback is the suspected sync stall) vs the sporadic stats readback — before any
/// structural change. Prints averaged ms to stderr every `WINDOW` active frames.
struct FluidProfile {
    on: Option<bool>,
    frames: u32,
    step_us: u64,
    comp_us: u64,
    stats_us: u64,
}

impl FluidProfile {
    const WINDOW: u32 = 120;
    const fn new() -> Self {
        Self {
            on: None,
            frames: 0,
            step_us: 0,
            comp_us: 0,
            stats_us: 0,
        }
    }
    fn enabled(&mut self) -> bool {
        if self.on.is_none() {
            self.on = Some(std::env::var("PH2D_FLUID_PROFILE").is_ok_and(|v| v != "0"));
        }
        self.on == Some(true)
    }
    fn record(&mut self, step_us: u64, comp_us: u64, stats_us: u64) {
        self.frames += 1;
        self.step_us += step_us;
        self.comp_us += comp_us;
        self.stats_us += stats_us;
        if self.frames >= Self::WINDOW {
            let f = f64::from(self.frames);
            let (s, c, st) = (
                self.step_us as f64 / f / 1000.0,
                self.comp_us as f64 / f / 1000.0,
                self.stats_us as f64 / f / 1000.0,
            );
            eprintln!(
                "[fluid] per-frame avg over {} frames: step={s:.3}ms composite(+readback)={c:.3}ms stats={st:.3}ms total={:.3}ms",
                self.frames,
                s + c + st
            );
            self.frames = 0;
            self.step_us = 0;
            self.comp_us = 0;
            self.stats_us = 0;
        }
    }
}

thread_local! {
    static PROFILE: RefCell<FluidProfile> = const { RefCell::new(FluidProfile::new()) };
}

/// How often (in frames) the resident path reads the GPU field stats back for the
/// dry-check. Drying takes ~0.3 s (≈ 18 frames @ 60 Hz), so a few frames of latency
/// on the drop is invisible — and the readback is the only per-frame sync we still
/// want sporadic (4K real-time arch §4 / E3). The composite envelope does NOT use
/// it (it's grown from the dab list), so cadence only affects when a dry field drops.
/// At 20: drying (~18 frames after pen-up) still drops the field promptly, but the
/// per-frame avg of the stats `device.poll(wait)` (a queue-drain, ~2.5 ms when it
/// fires) is ~3× lower than at 6 — it was the largest per-frame phase post-pipeline.
const DRY_CHECK_EVERY: u64 = 20;

/// Per-session GPU state for the live wet field: the resident solver, the K–M
/// compositor, the field size, the stroke epoch it was last reset for, and a frame
/// counter pacing the sporadic dry-check readback.
struct FluidSession {
    solver: FluidSolver,
    compositor: FluidCompositor,
    dims: (u32, u32),
    epoch: u64,
    frame: u64,
}

thread_local! {
    /// Rebuilt when the field resizes (a new canvas); reset (resident pigment zeroed
    /// + paper re-uploaded) on a new stroke epoch; dropped when no live field.
    /// Thread-local because the render loop is single-threaded (the established
    /// painter-bridge pattern for per-tool GPU state).
    static SESSION: RefCell<Option<FluidSession>> = const { RefCell::new(None) };
}

/// Drive + composite the live wet field on the GPU, called each frame after the
/// active tool's `on_tick`. No-op without an active painter or a live field (and it
/// then releases the session + hands the field back to the CPU path).
pub(crate) fn drive_fluid_gpu(tools: &mut ToolRegistry, gpu: &GpuContext) {
    let Some(painter) = tools.active_mut().and_then(|t| {
        t.as_any_mut()
            .downcast_mut::<ph2d_tool_painter::PainterTool>()
    }) else {
        return;
    };
    // **Graceful degrade (ADR-0049 §2.8/§2.9).** Only a software/CPU adapter is ruled
    // incapable; every real GPU stays eligible. VRAM-free probing isn't portable in
    // wgpu, so the floor is assumed met on any real GPU (refine with telemetry). When
    // incapable, the field falls back to the CPU path (the tool's on_tick).
    let tier = match gpu.adapter.get_info().device_type {
        wgpu::DeviceType::Cpu => ph2d_host::MemoryTier::Low,
        wgpu::DeviceType::IntegratedGpu | wgpu::DeviceType::VirtualGpu => {
            ph2d_host::MemoryTier::Mid
        }
        _ => ph2d_host::MemoryTier::High,
    };
    let capable = ph2d_host::MemoryBudget {
        vram_free_mb: 256,
        tier,
    }
    .fluid_capable();
    // W15.3 full-res: a capable GPU runs the NEXT fluid field at full canvas
    // resolution. Set every frame (even with no live field) so it's in effect before
    // the next `begin_stroke`.
    painter.set_fluid_hires(capable);

    let eligible = ph2d_painter_fluid::fluid_pass_eligible(true, capable, f32::INFINITY);
    if !eligible || !painter.fluid_brush_enabled() {
        // Not a GPU-fluid scenario → hand back to the CPU path + free the session.
        painter.set_gpu_fluid_driven(false);
        SESSION.with(|s| *s.borrow_mut() = None);
        return;
    }

    // ── PRE-WARM ──────────────────────────────────────────────────────────────
    // Fluid brush selected but no live field yet (hovering / between strokes): build
    // the solver + compositor NOW so the big composite shader compiles BEFORE the
    // first dab — no hitch when the stroke starts. Keep the session warm.
    if !painter.has_wet_field() {
        painter.set_gpu_fluid_driven(false);
        // Pre-generate + cache the paper-tooth field NOW (hovering) so the first
        // `begin_stroke` (the click) doesn't pay the O(grid) `grain_noise` (the ~⅓ s
        // click→stroke delay at 4K). Off the click path.
        painter.fluid_prewarm_paper();
        if let Some(dims) = painter.fluid_prewarm_dims() {
            SESSION.with(|cell| {
                let mut slot = cell.borrow_mut();
                if slot.as_ref().map(|s| s.dims) != Some(dims) {
                    *slot = Some(FluidSession {
                        solver: FluidSolver::new(&gpu.device, dims.0, dims.1),
                        compositor: FluidCompositor::new(&gpu.device),
                        dims,
                        epoch: u64::MAX,
                        frame: 0,
                    });
                }
            });
        }
        return;
    }

    painter.set_gpu_fluid_driven(true);
    let Some(dims) = painter.fluid_grid_dims() else {
        return;
    };
    let substeps = painter.fluid_idle_substeps();
    let epoch = painter.fluid_stroke_epoch();
    let scale = painter.fluid_field_scale();
    let coverage_k = painter.fluid_coverage_k();
    let (cw, ch) = painter.source_size();
    if cw == 0 || ch == 0 {
        return;
    }

    let profile = PROFILE.with(|p| p.borrow_mut().enabled());
    SESSION.with(|cell| {
        let mut slot = cell.borrow_mut();
        // (Re)build the session on a grid-size change (pre-warm usually already did).
        if slot.as_ref().map(|s| s.dims) != Some(dims) {
            *slot = Some(FluidSession {
                solver: FluidSolver::new(&gpu.device, dims.0, dims.1),
                compositor: FluidCompositor::new(&gpu.device),
                dims,
                epoch: u64::MAX,
                frame: 0,
            });
        }
        let Some(sess) = slot.as_mut() else {
            return;
        };
        // ── New stroke (epoch change): cheap per-stroke setup, ONCE ──
        if sess.epoch != epoch {
            // Resident path: pigment + water + deposited start each stroke empty (water
            // is GPU-resident now — `cs_splat` adds it, `cs_evaporate` dries it).
            sess.solver
                .clear_resident_pigment_gpu(&gpu.device, &gpu.queue);
            sess.solver
                .clear_resident_water_gpu(&gpu.device, &gpu.queue);
            sess.solver
                .clear_resident_deposited_gpu(&gpu.device, &gpu.queue);
            // ADR-0078 S3d: the shallow-water velocity + pressure start each stroke at rest
            // (no leftover momentum).
            sess.solver
                .clear_resident_velocity_gpu(&gpu.device, &gpu.queue);
            sess.frame = 0;
            // ADR-0079: drive ALL 15 solver controls (base diffusion + deposition +
            // shallow-water flow) from the ACTIVE BRUSH's per-brush `WatercolorParams`
            // (projected to `DiffusionParams`), replacing the old `FluidParams::default()`
            // + global `WATERCOLOR_*` consts. The artist's Brush Studio "Watercolor"
            // sliders now drive the live wash. `cs_combine` still feeds the compositor
            // `flowing + deposited` via `total_buffer()`; the velocity layer is dormant if
            // the brush sets `velocity = 0`.
            sess.solver
                .set_from_diffusion(&gpu.queue, &painter.fluid_diffusion_params());
            if let Some(paper) = painter.fluid_paper() {
                sess.solver.upload_paper(&gpu.queue, &paper);
            }
            if let Some(backdrop) = painter.fluid_backdrop() {
                let brush = prepare_wet_composite_from_stroke(painter.fluid_stroke_color_linear());
                // Coverage supersampling: 1 at full-res (edge already 1px-fine — saves
                // 4× the K–M cost), 2 at half-res to antialias the steeper edge.
                let ss = if scale <= 1 { 1 } else { 2 };
                sess.compositor.begin_stroke(
                    &gpu.device,
                    &gpu.queue,
                    dims.0,
                    dims.1,
                    cw,
                    ch,
                    scale,
                    coverage_k,
                    ss,
                    // Composite the TOTAL (flowing + deposited) so edge-darkening +
                    // granulation are visible (ADR-0078 S3c); equals flowing when
                    // nothing is deposited, so non-deposition strokes are unchanged.
                    sess.solver.total_buffer(),
                    backdrop,
                    &brush,
                );
            }
            sess.epoch = epoch;
        }

        // ── Per-frame hot loop (resident, no O(grid) CPU work, no upload) ──
        // Drain this frame's dabs + the monotonic envelope; `None` ⇒ never wet.
        let Some((dabs, region)) = painter.fluid_take_dabs() else {
            return;
        };
        // Map the tool's plain dabs → GPU dabs (the `r.max(0.5)` / radius>0 guard lives
        // in `DabGpu::new`, mirroring the CPU splat). `cs_splat` adds them to the
        // resident water + pigment; then diffuse/advect/evaporate run on the GPU.
        let gpu_dabs: Vec<DabGpu> = dabs
            .iter()
            .filter_map(|d| DabGpu::new(d.cx, d.cy, d.r, d.water, d.rgb))
            .collect();
        let t0 = profile.then(Instant::now);
        // Region-scoped (ADR-0078 S1): the sim runs only over the wet envelope (padded
        // inside the solver to ⊇ the composite region), so the per-frame cost is
        // O(wet frontier), not O(grid) — the dominant 4K cost (per the perf bench).
        sess.solver
            .step_resident_splat(&gpu.device, &gpu.queue, &gpu_dabs, substeps, region);
        let t1 = profile.then(Instant::now);
        // Composite PIPELINED (async readback, no per-frame device.poll(wait) stall) →
        // ~240 FPS. The real click→stroke delay was the per-stroke O(grid) paper
        // grain_noise (fixed by the paper cache), NOT the pipeline's 1-frame-late
        // readback (which is imperceptible — the same 1-frame lag the preview always
        // had). begin_stroke drains the prior stroke's in-flight map before reuse.
        let (band, rect) = sess
            .compositor
            .composite_frame_pipelined(&gpu.device, &gpu.queue, region);
        if !band.is_empty() {
            painter.fluid_apply_gpu_composite_rows(&band, rect);
        }
        let t2 = profile.then(Instant::now);
        // Sporadic dry-check: the GPU reduces max-water (no CPU O(grid) scan); when
        // it's below the dry threshold AND the stroke has ended, the field drops.
        sess.frame = sess.frame.wrapping_add(1);
        let mut stats_us = 0u64;
        if sess.frame % DRY_CHECK_EVERY == 0 {
            let ts = profile.then(Instant::now);
            let stats = sess.solver.read_field_stats(&gpu.device, &gpu.queue, 1.0e-3);
            painter.fluid_dry_check_and_drop_gpu(stats.max_water);
            stats_us = ts.map_or(0, |t| t.elapsed().as_micros() as u64);
        }
        if profile {
            let step_us = t1.unwrap().duration_since(t0.unwrap()).as_micros() as u64;
            let comp_us = t2.unwrap().duration_since(t1.unwrap()).as_micros() as u64;
            PROFILE.with(|p| p.borrow_mut().record(step_us, comp_us, stats_us));
        }
    });
}
