//! W15.3 GPU fluid drive (ADR-0049, feature `fluid`) — steps the painter's live
//! wet field on the GPU each frame, then runs the tool's composite.
//!
//! Feature-gated: the default build excludes this module entirely (and the
//! `ph2d-painter-fluid` crate), so the painter runs the CPU diffusion path. When
//! `--features fluid` is set, the render loop calls [`drive_fluid_gpu`] right after
//! the active tool's `on_tick`.
//!
//! Downcasts to the concrete `PainterTool` (allowlisted bridge, same exception
//! class as `painter_bridge.rs`): the GPU drive needs the concrete wet-field hooks
//! (`has_wet_field` / `fluid_grid_mut` / `composite_and_settle_fluid`).

use ph2d_editor::ToolRegistry;
use ph2d_gpu::GpuContext;
use ph2d_painter_fluid::{FluidParams, FluidSolver};
use std::cell::RefCell;

thread_local! {
    /// The persistent GPU solver for the active wet field, keyed by its grid size.
    /// Rebuilt when the field resizes (a new canvas); dropped when no live field
    /// (the wash dried or no fluid stroke). Thread-local because the render loop is
    /// single-threaded (the established painter-bridge pattern for per-tool state).
    static SESSION: RefCell<Option<(FluidSolver, (u32, u32))>> = const { RefCell::new(None) };
}

/// Drive the live wet field on the GPU, called each frame after the active tool's
/// `on_tick`. When the painter has a live field: marks the tool GPU-driven (so its
/// `on_tick`/`queue_pointer` skip the CPU diffusion), steps the field on the GPU
/// ([`FluidSolver::step_grid`], parity-proven against the CPU reference), and runs
/// the tool's composite ([`PainterTool::composite_and_settle_fluid`]). No-op
/// without an active painter or a live field (and it then releases the solver).
pub(crate) fn drive_fluid_gpu(tools: &mut ToolRegistry, gpu: &GpuContext) {
    let Some(painter) = tools
        .active_mut()
        .and_then(|t| t.as_any_mut().downcast_mut::<ph2d_tool_painter::PainterTool>())
    else {
        return;
    };
    // **Graceful degrade (ADR-0049 §2.8/§2.9).** Only a software/CPU adapter is
    // ruled incapable; every real GPU (discrete / integrated incl. Apple Silicon /
    // virtual / other-Metal) stays eligible. VRAM-free probing isn't portable in
    // wgpu, so the 32 MB floor is assumed met on any real GPU (the 1/2-res textures
    // are tens of MB) — refine with real telemetry. When incapable, the field falls
    // back to the CPU path (the tool's on_tick) — identity preserved.
    let tier = match gpu.adapter.get_info().device_type {
        wgpu::DeviceType::Cpu => ph2d_host::MemoryTier::Low,
        wgpu::DeviceType::IntegratedGpu | wgpu::DeviceType::VirtualGpu => ph2d_host::MemoryTier::Mid,
        _ => ph2d_host::MemoryTier::High,
    };
    let capable = ph2d_host::MemoryBudget { vram_free_mb: 256, tier }.fluid_capable();
    if !painter.has_wet_field()
        || !ph2d_painter_fluid::fluid_pass_eligible(true, capable, f32::INFINITY)
    {
        painter.set_gpu_fluid_driven(false);
        SESSION.with(|s| *s.borrow_mut() = None);
        return;
    }
    painter.set_gpu_fluid_driven(true);
    let Some(dims) = painter.fluid_grid_mut().map(|g| g.dims()) else {
        return;
    };
    let substeps = painter.fluid_idle_substeps();
    SESSION.with(|cell| {
        let mut sess = cell.borrow_mut();
        if sess.as_ref().map(|(_, d)| *d) != Some(dims) {
            *sess = Some((FluidSolver::new(&gpu.device, dims.0, dims.1), dims));
        }
        if let (Some((solver, _)), Some(grid)) = (sess.as_ref(), painter.fluid_grid_mut()) {
            solver.step_grid(&gpu.device, &gpu.queue, grid, &FluidParams::default(), substeps);
        }
    });
    painter.composite_and_settle_fluid();
}
