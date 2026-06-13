//! Minimal watercolor core drive (ADR-0086/0087, feature `wash`) — the PARALLEL,
//! drastically-simpler counterpart to `painter_fluid_bridge`. Reuses the tool's fluid
//! lifecycle (wet-field carrier, dab list, backdrop snapshot, epoch/dims — all allocated for a
//! `wash_enabled` brush by the same `begin_stroke` gate the v2 path uses) and the solver-agnostic
//! bake (`fluid_apply_gpu_composite_rows`).
//!
//! **v1 = correctness over perf:** per frame, ONE submit (`cs_splat` + `cs_step` + composite to
//! the preview texture), then a full-canvas readback baked into `canvas_rgba` (the canonical layer
//! the existing preview + Apply/undo consume). No `PreviewOverride` / slot / dirty-rect machinery
//! (those are the v2 perf fast-path; a follow-up). The normal painter preview shows `canvas_rgba`.
//!
//! Downcasts to the concrete `PainterTool` (allowlisted bridge, same exception class as
//! `painter_fluid_bridge`/`painter_bridge`).
#![cfg(feature = "wash")]

use super::sim_extract::PreviewOverride;
use ph2d_editor::ToolRegistry;
use ph2d_gpu::GpuContext;
use ph2d_painter_wash::{Dab, WashCompositor, WashParams, WashSolver};
use ph2d_render::SpriteRenderer;
use std::cell::RefCell;

thread_local! {
    /// Rebuilt on a grid-size change; cleared + re-bound on a new stroke epoch; dropped when the
    /// active brush isn't a wash brush. Single-threaded render loop (the painter-bridge pattern).
    static WASH_SESSION: RefCell<Option<WashSession>> = const { RefCell::new(None) };
}

struct WashSession {
    solver: WashSolver,
    compositor: WashCompositor,
    dims: (u32, u32),
    epoch: u64,
}

/// Project the artist's `DiffusionParams` onto the minimal `WashParams` (only the controls the
/// wash uses; the v2-only knobs — capillary/velocity/lift/… — are ignored). ADR-0087 §4.
fn wash_params_from(dp: &ph2d_painter_brush::diffusion::DiffusionParams) -> WashParams {
    WashParams {
        diffusivity: dp.diffusivity.clamp(0.0, 0.25), // CFL: explicit diffusion stable ≤ 0.25
        flow_outward: dp.flow_outward.max(0.0),
        evaporation: dp.evaporation.max(0.0),
        w_lo: dp.w_lo,
        w_hi: dp.w_hi,
        perm_valley: dp.perm_valley,
        perm_crest: dp.perm_crest,
        ..WashParams::default()
    }
}

