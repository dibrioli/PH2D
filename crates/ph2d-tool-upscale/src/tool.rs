//! [`UpscaleTool`] — stateful editor Tool for resolution upscaling.
//!
//! Model: live [`UpscaleParams`] + cached source snapshot + preview
//! thumbnail + cached preview output + one-shot `pending_apply` flag.
//! Mirrors `ph2d-tool-bgremoval` (sabor 3) without the protect-brush /
//! eyedropper extras.
//!
//! ## Apply flow
//!
//! 1. The shell pushes a fresh source snapshot whenever the active
//!    sprite changes via [`Self::set_source_snapshot`]; the thumbnail
//!    + preview rebuild automatically.
//! 2. Every panel event reaches the tool via
//!    [`Self::handle_panel_event`], routes through
//!    [`Self::apply_ui_edit`] (the single clamping site), and re-runs
//!    the preview against the cached thumbnail.
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

/// Side length (px) of the square thumbnail used for the panel
/// preview. Matches the BgRemoval thumbnail.
pub const THUMB_SIZE: u32 = 160;

/// Maximum side length (px) the live on-canvas preview will resample
/// to before running the upscale algorithm. Above this cap a slider
/// drag would re-run a multi-megabyte kernel every frame; the on-
/// canvas overlay is drawn scaled to the sprite footprint anyway, so
/// re-upscaling a downscaled copy is visually equivalent at slider-
/// drag time. Apply still bakes at full source resolution via
/// [`UpscaleTool::run_full_resolution`].
pub const PREVIEW_MAX_DIM: u32 = 512;

// NodeId range for the Upscale docked panel + preview thumbnail.
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

    /// Pre-scaled thumbnail derived from `source_rgba`. Aspect-fit into
    /// a `THUMB_SIZE × THUMB_SIZE` letterbox so the panel can paint a
    /// fixed-size preview regardless of sprite aspect. RGBA8.
    thumbnail_rgba: Vec<u8>,
    thumbnail_w: u32,
    thumbnail_h: u32,

    /// Preview output: result of the active algorithm on the thumbnail
    /// with the current params, capped at a sensible size. RGBA8.
    preview_rgba: Vec<u8>,
    preview_w: u32,
    preview_h: u32,

    /// Source RGBA downscaled to fit [`PREVIEW_MAX_DIM`] (aspect-fit,
    /// no letterbox). Input of the on-canvas live preview run.
    /// Rebuilt only when the source snapshot changes, so a slider drag
    /// re-upscales this smaller image instead of the full-res source.
    canvas_src_rgba: Vec<u8>,
    canvas_src_w: u32,
    canvas_src_h: u32,

    /// Set to `true` when the user activates Apply. Host polls via
    /// [`Self::take_pending_apply`] each frame; on `true` it runs the
    /// pipeline at full resolution against the active sprite and
    /// writes back a new Individual texture.
    pending_apply: bool,
    /// Set `true` by Reset / on_activate. Shell drains via
    /// `take_pending_panel_reset` and re-runs `Panel::populate` so
    /// the slider knob + chip text snap back to defaults.
    pending_panel_reset: bool,

    /// Set to `true` by any mutator that changes what the shell's
    /// on-canvas preview reflects (algorithm or scale). The shell
    /// polls via [`Self::take_params_dirty`] to gate live-preview
    /// reruns.
    params_dirty: bool,

    /// Cached output of the most-recent on-canvas preview run
    /// (`run_canvas_preview`). `RasterEditTool::current_preview` returns
    /// a slice into this buffer when the tool's dirty flag has been
    /// drained — the shell bridge no longer needs to manage the cache
    /// externally.
    ///
    /// Wave 10 / Etapa 2 (ADR-0041 follow-up): mirrors `BgRemovalTool`'s
    /// `cached_canvas_preview` field. Distinct from `preview_rgba` (which
    /// is the THUMB preview the panel paints in its docked slot); the
    /// canvas overlay needs the LARGER capped preview, hence a separate
    /// cache. Cleared on `set_source` and `deactivate` (audit fixes
    /// A1+A2 from Etapa 1.B).
    cached_canvas_preview: Option<(Vec<u8>, u32, u32)>,
}

