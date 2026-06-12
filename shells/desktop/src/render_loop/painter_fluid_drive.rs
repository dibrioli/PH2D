//! Per-frame fluid-drive helpers extracted from [`super::painter_fluid_bridge`] (HR-18
//! decomposition — the bridge's `drive_fluid_gpu` orchestration + the thread-local `SESSION`
//! stay there; these are the large cohesive sub-phases that operate on a `&mut FluidSession`).
#![cfg(feature = "fluid")]

use super::painter_fluid_support::{FluidSession, ensure_preview_slot, union_bbox};
use super::sim_extract::PreviewOverride;
use ph2d_gpu::GpuContext;
use ph2d_painter_fluid::DabGpu;
use ph2d_render::SpriteRenderer;

/// **New-stroke setup (epoch change), ONCE per stroke.** Clears the resident
/// pigment/water/deposited/velocity, uploads the active brush's `DiffusionParams` + paper +
/// (optional ADR-0084) backdrop-lift donor, and binds the compositor for the stroke. No-op when
/// the session epoch already matches (mid-stroke frames). Extracted verbatim from the bridge.
#[allow(clippy::too_many_arguments)] // a per-frame drive phase threads the frame's GPU state
pub(super) fn maybe_begin_fluid_stroke(
    sess: &mut FluidSession,
    gpu: &GpuContext,
    painter: &ph2d_tool_painter::PainterTool,
    epoch: u64,
    dims: (u32, u32),
    cw: u32,
    ch: u32,
    scale: u32,
    coverage_k: f32,
) {
    if sess.epoch == epoch {
        return;
    }
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
    sess.active_history.clear();
    sess.wet_bbox = None;
    // E4: `begin_stroke` below recreates the compositor's preview texture (seeded with the new
    // premultiplied backdrop), so nothing is published yet this stroke. `texture_mode_dirty` is
    // intentionally KEPT across epochs: if a new stroke begins before the previous transition
    // bake ran, `flush_pending_bake` (pointer-down, BEFORE this) already drained it — and a
    // surviving union still names rows canvas_rgba never received, so it stays correct.
    sess.texture_published = false;
    // **Re-seed the preview slot (ADR-0085 — the undo FLASH fix).** The slot is reused across a
    // fresh stroke (same dims); if it stayed `seeded`, this stroke's single-submit would only
    // dirty-rect its own region, leaving the PRIOR stroke's pixels visible in the rest of the slot
    // for a frame — e.g. an UNDONE stroke flashing back when you repaint. Forcing `seeded = false`
    // makes the first frame FULL-copy the (fresh, reverted) backdrop, so no stale pixels survive.
    sess.preview_slot_seeded = false;
    // ADR-0079: drive ALL 15 solver controls (base diffusion + deposition +
    // shallow-water flow) from the ACTIVE BRUSH's per-brush `WatercolorParams`
    // (projected to `DiffusionParams`), replacing the old `FluidParams::default()`
    // + global `WATERCOLOR_*` consts. The artist's Brush Studio "Watercolor"
    // sliders now drive the live wash. `cs_combine` still feeds the compositor
    // `flowing + deposited` via `total_buffer()`; the velocity layer is dormant if
    // the brush sets `velocity = 0`.
    let dp = painter.fluid_diffusion_params();
    sess.solver.set_from_diffusion(&gpu.queue, &dp);
    if let Some(paper) = painter.fluid_paper() {
        sess.solver.upload_paper(&gpu.queue, &paper);
    }
    // ADR-0084 backdrop lift: when the brush's `lift > 0`, seed the donor (`lift_source`)
    // from the current canvas backdrop vs the session's original PAPER (downsampled to the
    // grid by the SAME free fn the CPU seed uses → bit-identical CPU↔GPU, HR-5) and zero
    // `lifted_frac`; the wet brush then re-mobilizes that dry PAINT into the wash (+ the
    // compositor reveals the paper under the lifted pixels — never transparency). When no
    // paper snapshot exists, `paper == backdrop` ⇒ empty donor ⇒ inert (the safe fallback).
    // On the non-lift path zero BOTH so a fresh stroke with `lift = 0` has an inert donor +
    // `lifted_frac ≡ 0` → the compositor is byte-identical (the non-destructive default).
    if dp.lift > 0.0 {
        if let Some(backdrop) = painter.fluid_backdrop() {
            let paper = painter.fluid_paper_base().unwrap_or(backdrop);
            let cells = ph2d_painter_brush::diffusion::backdrop_to_lift_source(
                backdrop, paper, cw, ch, dims.0, dims.1,
            );
            sess.solver.clear_lift_gpu(&gpu.device, &gpu.queue);
            sess.solver.upload_lift_source(&gpu.queue, &cells);
        } else {
            sess.solver.clear_lift_gpu(&gpu.device, &gpu.queue);
        }
    } else {
        sess.solver.clear_lift_gpu(&gpu.device, &gpu.queue);
    }
    if let Some(backdrop) = painter.fluid_backdrop() {
        // ADR-0080: pigment colour is per-pixel (reduced from the field), so no
        // per-stroke brush — `begin_stroke` just binds the field + backdrop.
        // Coverage supersampling (N×N glaze samples/pixel). ss=2 EVERYWHERE (ADR-0085 + the
        // pixelated-rim fix): the composite now BILINEAR-samples the field at scale=1 too
        // (`sample_field_bicubic`), so the ss=2 sub-samples (`fx ± 0.25`) read interpolated edge
        // values and the wash rim anti-aliases instead of hard-stepping at the cell grid. (The old
        // scale=1 ss=1 fast path relied on a NEAREST read where sub-samples were redundant — that
        // is what pixelated every border-darkening param.)
        let ss = 2u32;
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
            // ADR-0084 paper-reveal: the session's original canvas content — lifted
            // pixels lerp back toward it (never toward transparency). Falls back to the
            // backdrop itself (`mix(b, b, lf) = b` ⇒ exact no-op) when no snapshot exists.
            painter.fluid_paper_base().unwrap_or(backdrop),
            // ADR-0084: bind the lift accumulator so the compositor reveals the paper
            // where dry paint was lifted. All-zero (cleared above) when `lift = 0`
            // ⇒ byte-identical output.
            Some(sess.solver.lifted_frac_buffer()),
            // Wet-paper sheen: bind the resident water so the preview-texture
            // passes can darken wet regions + brighten the meniscus (view-only;
            // the flag is driven per frame via `set_wet_sheen` below).
            Some(sess.solver.water_buffer()),
        );
    }
    sess.epoch = epoch;
}

