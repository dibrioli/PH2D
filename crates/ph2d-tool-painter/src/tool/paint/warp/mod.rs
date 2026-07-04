//! **Deform** (Liquify) — the brush-driven reshape sub-tool (Deform Wave 1). One inverse-warp kernel
//! ([`apply`]) driven by per-mode displacement fields ([`field`]) + a Reconstruct resampler
//! ([`reconstruct`]); only the field `D` changes between Push / Twist / Pinch / Wrinkle / Fold. Writes the
//! active raster directly (structural undo, one entry per stroke — mirrors `paint_begin`/`close_stroke`),
//! reuses the Selection mask for **Freeze/Protect**, and holds a session **pre-deform** buffer that
//! Reconstruct fades back to.
//!
//! State lives in [`DeformState`] on `PaintState` (mode-exclusive, like Selection). The panel section is
//! driven by the `is_deform` flag on the [`super::BrushSettings`] snapshot; the setters below are the
//! single clamp source (`apply_ui_edit`), routed from `handle_panel_event`.

mod apply;
mod field;
mod reconstruct;
mod transform;

use super::Region;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};
use std::sync::Arc;

/// The number of Reshape sub-modes (Push · Twist · Pinch · Wrinkle · Fold · Reconstruct) — the segmented
/// control's option count; [`field::DeformMode::from_u8`] clamps to it.
pub(crate) const DEFORM_MODE_COUNT: u8 = 6;

/// Map the size slider's `0..1` track to a brush radius in px (squared track → finer control when small),
/// over the shared brush size range. Kept local so Deform's size is independent of the paint brush.
fn deform_radius_px(size_norm: f32) -> f32 {
    let t = size_norm.clamp(0.0, 1.0);
    super::BRUSH_SIZE_MIN_PX + t * t * (super::BRUSH_SIZE_MAX_PX - super::BRUSH_SIZE_MIN_PX)
}

/// Deform settings + session state (mode-exclusive; the panel section reads it via the snapshot). The
/// `pre` buffer + `disp` map + `active` flag ARE captured in the `ModelSnapshot` (like the Selection mask),
/// so undo/redo rolls the whole warp back in lock-step with the pixels; the transient per-stroke cursor
/// fields (`last_pos`/`momentum_vec`) are not.
pub(crate) struct DeformState {
    /// Sub-mode (`0` Push · `1` Twist · `2` Pinch · `3` Wrinkle · `4` Fold · `5` Reconstruct).
    pub(crate) mode: u8,
    /// Size slider track (`0..1`; mapped to a px radius by [`deform_radius_px`]).
    pub(crate) size_norm: f32,
    /// Pressure (`0..1`) — master intensity, multiplied by the pen pressure per dab.
    pub(crate) pressure: f32,
    /// Distortion (`0..1`) — per-dab turbulence (Push perpendicular jitter · Pinch/Wrinkle/Fold gain noise).
    pub(crate) distortion: f32,
    /// Momentum (`0..1`) — how much of the previous dab's motion is carried into the next (inertia).
    pub(crate) momentum: f32,
    /// Strength (`0..1`, `0.5` = centre) — bipolar: Pinch `<0.5` sucks / `>0.5` bulges; Twist sign = CW/CCW.
    pub(crate) strength: f32,
    /// **Freeze**: protect the selected region (or its complement when [`Self::freeze_invert`]) from the warp.
    pub(crate) freeze_on: bool,
    pub(crate) freeze_invert: bool,

    // ── Session (transient) ──
    /// Pristine pixels at the session start — Reconstruct fades back to this, Reset restores it wholesale.
    pub(crate) pre: Arc<Vec<u8>>,
    /// A deform session is live (`pre` captured). Ends on Reset / Apply; Apply & Keep rebases `pre`.
    pub(crate) active: bool,
    /// Previous pointer sample within the current stroke — the Push drag delta + momentum source.
    pub(crate) last_pos: Option<[f32; 2]>,
    /// The last dab's effective motion, carried into the next dab scaled by Momentum.
    pub(crate) momentum_vec: [f32; 2],
    /// Per-stroke splitmix seed for the Distortion/Wrinkle noise (bumped each stroke; HR-5 deterministic).
    pub(crate) seed: u64,
    /// **Session cumulative displacement** (`[dx, dy]` px per texel) — the source of truth for the whole
    /// deform. Every render is `canvas[dst] = bilinear(pre, dst − disp[dst])`: a single resample from the
    /// pristine `pre`, so it never compounds blur. Field modes ADD to it; **Reconstruct REDUCES it** toward
    /// zero (so pixels slide BACK to their original spots — a real un-warp, not a cross-fade); Reset zeroes
    /// it. Sized `w*h`, allocated at session start; empty when idle. `Arc`-shared so the undo snapshot
    /// captures it cheaply (CoW) — undo/redo rolls the warp back in lock-step with the pixels, keeping
    /// Reconstruct available after an undo.
    pub(crate) disp: Arc<Vec<[f32; 2]>>,
}

