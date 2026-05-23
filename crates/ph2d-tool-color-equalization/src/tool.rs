//! [`ColorEqualizationTool`] — stateful editor Tool for Color EQ.
//!
//! Model: `ColorEqualizationParams` (CLAHE clip / tile grid + tonal
//! adjusts + auto-WB toggle) + a cached source snapshot + a live preview
//! thumbnail (512² aspect-fit, mirroring the BgRemoval cap). The Tool
//! reacts only to its panel widgets, never to the canvas directly.
//!
//! ## Apply flow
//!
//! On every panel edit, [`Self::apply_ui_edit`] mutates `params` and (when
//! `params_dirty`) rebuilds the preview against the cached small source.
//! On Apply, [`Self::apply_ui_edit`] sets `pending_apply`. The shell drains
//! it via [`Self::take_pending_apply`] each frame; on `true` it reads each
//! selected sprite's live `Sprite.source`, runs
//! [`Self::run_full_resolution`] against that pixel buffer, and swaps the
//! texture (the BgRemoval / Padding precedent). Multi-sprite broadcast is
//! handled shell-side — the shell emits one `OneShotImageOp` per
//! `hero.gizmo.iter_selected()`.
//!
//! `ImageEditTool` is intentionally NOT implemented (DIRETRIZ §3.8.3.1 —
//! production tools currently use the `as_any_mut` downcast pattern; the
//! generic image-edit channel is fan-out future work).

use ph2d_editor_core::floating_panel::{FloatingPanel, PanelAnchor, ToolId};
use ph2d_editor_core::tool::{PanelEvent, Tool};

use super::algorithm::{aspect_fit_within, resize_bilinear_rgba, run_pipeline};
use super::ids;
use super::params::{
    ColorEqualizationParams, ColorEqualizationUiEdit, ColorEqualizationUiSnapshot, apply_ui_edit,
    brightness_to_slider, clip_limit_to_slider, contrast_to_slider, saturation_to_slider,
    tile_grid_to_slider,
};

/// Side cap (px) for the live preview overlay. The preview re-runs the
/// full CLAHE + adjusts pipeline on every parameter change; doing that
/// at full source resolution makes each slider tick janky. The preview
/// is drawn scaled to the sprite footprint anyway, so it re-processes a
/// copy of the source downscaled to fit this box (aspect preserved, no
/// letterbox) instead, keeping slider drags smooth. Apply still bakes
/// at full source resolution via
/// [`ColorEqualizationTool::run_full_resolution`].
pub const PREVIEW_MAX_DIM: u32 = 512;

/// Editor Tool implementing the Color Equalization feature.
#[derive(Clone, Debug, Default)]
pub struct ColorEqualizationTool {
    /// User-tunable parameters, projected into the floating panel.
    pub params: ColorEqualizationParams,

    /// Latest source snapshot pushed by the host (set via
    /// [`Self::set_source_snapshot`]). Empty until the host calls.
    /// Layout: RGBA8, length `source_w * source_h * 4` (straight alpha).
    source_rgba: Vec<u8>,
    source_w: u32,
    source_h: u32,

    /// Source downscaled to fit [`PREVIEW_MAX_DIM`] (aspect preserved, no
    /// letterbox) — the input of the on-canvas live preview. Rebuilt only
    /// when the source snapshot changes, so a slider drag re-processes
    /// this small image instead of the full-res source.
    preview_src_rgba: Vec<u8>,
    preview_src_w: u32,
    preview_src_h: u32,

    /// Preview output — result of `run_pipeline` on `preview_src_rgba` with
    /// the current `params`. Layout matches `preview_src_*` dims. Reused
    /// across runs (HR-3 — the allocation persists).
    preview_rgba: Vec<u8>,
    preview_dirty: bool,

    /// Set to `true` when the user presses Apply on the panel. The host
    /// drains it via [`Self::take_pending_apply`] and bakes at full
    /// resolution against every selected sprite.
    pending_apply: bool,

    /// Set to `true` by any panel-edit mutator (param change OR
    /// Apply trigger). The shell uses this as the gate for rerunning the
    /// on-canvas live preview — mirrors `BgRemovalTool::take_params_dirty`.
    params_dirty: bool,
}

impl ColorEqualizationTool {
    /// Push a fresh source RGBA snapshot from the host (selection changed
    /// or tool became active). Rebuilds the capped preview source and
    /// marks the preview dirty so the next per-frame paint re-runs the
    /// pipeline against it.
    ///
    /// `rgba` must be straight-alpha RGBA8 of length `w * h * 4`.
    pub fn set_source_snapshot(&mut self, rgba: Vec<u8>, w: u32, h: u32) {
        assert_eq!(rgba.len(), (w as usize) * (h as usize) * 4);
        self.source_rgba = rgba;
        self.source_w = w;
        self.source_h = h;
        self.rebuild_preview_src();
        self.preview_dirty = true;
        // Cached on-canvas preview the shell holds was computed against
        // the previous selection — mark dirty so the next bridge tick
        // rebuilds it.
        self.params_dirty = true;
    }