/// **R1 single-submit hot path (mid-stroke, trivial stack).** Folds the sim step, the to-texture
/// composite AND the preview-slot copy into ONE encoder / ONE `queue.submit`, returning the
/// `PreviewOverride` that samples the freshly-composited slot THIS frame plus `texture_frame`
/// (whether the texture path published, so the E5 arm skips). Extracted verbatim from the bridge;
/// the caller guards on `trivial_hot && override_entity.is_some()`.
#[allow(clippy::too_many_arguments)] // a per-frame drive phase threads the frame's GPU state
pub(super) fn encode_single_submit_frame(
    sess: &mut FluidSession,
    gpu: &GpuContext,
    renderer: &mut SpriteRenderer,
    region: (u32, u32, u32, u32),
    gpu_dabs: &[DabGpu],
    substeps: u32,
    entity_bits: u64,
    cw: u32,
    ch: u32,
) -> (Option<PreviewOverride>, bool) {
    let mut override_out = None;
    let mut texture_frame = false;
    let enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fluid frame (R1 single submit)"),
        });
    // (1) Sim step (splat + substeps + combine) — encoded, not submitted.
    let mut enc = sess.solver.encode_resident_splat_step(
        &gpu.device,
        &gpu.queue,
        enc,
        gpu_dabs,
        substeps,
        region,
    );
    // (2) To-texture composite over the wet region (SAME encoder). `None` ⇒
    // empty region: still submit the stepped field, fall through to readback.
    if let Some((px_lo, py_lo, px_hi, py_hi)) = sess
        .compositor
        .encode_frame_to_texture(&gpu.queue, &mut enc, region)
    {
        // (3) Acquire/resize the slot (no GPU work); a FRESH or never-seeded
        // slot gets the full backdrop seeded ONCE, then per-frame dirty-rect
        // refreshes of only the wet rect (ADR-0085 §2.3-I2). All in `enc`.
        let (id, fresh) = ensure_preview_slot(renderer, &mut sess.preview_slot, cw, ch);
        let seed = fresh || !sess.preview_slot_seeded;
        let copy_ok = if let Some(tex) = sess.compositor.preview_texture() {
            if seed {
                renderer
                    .encode_copy_into_individual(&mut enc, id, tex, cw, ch)
                    .is_ok()
            } else {
                renderer
                    .encode_copy_region_into_individual(
                        &mut enc,
                        id,
                        tex,
                        px_lo,
                        py_lo,
                        px_lo,
                        py_lo,
                        px_hi - px_lo,
                        py_hi - py_lo,
                    )
                    .is_ok()
            }
        } else {
            false
        };
        gpu.queue.submit([enc.finish()]); // ← the SINGLE submit
        if copy_ok {
            sess.preview_slot_seeded = true;
            // Catch-up accumulator: union every grid region the texture path composited
            // (canvas_rgba is stale over it until the readback lane's single sync transition
            // bake — or `flush_pending_bake` at pointer-down — catches it up).
            sess.texture_mode_dirty = Some(match sess.texture_mode_dirty {
                Some(d) => union_bbox(d, region),
                None => region,
            });
            sess.texture_published = true;
            texture_frame = true;
            override_out = Some(PreviewOverride {
                entity_bits,
                texture_id: id,
                premultiplied: true,
            });
        } else {
            // Copy failed (or no preview texture yet) → release the just-acquired slot
            // and drop the seed, mirroring `copy_preview_into_slot`'s failure path (C5:
            // else the unused canvas-res slot is held until teardown while the readback
            // lane runs). The readback path (canvas_rgba + the CPU preview) keeps the
            // stroke alive at a 1-frame lag; a recovered frame re-acquires + re-seeds.
            if let Some((old, _, _)) = sess.preview_slot.take() {
                renderer.individual_mut().release(old);
            }
            sess.preview_slot_seeded = false;
        }
    } else {
        gpu.queue.submit([enc.finish()]);
    }
    (override_out, texture_frame)
}
