//! [`BgRemovalTool`] — stateful editor Tool for raster bg removal.
//!
//! Model: per-mode params + cached source snapshot + thumbnail
//! preview + scratch buffer. The Tool runs the algorithm pipeline
//! twice per Apply:
//!
//! - On `set_source_snapshot` and on every panel event, the Tool
//!   re-runs `algorithm::run_pipeline` on the 160×160 thumbnail.
//!   The result lands in `self.preview_rgba`, ready for the panel
//!   paint to display. The thumbnail is built once per snapshot via
//!   [`image::imageops::resize`] with `Triangle` (cheap box-quality,
//!   no ringing — good enough for a preview).
//! - On Apply trigger, the Tool sets `self.pending_apply = true`.
//!   The host drains via [`BgRemovalTool::take_pending_apply`], runs
//!   the pipeline at full resolution against the live `Sprite.source`,
//!   and swaps the texture per the Image Tools precedent
//!   ([`crate::tools::trim_transparency`]).
//!
//! All pointer / hover / canvas interaction is **out of scope** —
//! the tool reacts only to its panel widgets, never to the canvas
//! (consistent with the §5.5 ENTREGÁVEL contract).
//!
//! ## Apply trigger mechanism
//!
//! The Widget Gallery's [`PanelControl::Action`](crate::floating_panel::PanelControl::Action)
//! variant is paint-only (no `NodeId`, so the dispatcher cannot route
//! click events to it). The canonical workaround used here is a
//! single-shot **Toggle** wired to the Apply event: the Tool reads
//! [`PanelEvent::Toggle`](crate::tool::PanelEvent::Toggle) with
//! `on = true` as "fire Apply", sets `pending_apply`, then
//! rebuilds the panel with the Toggle's `on = false` so the visual
//! resets in the next paint. UX wart documented in
//! [`INTEGRATION.md`](INTEGRATION.md) §3.1 — Coord can swap this for
//! a proper PanelAction-with-NodeId once `floating_panel.rs` gets
//! that surface.

use crate::floating_panel::{FloatingPanel, PanelAnchor, PanelControl, PanelTab, ToolId};
use crate::tool::{PanelEvent, Tool};
use crate::widget::{RadioGroup, RadioOption, Slider, Toggle};
use ph2d_a11y::NodeId;

use super::algorithm::run_pipeline;
use super::params::{
    BgRemovalMode, BgRemovalParams, BgRemovalUiEdit, BgRemovalUiSnapshot, FEATHER_FULL_SCALE,
    GROW_FULL_SCALE, MAX_EXTRA_BG_COLORS, REFINE_RADIUS_FULL_SCALE, TOLERANCE_FULL_SCALE,
};
use super::scratch::BgRemovalScratch;

/// Side length (px) of the square thumbnail used for the panel preview.
pub const THUMB_SIZE: u32 = 160;

// NodeId range 500..599 reserved for bgremoval panel controls
// (clear of 100..199 brush/move and 1000..1099 grid_snap).
const MODE_GROUP_NODE: NodeId = NodeId(501);
const MODE_CHROMA_OPT: NodeId = NodeId(502);
const MODE_GRABCUT_OPT: NodeId = NodeId(503);
const TOLERANCE_NODE: NodeId = NodeId(504);
const FEATHER_NODE: NodeId = NodeId(505);
const REFINE_NODE: NodeId = NodeId(506);
const APPLY_NODE: NodeId = NodeId(507);

/// Editor Tool implementing the background-removal feature.
#[derive(Clone, Debug, Default)]
pub struct BgRemovalTool {
    /// User-tunable parameters, projected into the floating panel.
    pub params: BgRemovalParams,

    /// Latest source snapshot pushed by the host (`set_source_snapshot`).
    /// Empty until the host calls — in that case the Tool renders an
    /// empty preview thumbnail. Layout: RGBA8, length
    /// `source_w * source_h * 4`.
    source_rgba: Vec<u8>,
    source_w: u32,
    source_h: u32,

    /// Pre-scaled thumbnail derived from `source_rgba`. Always
    /// `THUMB_SIZE × THUMB_SIZE` RGBA8 (aspect-fit, letterboxed).
    /// Built once per `set_source_snapshot` call; re-used as the
    /// input of every preview pipeline run.
    thumbnail_rgba: Vec<u8>,
    thumbnail_w: u32,
    thumbnail_h: u32,

    /// Preview output — result of `run_pipeline` on `thumbnail_rgba`
    /// with the current `params`. The panel paint pass blits this.
    /// Length `THUMB_SIZE * THUMB_SIZE * 4`.
    preview_rgba: Vec<u8>,

    /// Reusable scratch for both the preview pipeline and the host's
    /// full-res Apply. Sized lazily.
    scratch: BgRemovalScratch,

    /// Set to `true` when the user activates the Apply toggle. Host
    /// polls via [`Self::take_pending_apply`] each frame; on `true`
    /// it runs the pipeline at full resolution against the active
    /// sprite and writes back a new Individual texture.
    pending_apply: bool,

    /// Whether the panel eyedropper is armed. While `true`, the shell's
    /// canvas click-drag handler samples the source pixel under the
    /// cursor and feeds it to [`Self::add_extra_color`]. Reset on
    /// deactivate / Apply so a stale armed state can't keep eating
    /// canvas clicks after the tool is dismissed.
    eyedropper_armed: bool,