impl Default for DeformState {
    fn default() -> Self {
        Self {
            mode: 0,         // Push
            size_norm: 0.5,  // mid brush
            pressure: 0.8,   // firm by default (Procreate-ish)
            distortion: 0.0, // OFF by default → clean Push (turbulence is opt-in; it's coherent, not grain)
            momentum: 0.0,   // no inertia by default
            strength: 0.5,   // centred → Pinch/Twist neutral (identity)
            freeze_on: false,
            freeze_invert: false,
            pre: Arc::new(Vec::new()),
            active: false,
            last_pos: None,
            momentum_vec: [0.0, 0.0],
            seed: 0,
            disp: Arc::new(Vec::new()),
        }
    }
}

impl PainterTool {
    // ── UI-edit setters (single clamp source; `handle_panel_event` forwards raw panel values here) ──

    /// Select the Reshape sub-mode from the segmented index (out-of-range clamps to the last mode).
    pub fn set_deform_mode(&mut self, m: u8) {
        self.paint.deform.mode = m.min(DEFORM_MODE_COUNT - 1);
    }
    /// Set the Deform brush size from the slider's `0..1` track.
    pub fn set_deform_size_norm(&mut self, t: f32) {
        self.paint.deform.size_norm = t.clamp(0.0, 1.0);
    }
    /// Set the Pressure (master intensity) `0..1`.
    pub fn set_deform_pressure(&mut self, t: f32) {
        self.paint.deform.pressure = t.clamp(0.0, 1.0);
    }
    /// Set the Distortion (per-dab turbulence) `0..1`.
    pub fn set_deform_distortion(&mut self, t: f32) {
        self.paint.deform.distortion = t.clamp(0.0, 1.0);
    }
    /// Set the Momentum (inertia) `0..1`.
    pub fn set_deform_momentum(&mut self, t: f32) {
        self.paint.deform.momentum = t.clamp(0.0, 1.0);
    }
    /// Set the bipolar Strength track `0..1` (`0.5` = neutral).
    pub fn set_deform_strength(&mut self, t: f32) {
        self.paint.deform.strength = t.clamp(0.0, 1.0);
    }
    /// Turn Freeze (protect the selection during the warp) on/off.
    pub fn set_deform_freeze(&mut self, on: bool) {
        self.paint.deform.freeze_on = on;
    }
    /// Invert the frozen region (protect the complement of the selection).
    pub fn set_deform_freeze_invert(&mut self, on: bool) {
        self.paint.deform.freeze_invert = on;
    }

    // ── Session verbs (Card D) ──

    /// **Reset** — discard the whole session's deformation, restoring the pre-deform pixels. One structural
    /// undo entry; no-op when no session is active.
    pub fn deform_reset(&mut self) {
        if !self.paint.deform.active {
            return;
        }
        let before = self.snapshot_model();
        self.canvas_rgba = Arc::clone(&self.paint.deform.pre);
        self.end_deform_session();
        self.mark_full_dirty();
        self.commit_structural_edit(before);
    }
    /// **Apply** — finalize: keep the deformed pixels (already committed) and close the session, so the next
    /// stroke captures a fresh pre-deform baseline. Undo still reaches earlier strokes via their snapshots.
    pub fn deform_apply(&mut self) {
        self.end_deform_session();
    }
    /// **Apply & Keep** — bank the current pixels as the new session baseline (Reconstruct now un-warps
    /// toward *this* state) and keep deforming: rebase `pre` to the current canvas and zero the accumulated
    /// displacement. No-op without an active session.
    pub fn deform_apply_keep(&mut self) {
        if self.paint.deform.active {
            self.paint.deform.pre = Arc::clone(&self.canvas_rgba);
            for d in Arc::make_mut(&mut self.paint.deform.disp) {
                *d = [0.0, 0.0];
            }
        }
    }

