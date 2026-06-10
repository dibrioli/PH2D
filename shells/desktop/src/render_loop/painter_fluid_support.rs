//! Support pieces for the W15.3 GPU fluid drive (`painter_fluid_bridge`) — the
//! opt-in per-phase profiler, bbox helpers, and the preview-slot plumbing. Split
//! out of the bridge for HR-18 (≤600 LOC per shell file); the bridge keeps the
//! per-frame drive logic, this module keeps the leaf utilities.

use ph2d_render::SpriteRenderer;
use std::cell::RefCell;

/// Opt-in per-phase profiler for the fluid drive (`PH2D_FLUID_PROFILE=1`). Confirms
/// where the per-frame wall-clock goes — sim step vs the composite (whose `device.poll`
/// readback is the suspected sync stall) vs the sporadic stats readback — before any
/// structural change. Prints averaged ms to stderr every `WINDOW` active frames.
pub(super) struct FluidProfile {
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
    pub(super) fn enabled(&mut self) -> bool {
        if self.on.is_none() {
            self.on = Some(std::env::var("PH2D_FLUID_PROFILE").is_ok_and(|v| v != "0"));
        }
        self.on == Some(true)
    }
    pub(super) fn record(&mut self, step_us: u64, comp_us: u64, stats_us: u64) {
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
    pub(super) static PROFILE: RefCell<FluidProfile> = const { RefCell::new(FluidProfile::new()) };
}

/// Inclusive union of two grid-cell bboxes `(x0, y0, x1, y1)`.
pub(super) fn union_bbox(a: (u32, u32, u32, u32), b: (u32, u32, u32, u32)) -> (u32, u32, u32, u32) {
    (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3))
}

/// Grow an inclusive grid-cell bbox by `pad` cells on each side, clamped to `dims`.
pub(super) fn grow_bbox(
    b: (u32, u32, u32, u32),
    pad: u32,
    dims: (u32, u32),
) -> (u32, u32, u32, u32) {
    (
        b.0.saturating_sub(pad),
        b.1.saturating_sub(pad),
        (b.2 + pad).min(dims.0.saturating_sub(1)),
        (b.3 + pad).min(dims.1.saturating_sub(1)),
    )
}

/// E4 (ADR-0078 S2): lazily acquire/resize the `IndividualTextureStore` slot and
/// GPU-copy the fluid compositor's premultiplied preview texture into it (the
/// rgba8unorm → Rgba8UnormSrgb copy is format-compatible; the renderer samples
/// it this same frame, before it is ever sampled — the `acquire_empty` contract).
/// Returns the slot id, or `None` on a copy error (slot released; the caller
/// falls back to the readback path, which keeps the preview alive).
pub(super) fn copy_preview_into_slot(
    renderer: &mut SpriteRenderer,
    slot: &mut Option<(u32, u32, u32)>,
    tex: &wgpu::Texture,
    cw: u32,
    ch: u32,
) -> Option<u32> {
    let id = match *slot {
        Some((id, w, h)) if w == cw && h == ch => id,
        _ => {
            if let Some((old, _, _)) = slot.take() {
                renderer.individual_mut().release(old);
            }
            let id = renderer.acquire_individual_empty(cw, ch);
            *slot = Some((id, cw, ch));
            id
        }
    };
    match renderer.copy_texture_into_individual(id, tex, cw, ch) {
        Ok(()) => Some(id),
        Err(e) => {
            eprintln!("warn: fluid preview texture→slot copy failed ({e}); using readback path");
            renderer.individual_mut().release(id);
            *slot = None;
            None
        }
    }
}