    /// Whether the protection brush is armed. While `true`, the shell's
    /// canvas click-drag handler paints into [`Self::protect_mask`] via
    /// [`Self::paint_protect_at_uv`] instead of running the normal pick /
    /// gizmo / selection logic. Reset on deactivate / Apply (mirrors
    /// `eyedropper_armed`).
    protect_brush_armed: bool,

    /// Freehand protection mask at the SOURCE resolution
    /// (`protect_mask_w × protect_mask_h`, one byte/pixel; `255` =
    /// protected/forced-foreground, `0` = unprotected). Empty until the
    /// user paints. SCAFFOLD: storage + accessors exist; the Implementer
    /// owns (a) the dab logic in `paint_protect_at_uv`, (b) downscaling
    /// this to the thumbnail / threading it into `run_pipeline` (Chroma
    /// force-keep + GrabCut `FgHard`), and (c) tests.
    protect_mask: Vec<u8>,
    protect_mask_w: u32,
    protect_mask_h: u32,
}

/// Squared RGB Euclidean distance below which two extra colours are
/// treated as duplicates (skip-on-add). `24²` ≈ a barely-perceptible
/// step; stops a click-drag from appending hundreds of near-identical
/// samples across a smooth gradient. // LITERAL-OK: dedup perceptual budget
const EXTRA_COLOR_DEDUP_DIST_SQ: i32 = 24 * 24;

impl BgRemovalTool {
    /// Push a fresh source RGBA snapshot from the host. Called when
    /// the selection changes or the tool becomes active. Rebuilds
    /// the thumbnail and re-renders the preview with the current
    /// params.
    ///
    /// `rgba` must be straight-alpha RGBA8 of length `w * h * 4`.
    pub fn set_source_snapshot(&mut self, rgba: Vec<u8>, w: u32, h: u32) {
        assert_eq!(rgba.len(), (w as usize) * (h as usize) * 4);
        self.source_rgba = rgba;
        self.source_w = w;
        self.source_h = h;
        self.rebuild_thumbnail();
        self.rerun_preview();
    }

    /// Whether the host has pushed a source snapshot at least once.
    pub fn has_source(&self) -> bool {
        !self.source_rgba.is_empty()
    }

    /// Borrow the current thumbnail preview (RGBA8,
    /// `THUMB_SIZE × THUMB_SIZE`). Returns an empty slice when
    /// `has_source()` is false.
    pub fn preview_rgba(&self) -> &[u8] {
        &self.preview_rgba
    }

    /// Drain the pending-apply flag. Returns `true` exactly once
    /// after each Apply trigger. Host calls this in its per-frame
    /// drain loop; on `true` it runs the pipeline at full resolution.
    pub fn take_pending_apply(&mut self) -> bool {
        let p = self.pending_apply;
        self.pending_apply = false;
        p
    }

    /// Whether the panel eyedropper is armed (shell samples canvas
    /// click-drags into extra colours while `true`).
    pub fn is_eyedropper_armed(&self) -> bool {
        self.eyedropper_armed
    }

    /// Set the eyedropper armed state (shell mirror of the panel toggle).
    pub fn set_eyedropper_armed(&mut self, armed: bool) {
        self.eyedropper_armed = armed;
    }

    // ── Protection brush (SCAFFOLD — Coordinator) ──────────────────────
    // Contract surface the panel + shell compile against. The Implementer
    // fills the dab/threading bodies + tests; do NOT change these public
    // signatures without reporting (the shell `input_dispatch` + overlay
    // call them). Mirrors the eyedropper arm/sample pattern.

    /// Whether the protection brush is armed (shell paints canvas
    /// click-drags into the protection mask while `true`).
    pub fn is_protect_armed(&self) -> bool {
        self.protect_brush_armed
    }

    /// Set the protection-brush armed state (shell mirror of the panel
    /// toggle). Arming the brush disarms the eyedropper so the two canvas
    /// modes never fight over the same click.
    pub fn set_protect_armed(&mut self, armed: bool) {
        self.protect_brush_armed = armed;
        if armed {
            self.eyedropper_armed = false;
        }
    }

    /// Whether the protection mask currently holds any painted pixels.
    pub fn has_protect_mask(&self) -> bool {
        self.protect_mask.iter().any(|&v| v != 0)
    }

    /// Borrow the source-resolution protection mask for the shell's
    /// on-canvas overlay: `(mask, w, h)`, one byte/pixel (`255` =
    /// protected). Empty slice + `(0, 0)` when nothing is painted.
    pub fn protect_mask_source(&self) -> (&[u8], u32, u32) {
        (&self.protect_mask, self.protect_mask_w, self.protect_mask_h)
    }

    /// Paint a brush dab into the protection mask at normalized UV
    /// `(u, v)` (`[0,1]` each, origin top-left) with `radius_px` measured
    /// at SOURCE resolution. Called by the shell on canvas click-drag
    /// while the brush is armed (mirrors `add_extra_color` /
    /// `sample_source_at_uv`).
    ///
    /// SCAFFOLD STUB — the Implementer fills: lazy-size `protect_mask` to
    /// the source dims, stamp a filled disc of `255` around the UV, and
    /// rerun the preview so the overlay + matte update live.
    pub fn paint_protect_at_uv(&mut self, _u: f32, _v: f32, _radius_px: f32) {
        // TODO(impl): size protect_mask to source dims; stamp disc; rerun preview.
    }