/// Drive + composite the live wash on the GPU, called each frame after the active tool's
/// `on_tick`. No-op (and drops the session) unless a wash brush has a live field. Returns `None`
/// (v1 bakes into `canvas_rgba`; no preview override).
pub(crate) fn drive_wash_gpu(
    tools: &mut ToolRegistry,
    gpu: &GpuContext,
    _renderer: &mut SpriteRenderer,
    _override_entity: Option<u64>,
) -> Option<PreviewOverride> {
    let painter = tools
        .active_mut()
        .and_then(|t| t.as_any_mut().downcast_mut::<ph2d_tool_painter::PainterTool>())?;

    let drop_session = || WASH_SESSION.with(|s| *s.borrow_mut() = None);

    if !painter.wash_brush_enabled() {
        drop_session();
        return None;
    }
    // The wet-field carrier is only allocated when `fluid_hires` is set (the tool's begin_stroke
    // gate). The fluid bridge sets it for fluid brushes; the wash bridge must set it for wash
    // brushes (so a `--features wash` build without `fluid` still allocates the field).
    let capable = !matches!(gpu.adapter.get_info().device_type, wgpu::DeviceType::Cpu);
    painter.set_fluid_hires(capable);
    if !capable || !painter.has_wet_field() {
        drop_session();
        return None;
    }

    let dims = painter.fluid_grid_dims()?;
    let (cw, ch) = painter.source_size();
    if cw == 0 || ch == 0 {
        return None;
    }
    let epoch = painter.fluid_stroke_epoch();
    let scale = painter.fluid_field_scale().max(1);
    let coverage_k = painter.fluid_coverage_k();
    let substeps = if painter.is_stroke_active() {
        painter.fluid_painting_substeps()
    } else {
        painter.fluid_idle_substeps()
    };
    let wp = wash_params_from(&painter.fluid_diffusion_params());
    // Backdrop snapshot (RGBA8) → packed u32 (manual, alignment-safe — no bytemuck on the shell).
    let backdrop: Vec<u32> = painter
        .fluid_backdrop()?
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();
    let (dabs, dab_region) = painter.fluid_take_dabs()?;

    WASH_SESSION.with(|cell| {
        let mut slot = cell.borrow_mut();
        if slot.as_ref().map(|s| s.dims) != Some(dims) {
            *slot = Some(WashSession {
                solver: WashSolver::new(&gpu.device, dims.0, dims.1),
                compositor: WashCompositor::new(&gpu.device),
                dims,
                epoch: u64::MAX, // ≠ any real epoch ⇒ begin_stroke fires below
            });
        }
        let sess = slot.as_mut()?;

        // New stroke (epoch change): clear the field + (re)bind the compositor over the fresh
        // backdrop. No-op mid-stroke (the resident field persists + keeps blooming).
        if sess.epoch != epoch {
            sess.solver.clear(&gpu.device, &gpu.queue);
            sess.compositor.begin_stroke(
                &gpu.device,
                &gpu.queue,
                dims.0,
                dims.1,
                cw,
                ch,
                &backdrop,
                coverage_k,
                sess.solver.pig_buffer(),
            );
            sess.epoch = epoch;
        }
        // Live slider sync: re-upload the params every frame (cheap UBO write).
        sess.solver.set_params(&gpu.queue, wp);

        let gpu_dabs: Vec<Dab> = dabs
            .iter()
            .map(|d| Dab::from_color_mass(d.cx, d.cy, d.r.max(0.5), d.water, d.color, d.mass))
            .collect();

        // **Region-scope (the perf fix).** Step + composite + bake only the wet envelope (the
        // monotonic dab bbox) + a wick pad — O(stroke), not O(canvas). `dab_region` is a grid
        // bbox `(x0,y0,x1,y1)` inclusive; map it to the grid step region (ox,oy,w,h) and the
        // canvas rect (× scale).
        let pad = 8u32;
        let (gx0, gy0, gx1, gy1) = dab_region;
        let gx0 = gx0.saturating_sub(pad);
        let gy0 = gy0.saturating_sub(pad);
        let gx1 = (gx1 + pad).min(dims.0 - 1);
        let gy1 = (gy1 + pad).min(dims.1 - 1);
        let g_region = (gx0, gy0, gx1 - gx0 + 1, gy1 - gy0 + 1);
        let cx0 = (gx0 * scale).min(cw);
        let cy0 = (gy0 * scale).min(ch);
        let cx1 = ((gx1 + 1) * scale).min(cw);
        let cy1 = ((gy1 + 1) * scale).min(ch);

        // Single submit: splat + substeps + composite over the wet region.
        let enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("wash frame") });
        let mut enc = sess
            .solver
            .encode_step(&gpu.queue, enc, &gpu_dabs, substeps, g_region);
        sess.compositor
            .encode_composite(&gpu.queue, &mut enc, (cx0, cy0, cx1 - cx0, cy1 - cy0));
        gpu.queue.submit([enc.finish()]);

        // Bake just the wet BAND into `canvas_rgba` (the canonical layer the normal preview +
        // Apply/undo consume) — full-width rows `[cy0, cy1)`, columns clipped to the rect.
        if let Some(words) = sess.compositor.read_preview_band(&gpu.device, &gpu.queue, cy0, cy1) {
            let band: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
            painter.fluid_apply_gpu_composite_rows(&band, (cx0, cy0, cx1, cy1));
        }
        None
    })
}