    /// Whether the host has pushed a source snapshot at least once.
    pub fn has_source(&self) -> bool {
        !self.source_rgba.is_empty()
    }

    /// Source texture resolution `(w, h)` of the active snapshot, or
    /// `(0, 0)` before any source is pushed.
    pub fn source_size(&self) -> (u32, u32) {
        (self.source_w, self.source_h)
    }

    /// Drain the pending-apply flag. Returns `true` exactly once after
    /// each Apply trigger; host calls this in its per-frame drain loop;
    /// on `true` it runs [`Self::run_full_resolution`] against each
    /// selected sprite.
    pub fn take_pending_apply(&mut self) -> bool {
        let p = self.pending_apply;
        self.pending_apply = false;
        p
    }

    /// Drain the params-dirty flag. Returns `true` exactly once when any
    /// panel-edit mutator has run since the last call.
    pub fn take_params_dirty(&mut self) -> bool {
        std::mem::take(&mut self.params_dirty)
    }

    /// Project the live params into the snapshot the typed
    /// `ph2d-panel-color-equalization` paints. Host publishes a fresh
    /// snapshot once per frame while the tool is active.
    pub fn ui_snapshot(&self) -> ColorEqualizationUiSnapshot {
        ColorEqualizationUiSnapshot {
            clip_limit01: clip_limit_to_slider(self.params.clip_limit),
            tile_grid01: tile_grid_to_slider(self.params.tile_grid_size),
            brightness01: brightness_to_slider(self.params.brightness),
            contrast01: contrast_to_slider(self.params.contrast),
            saturation01: saturation_to_slider(self.params.saturation),
            auto_wb: self.params.auto_wb,
            clip_limit: self.params.clip_limit,
            tile_grid_size: self.params.tile_grid_size,
            brightness: self.params.brightness,
            contrast: self.params.contrast,
            saturation: self.params.saturation,
        }
    }

    /// Apply one panel-originated edit against the live params. Re-runs
    /// the preview when a param actually changed and a source is loaded.
    /// `Apply` arms the pending-apply latch the host drains via
    /// [`Self::take_pending_apply`].
    pub fn apply_ui_edit(&mut self, edit: ColorEqualizationUiEdit) {
        if matches!(edit, ColorEqualizationUiEdit::Apply) {
            self.pending_apply = true;
            self.params_dirty = true;
            return;
        }
        if apply_ui_edit(&mut self.params, edit) && self.has_source() {
            self.preview_dirty = true;
        }
        self.params_dirty = true;
    }

    /// Borrow the current preview RGBA + its dimensions. Returns an empty
    /// slice + `(0, 0)` until a source is pushed. Lazily reruns the
    /// pipeline when `preview_dirty` is set.
    pub fn preview_rgba(&mut self) -> (&[u8], u32, u32) {
        if self.preview_dirty && self.has_source() {
            self.preview_rgba.clear();
            run_pipeline(
                &self.preview_src_rgba,
                self.preview_src_w,
                self.preview_src_h,
                &self.params,
                &mut self.preview_rgba,
            );
            self.preview_dirty = false;
        }
        (&self.preview_rgba, self.preview_src_w, self.preview_src_h)
    }

    /// Run the full-resolution pipeline against the cached source and
    /// write the result into `out` (resized to fit). Returns the `(w, h)`
    /// of the output.
    ///
    /// Called by the shell drain on Apply for EACH selected sprite (one
    /// `OneShotImageOp` per sprite). The shell is expected to call
    /// [`Self::set_source_snapshot`] with that sprite's pixels before
    /// invoking this so the bake matches the live source.
    pub fn run_full_resolution(&mut self, out: &mut Vec<u8>) -> (u32, u32) {
        assert!(self.has_source(), "set_source_snapshot must run first");
        run_pipeline(
            &self.source_rgba,
            self.source_w,
            self.source_h,
            &self.params,
            out,
        );
        (self.source_w, self.source_h)
    }

    /// Build [`Self::preview_src_rgba`] — the source downscaled to fit
    /// [`PREVIEW_MAX_DIM`] (aspect preserved, no letterbox). No-op without
    /// a source.
    fn rebuild_preview_src(&mut self) {
        if !self.has_source() {
            self.preview_src_w = 0;
            self.preview_src_h = 0;
            self.preview_src_rgba.clear();
            return;
        }
        let (dw, dh) = aspect_fit_within(self.source_w, self.source_h, PREVIEW_MAX_DIM);
        if dw == self.source_w && dh == self.source_h {
            self.preview_src_rgba.clear();
            self.preview_src_rgba.extend_from_slice(&self.source_rgba);
        } else {
            self.preview_src_rgba =
                resize_bilinear_rgba(&self.source_rgba, self.source_w, self.source_h, dw, dh);
        }
        self.preview_src_w = dw;
        self.preview_src_h = dh;
    }
}