    /// Wipe the painted protection mask. Reruns the preview when a source
    /// is loaded so the matte drops the forced-keep region immediately.
    pub fn clear_protect_mask(&mut self) {
        self.protect_mask.clear();
        self.protect_mask_w = 0;
        self.protect_mask_h = 0;
        if self.has_source() {
            self.rerun_preview();
        }
    }

    /// Borrow the current extra background colours (sRGB 8-bit).
    pub fn extra_colors(&self) -> &[[u8; 3]] {
        &self.params.extra_bg_colors
    }

    /// Append a user-picked extra background colour. No-op when the
    /// colour duplicates (exactly or within
    /// [`EXTRA_COLOR_DEDUP_DIST_SQ`]) one already stored, or when the
    /// list is already at [`MAX_EXTRA_BG_COLORS`]. Re-runs the preview
    /// when something was actually added and a source is loaded.
    pub fn add_extra_color(&mut self, rgb: [u8; 3]) {
        if self.params.extra_bg_colors.len() >= MAX_EXTRA_BG_COLORS {
            return;
        }
        let is_dup = self.params.extra_bg_colors.iter().any(|c| {
            let dr = c[0] as i32 - rgb[0] as i32;
            let dg = c[1] as i32 - rgb[1] as i32;
            let db = c[2] as i32 - rgb[2] as i32;
            dr * dr + dg * dg + db * db <= EXTRA_COLOR_DEDUP_DIST_SQ
        });
        if is_dup {
            return;
        }
        self.params.extra_bg_colors.push(rgb);
        if self.has_source() {
            self.rerun_preview();
        }
    }

    /// Remove the extra background colour at `idx` (bounds-checked).
    /// Re-runs the preview when the index was valid and a source is
    /// loaded.
    pub fn remove_extra_color(&mut self, idx: usize) {
        if idx >= self.params.extra_bg_colors.len() {
            return;
        }
        self.params.extra_bg_colors.remove(idx);
        if self.has_source() {
            self.rerun_preview();
        }
    }