impl UpscaleTool {
    /// Push a fresh source RGBA snapshot from the host. Rebuilds the
    /// thumbnail + canvas-preview-source and re-renders the preview
    /// with the current params.
    ///
    /// `rgba` must be straight-alpha RGBA8 of length `w * h * 4`.
    pub fn set_source_snapshot(&mut self, rgba: Vec<u8>, w: u32, h: u32) {
        assert_eq!(rgba.len(), (w as usize) * (h as usize) * 4);
        self.source_rgba = rgba;
        self.source_w = w;
        self.source_h = h;
        self.rebuild_thumbnail();
        self.rebuild_canvas_src();
        self.rerun_preview();
        self.params_dirty = true;
        // Wave 10 / Etapa 2 audit fix [A1] (mirror of BgRemoval Etapa 1.B):
        // explicitly invalidate the canvas-preview cache so a stale frame
        // from the previous selection's source can NEVER paint over the
        // newly pushed pixels.
        self.cached_canvas_preview = None;
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

    /// Borrow the current preview RGBA buffer (size
    /// `preview_w × preview_h × 4`). Empty when no source is loaded.
    pub fn preview_rgba(&self) -> &[u8] {
        &self.preview_rgba
    }

    /// Current preview dimensions (panel paints at these dims, scaled
    /// to the thumb slot).
    pub fn preview_size(&self) -> (u32, u32) {
        (self.preview_w, self.preview_h)
    }

    /// Drain the pending-apply flag. Returns `true` exactly once after
    /// each Apply trigger; the host's drain loop runs the full-res
    /// pipeline when this returns `true`.
    pub fn take_pending_apply(&mut self) -> bool {
        let p = self.pending_apply;
        self.pending_apply = false;
        p
    }

    /// Drain the params-dirty flag. Returns `true` exactly once when
    /// any mutator has run since the last call; the shell uses this
    /// as the gate for rerunning the on-canvas live preview.
    pub fn take_pending_panel_reset(&mut self) -> bool {
        std::mem::take(&mut self.pending_panel_reset)
    }

    pub fn take_params_dirty(&mut self) -> bool {
        std::mem::take(&mut self.params_dirty)
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

    /// Run the active algorithm at the capped on-canvas preview
    /// resolution and write the result into `out`. Returns the output
    /// `(w, h)`.
    ///
    /// Perf: both INPUT and OUTPUT are capped at [`PREVIEW_MAX_DIM`].
    /// `canvas_src_rgba` is the source aspect-fit to that box (built
    /// once per source push); at high scale factors (`16×`) the naive
    /// output would be `8192²` and stall the slider drag, so we
    /// dynamically further downscale the working source so
    /// `working_dim × factor ≤ PREVIEW_MAX_DIM`. The shell's overlay
    /// draws this preview scaled to the sprite's on-canvas footprint,
    /// so a smaller working buffer doesn't change the perceived
    /// preview quality at this factor.
    ///
    /// No-op (returns `(0, 0)`, clears `out`) when no source is loaded.
    pub fn run_canvas_preview(&mut self, out: &mut Vec<u8>) -> (u32, u32) {
        if self.canvas_src_rgba.is_empty() {
            out.clear();
            return (0, 0);
        }
        let factor = self
            .params
            .algorithm
            .project_scale(self.params.scale_factor)
            .max(1.0);
        // Working source cap so the algorithm's output stays under
        // PREVIEW_MAX_DIM: `out_dim = src_dim × factor ≤ MAX` ⇒
        // `src_dim ≤ MAX / factor`. Always ≥ 1 px so a degenerate
        // factor doesn't collapse the buffer.
        let working_max_dim = ((PREVIEW_MAX_DIM as f32) / factor).floor().max(1.0) as u32;
        let (src_w, src_h) = (self.canvas_src_w, self.canvas_src_h);
        if src_w <= working_max_dim && src_h <= working_max_dim {
            // canvas_src already fits — run the algorithm directly.
            let r = self.run_algorithm(&self.canvas_src_rgba, src_w, src_h);
            out.clear();
            out.extend_from_slice(&r.pixels);
            return (r.width, r.height);
        }
        // Downscale canvas_src → working_src for this factor. Cheap
        // `Triangle` filter (the algorithm-under-test is what the user
        // sees; the working buffer's resampler is invisible).
        let (dw, dh) = aspect_fit_within(src_w, src_h, working_max_dim);
        let src = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
            src_w,
            src_h,
            self.canvas_src_rgba.clone(),
        )
        .expect("canvas_src_rgba length matches canvas_src_w * canvas_src_h * 4");
        let resized: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
            image::imageops::resize(&src, dw, dh, image::imageops::FilterType::Triangle);
        let r = self.run_algorithm(resized.as_raw(), dw, dh);
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
    /// Re-runs the preview when the edit actually changes the output.
    /// `Apply` arms the pending-apply flag; the host drains via
    /// [`Self::take_pending_apply`]. Inverse of [`Self::ui_snapshot`].
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
        if self.has_source() {
            self.rerun_preview();
        }
        self.params_dirty = true;
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

    /// Aspect-fit `source_rgba` into a `THUMB_SIZE × THUMB_SIZE` RGBA8
    /// buffer with transparent letterbox borders. Cheap `Triangle`
    /// resample (no ringing; the preview shows the algorithm's
    /// behaviour, not the resampler's).
    fn rebuild_thumbnail(&mut self) {
        if !self.has_source() {
            self.thumbnail_w = 0;
            self.thumbnail_h = 0;
            self.thumbnail_rgba.clear();
            return;
        }
        let target = THUMB_SIZE;
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

    /// Rebuild [`Self::canvas_src_rgba`] — the source downscaled to fit
    /// [`PREVIEW_MAX_DIM`] (aspect-fit, no letterbox).
    fn rebuild_canvas_src(&mut self) {
        if !self.has_source() {
            self.canvas_src_w = 0;
            self.canvas_src_h = 0;
            self.canvas_src_rgba.clear();
            return;
        }
        let (dw, dh) = aspect_fit_within(self.source_w, self.source_h, PREVIEW_MAX_DIM);
        self.canvas_src_rgba.clear();
        if dw == self.source_w && dh == self.source_h {
            self.canvas_src_rgba.extend_from_slice(&self.source_rgba);
        } else {
            let src = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
                self.source_w,
                self.source_h,
                self.source_rgba.clone(),
            )
            .expect("source_rgba length matches source_w * source_h * 4");
            let resized: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
                image::imageops::resize(&src, dw, dh, image::imageops::FilterType::Triangle);
            self.canvas_src_rgba.extend_from_slice(resized.as_raw());
        }
        self.canvas_src_w = dw;
        self.canvas_src_h = dh;
    }

    /// Re-run the active algorithm on the cached thumbnail with the
    /// current params; result lands in `preview_rgba`. No-op when the
    /// thumbnail hasn't been built yet.
    fn rerun_preview(&mut self) {
        if self.thumbnail_rgba.is_empty() {
            self.preview_rgba.clear();
            self.preview_w = 0;
            self.preview_h = 0;
            return;
        }
        let r = self.run_algorithm(&self.thumbnail_rgba, self.thumbnail_w, self.thumbnail_h);
        self.preview_rgba = r.pixels;
        self.preview_w = r.width;
        self.preview_h = r.height;
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
        // Clear one-shot drain flags so a Cancel-mid-Apply (or any
        // deactivation while the bridge hasn't yet drained) does not
        // fire a phantom bake nor a spurious canvas-preview rerun on
        // the next activation.
        self.pending_apply = false;
        self.params_dirty = false;
        // Wave 10 / Etapa 2 audit fix [A2] (mirror of BgRemoval Etapa 1.B):
        // align the invariant "deactivate clears canvas-preview cache"
        // across BOTH paths — Tool::on_deactivate (registry switch) and
        // RasterEditTool::deactivate (tool-runtime drive_deactivate_cleanup).
        self.cached_canvas_preview = None;
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
/// Mapping:
/// - `set_source` → wraps `set_source_snapshot` (which internally
///   triggers `rerun_preview` to refresh `preview_rgba`).
/// - `current_preview` → drains `params_dirty`; if dirty + has source +
///   non-empty cache, returns a ref into the already-populated
///   `preview_rgba` field. Upscale eager-recomputes on every
///   `apply_ui_edit` via `rerun_preview`, so no extra `run_canvas_preview`
///   call is needed in the dirty path.
/// - `take_pending_commit` → wraps `take_pending_apply`.
/// - `run_full` → wraps `run_full_resolution(&mut Vec)`.
/// - `deactivate` → mirrors `Tool::on_deactivate` (drain pending_apply
///   + params_dirty).
impl RasterEditTool for UpscaleTool {
    fn set_source(&mut self, rgba: Vec<u8>, width: u32, height: u32) {
        self.set_source_snapshot(rgba, width, height);
    }

    fn current_preview(&mut self) -> Option<(&[u8], u32, u32)> {
        if !self.take_params_dirty() {
            return None;
        }
        if !self.has_source() || self.canvas_src_rgba.is_empty() {
            return None;
        }
        let mut out = Vec::new();
        let (w, h) = self.run_canvas_preview(&mut out);
        if w == 0 || h == 0 {
            return None;
        }
        self.cached_canvas_preview = Some((out, w, h));
        self.cached_canvas_preview
            .as_ref()
            .map(|(p, w, h)| (p.as_slice(), *w, *h))
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
        // Mirror Tool::on_deactivate so the generic shell-runtime
        // deactivate path is equivalent to the registry-switch path.
        self.pending_apply = false;
        self.params_dirty = false;
        self.cached_canvas_preview = None;
    }
}

/// Aspect-fit into a `target × target` square: the longer side maps to
/// `target`, the shorter side scales proportionally. Outputs clamped
/// to ≥ 1.
fn aspect_fit(sw: u32, sh: u32, target: u32) -> (u32, u32) {
    if sw == 0 || sh == 0 || target == 0 {
        return (target.max(1), target.max(1));
    }
    if sw >= sh {
        let scaled_h = ((sh as u64 * target as u64) / sw as u64) as u32;
        (target, scaled_h.max(1))
    } else {
        let scaled_w = ((sw as u64 * target as u64) / sh as u64) as u32;
        (scaled_w.max(1), target)
    }
}

/// Aspect-fit into a `max_dim × max_dim` box — like [`aspect_fit`] but
/// never upscales (an input already within the box passes through
/// unchanged).
fn aspect_fit_within(sw: u32, sh: u32, max_dim: u32) -> (u32, u32) {
    if sw == 0 || sh == 0 || max_dim == 0 {
        return (sw.max(1), sh.max(1));
    }
    if sw <= max_dim && sh <= max_dim {
        return (sw, sh);
    }
    if sw >= sh {
        let dh = ((sh as u64 * max_dim as u64) / sw as u64).max(1) as u32;
        (max_dim, dh)
    } else {
        let dw = ((sw as u64 * max_dim as u64) / sh as u64).max(1) as u32;
        (dw, max_dim)
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
        assert!(t.preview_rgba().is_empty());
        assert!(!t.take_pending_apply());
        assert!(!t.take_params_dirty());
    }

    #[test]
    fn set_source_snapshot_marks_has_source_and_builds_preview() {
        let mut t = UpscaleTool::default();
        let buf = vec![128u8; 8 * 8 * 4];
        t.set_source_snapshot(buf, 8, 8);
        assert!(t.has_source());
        // Preview is non-empty (default Lanczos3 @ 2× of a 160² thumb).
        assert!(!t.preview_rgba().is_empty());
        let (pw, ph) = t.preview_size();
        assert!(pw > 0 && ph > 0);
        // Params-dirty fires once on snapshot push.
        assert!(t.take_params_dirty());
        assert!(!t.take_params_dirty());
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
    fn algorithm_edit_persists_and_marks_dirty() {
        let mut t = UpscaleTool::default();
        assert_eq!(t.params.algorithm, UpscaleAlgorithm::Lanczos3);
        t.apply_ui_edit(UpscaleUiEdit::SetAlgorithm(UpscaleAlgorithm::Xbr));
        assert_eq!(t.params.algorithm, UpscaleAlgorithm::Xbr);
        assert!(t.take_params_dirty());
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
        assert!(!t.take_params_dirty());
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
    fn run_canvas_preview_caps_output_to_preview_max() {
        let mut t = UpscaleTool::default();
        // Build a source bigger than PREVIEW_MAX_DIM so the canvas-src
        // path actually downscales.
        let big = (PREVIEW_MAX_DIM + 50) as usize;
        let buf = vec![32u8; big * big * 4];
        t.set_source_snapshot(buf, big as u32, big as u32);
        // Both input AND output are capped at PREVIEW_MAX_DIM: at
        // factor 2 the working src downscales to MAX/2, output = MAX.
        t.apply_ui_edit(UpscaleUiEdit::Scale(2.0));
        let mut out = Vec::new();
        let (w, h) = t.run_canvas_preview(&mut out);
        assert_eq!((w, h), (PREVIEW_MAX_DIM, PREVIEW_MAX_DIM));

        // And at factor 16 the cap still holds — without the cap the
        // naive output would be `8192²` and stall the slider drag.
        t.apply_ui_edit(UpscaleUiEdit::Scale(16.0));
        let (w, h) = t.run_canvas_preview(&mut out);
        assert!(
            w <= PREVIEW_MAX_DIM && h <= PREVIEW_MAX_DIM,
            "canvas preview must cap output at PREVIEW_MAX_DIM (got {w}×{h})"
        );
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

    #[test]
    fn aspect_fit_landscape_and_portrait() {
        assert_eq!(aspect_fit(4, 2, 16), (16, 8));
        assert_eq!(aspect_fit(2, 4, 16), (8, 16));
        assert_eq!(aspect_fit(0, 0, 16), (16, 16));
    }

    #[test]
    fn aspect_fit_within_does_not_upscale_small_inputs() {
        // 100 × 50 with cap 200 → unchanged.
        assert_eq!(aspect_fit_within(100, 50, 200), (100, 50));
        // 400 × 200 with cap 200 → 200 × 100.
        assert_eq!(aspect_fit_within(400, 200, 200), (200, 100));
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
    fn raster_edit_current_preview_drains_dirty() {
        let mut t = UpscaleTool::default();
        RasterEditTool::set_source(&mut t, solid_rgba(4, 4, [128, 128, 128]), 4, 4);
        let frame = RasterEditTool::current_preview(&mut t);
        assert!(
            frame.is_some(),
            "first call after set_source must return Some"
        );
        let (pixels, w, h) = frame.unwrap();
        assert!(pixels.len() >= 4 && w > 0 && h > 0);
        let frame2 = RasterEditTool::current_preview(&mut t);
        assert!(frame2.is_none(), "dirty drained — no new frame");
    }

    #[test]
    fn raster_edit_current_preview_none_without_source() {
        // simulate dirty without source
        let mut t = UpscaleTool {
            params_dirty: true,
            ..UpscaleTool::default()
        };
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
    fn raster_edit_deactivate_clears_transient_flags() {
        let mut t = UpscaleTool::default();
        t.apply_ui_edit(UpscaleUiEdit::Apply);
        RasterEditTool::deactivate(&mut t);
        assert!(!t.take_pending_apply());
        assert!(!t.take_params_dirty());
    }
}