impl Tool for ColorEqualizationTool {
    fn id(&self) -> ToolId {
        ToolId::new("color_equalization")
    }

    fn label(&self) -> &str {
        "Color Equalization"
    }

    fn icon_slug(&self) -> &str {
        "color-equalization"
    }

    fn build_panel(&self) -> FloatingPanel {
        // The real UI is the typed `ph2d-panel-color-equalization` crate;
        // the FloatingPanel here is a minimal shell so `Tool::build_panel`
        // has a value (mirrors `PaddingTool::build_panel`). The docked
        // panel reads its layout from the snapshot the host publishes.
        let mut panel = FloatingPanel::new(self.id(), "Color Equalization");
        panel.anchor = PanelAnchor::BottomCenter;
        panel
    }

    fn on_deactivate(&mut self) {
        // Clear the one-shot drain flags so a cancel-mid-Apply (or any
        // deactivation while the bridge hasn't yet drained) does not fire
        // a phantom bake nor a spurious preview rerun on next activation.
        // Params persist (mirrors Padding / BgRemoval).
        self.pending_apply = false;
        self.params_dirty = false;
    }

    fn handle_panel_event(&mut self, event: PanelEvent) {
        // Sliders carry the normalized 0..1 track; the paired number chips
        // carry the natural unit. `apply_ui_edit` (in `params.rs`)
        // centralizes the clamps — every NodeId match here just routes to
        // the matching variant.
        match event {
            // Sliders.
            PanelEvent::SetValue(id, v) if id == ids::CEQ_CLIP_LIMIT => {
                self.apply_ui_edit(ColorEqualizationUiEdit::ClipLimitSlider(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_TILE_GRID => {
                self.apply_ui_edit(ColorEqualizationUiEdit::TileGridSlider(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_BRIGHTNESS => {
                self.apply_ui_edit(ColorEqualizationUiEdit::BrightnessSlider(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_CONTRAST => {
                self.apply_ui_edit(ColorEqualizationUiEdit::ContrastSlider(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_SATURATION => {
                self.apply_ui_edit(ColorEqualizationUiEdit::SaturationSlider(v as f32));
            }
            // Number chips (natural unit).
            PanelEvent::SetValue(id, v) if id == ids::CEQ_CLIP_LIMIT_NUM => {
                self.apply_ui_edit(ColorEqualizationUiEdit::ClipLimit(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_TILE_GRID_NUM => {
                self.apply_ui_edit(ColorEqualizationUiEdit::TileGrid(v.round() as u32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_BRIGHTNESS_NUM => {
                self.apply_ui_edit(ColorEqualizationUiEdit::Brightness(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_CONTRAST_NUM => {
                self.apply_ui_edit(ColorEqualizationUiEdit::Contrast(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == ids::CEQ_SATURATION_NUM => {
                self.apply_ui_edit(ColorEqualizationUiEdit::Saturation(v as f32));
            }
            // Toggles + buttons.
            PanelEvent::Click(id) if id == ids::CEQ_AUTO_WB => {
                self.apply_ui_edit(ColorEqualizationUiEdit::ToggleAutoWb);
            }
            PanelEvent::Toggle(id, _) if id == ids::CEQ_AUTO_WB => {
                self.apply_ui_edit(ColorEqualizationUiEdit::ToggleAutoWb);
            }
            PanelEvent::Click(id) if id == ids::CEQ_APPLY => {
                self.apply_ui_edit(ColorEqualizationUiEdit::Apply);
            }
            _ => {}
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: u32, h: u32, rgb: [u8; 3]) -> Vec<u8> {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..(w * h) {
            v.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
        v
    }

    #[test]
    fn default_tool_has_no_source_and_no_pending() {
        let mut t = ColorEqualizationTool::default();
        assert!(!t.has_source());
        assert_eq!(t.preview_rgba().0.len(), 0);
        assert!(!t.take_pending_apply());
        assert!(!t.take_params_dirty());
    }

    #[test]
    fn id_label_icon() {
        let t = ColorEqualizationTool::default();
        assert_eq!(t.id(), ToolId::new("color_equalization"));
        assert_eq!(t.label(), "Color Equalization");
        assert_eq!(t.icon_slug(), "color-equalization");
    }

    #[test]
    fn ui_snapshot_mirrors_defaults() {
        let t = ColorEqualizationTool::default();
        let s = t.ui_snapshot();
        let dft = ColorEqualizationUiSnapshot::default();
        assert_eq!(s, dft);
    }

    #[test]
    fn slider_event_updates_params() {
        let mut t = ColorEqualizationTool::default();
        // Brightness slider mid-range (0.75 → 0.5 brightness).
        t.handle_panel_event(PanelEvent::SetValue(ids::CEQ_BRIGHTNESS, 0.75));
        assert!((t.params.brightness - 0.5).abs() < 1e-5);
        // Tile grid chip clamps.
        t.handle_panel_event(PanelEvent::SetValue(ids::CEQ_TILE_GRID_NUM, 100.0));
        assert_eq!(t.params.tile_grid_size, 16);
    }

    #[test]
    fn apply_arms_pending_once() {
        let mut t = ColorEqualizationTool::default();
        assert!(!t.take_pending_apply());
        t.handle_panel_event(PanelEvent::Click(ids::CEQ_APPLY));
        assert!(t.take_pending_apply());
        assert!(!t.take_pending_apply());
    }

    #[test]
    fn auto_wb_toggle_event_flips_param() {
        let mut t = ColorEqualizationTool::default();
        assert!(!t.params.auto_wb);
        t.handle_panel_event(PanelEvent::Click(ids::CEQ_AUTO_WB));
        assert!(t.params.auto_wb);
        t.handle_panel_event(PanelEvent::Toggle(ids::CEQ_AUTO_WB, false));
        assert!(!t.params.auto_wb);
    }

    #[test]
    fn deactivate_clears_pending_but_keeps_params() {
        let mut t = ColorEqualizationTool::default();
        t.handle_panel_event(PanelEvent::SetValue(ids::CEQ_BRIGHTNESS, 1.0));
        t.handle_panel_event(PanelEvent::Click(ids::CEQ_APPLY));
        t.on_deactivate();
        assert!(!t.take_pending_apply());
        assert!(!t.take_params_dirty());
        // Params persist.
        assert!((t.params.brightness - 1.0).abs() < 1e-5);
    }

    #[test]
    fn set_source_snapshot_marks_has_source_true() {
        let mut t = ColorEqualizationTool::default();
        let buf = solid(8, 8, [120, 80, 200]);
        t.set_source_snapshot(buf, 8, 8);
        assert!(t.has_source());
        assert_eq!(t.source_size(), (8, 8));
    }

    #[test]
    fn preview_is_built_lazily_after_param_edit() {
        let mut t = ColorEqualizationTool::default();
        t.set_source_snapshot(solid(8, 8, [180, 120, 60]), 8, 8);
        // Drain the initial dirty marker that source-push armed.
        let _ = t.preview_rgba();
        t.handle_panel_event(PanelEvent::SetValue(ids::CEQ_BRIGHTNESS, 1.0));
        let (rgba, w, h) = t.preview_rgba();
        assert_eq!(w, 8);
        assert_eq!(h, 8);
        assert!(!rgba.is_empty());
        // Pixels should have been lifted by brightness.
        assert!(rgba[0] > 180);
    }

    #[test]
    fn preview_caps_at_max_dim_for_large_sources() {
        let mut t = ColorEqualizationTool::default();
        // 1024² source → preview at 512² (PREVIEW_MAX_DIM).
        t.set_source_snapshot(solid(1024, 1024, [128, 128, 128]), 1024, 1024);
        let (rgba, w, h) = t.preview_rgba();
        assert_eq!(w, PREVIEW_MAX_DIM);
        assert_eq!(h, PREVIEW_MAX_DIM);
        assert_eq!(rgba.len(), (PREVIEW_MAX_DIM * PREVIEW_MAX_DIM * 4) as usize);
    }

    #[test]
    fn run_full_resolution_returns_source_dims() {
        let mut t = ColorEqualizationTool::default();
        t.set_source_snapshot(solid(7, 11, [50, 100, 200]), 7, 11);
        let mut out = Vec::new();
        let (w, h) = t.run_full_resolution(&mut out);
        assert_eq!((w, h), (7, 11));
        assert_eq!(out.len(), 7 * 11 * 4);
    }

    #[test]
    fn downcast_via_as_any_mut_round_trips() {
        // Mirrors the shell's downcast path for raster I/O (DIRETRIZ
        // §3.8.3.1 — production tools still use this pattern).
        let mut boxed: Box<dyn Tool> = Box::new(ColorEqualizationTool::default());
        let any = boxed.as_any_mut();
        let tool = any.downcast_mut::<ColorEqualizationTool>().unwrap();
        tool.set_source_snapshot(solid(4, 4, [10, 20, 30]), 4, 4);
        assert!(tool.has_source());
    }
}