    /// Sample the stored SOURCE snapshot at normalized UV `(u, v)`
    /// (`[0,1]` each, origin top-left), nearest-pixel. Returns the RGB
    /// of that pixel, or `None` when no source is loaded or the UV is
    /// out of range. Samples the SOURCE — never the framebuffer — so the
    /// picked colour is the true sprite colour, not the composited
    /// preview (which carries the in-progress transparency).
    pub fn sample_source_at_uv(&self, u: f32, v: f32) -> Option<[u8; 3]> {
        if !self.has_source() || !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) {
            return None;
        }
        // Nearest-pixel: map [0,1] onto [0, dim-1].
        let px = ((u * (self.source_w as f32 - 1.0)).round() as i64)
            .clamp(0, self.source_w as i64 - 1) as usize;
        let py = ((v * (self.source_h as f32 - 1.0)).round() as i64)
            .clamp(0, self.source_h as i64 - 1) as usize;
        let base = (py * self.source_w as usize + px) * 4;
        Some([
            self.source_rgba[base],
            self.source_rgba[base + 1],
            self.source_rgba[base + 2],
        ])
    }

    /// Run the full-resolution pipeline on the cached `source_rgba`
    /// (called from the host's drain handler) and write the result
    /// into `out`. `out` is grown to `source_w * source_h * 4` if
    /// needed.
    ///
    /// Returns the `(w, h)` of the output.
    pub fn run_full_resolution(&mut self, out: &mut Vec<u8>) -> (u32, u32) {
        assert!(self.has_source(), "set_source_snapshot must run first");
        run_pipeline(
            &self.source_rgba,
            self.source_w,
            self.source_h,
            &self.params,
            &mut self.scratch,
        );
        out.clear();
        out.extend_from_slice(&self.scratch.output_rgba);
        (self.source_w, self.source_h)
    }

    /// Aspect-fit `source_rgba` into a `THUMB_SIZE × THUMB_SIZE` RGBA8
    /// buffer with transparent letterbox borders. Uses
    /// `image::imageops::resize` with `Triangle` (cheap box-quality,
    /// no ringing — fine for a 160-px preview that gets re-segmented
    /// every panel-event frame).
    ///
    /// No-op when the host hasn't pushed a source snapshot yet.
    ///
    /// Allocations: one `ImageBuffer` for the source view and one for
    /// the resized output (both freed before return). The owned
    /// `self.thumbnail_rgba` is `clear()`-ed and re-extended so its
    /// capacity persists across calls (HR-3 in the steady state where
    /// every Apply sees the same source size).
    fn rebuild_thumbnail(&mut self) {
        if !self.has_source() {
            self.thumbnail_w = 0;
            self.thumbnail_h = 0;
            self.thumbnail_rgba.clear();
            return;
        }
        let target = THUMB_SIZE;
        // Aspect-fit: scale the LONGER side to `target`, the shorter
        // side gets proportional scaling. Degenerate dims fall back
        // to 1 px so the resize call doesn't panic.
        let (sw, sh) = aspect_fit(self.source_w, self.source_h, target);
        let src = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
            self.source_w,
            self.source_h,
            self.source_rgba.clone(),
        )
        .expect("source_rgba length matches source_w * source_h * 4");
        let resized: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
            if sw == self.source_w && sh == self.source_h {
                src
            } else {
                image::imageops::resize(&src, sw, sh, image::imageops::FilterType::Triangle)
            };
        // Letterbox into target × target with transparent borders.
        let pad_x = (target - sw) / 2;
        let pad_y = (target - sh) / 2;
        let total_bytes = (target as usize) * (target as usize) * 4;
        self.thumbnail_rgba.clear();
        self.thumbnail_rgba.resize(total_bytes, 0);
        for row in 0..sh {
            let dst_y = (pad_y + row) as usize;
            let dst_start = (dst_y * (target as usize) + pad_x as usize) * 4;
            let src_start = (row as usize) * (sw as usize) * 4;
            let row_bytes = (sw as usize) * 4;
            self.thumbnail_rgba[dst_start..dst_start + row_bytes]
                .copy_from_slice(&resized.as_raw()[src_start..src_start + row_bytes]);
        }
        self.thumbnail_w = target;
        self.thumbnail_h = target;
    }

    /// Re-run the segmentation pipeline against the cached thumbnail
    /// with the current `params`. Output lands in `self.preview_rgba`,
    /// always `THUMB_SIZE * THUMB_SIZE * 4` bytes.
    ///
    /// No-op when `rebuild_thumbnail` hasn't produced a buffer yet.
    fn rerun_preview(&mut self) {
        if self.thumbnail_rgba.is_empty() {
            self.preview_rgba.clear();
            return;
        }
        run_pipeline(
            &self.thumbnail_rgba,
            self.thumbnail_w,
            self.thumbnail_h,
            &self.params,
            &mut self.scratch,
        );
        self.preview_rgba.clear();
        self.preview_rgba
            .extend_from_slice(&self.scratch.output_rgba);
    }

    /// Project the current full-scale params into the normalized
    /// snapshot the typed `ph2d-panel-bgremoval` paints. Published by
    /// the host once per frame while the tool is active (forward of
    /// [`Self::apply_ui_edit`]).
    pub fn ui_snapshot(&self) -> BgRemovalUiSnapshot {
        BgRemovalUiSnapshot {
            mode: self.params.mode,
            tolerance01: (self.params.chroma.tolerance / TOLERANCE_FULL_SCALE).clamp(0.0, 1.0),
            feather01: (self.params.chroma.feather / FEATHER_FULL_SCALE).clamp(0.0, 1.0),
            refine01: (self.params.refinement.radius as f32 / REFINE_RADIUS_FULL_SCALE)
                .clamp(0.0, 1.0),
            grow01: (self.params.grow_px / (2.0 * GROW_FULL_SCALE) + 0.5).clamp(0.0, 1.0),
            extra_colors: self.params.extra_bg_colors.clone(),
            eyedropper_armed: self.eyedropper_armed,
            protect_brush_armed: self.protect_brush_armed,
            has_protect_mask: self.has_protect_mask(),
        }
    }

    /// Apply one panel-originated edit (normalized slider value / mode /
    /// Apply) against the live params. Re-runs the thumbnail preview when
    /// a param actually changed and a source snapshot is loaded. `Apply`
    /// arms the pending-apply flag the host drains via
    /// [`Self::take_pending_apply`]. Inverse of [`Self::ui_snapshot`].
    pub fn apply_ui_edit(&mut self, edit: BgRemovalUiEdit) {
        let mut changed = false;
        match edit {
            BgRemovalUiEdit::Mode(mode) => {
                if self.params.mode != mode {
                    self.params.mode = mode;
                    changed = true;
                }
            }
            BgRemovalUiEdit::Tolerance(v) => {
                self.params.chroma.tolerance = v.clamp(0.0, 1.0) * TOLERANCE_FULL_SCALE;
                changed = true;
            }
            BgRemovalUiEdit::Feather(v) => {
                let v = v.clamp(0.0, 1.0);
                // Drives BOTH feather paths so the slider is never a
                // no-op regardless of Refine:
                //   • Refine == 0 (compose soft-band path): widens the
                //     `[tol, tol+feather]` ΔE transition band.
                //   • Refine  > 0 (guided-filter path, the default):
                //     maps to the filter's regularisation ε on a log
                //     scale — the canonical "guided feathering" control
                //     (He et al. 2013). Larger ε ⇒ the matte ignores
                //     fine guide edges and blurs softer; smaller ε ⇒
                //     edges hug the luma guide tightly. Range 1e-6
                //     (sharp) … 1e-1 (very soft).
                self.params.chroma.feather = v * FEATHER_FULL_SCALE;
                self.params.refinement.epsilon = 1.0e-6 * 10.0_f32.powf(5.0 * v);
                changed = true;
            }
            BgRemovalUiEdit::Refine(v) => {
                self.params.refinement.radius =
                    (v.clamp(0.0, 1.0) * REFINE_RADIUS_FULL_SCALE).round() as u32;
                changed = true;
            }
            BgRemovalUiEdit::Grow(v) => {
                // Bipolar: 0.5 = neutral, maps to ∓GROW_FULL_SCALE px.
                self.params.grow_px = (v.clamp(0.0, 1.0) - 0.5) * 2.0 * GROW_FULL_SCALE;
                changed = true;
            }
            BgRemovalUiEdit::ToggleEyedropper => {
                // Flip the armed state. No preview rerun needed — arming
                // the picker doesn't change params; sampling does (via
                // `add_extra_color`).
                self.eyedropper_armed = !self.eyedropper_armed;
            }
            BgRemovalUiEdit::RemoveExtraColor(idx) => {
                // `remove_extra_color` already reruns the preview itself.
                self.remove_extra_color(idx);
            }
            BgRemovalUiEdit::ToggleProtectBrush => {
                // Flip the armed state; arming disarms the eyedropper
                // (`set_protect_armed` enforces the mutual exclusion). No
                // preview rerun — arming changes no params; painting does.
                self.set_protect_armed(!self.protect_brush_armed);
            }
            BgRemovalUiEdit::ClearProtectMask => {
                // `clear_protect_mask` reruns the preview itself.
                self.clear_protect_mask();
            }
            BgRemovalUiEdit::Apply => {
                self.pending_apply = true;
                // A commit ends the picking session — disarm so a stale
                // eyedropper / protect brush doesn't keep eating canvas
                // clicks on the freshly baked sprite.
                self.eyedropper_armed = false;
                self.protect_brush_armed = false;
            }
        }
        if changed && self.has_source() {
            self.rerun_preview();
        }
    }

    /// Build the Mode RadioGroup (Chroma / Smart Cut) seeded with the
    /// currently-selected mode.
    fn build_mode_radio(&self) -> RadioGroup<String> {
        let selected = match self.params.mode {
            BgRemovalMode::Chroma => "chroma",
            BgRemovalMode::GrabCut => "grabcut",
        };
        RadioGroup::new(
            MODE_GROUP_NODE,
            "Mode",
            vec![
                RadioOption::new(MODE_CHROMA_OPT, "chroma".to_string(), "Chroma"),
                RadioOption::new(MODE_GRABCUT_OPT, "grabcut".to_string(), "Smart Cut"),
            ],
        )
        .selected(selected.to_string())
    }
}

