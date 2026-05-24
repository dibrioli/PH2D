//! [`UpscaleTool`] — stateful editor Tool for resolution upscaling.
//!
//! Model: live [`UpscaleParams`] + cached source snapshot + one-shot
//! `pending_apply` flag. Apply-only by design — there is NO live
//! preview overlay and NO realtime feedback during slider drag. The
//! result only materializes on Apply.
//!
//! Wave 10 / Etapa 2 follow-up (Enio smoke 2026-05-24): "Upscale não
//! necessita de preview nem de mudanças em tempo real. Apenas no
//! apply." The earlier infrastructure (thumbnail / canvas preview /
//! params_dirty flag) was deleted in this pass.
//!
//! ## Apply flow
//!
//! 1. The shell pushes a fresh source snapshot whenever the active
//!    sprite changes via [`Self::set_source_snapshot`].
//! 2. Every panel event reaches the tool via
//!    [`Self::handle_panel_event`] → [`Self::apply_ui_edit`] (the
//!    single clamping site). The live params update, but NOTHING runs
//!    on the canvas — the user sees the original sprite untouched
//!    until they press Apply.
//! 3. The Apply button sets `pending_apply`. The shell drains via
//!    [`Self::take_pending_apply`] and bakes at full resolution by
//!    calling [`Self::run_full_resolution`] — which runs the active
//!    algorithm over the live source RGBA and writes back a new
//!    Individual texture (mirrors the BgRemoval / Padding bake path,
//!    reached via `as_any_mut` downcast).

use ph2d_editor_core::floating_panel::{FloatingPanel, PanelAnchor, ToolId};
use ph2d_editor_core::tool::{PanelEvent, RasterEditTool, Tool};
use ph2d_tool_registry::hash_node_id;

use ph2d_a11y::NodeId;

use crate::algorithm::{UpscaleResult, upscale_lanczos3, upscale_nearest, upscale_xbr};
use crate::params::{UpscaleAlgorithm, UpscaleParams, UpscaleUiEdit, UpscaleUiSnapshot};

// NodeId range for the Upscale docked panel.
// FNV-1a-derived (`hash_node_id`) — no slot in `editor-core/ids.rs` is
// touched, keeping the tool drop-in.
const NODE_ALGO_LANCZOS3: NodeId = hash_node_id("upscale.algo.lanczos3");
const NODE_ALGO_NEAREST: NodeId = hash_node_id("upscale.algo.nearest");
const NODE_ALGO_XBR: NodeId = hash_node_id("upscale.algo.xbr");
const NODE_SCALE: NodeId = hash_node_id("upscale.scale");
const NODE_SCALE_NUM: NodeId = hash_node_id("upscale.scale.num");
const NODE_APPLY: NodeId = hash_node_id("upscale.apply");
const NODE_CANCEL: NodeId = hash_node_id("upscale.cancel");
const NODE_RESET: NodeId = hash_node_id("upscale.reset");

/// Panel widget `NodeId`s as a const-evaluated table (used by both the
/// panel crate to register/paint and the tool to route events).
pub mod ids {
    use ph2d_a11y::NodeId;

    /// Algorithm segmented selection — Lanczos3 (default).
    pub const UPS_ALGO_LANCZOS3: NodeId = super::NODE_ALGO_LANCZOS3;
    /// Algorithm segmented selection — Nearest neighbour.
    pub const UPS_ALGO_NEAREST: NodeId = super::NODE_ALGO_NEAREST;
    /// Algorithm segmented selection — xBR (Scale2x/3x/4x in v1).
    pub const UPS_ALGO_XBR: NodeId = super::NODE_ALGO_XBR;
    /// Scale slider (normalized 0..1, mapped via
    /// `params::slider_to_scale`).
    pub const UPS_SCALE: NodeId = super::NODE_SCALE;
    /// Scale number chip (raw scale factor as a `f64`).
    pub const UPS_SCALE_NUM: NodeId = super::NODE_SCALE_NUM;
    /// Apply button.
    pub const UPS_APPLY: NodeId = super::NODE_APPLY;
    /// Cancel button.
    pub const UPS_CANCEL: NodeId = super::NODE_CANCEL;
    /// Reset-all button — algorithm + scale back to defaults.
    pub const UPS_RESET: NodeId = super::NODE_RESET;
}

