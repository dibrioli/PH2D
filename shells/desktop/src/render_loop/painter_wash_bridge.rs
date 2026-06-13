//! Minimal watercolor core drive (ADR-0086/0087, feature `wash`) — the PARALLEL, drastically
//! simpler counterpart to `painter_fluid_bridge`. Reuses the tool's fluid lifecycle (wet-field
//! carrier, dab list, backdrop snapshot, epoch/dims) and ports the v2 zero-stall hot path:
//!
//! - **ONE submit/frame**: `cs_splat` (region) + `cs_step`×substeps (region) + the to-texture
//!   composite (region) + the preview-slot copy (full SEED once, then DIRTY-RECT) all in one encoder
//!   → a `PreviewOverride` the renderer samples in place of the sprite. NO per-frame readback / poll.
//! - **Active-region window**: the per-frame work follows the recent dabs (not the monotonic
//!   envelope), so a long stroke / settled wash costs O(brush), not O(canvas).
//! - **Backdrop built ONLY on stroke start** (not every frame — that full-canvas `Vec` copy was a
//!   per-frame CPU stall).
//! - **Finalize**: a short bloom window after pen-up, then ONE readback bakes `canvas_rgba` and the
//!   session idles for free until the next stroke.
//!
//! Set `PH2D_WASH_PROFILE=1` to print the per-frame CPU cost + region size every 120 frames.
//! Downcasts to the concrete `PainterTool` (allowlisted bridge).
#![cfg(feature = "wash")]

use super::sim_extract::PreviewOverride;
use ph2d_editor::ToolRegistry;
use ph2d_gpu::GpuContext;
use ph2d_painter_wash::{Dab, WashCompositor, WashParams, WashSolver};
use ph2d_render::SpriteRenderer;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::time::Instant;

/// Frames a dab's area stays in the per-frame work region after it lands (then it freezes — its
/// last composite persists in the preview slot). Doubles as the post-pen-up bloom window.
const ACTIVE_WINDOW: u64 = 30;
/// Wick pad (grid cells) grown around the active region.
const REGION_PAD: u32 = 8;

thread_local! {
    static WASH_SESSION: RefCell<Option<WashSession>> = const { RefCell::new(None) };
    static PROFILE: RefCell<Profile> = const { RefCell::new(Profile::new()) };
}

struct Profile {
    on: Option<bool>,
    frames: u64,
    cpu_us: u64,
    last_region_cells: u64,
}
impl Profile {
    const fn new() -> Self {
        Self { on: None, frames: 0, cpu_us: 0, last_region_cells: 0 }
    }
    fn enabled(&mut self) -> bool {
        *self.on.get_or_insert_with(|| std::env::var("PH2D_WASH_PROFILE").is_ok())
    }
}

struct WashSession {
    solver: WashSolver,
    compositor: WashCompositor,
    dims: (u32, u32),
    epoch: u64,
    slot: Option<(u32, u32, u32)>,
    seeded: bool,
    /// (frame, grid bbox) of recent dabs — the active-region window.
    active_history: VecDeque<(u64, (u32, u32, u32, u32))>,
    frame: u64,
    idle_frames: u32,
    finalized: bool,
}

impl WashSession {
    fn release_slot(&mut self, renderer: &mut SpriteRenderer) {
        if let Some((id, _, _)) = self.slot.take() {
            renderer.individual_mut().release(id);
        }
    }
}

fn union(a: (u32, u32, u32, u32), b: (u32, u32, u32, u32)) -> (u32, u32, u32, u32) {
    (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3))
}

fn wash_params_from(dp: &ph2d_painter_brush::diffusion::DiffusionParams) -> WashParams {
    WashParams {
        diffusivity: dp.diffusivity.clamp(0.0, 0.25),
        flow_outward: dp.flow_outward.max(0.0),
        evaporation: dp.evaporation.max(0.0),
        w_lo: dp.w_lo,
        w_hi: dp.w_hi,
        perm_valley: dp.perm_valley,
        perm_crest: dp.perm_crest,
        ..WashParams::default()
    }
}

fn drop_session(renderer: &mut SpriteRenderer) {
    WASH_SESSION.with(|c| {
        let mut b = c.borrow_mut();
        if let Some(s) = b.as_mut() {
            s.release_slot(renderer);
        }
        *b = None;
    });
}