impl Tool for BgRemovalTool {
    fn id(&self) -> ToolId {
        ToolId::new("bgremoval")
    }

    fn label(&self) -> &str {
        "Bg Removal"
    }

    fn icon_slug(&self) -> &str {
        "bgremoval"
    }

    fn build_panel(&self) -> FloatingPanel {
        let mode = self.build_mode_radio();

        let mut tolerance = Slider::new(TOLERANCE_NODE, "Tolerance");
        tolerance.value = (self.params.chroma.tolerance / 0.30).clamp(0.0, 1.0);

        let mut feather = Slider::new(FEATHER_NODE, "Feather");
        feather.value = (self.params.chroma.feather / 0.20).clamp(0.0, 1.0);

        let mut refine = Slider::new(REFINE_NODE, "Refine");
        refine.value = (self.params.refinement.radius as f32 / 100.0).clamp(0.0, 1.0);

        // Apply uses Toggle as one-shot trigger: on=false in every
        // rebuild; turning on fires `pending_apply` and the next
        // build_panel reverts to off (see INTEGRATION.md §3.1).
        let apply = Toggle::new(APPLY_NODE, "Apply");

        let mut panel = FloatingPanel::new(self.id(), "Bg Removal")
            .with_tabs(vec![PanelTab {
                label: self.params.mode.label().to_string(),
                icon: None,
                active: true,
            }])
            .with_controls(vec![
                PanelControl::RadioGroup(mode),
                PanelControl::Slider(tolerance),
                PanelControl::Slider(feather),
                PanelControl::Slider(refine),
                PanelControl::Toggle(apply),
            ]);
        panel.anchor = PanelAnchor::BottomCenter;
        panel.width = 600.0;
        panel.height = 110.0;
        panel
    }

    fn handle_panel_event(&mut self, event: PanelEvent) {
        let mut changed = false;
        match event {
            PanelEvent::SetValue(id, v) if id == TOLERANCE_NODE => {
                self.params.chroma.tolerance = (v.clamp(0.0, 1.0) as f32) * 0.30;
                changed = true;
            }
            PanelEvent::SetValue(id, v) if id == FEATHER_NODE => {
                self.params.chroma.feather = (v.clamp(0.0, 1.0) as f32) * 0.20;
                changed = true;
            }
            PanelEvent::SetValue(id, v) if id == REFINE_NODE => {
                self.params.refinement.radius = (v.clamp(0.0, 1.0) * 100.0).round() as u32;
                changed = true;
            }
            PanelEvent::SelectOption(id, value) if id == MODE_GROUP_NODE => {
                self.params.mode = match value.as_str() {
                    "grabcut" => BgRemovalMode::GrabCut,
                    _ => BgRemovalMode::Chroma,
                };
                changed = true;
            }
            // One-shot trigger; the next build_panel emits a
            // fresh Toggle with on=false so the visual resets.
            PanelEvent::Toggle(id, on) if id == APPLY_NODE && on => {
                self.pending_apply = true;
            }
            _ => {}
        }
        if changed && self.has_source() {
            self.rerun_preview();
        }
    }