/// Editor Tool implementing the stateful Upscale feature.
#[derive(Clone, Debug, Default)]
pub struct UpscaleTool {
    /// User-tunable parameters (algorithm + scale factor).
    pub params: UpscaleParams,

    /// Latest source snapshot pushed by the host (`set_source_snapshot`).
    /// Empty until the host calls. Layout: RGBA8, length
    /// `source_w * source_h * 4`.
    source_rgba: Vec<u8>,
    source_w: u32,
    source_h: u32,

    /// Set to `true` when the user activates Apply. Host polls via
    /// [`Self::take_pending_apply`] each frame; on `true` it runs the
    /// pipeline at full resolution against the active sprite and
    /// writes back a new Individual texture.
    pending_apply: bool,
    /// Set `true` by Reset / on_activate. Shell drains via
    /// `take_pending_panel_reset` and re-runs `Panel::populate` so
    /// the slider knob + chip text snap back to defaults.
    pending_panel_reset: bool,
}

impl UpscaleTool {
    /// Push a fresh source RGBA snapshot from the host. Just stores
    /// the pixels — Apply-only model means there's no preview cook to
    /// trigger here.
    ///
    /// `rgba` must be straight-alpha RGBA8 of length `w * h * 4`.
    pub fn set_source_snapshot(&mut self, rgba: Vec<u8>, w: u32, h: u32) {
        assert_eq!(rgba.len(), (w as usize) * (h as usize) * 4);
        self.source_rgba = rgba;
        self.source_w = w;
        self.source_h = h;
    }

    /// Whether the host has pushed a source snapshot at least once.
    pub fn has_source(&self) -> bool {
        !self.source_rgba.is_empty()
    }

    /// Source dimensions of the active snapshot, or `(0, 0)` before
    /// any source is pushed.
    pub fn source_size(&self) -> (u32, u32) {
        (self.source_w, self.source_h)
    }

    /// Drain the pending-apply flag. Returns `true` exactly once after
    /// each Apply trigger; the host's drain loop runs the full-res
    /// pipeline when this returns `true`.
    pub fn take_pending_apply(&mut self) -> bool {
        let p = self.pending_apply;
        self.pending_apply = false;
        p
    }

    /// Drain the panel-reset flag. Returns `true` exactly once when
    /// Reset / on_activate has fired; the shell re-runs
    /// `Panel::populate` so the slider knob + chip text snap back to
    /// defaults.
    pub fn take_pending_panel_reset(&mut self) -> bool {
        std::mem::take(&mut self.pending_panel_reset)
    }

    /// Run the active algorithm at full source resolution and write
    /// the result into `out`. Returns the output `(w, h)`.
    ///
    /// `out` is overwritten (cleared, then extended); the caller can
    /// reuse the same allocation across applies.
    pub fn run_full_resolution(&mut self, out: &mut Vec<u8>) -> (u32, u32) {
        assert!(
            self.has_source(),
            "set_source_snapshot must run before run_full_resolution"
        );
        let r = self.run_algorithm(&self.source_rgba, self.source_w, self.source_h);
        out.clear();
        out.extend_from_slice(&r.pixels);
        (r.width, r.height)
    }

    /// Project the live params into the snapshot the panel paints.
    /// Published by the host once per frame while the tool is active.
    pub fn ui_snapshot(&self) -> UpscaleUiSnapshot {
        UpscaleUiSnapshot {
            algorithm: self.params.algorithm,
            scale_factor: self.params.scale_factor,
            effective_factor: self
                .params
                .algorithm
                .project_scale(self.params.scale_factor),
        }
    }

    /// Apply one panel-originated edit against the live params.
    /// `Apply` arms the pending-apply flag; the host drains via
    /// [`Self::take_pending_apply`]. Inverse of [`Self::ui_snapshot`].
    ///
    /// Wave 10 / Etapa 2 follow-up: no preview re-run on slider /
    /// algorithm changes — the tool is Apply-only by user request.
    /// The result only materializes via `run_full_resolution` after
    /// the Apply button is pressed.
    pub fn apply_ui_edit(&mut self, edit: UpscaleUiEdit) {
        match edit {
            UpscaleUiEdit::SetAlgorithm(a) => {
                self.params.algorithm = a;
            }
            UpscaleUiEdit::Scale(v) => {
                self.params.scale_factor = v;
            }
            UpscaleUiEdit::Apply => {
                self.pending_apply = true;
            }
            UpscaleUiEdit::ResetAll => {
                self.params = crate::params::UpscaleParams::default();
                self.pending_panel_reset = true;
            }
        }
    }

