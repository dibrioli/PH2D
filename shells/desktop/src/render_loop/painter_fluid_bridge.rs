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
use ph2d_painter_fluid::{FluidCompositor, FluidParams, FluidSolver};
use std::cell::RefCell;

/// Per-session GPU state for the live wet field: the resident solver, the K–M
/// compositor, the field size, and the stroke epoch it was last reset for.
struct FluidSession {
    solver: FluidSolver,
    compositor: FluidCompositor,
    dims: (u32, u32),
    epoch: u64,
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
    let Some(painter) = tools
        .active_mut()
        .and_then(|t| t.as_any_mut().downcast_mut::<ph2d_tool_painter::PainterTool>())
    else {
        return;
    };
    // **Graceful degrade (ADR-0049 §2.8/§2.9).** Only a software/CPU adapter is ruled
    // incapable; every real GPU stays eligible. VRAM-free probing isn't portable in
    // wgpu, so the floor is assumed met on any real GPU (refine with telemetry). When
    // incapable, the field falls back to the CPU path (the tool's on_tick).
    let tier = match gpu.adapter.get_info().device_type {
        wgpu::DeviceType::Cpu => ph2d_host::MemoryTier::Low,
        wgpu::DeviceType::IntegratedGpu | wgpu::DeviceType::VirtualGpu => ph2d_host::MemoryTier::Mid,
        _ => ph2d_host::MemoryTier::High,
    };
    let capable = ph2d_host::MemoryBudget { vram_free_mb: 256, tier }.fluid_capable();
    // W15.3 full-res: a capable GPU runs the NEXT fluid field at full canvas
    // resolution (sharp edges / fine bleeds). Set every frame (even with no live
    // field) so it's in effect before the next `begin_stroke`.
    painter.set_fluid_hires(capable);
    if !painter.has_wet_field()
        || !ph2d_painter_fluid::fluid_pass_eligible(true, capable, f32::INFINITY)
    {
        painter.set_gpu_fluid_driven(false);
        SESSION.with(|s| *s.borrow_mut() = None);
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
    let evap_per_frame = painter.fluid_evaporation() * substeps as f32;
    let (cw, ch) = painter.source_size();
    if cw == 0 || ch == 0 {
        return;
    }

    SESSION.with(|cell| {
        let mut slot = cell.borrow_mut();
        // (Re)build the session on a size change; force a reset by mismatching epoch.
        if slot.as_ref().map(|s| s.dims) != Some(dims) {
            *slot = Some(FluidSession {
                solver: FluidSolver::new(&gpu.device, dims.0, dims.1),
                compositor: FluidCompositor::new(&gpu.device),
                dims,
                epoch: u64::MAX,
            });
        }
        let Some(sess) = slot.as_mut() else {
            return;
        };
        sess.solver.set_params(&gpu.queue, &FluidParams::default());
        // New stroke (or new session) → reset the resident pigment + upload paper.
        if sess.epoch != epoch {
            sess.solver.clear_resident_pigment(&gpu.queue);
            if let Some(paper) = painter.fluid_paper() {
                sess.solver.upload_paper(&gpu.queue, &paper);
            }
            sess.epoch = epoch;
        }

        // This frame's dabs + water mirror (clears the grid pigment, evaporates water).
        let Some(inp) = painter.fluid_frame_step_inputs(evap_per_frame) else {
            // Bare field — let the dry-check drop it.
            painter.fluid_dry_check_and_drop();
            return;
        };
        sess.solver
            .step_resident(&gpu.device, &gpu.queue, &inp.water, &inp.deposit, substeps);

        // Composite reading the resident pigment → the wet row band → canvas_rgba.
        let composited = painter.fluid_backdrop().map(|backdrop| {
            let brush = prepare_wet_composite_from_stroke(painter.fluid_stroke_color_linear());
            sess.compositor.composite_buffer_rows(
                &gpu.device,
                &gpu.queue,
                dims.0,
                dims.1,
                cw,
                ch,
                scale,
                coverage_k,
                sess.solver.pigment_buffer(),
                backdrop,
                &brush,
                inp.region,
            )
        });
        if let Some((band, rect)) = composited
            && !band.is_empty()
        {
            painter.fluid_apply_gpu_composite_rows(&band, rect);
        }

        // Dry-check on the CPU water mirror; drop the field when it dries (its final
        // pigment was just composited).
        painter.fluid_dry_check_and_drop();
    });
}