    fn on_deactivate(&mut self) {
        // Disarm the eyedropper + protect brush so reactivating later
        // starts clean and a stale armed state can't intercept canvas
        // clicks meant for another tool.
        self.eyedropper_armed = false;
        self.protect_brush_armed = false;
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Compute the (w, h) that fit inside a `target × target` square,
/// preserving the input aspect ratio. The longer side lands on
/// `target`; the shorter side scales proportionally. Outputs are
/// clamped to at least 1 px so the resize call never sees a 0
/// dimension.
fn aspect_fit(sw: u32, sh: u32, target: u32) -> (u32, u32) {
    if sw == 0 || sh == 0 || target == 0 {
        return (target.max(1), target.max(1));
    }
    if sw >= sh {
        // landscape (or square) → width clamps to target.
        let scaled_h = ((sh as u64 * target as u64) / sw as u64) as u32;
        (target, scaled_h.max(1))
    } else {
        // portrait → height clamps to target.
        let scaled_w = ((sw as u64 * target as u64) / sh as u64) as u32;
        (scaled_w.max(1), target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tool_has_no_source_and_no_pending() {
        let t = BgRemovalTool::default();
        assert!(!t.has_source());
        assert!(t.preview_rgba().is_empty());
    }

    #[test]
    fn id_label_icon() {
        let t = BgRemovalTool::default();
        assert_eq!(t.id(), ToolId::new("bgremoval"));
        assert_eq!(t.label(), "Bg Removal");
        assert_eq!(t.icon_slug(), "bgremoval");
    }

    #[test]
    fn panel_has_five_canonical_controls() {
        let p = BgRemovalTool::default().build_panel();
        assert_eq!(p.tool_id, ToolId::new("bgremoval"));
        assert_eq!(p.title, "Bg Removal");
        assert_eq!(p.tabs.len(), 1);
        assert_eq!(p.controls.len(), 5);
        assert!(matches!(p.controls[0], PanelControl::RadioGroup(_)));
        assert!(matches!(p.controls[1], PanelControl::Slider(_)));
        assert!(matches!(p.controls[2], PanelControl::Slider(_)));
        assert!(matches!(p.controls[3], PanelControl::Slider(_)));
        assert!(matches!(p.controls[4], PanelControl::Toggle(_)));
        let labels: Vec<&str> = p.controls.iter().map(|c| c.label()).collect();
        assert_eq!(
            labels,
            vec!["Mode", "Tolerance", "Feather", "Refine", "Apply"]
        );
    }

    #[test]
    fn panel_radio_options_match_modes() {
        let p = BgRemovalTool::default().build_panel();
        match &p.controls[0] {
            PanelControl::RadioGroup(g) => {
                assert_eq!(g.options.len(), 2);
                assert_eq!(g.options[0].label, "Chroma");
                assert_eq!(g.options[1].label, "Smart Cut");
                assert_eq!(g.selected.as_deref(), Some("chroma"));
            }
            _ => panic!("expected RadioGroup at index 0"),
        }
    }

    #[test]
    fn slider_event_updates_params_and_clamps() {
        let mut t = BgRemovalTool::default();
        // Tolerance: slider value 0..1 maps to tolerance 0..0.30.
        t.handle_panel_event(PanelEvent::SetValue(TOLERANCE_NODE, 0.5));
        assert!((t.params.chroma.tolerance - 0.15).abs() < 1e-5);
        // Slider value out-of-range is clamped.
        t.handle_panel_event(PanelEvent::SetValue(TOLERANCE_NODE, 1.5));
        assert!((t.params.chroma.tolerance - 0.30).abs() < 1e-5);
    }

    #[test]
    fn apply_toggle_one_shot_trigger() {
        let mut t = BgRemovalTool::default();
        assert!(!t.take_pending_apply());
        // Toggle on → fire apply.
        t.handle_panel_event(PanelEvent::Toggle(APPLY_NODE, true));
        assert!(t.take_pending_apply());
        // Drained: second call returns false.
        assert!(!t.take_pending_apply());
        // Toggle off (or a stray "off" event) should not fire apply.
        t.handle_panel_event(PanelEvent::Toggle(APPLY_NODE, false));
        assert!(!t.take_pending_apply());
    }

    #[test]
    fn apply_toggle_rebuilds_with_off_state() {
        let mut t = BgRemovalTool::default();
        t.handle_panel_event(PanelEvent::Toggle(APPLY_NODE, true));
        // pending was consumed; the next build_panel must emit
        // Toggle(on=false) so the UI does not stick lit.
        let panel = t.build_panel();
        match &panel.controls[4] {
            PanelControl::Toggle(tg) => assert!(!tg.on, "Apply toggle must reset to off"),
            _ => panic!("expected Toggle at index 4"),
        }
    }

    #[test]
    fn set_source_snapshot_marks_has_source_true() {
        let mut t = BgRemovalTool::default();
        let buf = vec![255u8; 8 * 8 * 4];
        t.set_source_snapshot(buf, 8, 8);
        assert!(t.has_source());
    }

    #[test]
    fn set_source_snapshot_builds_thumbnail_and_preview() {
        // Push a 32×32 opaque-white source; the thumbnail must
        // letterbox to 160×160 and the preview pipeline must produce
        // a same-size buffer.
        let mut t = BgRemovalTool::default();
        let buf = vec![255u8; 32 * 32 * 4];
        t.set_source_snapshot(buf, 32, 32);
        assert_eq!(t.thumbnail_w, THUMB_SIZE);
        assert_eq!(t.thumbnail_h, THUMB_SIZE);
        assert_eq!(
            t.thumbnail_rgba.len(),
            (THUMB_SIZE as usize) * (THUMB_SIZE as usize) * 4
        );
        assert_eq!(
            t.preview_rgba().len(),
            (THUMB_SIZE as usize) * (THUMB_SIZE as usize) * 4
        );
    }

    #[test]
    fn slider_event_triggers_preview_rerun() {
        // Mutating a param after a source snapshot is live re-runs
        // the preview pipeline. Tests the `changed && has_source`
        // gate in `handle_panel_event`.
        let mut t = BgRemovalTool::default();
        let buf = vec![255u8; 32 * 32 * 4];
        t.set_source_snapshot(buf, 32, 32);
        let baseline = t.preview_rgba().to_vec();
        t.handle_panel_event(PanelEvent::SetValue(TOLERANCE_NODE, 0.9));
        // Preview ran again (length preserved; content may differ —
        // we don't assert on content, just on the contract that it
        // didn't get wiped to empty).
        assert_eq!(t.preview_rgba().len(), baseline.len());
    }

    #[test]
    fn aspect_fit_landscape_keeps_target_width() {
        let (w, h) = aspect_fit(400, 200, 160);
        assert_eq!(w, 160);
        assert_eq!(h, 80);
    }

    #[test]
    fn aspect_fit_portrait_keeps_target_height() {
        let (w, h) = aspect_fit(200, 400, 160);
        assert_eq!(h, 160);
        assert_eq!(w, 80);
    }

    #[test]
    fn aspect_fit_square_passes_through_target() {
        assert_eq!(aspect_fit(256, 256, 160), (160, 160));
    }

    #[test]
    fn aspect_fit_degenerate_returns_target_minimum() {
        // Zero dims should not panic — should produce a 1×1+ fallback
        // so the downstream resize call has a defined target.
        let (w, h) = aspect_fit(0, 0, 160);
        assert!(w >= 1 && h >= 1);
    }

    #[test]
    fn mode_radio_select_swaps_mode() {
        let mut t = BgRemovalTool::default();
        assert_eq!(t.params.mode, BgRemovalMode::Chroma);
        t.handle_panel_event(PanelEvent::SelectOption(MODE_GROUP_NODE, "grabcut".into()));
        assert_eq!(t.params.mode, BgRemovalMode::GrabCut);
        t.handle_panel_event(PanelEvent::SelectOption(MODE_GROUP_NODE, "chroma".into()));
        assert_eq!(t.params.mode, BgRemovalMode::Chroma);
    }

    #[test]
    fn ui_snapshot_reflects_default_params() {
        let t = BgRemovalTool::default();
        let s = t.ui_snapshot();
        assert_eq!(s.mode, BgRemovalMode::Chroma);
        // Tuned defaults (Enio 2026-05-20): tolerance 0.6, feather 0.9,
        // refine 0.01, grow chip −0.10 ⇒ grow01 0.45.
        assert!((s.tolerance01 - 0.6).abs() < 1e-5);
        assert!((s.feather01 - 0.9).abs() < 1e-5);
        assert!((s.refine01 - 0.01).abs() < 1e-5);
        assert!((s.grow01 - 0.45).abs() < 1e-5);
    }

    #[test]
    fn grow_edit_maps_bipolar_and_round_trips() {
        let mut t = BgRemovalTool::default();
        // Centre = neutral.
        t.apply_ui_edit(BgRemovalUiEdit::Grow(0.5));
        assert!(t.params.grow_px.abs() < 1e-5);
        // Full shrink (0.0) → −GROW_FULL_SCALE px; full grow (1.0) → +.
        t.apply_ui_edit(BgRemovalUiEdit::Grow(0.0));
        assert!((t.params.grow_px + GROW_FULL_SCALE).abs() < 1e-5);
        assert!(t.ui_snapshot().grow01.abs() < 1e-5);
        t.apply_ui_edit(BgRemovalUiEdit::Grow(1.0));
        assert!((t.params.grow_px - GROW_FULL_SCALE).abs() < 1e-5);
        assert!((t.ui_snapshot().grow01 - 1.0).abs() < 1e-5);
    }

    #[test]
    fn apply_ui_edit_round_trips_through_ui_snapshot() {
        let mut t = BgRemovalTool::default();
        t.apply_ui_edit(BgRemovalUiEdit::Tolerance(0.5));
        t.apply_ui_edit(BgRemovalUiEdit::Feather(0.25));
        t.apply_ui_edit(BgRemovalUiEdit::Refine(0.8));
        t.apply_ui_edit(BgRemovalUiEdit::Mode(BgRemovalMode::GrabCut));
        let s = t.ui_snapshot();
        assert_eq!(s.mode, BgRemovalMode::GrabCut);
        assert!((s.tolerance01 - 0.5).abs() < 1e-5);
        assert!((s.feather01 - 0.25).abs() < 1e-5);
        // Refine maps through an integer radius (0.8*100=80 → 80/100).
        assert!((s.refine01 - 0.8).abs() < 1e-2);
        // Full-scale values land where the slider maps expect.
        assert!((t.params.chroma.tolerance - 0.15).abs() < 1e-5);
        assert!((t.params.chroma.feather - 0.05).abs() < 1e-5);
        assert_eq!(t.params.refinement.radius, 80);
    }

    #[test]
    fn apply_ui_edit_clamps_out_of_range() {
        let mut t = BgRemovalTool::default();
        t.apply_ui_edit(BgRemovalUiEdit::Tolerance(2.0));
        assert!((t.ui_snapshot().tolerance01 - 1.0).abs() < 1e-5);
        t.apply_ui_edit(BgRemovalUiEdit::Tolerance(-1.0));
        assert!(t.ui_snapshot().tolerance01.abs() < 1e-5);
    }

    #[test]
    fn apply_ui_edit_apply_arms_pending() {
        let mut t = BgRemovalTool::default();
        assert!(!t.take_pending_apply());
        t.apply_ui_edit(BgRemovalUiEdit::Apply);
        assert!(t.take_pending_apply());
        assert!(!t.take_pending_apply());
    }

    #[test]
    fn mode_change_updates_active_tab_label() {
        let mut t = BgRemovalTool::default();
        let p = t.build_panel();
        assert_eq!(p.tabs[0].label, "Chroma");
        t.handle_panel_event(PanelEvent::SelectOption(MODE_GROUP_NODE, "grabcut".into()));
        let p = t.build_panel();
        assert_eq!(p.tabs[0].label, "Smart Cut");
    }

    // ── Eyedropper / extra-colour tests ────────────────────────────

    #[test]
    fn add_extra_color_dedups_near_duplicates_and_caps() {
        let mut t = BgRemovalTool::default();
        t.add_extra_color([100, 100, 100]);
        // A colour within ~24 RGB of an existing one is skipped.
        t.add_extra_color([110, 100, 100]);
        assert_eq!(t.extra_colors().len(), 1, "near-duplicate must be skipped");
        // A clearly different colour is appended.
        t.add_extra_color([10, 200, 30]);
        assert_eq!(t.extra_colors().len(), 2);

        // Cap at MAX_EXTRA_BG_COLORS with well-separated colours.
        // Grid in (R, G) with a 64-step so every pair is ≥ 64 apart
        // on at least one channel — far beyond the dedup radius.
        let mut t2 = BgRemovalTool::default();
        for i in 0..(MAX_EXTRA_BG_COLORS + 5) {
            let r = ((i % 4) * 64) as u8;
            let g = ((i / 4) * 64) as u8;
            t2.add_extra_color([r, g, 0]);
        }
        assert_eq!(t2.extra_colors().len(), MAX_EXTRA_BG_COLORS);
    }

    #[test]
    fn remove_extra_color_removes_right_index_and_is_bounds_checked() {
        let mut t = BgRemovalTool::default();
        t.add_extra_color([200, 0, 0]);
        t.add_extra_color([0, 200, 0]);
        t.add_extra_color([0, 0, 200]);
        t.remove_extra_color(1); // remove green
        assert_eq!(t.extra_colors(), &[[200, 0, 0], [0, 0, 200]]);
        // Out-of-bounds is a no-op.
        t.remove_extra_color(99);
        assert_eq!(t.extra_colors().len(), 2);
    }

    #[test]
    fn sample_source_at_uv_maps_corners() {
        // 2×2 source with 4 distinct colours: TL red, TR green,
        // BL blue, BR white.
        let mut t = BgRemovalTool::default();
        let buf = vec![
            255, 0, 0, 255, // (0,0) red
            0, 255, 0, 255, // (1,0) green
            0, 0, 255, 255, // (0,1) blue
            255, 255, 255, 255, // (1,1) white
        ];
        t.set_source_snapshot(buf, 2, 2);
        assert_eq!(t.sample_source_at_uv(0.0, 0.0), Some([255, 0, 0]));
        assert_eq!(t.sample_source_at_uv(1.0, 0.0), Some([0, 255, 0]));
        assert_eq!(t.sample_source_at_uv(0.0, 1.0), Some([0, 0, 255]));
        assert_eq!(t.sample_source_at_uv(1.0, 1.0), Some([255, 255, 255]));
        // Out of range → None.
        assert_eq!(t.sample_source_at_uv(1.5, 0.0), None);
        assert_eq!(t.sample_source_at_uv(0.0, -0.1), None);
    }

    #[test]
    fn sample_source_at_uv_none_without_source() {
        let t = BgRemovalTool::default();
        assert_eq!(t.sample_source_at_uv(0.5, 0.5), None);
    }

    #[test]
    fn ui_snapshot_reflects_extra_colors_and_armed() {
        let mut t = BgRemovalTool::default();
        assert!(t.ui_snapshot().extra_colors.is_empty());
        assert!(!t.ui_snapshot().eyedropper_armed);
        t.add_extra_color([1, 2, 3]);
        t.set_eyedropper_armed(true);
        let s = t.ui_snapshot();
        assert_eq!(s.extra_colors, vec![[1, 2, 3]]);
        assert!(s.eyedropper_armed);
    }

    #[test]
    fn toggle_eyedropper_edit_flips_armed() {
        let mut t = BgRemovalTool::default();
        assert!(!t.is_eyedropper_armed());
        t.apply_ui_edit(BgRemovalUiEdit::ToggleEyedropper);
        assert!(t.is_eyedropper_armed());
        t.apply_ui_edit(BgRemovalUiEdit::ToggleEyedropper);
        assert!(!t.is_eyedropper_armed());
    }

    #[test]
    fn remove_extra_color_edit_removes_index() {
        let mut t = BgRemovalTool::default();
        t.add_extra_color([200, 0, 0]);
        t.add_extra_color([0, 200, 0]);
        t.apply_ui_edit(BgRemovalUiEdit::RemoveExtraColor(0));
        assert_eq!(t.extra_colors(), &[[0, 200, 0]]);
    }

    #[test]
    fn apply_disarms_eyedropper() {
        let mut t = BgRemovalTool::default();
        t.set_eyedropper_armed(true);
        t.apply_ui_edit(BgRemovalUiEdit::Apply);
        assert!(!t.is_eyedropper_armed());
    }

    #[test]
    fn on_deactivate_disarms_eyedropper() {
        let mut t = BgRemovalTool::default();
        t.set_eyedropper_armed(true);
        Tool::on_deactivate(&mut t);
        assert!(!t.is_eyedropper_armed());
    }
}