    /// Dispatch to the active algorithm.
    fn run_algorithm(&self, rgba: &[u8], w: u32, h: u32) -> UpscaleResult {
        let factor = self
            .params
            .algorithm
            .project_scale(self.params.scale_factor);
        match self.params.algorithm {
            UpscaleAlgorithm::Lanczos3 => upscale_lanczos3(rgba, w, h, factor),
            UpscaleAlgorithm::Nearest => upscale_nearest(rgba, w, h, factor),
            UpscaleAlgorithm::Xbr => upscale_xbr(rgba, w, h, factor),
        }
    }
}

impl Tool for UpscaleTool {
    fn id(&self) -> ToolId {
        ToolId::new("upscale")
    }

    fn label(&self) -> &str {
        "Upscale"
    }

    fn icon_slug(&self) -> &str {
        "upscale"
    }

    fn build_panel(&self) -> FloatingPanel {
        // The real UI is the typed `ph2d-panel-upscale` crate; the
        // legacy FloatingPanel path is retained as a tiny shell so
        // `Tool::build_panel` has a value to return (mirrors Padding).
        let mut panel = FloatingPanel::new(self.id(), "Upscale");
        panel.anchor = PanelAnchor::BottomCenter;
        panel
    }

    fn on_activate(&mut self) {
        // Defaults load on every fresh panel open (algorithm + scale
        // back to `UpscaleParams::default()`).
        self.apply_ui_edit(UpscaleUiEdit::ResetAll);
    }

    fn on_deactivate(&mut self) {
        // Clear pending_apply so a Cancel-mid-Apply doesn't fire a
        // phantom bake on the next activation.
        self.pending_apply = false;
    }

    fn handle_panel_event(&mut self, event: PanelEvent) {
        // Docked-panel `UPS_*` NodeIds route through `apply_ui_edit` so
        // the slider → scale projection + algorithm-segmented mapping
        // live in exactly one place (params.rs). Cancel is NOT handled
        // here — the panel pushes `EditorAction::CancelActiveTool`
        // directly (it's a tool-lifecycle event, not a params edit).
        match event {
            PanelEvent::Click(id) if id == NODE_ALGO_LANCZOS3 => {
                self.apply_ui_edit(UpscaleUiEdit::SetAlgorithm(UpscaleAlgorithm::Lanczos3));
            }
            PanelEvent::Click(id) if id == NODE_ALGO_NEAREST => {
                self.apply_ui_edit(UpscaleUiEdit::SetAlgorithm(UpscaleAlgorithm::Nearest));
            }
            PanelEvent::Click(id) if id == NODE_ALGO_XBR => {
                self.apply_ui_edit(UpscaleUiEdit::SetAlgorithm(UpscaleAlgorithm::Xbr));
            }
            // Scale slider — `v` is the normalized track `0..1`,
            // mapped to a scale factor via `slider_to_scale`. The
            // panel's forwarder thin (Widget Gallery §4.2) only ever
            // emits this id; the chip shares the same track storage
            // via `link_slider_number` and never reaches the tool as
            // a separate `_NUM` event.
            PanelEvent::SetValue(id, v) if id == NODE_SCALE => {
                let factor = crate::params::slider_to_scale(v as f32);
                self.apply_ui_edit(UpscaleUiEdit::Scale(factor));
            }
            PanelEvent::Click(id) if id == NODE_APPLY => {
                self.apply_ui_edit(UpscaleUiEdit::Apply);
            }
            PanelEvent::Click(id) if id == NODE_RESET => {
                self.apply_ui_edit(UpscaleUiEdit::ResetAll);
            }
            _ => {}
        }
    }