pub(crate) fn drive_wash_gpu(
    tools: &mut ToolRegistry,
    gpu: &GpuContext,
    renderer: &mut SpriteRenderer,
    override_entity: Option<u64>,
) -> Option<PreviewOverride> {
    let painter = tools
        .active_mut()
        .and_then(|t| t.as_any_mut().downcast_mut::<ph2d_tool_painter::PainterTool>())?;

    if !painter.wash_brush_enabled() {
        drop_session(renderer);
        return None;
    }
    let capable = !matches!(gpu.adapter.get_info().device_type, wgpu::DeviceType::Cpu);
    painter.set_fluid_hires(capable);
    if !capable || !painter.has_wet_field() {
        drop_session(renderer);
        return None;
    }
    let active = painter.is_stroke_active();
    if !active && WASH_SESSION.with(|c| c.borrow().as_ref().is_some_and(|s| s.finalized)) {
        return None; // settled + baked ⇒ zero idle cost until the next stroke
    }

    let dims = painter.fluid_grid_dims()?;
    let (cw, ch) = painter.source_size();
    if cw == 0 || ch == 0 {
        return None;
    }
    let epoch = painter.fluid_stroke_epoch();
    let scale = painter.fluid_field_scale().max(1);
    let coverage_k = painter.fluid_coverage_k();
    let substeps = if active {
        painter.fluid_painting_substeps()
    } else {
        painter.fluid_idle_substeps()
    };
    let wp = wash_params_from(&painter.fluid_diffusion_params());
    // **Backdrop ONLY on stroke start** (epoch/dims change) — building this full-canvas Vec every
    // frame was a per-frame CPU stall.
    let need_backdrop = WASH_SESSION.with(|c| c.borrow().as_ref().map(|s| (s.dims, s.epoch)) != Some((dims, epoch)));
    let backdrop: Option<Vec<u32>> = if need_backdrop {
        Some(painter.fluid_backdrop()?.chunks_exact(4).map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect())
    } else {
        None
    };
    let (dabs, _envelope) = painter.fluid_take_dabs()?;
    let t0 = PROFILE.with(|p| p.borrow_mut().enabled()).then(Instant::now);

    let out = WASH_SESSION.with(|cell| {
        let mut slot_cell = cell.borrow_mut();
        if slot_cell.as_ref().map(|s| s.dims) != Some(dims) {
            if let Some(old) = slot_cell.as_mut() {
                old.release_slot(renderer);
            }
            *slot_cell = Some(WashSession {
                solver: WashSolver::new(&gpu.device, dims.0, dims.1),
                compositor: WashCompositor::new(&gpu.device),
                dims,
                epoch: u64::MAX,
                slot: None,
                seeded: false,
                active_history: VecDeque::new(),
                frame: 0,
                idle_frames: 0,
                finalized: false,
            });
        }
        let sess = slot_cell.as_mut()?;

        if sess.epoch != epoch {
            let backdrop = backdrop.as_deref()?; // present iff need_backdrop (epoch changed)
            sess.solver.clear(&gpu.device, &gpu.queue);
            sess.compositor.begin_stroke(
                &gpu.device, &gpu.queue, dims.0, dims.1, cw, ch, backdrop, coverage_k,
                sess.solver.pig_buffer(),
            );
            sess.epoch = epoch;
            sess.seeded = false;
            sess.idle_frames = 0;
            sess.finalized = false;
            sess.active_history.clear();
        }
        if active {
            sess.idle_frames = 0;
            sess.finalized = false;
        }
        sess.solver.set_params(&gpu.queue, wp);

        let gpu_dabs: Vec<Dab> = dabs
            .iter()
            .map(|d| Dab::from_color_mass(d.cx, d.cy, d.r.max(0.5), d.water, d.color, d.mass))
            .collect();

        // ── Active-region window: union of recent dab bboxes (NOT the monotonic envelope) ──
        if !gpu_dabs.is_empty() {
            let (mut x0, mut y0, mut x1, mut y1) = (dims.0 - 1, dims.1 - 1, 0u32, 0u32);
            for d in &dabs {
                x0 = x0.min((d.cx - d.r).floor().clamp(0.0, (dims.0 - 1) as f32) as u32);
                y0 = y0.min((d.cy - d.r).floor().clamp(0.0, (dims.1 - 1) as f32) as u32);
                x1 = x1.max((d.cx + d.r).ceil().clamp(0.0, (dims.0 - 1) as f32) as u32);
                y1 = y1.max((d.cy + d.r).ceil().clamp(0.0, (dims.1 - 1) as f32) as u32);
            }
            sess.active_history.push_back((sess.frame, (x0, y0, x1, y1)));
        }
        let cur = sess.frame;
        while sess.active_history.front().is_some_and(|&(f, _)| cur.saturating_sub(f) > ACTIVE_WINDOW) {
            sess.active_history.pop_front();
        }
        sess.frame = sess.frame.wrapping_add(1);
        let region = sess.active_history.iter().map(|(_, b)| *b).reduce(union);

        // Finalize: pen up + nothing recent left to bloom ⇒ bake once + stop.
        if !active {
            sess.idle_frames = sess.idle_frames.saturating_add(1);
            if region.is_none() || sess.idle_frames > ACTIVE_WINDOW as u32 {
                if let Some(words) = sess.compositor.read_preview(&gpu.device, &gpu.queue) {
                    let band: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
                    painter.fluid_apply_gpu_composite_rows(&band, (0, 0, cw, ch));
                }
                sess.finalized = true;
                sess.release_slot(renderer);
                return None;
            }
        }
        let (gx0, gy0, gx1, gy1) = region?;
        let gx0 = gx0.saturating_sub(REGION_PAD);
        let gy0 = gy0.saturating_sub(REGION_PAD);
        let gx1 = (gx1 + REGION_PAD).min(dims.0 - 1);
        let gy1 = (gy1 + REGION_PAD).min(dims.1 - 1);
        let g_region = (gx0, gy0, gx1 - gx0 + 1, gy1 - gy0 + 1);
        let cx0 = (gx0 * scale).min(cw);
        let cy0 = (gy0 * scale).min(ch);
        let cx1 = ((gx1 + 1) * scale).min(cw);
        let cy1 = ((gy1 + 1) * scale).min(ch);
        PROFILE.with(|p| p.borrow_mut().last_region_cells = u64::from(cx1 - cx0) * u64::from(cy1 - cy0));

        // Need the entity to override; without it skip this frame (rare).
        let entity_bits = override_entity?;
        // Acquire/resize the slot (no GPU work).
        let (id, fresh) = match sess.slot {
            Some((id, w, h)) if w == cw && h == ch => (id, false),
            _ => {
                if let Some((old, _, _)) = sess.slot.take() {
                    renderer.individual_mut().release(old);
                }
                let id = renderer.acquire_individual_empty(cw, ch);
                sess.slot = Some((id, cw, ch));
                (id, true)
            }
        };
        let seed = fresh || !sess.seeded;

        // ── ONE encoder: step + composite + slot copy ──
        let enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("wash frame") });
        let mut enc = sess.solver.encode_step(&gpu.queue, enc, &gpu_dabs, substeps, g_region);
        let copy_ok = if seed {
            // Seed: composite the FULL canvas (backdrop everywhere + the wash) + full slot copy.
            sess.compositor.encode_composite(&gpu.queue, &mut enc, (0, 0, cw, ch));
            let tex = sess.compositor.preview_texture();
            tex.is_some_and(|t| renderer.encode_copy_into_individual(&mut enc, id, t, cw, ch).is_ok())
        } else {
            // Dirty-rect: composite + copy only the active region.
            sess.compositor.encode_composite(&gpu.queue, &mut enc, (cx0, cy0, cx1 - cx0, cy1 - cy0));
            let tex = sess.compositor.preview_texture();
            tex.is_some_and(|t| {
                renderer.encode_copy_region_into_individual(&mut enc, id, t, cx0, cy0, cx0, cy0, cx1 - cx0, cy1 - cy0).is_ok()
            })
        };
        gpu.queue.submit([enc.finish()]);

        if copy_ok {
            sess.seeded = true;
            Some(PreviewOverride { entity_bits, texture_id: id, premultiplied: true })
        } else {
            sess.release_slot(renderer);
            sess.seeded = false;
            None
        }
    });

    if let Some(t0) = t0 {
        PROFILE.with(|p| {
            let mut p = p.borrow_mut();
            p.frames += 1;
            p.cpu_us += t0.elapsed().as_micros() as u64;
            if p.frames % 120 == 0 {
                eprintln!(
                    "[wash] cpu {:.2} ms/frame  region {} cells  dims {}x{}",
                    p.cpu_us as f64 / 120.0 / 1000.0,
                    p.last_region_cells,
                    dims.0,
                    dims.1,
                );
                p.cpu_us = 0;
            }
        });
    }
    out
}