    /// Route a Deform-panel event to its verb (the tool side of the seam). Segmented mode = `Click` on an
    /// option id (its array position is the mode); sliders = `SetValue`; toggles/buttons = `Click`. Returns
    /// `true` iff consumed. Mirrors `route_selection_event`; hung off the `handle_panel_event` chain.
    pub(crate) fn route_deform_event(
        &mut self,
        event: &ph2d_editor_core::tool::PanelEvent,
    ) -> bool {
        use ph2d_editor_core::ids as core_ids;
        use ph2d_editor_core::tool::PanelEvent;
        match event {
            PanelEvent::Click(id) if core_ids::PAINTER_DEFORM_MODE_IDS.contains(id) => {
                let idx = core_ids::PAINTER_DEFORM_MODE_IDS
                    .iter()
                    .position(|x| x == id)
                    .unwrap_or(0) as u8;
                self.set_deform_mode(idx);
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_DEFORM_FREEZE => {
                let on = !self.paint.deform.freeze_on;
                self.set_deform_freeze(on);
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_DEFORM_FREEZE_INVERT => {
                let on = !self.paint.deform.freeze_invert;
                self.set_deform_freeze_invert(on);
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_DEFORM_RESET => {
                self.deform_reset();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_DEFORM_APPLY => {
                self.deform_apply();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_DEFORM_APPLY_KEEP => {
                self.deform_apply_keep();
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_DEFORM_SIZE_SLIDER => {
                self.set_deform_size_norm(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_DEFORM_PRESSURE_SLIDER => {
                self.set_deform_pressure(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_DEFORM_DISTORTION_SLIDER => {
                self.set_deform_distortion(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_DEFORM_MOMENTUM_SLIDER => {
                self.set_deform_momentum(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_DEFORM_STRENGTH_SLIDER => {
                self.set_deform_strength(*v as f32);
                true
            }
            _ => false,
        }
    }

    /// Whether Freeze should protect texels this dab — on AND an active selection restricts painting.
    pub(super) fn deform_freeze_effective(&self) -> bool {
        self.paint.deform.freeze_on && self.selection_restricts_paint()
    }
    /// Whether the frozen region is inverted (protect the selection complement).
    pub(super) fn deform_freeze_inverted(&self) -> bool {
        self.paint.deform.freeze_invert
    }

    // ── Pointer lifecycle ──

    /// The canvas-pointer entry for Deform mode (routed from `canvas_pointer.rs`). Down/Up bracket one
    /// structural undo entry; each Move stamps a warp dab that accumulates into the layer.
    pub(super) fn warp_pointer(&mut self, ev: CanvasPointer) -> bool {
        match ev.phase {
            PointerPhase::Down => {
                self.warp_begin(ev);
                true
            }
            PointerPhase::Move => self.warp_extend(ev),
            PointerPhase::Up => {
                self.warp_end(ev);
                true
            }
            PointerPhase::Hover => false,
        }
    }

    fn warp_begin(&mut self, ev: CanvasPointer) {
        let before = self.snapshot_model();
        self.paint.stroke_undo = Some(before);
        self.ensure_deform_session();
        self.paint.deform.seed = self.paint.deform.seed.wrapping_add(1);
        self.paint.deform.last_pos = Some(ev.pos);
        self.paint.deform.momentum_vec = [0.0, 0.0];
        // First dab has no motion — Pinch/Twist still deform on a stationary press; Push waits for a drag.
        self.warp_dab_at(ev.pos, [0.0, 0.0], ev.pressure);
    }

    fn warp_extend(&mut self, ev: CanvasPointer) -> bool {
        let Some(prev) = self.paint.deform.last_pos else {
            return false; // stray Move with no open stroke
        };
        let mv = [ev.pos[0] - prev[0], ev.pos[1] - prev[1]];
        self.warp_dab_at(ev.pos, mv, ev.pressure);
        self.paint.deform.momentum_vec = mv;
        self.paint.deform.last_pos = Some(ev.pos);
        true
    }

    fn warp_end(&mut self, ev: CanvasPointer) {
        self.warp_extend(ev);
        self.paint.deform.last_pos = None;
        self.paint.deform.momentum_vec = [0.0, 0.0];
        // The session displacement + `pre` PERSIST across strokes (so Reconstruct can un-warp the whole
        // session and Reset reverts it all); they're freed by Reset / Apply / a mode switch / undo.
        if let Some(before) = self.paint.stroke_undo.take() {
            self.commit_structural_edit(before);
        }
    }

    /// Stamp one dab: dispatch to the field kernel (Push/Twist/Pinch/Wrinkle/Fold) or the Reconstruct
    /// resampler, at `pos` with drag delta `mv` and pen `pen_pressure`.
    fn warp_dab_at(&mut self, pos: [f32; 2], mv: [f32; 2], pen_pressure: f32) {
        let d = &self.paint.deform;
        let radius = deform_radius_px(d.size_norm);
        let pressure = (d.pressure * pen_pressure).clamp(0.0, 1.0);
        let mode = field::DeformMode::from_u8(d.mode);
        if matches!(mode, field::DeformMode::Reconstruct) {
            self.warp_reconstruct_dab(pos, radius, pressure);
            return;
        }
        let signed = (d.strength - 0.5) * 2.0; // 0.5-centred → [-1, 1]
        let f = field::DabField::new(
            mode,
            pos,
            radius,
            mv,
            d.momentum_vec,
            signed,
            pressure,
            d.distortion,
            d.momentum,
            d.seed,
        );
        self.warp_apply_dab(&f, pos, radius);
    }

    // ── Session helpers ──

    /// Start a deform session if none is live: snapshot the current pixels as the pristine `pre` (a cheap
    /// `Arc` clone — the first write copies-on-write, so `pre` stays frozen at session start) and allocate a
    /// zeroed session displacement map. Every render samples `pre` through that map (single resample →
    /// sharp). No-op if a session is already live (so multi-stroke sessions accumulate into one map).
    fn ensure_deform_session(&mut self) {
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        if !self.paint.deform.active || self.paint.deform.pre.len() != n * 4 {
            self.paint.deform.pre = Arc::clone(&self.canvas_rgba);
            self.paint.deform.disp = Arc::new(vec![[0.0, 0.0]; n]);
            self.paint.deform.active = true;
        }
    }

    /// Snapshot the deform session for the undo model (the buffers are `tool::paint`-private, so the
    /// general `snapshot_model` reaches them through here — mirror of `selection_for_snapshot`).
    #[allow(clippy::type_complexity)] // (disp, pre, active) — mirrors `selection_for_snapshot`'s tuple
    pub(crate) fn deform_for_snapshot(&self) -> (Arc<Vec<[f32; 2]>>, Arc<Vec<u8>>, bool) {
        (
            Arc::clone(&self.paint.deform.disp),
            Arc::clone(&self.paint.deform.pre),
            self.paint.deform.active,
        )
    }

    /// Reinstate the deform session from an undo model — rolls the warp back in lock-step with the pixels
    /// so Reconstruct stays usable after an undo. Drops the transient per-stroke cursor state.
    pub(crate) fn restore_deform(
        &mut self,
        disp: Arc<Vec<[f32; 2]>>,
        pre: Arc<Vec<u8>>,
        active: bool,
    ) {
        self.paint.deform.disp = disp;
        self.paint.deform.pre = pre;
        self.paint.deform.active = active;
        self.paint.deform.last_pos = None;
        self.paint.deform.momentum_vec = [0.0, 0.0];
    }

    /// End the deform session and free its buffers. Called by Reset / Apply, on a mode switch away from
    /// Deform, and on undo/redo (`runtime::restore_model`) — the session displacement is NOT in the undo
    /// snapshot, so a restore must drop it to avoid re-applying a stale warp.
    pub(crate) fn end_deform_session(&mut self) {
        self.paint.deform.active = false;
        self.paint.deform.pre = Arc::new(Vec::new());
        self.paint.deform.disp = Arc::new(Vec::new());
        self.paint.deform.last_pos = None;
        self.paint.deform.momentum_vec = [0.0, 0.0];
    }

    /// Mark the whole canvas dirty (used after a wholesale Reset).
    fn mark_full_dirty(&mut self) {
        let (w, h) = self.source_size;
        if w > 0 && h > 0 {
            self.mark_dirty(Region { x: 0, y: 0, w, h });
        }
    }
}