    fn as_raster_edit_mut(&mut self) -> Option<&mut dyn RasterEditTool> {
        // Wave 10 / Etapa 2 (ADR-0041): Upscale joins BgRemoval + CEQ on
        // the RasterEditTool generic channel. The shell's tool-runtime
        // helpers compose over this upcast — bridges no longer need
        // `downcast_mut::<UpscaleTool>` for the generic raster I/O
        // lifecycle. Upscale-specific concerns (panel snapshot publish,
        // preview_size for thumb slot) still go via `as_any_mut` downcast
        // — ADR-0040 §3 documented exception.
        Some(self)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Wave 10 / Etapa 2 (ADR-0041): Upscale drives its raster I/O
/// lifecycle through the generic `RasterEditTool` channel.
///
/// Apply-only — `current_preview` always returns `None`. The shell's
/// `drive_preview_cache` therefore never installs a frame in the
/// preview cache, so no canvas overlay paints during slider drag.
/// The actual upscale only runs on Apply via `run_full`.
///
/// Mapping:
/// - `set_source` → wraps `set_source_snapshot` (just stores pixels).
/// - `current_preview` → always `None` (no realtime preview).
/// - `take_pending_commit` → wraps `take_pending_apply`.
/// - `run_full` → wraps `run_full_resolution(&mut Vec)`.
/// - `deactivate` → drain pending_apply.
impl RasterEditTool for UpscaleTool {
    fn set_source(&mut self, rgba: Vec<u8>, width: u32, height: u32) {
        self.set_source_snapshot(rgba, width, height);
    }

    fn current_preview(&mut self) -> Option<(&[u8], u32, u32)> {
        // Apply-only by design (Etapa 2 follow-up smoke): never expose
        // a live preview frame. The shell's preview-cache stays empty
        // for the lifetime of the tool's activation.
        None
    }

    fn take_pending_commit(&mut self) -> bool {
        self.take_pending_apply()
    }

    fn run_full(&mut self) -> (Vec<u8>, u32, u32) {
        let mut out = Vec::new();
        let (w, h) = self.run_full_resolution(&mut out);
        (out, w, h)
    }

    fn deactivate(&mut self) {
        self.pending_apply = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::slider_to_scale;

    #[test]
    fn id_label_icon() {
        let t = UpscaleTool::default();
        assert_eq!(t.id(), ToolId::new("upscale"));
        assert_eq!(t.label(), "Upscale");
        assert_eq!(t.icon_slug(), "upscale");
    }

    #[test]
    fn default_tool_has_no_source_and_no_pending() {
        let mut t = UpscaleTool::default();
        assert!(!t.has_source());
        assert!(!t.take_pending_apply());
    }

    #[test]
    fn set_source_snapshot_marks_has_source_and_stores_dims() {
        let mut t = UpscaleTool::default();
        let buf = vec![128u8; 8 * 8 * 4];
        t.set_source_snapshot(buf, 8, 8);
        assert!(t.has_source());
        assert_eq!(t.source_size(), (8, 8));
    }

    #[test]
    fn apply_arms_pending_once() {
        let mut t = UpscaleTool::default();
        assert!(!t.take_pending_apply());
        t.apply_ui_edit(UpscaleUiEdit::Apply);
        assert!(t.take_pending_apply());
        assert!(!t.take_pending_apply());
    }

    #[test]
    fn algorithm_edit_persists() {
        let mut t = UpscaleTool::default();
        assert_eq!(t.params.algorithm, UpscaleAlgorithm::Lanczos3);
        t.apply_ui_edit(UpscaleUiEdit::SetAlgorithm(UpscaleAlgorithm::Xbr));
        assert_eq!(t.params.algorithm, UpscaleAlgorithm::Xbr);
    }

    #[test]
    fn scale_edit_clamps_and_persists() {
        let mut t = UpscaleTool::default();
        // Pass through valid value.
        t.apply_ui_edit(UpscaleUiEdit::Scale(3.0));
        assert!((t.params.scale_factor - 3.0).abs() < f32::EPSILON);
        // Snapshot mirrors the live scale (params clamping happens at
        // project_scale, not in apply_ui_edit — out-of-range slider
        // values are stored as-is for round-trip clarity).
        t.apply_ui_edit(UpscaleUiEdit::Scale(99.0));
        let s = t.ui_snapshot();
        // effective_factor is the clamped value the algorithm uses.
        assert!((s.effective_factor - 16.0).abs() < f32::EPSILON);
    }

    #[test]
    fn handle_panel_event_routes_algorithm_segmented() {
        let mut t = UpscaleTool::default();
        t.handle_panel_event(PanelEvent::Click(NODE_ALGO_NEAREST));
        assert_eq!(t.params.algorithm, UpscaleAlgorithm::Nearest);
        t.handle_panel_event(PanelEvent::Click(NODE_ALGO_XBR));
        assert_eq!(t.params.algorithm, UpscaleAlgorithm::Xbr);
        t.handle_panel_event(PanelEvent::Click(NODE_ALGO_LANCZOS3));
        assert_eq!(t.params.algorithm, UpscaleAlgorithm::Lanczos3);
    }

    #[test]
    fn handle_panel_event_routes_scale_slider() {
        // Widget Gallery convention §4.2: chip + slider share `0..1`
        // storage via `link_slider_number`, so the panel forwarder
        // only emits the slider id. The tool projects the track via
        // `slider_to_scale`.
        let mut t = UpscaleTool::default();
        t.handle_panel_event(PanelEvent::SetValue(NODE_SCALE, 0.5));
        let expected = slider_to_scale(0.5);
        assert!((t.params.scale_factor - expected).abs() < f32::EPSILON);
        t.handle_panel_event(PanelEvent::SetValue(NODE_SCALE, 1.0));
        assert!((t.params.scale_factor - 16.0).abs() < f32::EPSILON);
    }

    #[test]
    fn handle_panel_event_routes_apply_button() {
        let mut t = UpscaleTool::default();
        t.handle_panel_event(PanelEvent::Click(NODE_APPLY));
        assert!(t.take_pending_apply());
    }

    #[test]
    fn deactivate_clears_pending_but_keeps_params() {
        let mut t = UpscaleTool::default();
        t.apply_ui_edit(UpscaleUiEdit::SetAlgorithm(UpscaleAlgorithm::Xbr));
        t.apply_ui_edit(UpscaleUiEdit::Scale(4.0));
        t.apply_ui_edit(UpscaleUiEdit::Apply);
        t.on_deactivate();
        assert_eq!(t.params.algorithm, UpscaleAlgorithm::Xbr);
        assert!((t.params.scale_factor - 4.0).abs() < f32::EPSILON);
        assert!(!t.take_pending_apply());
    }

    #[test]
    fn current_preview_always_returns_none() {
        // Apply-only contract — never expose a live preview frame.
        let mut t = UpscaleTool::default();
        t.set_source_snapshot(vec![100u8; 4 * 4 * 4], 4, 4);
        t.apply_ui_edit(UpscaleUiEdit::Scale(2.0));
        t.apply_ui_edit(UpscaleUiEdit::SetAlgorithm(UpscaleAlgorithm::Xbr));
        assert!(RasterEditTool::current_preview(&mut t).is_none());
    }

    #[test]
    fn run_full_resolution_writes_expected_dims() {
        let mut t = UpscaleTool::default();
        let buf = vec![200u8; 4 * 4 * 4];
        t.set_source_snapshot(buf, 4, 4);
        t.apply_ui_edit(UpscaleUiEdit::Scale(2.0));
        let mut out = Vec::new();
        let (w, h) = t.run_full_resolution(&mut out);
        assert_eq!((w, h), (8, 8));
        assert_eq!(out.len(), 8 * 8 * 4);
    }

    #[test]
    fn run_full_resolution_works_after_per_sprite_source_swap() {
        // Mirrors the shell's multi-Apply drain pattern: one UpscaleTool
        // instance bakes N sprites in sequence via set_source_snapshot
        // → run_full_resolution per entity. Each bake must reflect the
        // CURRENT snapshot dims, not leak source_w/source_h from the
        // previous sprite. Regression cover (§12.6 + Agent D gap).
        let mut t = UpscaleTool::default();
        t.apply_ui_edit(UpscaleUiEdit::Scale(2.0));

        // Sprite 1: 4×4.
        t.set_source_snapshot(vec![100u8; 4 * 4 * 4], 4, 4);
        let mut out1 = Vec::new();
        let (w1, h1) = t.run_full_resolution(&mut out1);
        assert_eq!((w1, h1), (8, 8));
        assert_eq!(out1.len(), 8 * 8 * 4);

        // Sprite 2: different dims (5×7) — must re-bake against new
        // snapshot, not reuse sprite-1 dims.
        t.set_source_snapshot(vec![50u8; 5 * 7 * 4], 5, 7);
        let mut out2 = Vec::new();
        let (w2, h2) = t.run_full_resolution(&mut out2);
        assert_eq!((w2, h2), (10, 14), "per-sprite source swap leaked dims");
        assert_eq!(out2.len(), 10 * 14 * 4);
    }

    #[test]
    fn on_activate_resets_params_and_arms_panel_repopulate() {
        // Regression cover (§12.3 / §12.4 UI_Bugs): `on_activate` must
        // route through `apply_ui_edit::ResetAll` so (a) params snap to
        // defaults AND (b) `pending_panel_reset` arms so the shell
        // bridge re-runs `Panel::populate(store)` and the slider knobs
        // visually snap back to defaults.
        let mut t = UpscaleTool::default();
        // Dirty the state — simulate a prior session.
        t.apply_ui_edit(UpscaleUiEdit::SetAlgorithm(UpscaleAlgorithm::Xbr));
        t.apply_ui_edit(UpscaleUiEdit::Scale(8.0));
        assert_eq!(t.params.algorithm, UpscaleAlgorithm::Xbr);
        assert!((t.params.scale_factor - 8.0).abs() < f32::EPSILON);
        // Drain any stray reset flag first.
        let _ = t.take_pending_panel_reset();

        t.on_activate();

        let dft = UpscaleParams::default();
        assert_eq!(t.params.algorithm, dft.algorithm);
        assert!((t.params.scale_factor - dft.scale_factor).abs() < f32::EPSILON);
        assert!(
            t.take_pending_panel_reset(),
            "on_activate must arm pending_panel_reset so the shell repopulates"
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // Wave 10 / Etapa 2 — RasterEditTool impl tests (ADR-0041 follow-up)
    // ─────────────────────────────────────────────────────────────────────

    fn solid_rgba(w: u32, h: u32, rgb: [u8; 3]) -> Vec<u8> {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..(w * h) {
            v.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
        v
    }

    #[test]
    fn as_raster_edit_mut_returns_some_for_upscale() {
        let mut t = UpscaleTool::default();
        assert!(<dyn Tool as Tool>::as_raster_edit_mut(&mut t).is_some());
    }

    #[test]
    fn raster_edit_set_source_delegates() {
        let mut t = UpscaleTool::default();
        RasterEditTool::set_source(&mut t, solid_rgba(8, 8, [100, 150, 200]), 8, 8);
        assert!(t.has_source());
        assert_eq!(t.source_size(), (8, 8));
    }

    #[test]
    fn raster_edit_current_preview_always_none() {
        // Apply-only contract: never returns Some, regardless of state.
        let mut t = UpscaleTool::default();
        assert!(RasterEditTool::current_preview(&mut t).is_none());
        RasterEditTool::set_source(&mut t, solid_rgba(4, 4, [128, 128, 128]), 4, 4);
        assert!(RasterEditTool::current_preview(&mut t).is_none());
        t.apply_ui_edit(UpscaleUiEdit::SetAlgorithm(UpscaleAlgorithm::Xbr));
        assert!(RasterEditTool::current_preview(&mut t).is_none());
    }

    #[test]
    fn raster_edit_take_pending_commit_drains() {
        let mut t = UpscaleTool::default();
        t.apply_ui_edit(UpscaleUiEdit::Apply);
        assert!(RasterEditTool::take_pending_commit(&mut t));
        assert!(!RasterEditTool::take_pending_commit(&mut t));
    }

    #[test]
    fn raster_edit_run_full_returns_owned_buffer() {
        let mut t = UpscaleTool::default();
        RasterEditTool::set_source(&mut t, solid_rgba(4, 4, [50, 100, 150]), 4, 4);
        let (out, w, h) = RasterEditTool::run_full(&mut t);
        assert!(w >= 4 && h >= 4); // upscaled
        assert_eq!(out.len(), (w as usize) * (h as usize) * 4);
    }

    #[test]
    fn raster_edit_deactivate_clears_pending_apply() {
        let mut t = UpscaleTool::default();
        t.apply_ui_edit(UpscaleUiEdit::Apply);
        RasterEditTool::deactivate(&mut t);
        assert!(!t.take_pending_apply());
    }
}
