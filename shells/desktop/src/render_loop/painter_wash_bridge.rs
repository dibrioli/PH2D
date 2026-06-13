//! Minimal watercolor core drive (ADR-0086/0087, feature `wash`) — the PARALLEL, drastically
//! simpler counterpart to `painter_fluid_bridge`. Reuses the tool's fluid lifecycle (wet-field
//! carrier, dab list, backdrop snapshot, epoch/dims — all allocated for a `wash_enabled` brush by
//! the same `begin_stroke` gate the v2 path uses).
//!
//! **Zero-readback hot path (the perf fix).** Per frame the wash composites to its preview TEXTURE
//! and GPU-copies it into an `IndividualTextureStore` slot (no readback, no per-frame
//! `device.poll(wait)` stall — that blocking sync was the "8/10 FPS even idle" collapse). The
//! returned `PreviewOverride` makes the renderer sample the slot in place of the sprite this frame.
//! `canvas_rgba` is baked ONCE, on stroke FINALIZE (a short bloom window after pen-up): a single
//! readback → `fluid_apply_gpu_composite_rows`, then the session stops doing GPU work (idle FPS is
//! full) until the next stroke. Downcasts to the concrete `PainterTool` (allowlisted bridge).
#![cfg(feature = "wash")]

use super::sim_extract::PreviewOverride;
use ph2d_editor::ToolRegistry;
use ph2d_gpu::GpuContext;
use ph2d_painter_wash::{Dab, WashCompositor, WashParams, WashSolver};
use ph2d_render::SpriteRenderer;
use std::cell::RefCell;

/// Idle frames (pointer up) the wash keeps blooming before it finalizes (bakes + stops). Short —
/// the wash settles fast and freezing it then makes idle cost zero.
const BLOOM_FRAMES: u32 = 24;
/// Wick pad (grid cells) grown around the dab envelope for the work region.
const REGION_PAD: u32 = 8;

thread_local! {
    static WASH_SESSION: RefCell<Option<WashSession>> = const { RefCell::new(None) };
}

struct WashSession {
    solver: WashSolver,
    compositor: WashCompositor,
    dims: (u32, u32),
    epoch: u64,
    /// Renderer preview slot `(id, w, h)` the composited texture is copied into.
    slot: Option<(u32, u32, u32)>,
    idle_frames: u32,
    /// Set once the stroke has settled + baked into `canvas_rgba`; the session then idles for free.
    finalized: bool,
}

impl WashSession {
    fn release_slot(&mut self, renderer: &mut SpriteRenderer) {
        if let Some((id, _, _)) = self.slot.take() {
            renderer.individual_mut().release(id);
        }
    }
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

/// Drop the session, releasing its renderer slot first (refcounted store).
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
    // Finalized + idle ⇒ the stroke already baked into `canvas_rgba`; do NOTHING (idle FPS is
    // full). A new stroke (pointer down ⇒ active) resumes below.
    let finalized_idle = WASH_SESSION.with(|c| {
        c.borrow().as_ref().is_some_and(|s| s.finalized)
    });
    if finalized_idle && !active {
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
    let substeps = if active {
        painter.fluid_painting_substeps()
    } else {
        painter.fluid_idle_substeps()
    };
    let wp = wash_params_from(&painter.fluid_diffusion_params());
    let backdrop: Vec<u32> = painter
        .fluid_backdrop()?
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();
    let (dabs, dab_region) = painter.fluid_take_dabs()?;

    WASH_SESSION.with(|cell| {
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
                idle_frames: 0,
                finalized: false,
            });
        }
        let sess = slot_cell.as_mut()?;

        // New stroke (fresh field): clear + (re)bind the compositor + SEED the full preview texture
        // with the backdrop (so the slot copy shows the whole canvas, not just the wet region).
        if sess.epoch != epoch {
            sess.solver.clear(&gpu.device, &gpu.queue);
            sess.compositor.begin_stroke(
                &gpu.device, &gpu.queue, dims.0, dims.1, cw, ch, &backdrop, coverage_k,
                sess.solver.pig_buffer(),
            );
            let mut seed = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("wash seed"),
            });
            sess.compositor.encode_composite(&gpu.queue, &mut seed, (0, 0, cw, ch));
            gpu.queue.submit([seed.finish()]);
            sess.epoch = epoch;
            sess.idle_frames = 0;
            sess.finalized = false;
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

        // Region-scope step + composite to the wet envelope + pad (O(stroke)).
        let (gx0, gy0, gx1, gy1) = dab_region;
        let gx0 = gx0.saturating_sub(REGION_PAD);
        let gy0 = gy0.saturating_sub(REGION_PAD);
        let gx1 = (gx1 + REGION_PAD).min(dims.0 - 1);
        let gy1 = (gy1 + REGION_PAD).min(dims.1 - 1);
        let g_region = (gx0, gy0, gx1 - gx0 + 1, gy1 - gy0 + 1);
        let cx0 = (gx0 * scale).min(cw);
        let cy0 = (gy0 * scale).min(ch);
        let cx1 = ((gx1 + 1) * scale).min(cw);
        let cy1 = ((gy1 + 1) * scale).min(ch);

        let enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("wash frame") });
        let mut enc = sess.solver.encode_step(&gpu.queue, enc, &gpu_dabs, substeps, g_region);
        sess.compositor
            .encode_composite(&gpu.queue, &mut enc, (cx0, cy0, cx1 - cx0, cy1 - cy0));
        gpu.queue.submit([enc.finish()]);

        // Idle accounting → finalize (bake once + stop). Done BEFORE the override so the finalize
        // frame returns None (the renderer falls to the freshly-baked `canvas_rgba`, no flash).
        if !active {
            sess.idle_frames = sess.idle_frames.saturating_add(1);
            if sess.idle_frames > BLOOM_FRAMES {
                if let Some(words) = sess.compositor.read_preview(&gpu.device, &gpu.queue) {
                    let band: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
                    painter.fluid_apply_gpu_composite_rows(&band, (0, 0, cw, ch));
                }
                sess.finalized = true;
                sess.release_slot(renderer);
                return None;
            }
        }

        // Zero-readback preview: GPU-copy the composited texture into the slot, sample it as the
        // sprite this frame. Needs the entity the painter owns; without it, no override this frame.
        let entity_bits = override_entity?;
        let tex = sess.compositor.preview_texture()?;
        let id = match sess.slot {
            Some((id, w, h)) if w == cw && h == ch => id,
            _ => {
                if let Some((old, _, _)) = sess.slot.take() {
                    renderer.individual_mut().release(old);
                }
                let id = renderer.acquire_individual_empty(cw, ch);
                sess.slot = Some((id, cw, ch));
                id
            }
        };
        match renderer.copy_texture_into_individual(id, tex, cw, ch) {
            Ok(()) => Some(PreviewOverride { entity_bits, texture_id: id, premultiplied: true }),
            Err(_) => {
                sess.release_slot(renderer);
                None
            }
        }
    })
}
