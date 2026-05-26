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
//!   and swaps the texture per the Image Tools precedent (`ph2d-tool-trim-transparency`).
//!
//! All pointer / hover / canvas interaction is **out of scope** —
//! the tool reacts only to its panel widgets, never to the canvas
//! (consistent with the §5.5 ENTREGÁVEL contract).
//!
//! ## Apply trigger mechanism
//!
//! The Widget Gallery's `ph2d_editor_core::floating_panel::PanelControl::Action`
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

use ph2d_a11y::NodeId;
use ph2d_editor_core::floating_panel::{
    FloatingPanel, PanelAnchor, PanelControl, PanelTab, ToolId,
};
use ph2d_editor_core::ids;
use ph2d_editor_core::tool::{PanelEvent, RasterEditTool, Tool};
use ph2d_editor_core::widget::{Slider, Toggle};

use super::algorithm::islands::{self, IslandPayload};
use super::algorithm::run_pipeline;
use super::params::{
    BRUSH_SIZE_FULL_SCALE, BgRemovalParams, BgRemovalUiEdit, BgRemovalUiSnapshot, BrushFalloff,
    DEFAULT_BRUSH_SIZE01, FEATHER_FULL_SCALE, GROW_FULL_SCALE, MAX_EXTRA_BG_COLORS,
    MIN_ISLAND_PIXELS_FULL_SCALE, REFINE_RADIUS_FULL_SCALE, TOLERANCE_FULL_SCALE,
};
use super::scratch::BgRemovalScratch;

/// Side length (px) of the square thumbnail used for the panel preview.
pub const THUMB_SIZE: u32 = 160;

/// Side cap (px) for the live on-canvas preview overlay. The overlay
/// re-runs the whole pipeline on every parameter change; doing that at
/// full source resolution makes each slider tick janky. The overlay is
/// drawn *scaled* to the sprite footprint anyway, so it re-segments a
/// copy of the source downscaled to fit this box (aspect preserved, no
/// letterbox) instead — keeping slider drags smooth. Apply still bakes
/// at full source resolution via [`BgRemovalTool::run_full_resolution`].
pub const PREVIEW_MAX_DIM: u32 = 512;

// NodeId range 500..599 reserved for bgremoval panel controls
// (clear of 100..199 brush/move and 1000..1099 grid_snap).
const TOLERANCE_NODE: NodeId = NodeId(504);
const FEATHER_NODE: NodeId = NodeId(505);
const REFINE_NODE: NodeId = NodeId(506);
const APPLY_NODE: NodeId = NodeId(507);

/// Editor Tool implementing the background-removal feature.
///
/// `Default` is hand-written (not derived) because the protection-brush
/// state has non-zero defaults: the brush starts at a usable size and the
/// painted mask is shown by default.
#[derive(Clone, Debug)]
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

    /// Set to `true` by any mutator that touches state the shell's
    /// on-canvas preview reflects (params, extra-bg colours, painted
    /// protection mask). Host polls via [`Self::take_params_dirty`]
    /// each frame as the gate for rerunning `run_canvas_preview` —
    /// replaces the old `!bgremoval_ui_edits.is_empty()` check the
    /// shell used before ADR-0040 TG-B routed panel events through
    /// `handle_panel_event` directly.
    params_dirty: bool,

    /// `true` when the params were just reset to defaults (Reset
    /// button OR `on_activate`). The shell bridge drains via
    /// [`Self::take_pending_panel_reset`] and re-runs
    /// `Panel::populate(store)` so the slider knob / chip text
    /// positions snap back to defaults — without this, only the
    /// params struct resets while the WidgetStore retains whatever
    /// drag position the user last left.
    pending_panel_reset: bool,

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

    /// Whether a protection-brush dab-drag is in progress (set on
    /// pointer-down, cleared on pointer-up by the shell). Transient
    /// pointer state — it lives here rather than on the shell's `App`
    /// because the protection feature does not edit the `App` struct, and
    /// the tool is the natural per-tool home for the flag. Distinct from
    /// `protect_brush_armed` (whether the brush is *selected* at all).
    protect_painting: bool,

    /// Freehand protection mask at the SOURCE resolution
    /// (`protect_mask_w × protect_mask_h`, one byte/pixel; `255` =
    /// protected/forced-foreground, `0` = unprotected). Empty until the
    /// user paints. Threaded into `run_pipeline` as the compose
    /// force-keep mask (a painted region stays opaque).
    protect_mask: Vec<u8>,
    protect_mask_w: u32,
    protect_mask_h: u32,

    /// Source RGBA downscaled to fit [`PREVIEW_MAX_DIM`] (aspect kept,
    /// no letterbox) — the input of the on-canvas live preview. Rebuilt
    /// only when the source snapshot changes, so a slider drag
    /// re-segments this small image instead of the full-res source.
    /// Empty until the host pushes a source.
    canvas_src_rgba: Vec<u8>,
    canvas_src_w: u32,
    canvas_src_h: u32,
    /// Protection mask nearest-resampled to the canvas-preview dims.
    /// Re-filled each canvas-preview run; kept as a field so the
    /// allocation persists across runs (HR-3).
    canvas_protect: Vec<u8>,

    /// Protection-brush radius in SOURCE pixels (what `paint_protect_at_uv`
    /// consumes). Driven by the panel Brush Size slider; default ≈
    /// [`DEFAULT_BRUSH_SIZE01`] × [`BRUSH_SIZE_FULL_SCALE`].
    brush_radius: f32,
    /// Protection-brush dab falloff profile (Smooth / Sphere / Sharp /
    /// Hard) — applied to both paint and erase.
    falloff: BrushFalloff,
    /// Current drag is an ERASE drag (set by the shell on a secondary-
    /// button down). Transient, like `protect_painting`.
    protect_erase_mode: bool,
    /// Whether the painted protection mask is drawn as an on-canvas tint
    /// overlay (the shell gates the overlay on this). Default `true`.
    show_mask: bool,

    /// Per-island RGBA payloads stashed by `run_full_resolution` when
    /// `params.separate_islands` is on. The shell drains them via
    /// [`Self::take_pending_islands`] right after baking the main result
    /// (or alongside it) and spawns one new sprite per entry. Empty when
    /// the toggle is off, when an Apply hasn't run yet, or after the
    /// host has drained.
    pending_islands: Vec<IslandPayload>,

    /// Cached output of the most-recent `run_canvas_preview` invocation.
    /// `RasterEditTool::current_preview` returns a slice into this buffer
    /// when the tool's dirty flag has been drained — so the bridge
    /// doesn't need to maintain its own per-tool preview cache outside.
    ///
    /// Wave 10 / Etapa 1.B (ADR-0041 follow-up): the cache moved from
    /// `shells/desktop/src/app_state.rs::BgremovalPreview` to live inside
    /// the tool — this is what lets `ph2d-tool-runtime::drive_preview_cache`
    /// stay generic.
    cached_canvas_preview: Option<(Vec<u8>, u32, u32)>,

    /// Last (u, v) painted in the current stroke — anchor used by
    /// `stamp_protect` to interpolate intermediate dabs between the
    /// previous cursor position and the current one. Without this, a
    /// fast drag produces visibly spaced discs along the path (Enio
    /// 2026-05-26: "máscara apresenta pintura não regular, como se o
    /// espaço entre os pontos de pintura fossem muito grandes").
    /// `None` outside an active stroke; reset by
    /// [`Self::set_protect_painting`] on pointer-up.
    last_protect_uv: Option<(f32, f32)>,
}

impl Default for BgRemovalTool {
    fn default() -> Self {
        Self {
            params: BgRemovalParams::default(),
            source_rgba: Vec::new(),
            source_w: 0,
            source_h: 0,
            thumbnail_rgba: Vec::new(),
            thumbnail_w: 0,
            thumbnail_h: 0,
            preview_rgba: Vec::new(),
            scratch: BgRemovalScratch::default(),
            pending_apply: false,
            params_dirty: false,
            pending_panel_reset: false,
            eyedropper_armed: false,
            protect_brush_armed: false,
            protect_painting: false,
            protect_mask: Vec::new(),
            protect_mask_w: 0,
            protect_mask_h: 0,
            canvas_src_rgba: Vec::new(),
            canvas_src_w: 0,
            canvas_src_h: 0,
            canvas_protect: Vec::new(),
            brush_radius: DEFAULT_BRUSH_SIZE01 * BRUSH_SIZE_FULL_SCALE,
            falloff: BrushFalloff::default(),
            protect_erase_mode: false,
            show_mask: true,
            pending_islands: Vec::new(),
            cached_canvas_preview: None,
            last_protect_uv: None,
        }
    }
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
    /// `pixels` must be straight-alpha `SrgbRgba` of length `w * h`.
    /// Internally re-stored as `Vec<u8>` (downstream consumes bytes);
    /// the cast is zero-copy via `bytemuck::cast_vec`.
    pub fn set_source_snapshot(&mut self, pixels: Vec<ph2d_color::SrgbRgba>, w: u32, h: u32) {
        assert_eq!(pixels.len(), (w as usize) * (h as usize));
        // The protection mask is spatial — a genuinely different image
        // invalidates it. Re-feeding the SAME dimensions (e.g. the Apply
        // re-read of the same sprite) preserves it so the bake honours
        // the painted region.
        if w != self.protect_mask_w || h != self.protect_mask_h {
            self.protect_mask.clear();
            self.protect_mask_w = 0;
            self.protect_mask_h = 0;
        }
        self.source_rgba = bytemuck::allocation::cast_vec(pixels);
        self.source_w = w;
        self.source_h = h;
        self.rebuild_thumbnail();
        self.rebuild_canvas_src();
        self.rerun_preview();
        // The cached on-canvas preview the shell holds was computed against
        // the previous selection's source — mark dirty so the next bridge
        // tick rebuilds it. (Shell also drops `bgremoval_preview` when
        // `last_bgremoval_pushed_entity` changes, so this is a belt-and-
        // suspenders for the path where the snapshot push fires for some
        // other reason but the cached preview is now stale.)
        self.params_dirty = true;
        // Wave 10 / Etapa 1.B audit fix [A1]: explicitly invalidate the
        // tool's internal canvas-preview cache so a stale frame from the
        // previous selection can NEVER paint over the new sprite, even
        // in the path where the shell read failed mid-frame and never
        // refilled its own cache. Without this, the next current_preview
        // call would short-circuit if dirty was somehow consumed first.
        self.cached_canvas_preview = None;
    }

    /// Whether the host has pushed a source snapshot at least once.
    pub fn has_source(&self) -> bool {
        !self.source_rgba.is_empty()
    }

    /// Source texture resolution `(w, h)` of the active snapshot, or
    /// `(0, 0)` before any source is pushed. The shell uses this to map
    /// an on-screen protection-brush radius into source pixels (the unit
    /// [`Self::paint_protect_at_uv`] expects) on the very first dab,
    /// before the protection mask itself is sized.
    pub fn source_size(&self) -> (u32, u32) {
        (self.source_w, self.source_h)
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
    /// Drain the per-island RGBA payloads produced by the last Apply
    /// when `params.separate_islands` was on. Returns an empty Vec when
    /// the toggle is off, no Apply has run yet, or the host already
    /// drained. The shell typically calls this right after baking the
    /// main result and spawns one new sprite per returned payload
    /// (legacy parity — biggest island stays in the original sprite,
    /// rest get sibling sprites positioned at their bounding-box origins).
    pub fn take_pending_islands(&mut self) -> Vec<IslandPayload> {
        std::mem::take(&mut self.pending_islands)
    }

    pub fn take_pending_apply(&mut self) -> bool {
        let p = self.pending_apply;
        self.pending_apply = false;
        p
    }

    /// Drain the params-dirty flag. Returns `true` exactly once when
    /// any panel-edit / extra-colour / protect-mask mutator has run
    /// since the last call. The shell uses this as the gate for
    /// rerunning the on-canvas live preview (ADR-0040 TG-B replacement
    /// for the old `!bgremoval_ui_edits.is_empty()` check).
    pub fn take_pending_panel_reset(&mut self) -> bool {
        std::mem::take(&mut self.pending_panel_reset)
    }

    pub fn take_params_dirty(&mut self) -> bool {
        std::mem::take(&mut self.params_dirty)
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

    /// Whether a protection-brush dab-drag is currently in progress
    /// (shell paints on every cursor-move while `true`).
    pub fn is_protect_painting(&self) -> bool {
        self.protect_painting
    }

    /// Set the protection-brush dab-drag state (shell sets `true` on
    /// pointer-down over the sprite, `false` on pointer-up). Clearing
    /// the flag also drops `last_protect_uv` so the next stroke
    /// doesn't draw an interpolated line from the previous stroke's
    /// final dab to the new starting position.
    pub fn set_protect_painting(&mut self, painting: bool) {
        if !painting {
            self.last_protect_uv = None;
        }
        self.protect_painting = painting;
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
    /// Lazy-sizes `protect_mask` to the source dims and stamps a brush dab
    /// at UV `(u, v)` with `radius_px` (SOURCE px). The dab strength
    /// follows the active [`BrushFalloff`] over the normalized distance
    /// `d = dist/radius`, accumulating with `max` so overlapping dabs
    /// build up to full protection.
    ///
    /// Does NOT re-run the pipeline — painting only mutates the mask. The
    /// on-canvas tint overlay reads the mask live each frame (cheap); the
    /// matte re-segments once the stroke ends (the shell drops its cached
    /// preview on pointer-up) — so painting stays cheap (no per-dab
    /// re-segmentation).
    pub fn paint_protect_at_uv(&mut self, u: f32, v: f32, radius_px: f32) {
        self.stamp_protect(u, v, radius_px, false);
    }

    /// Erase from the protection mask at UV `(u, v)` — the inverse of
    /// [`Self::paint_protect_at_uv`]: subtracts the falloff strength
    /// (`saturating_sub`) so the centre erases fully and the rim only
    /// nibbles. No pipeline re-run (same rationale as paint).
    pub fn erase_protect_at_uv(&mut self, u: f32, v: f32, radius_px: f32) {
        self.stamp_protect(u, v, radius_px, true);
    }

    /// Shared dab kernel for paint (`erase = false`) / erase (`erase =
    /// true`). Lazy-sizes the mask, then walks the segment from the
    /// previous stroke anchor to `(u, v)` placing intermediate dabs
    /// every `STAMP_SPACING_FRAC * radius` SOURCE pixels. Without the
    /// interpolation a fast drag draws discrete discs along the path
    /// (Enio 2026-05-26: "espaço entre os pontos de pintura fossem
    /// muito grandes"); 4 dabs per radius (0.25 spacing) gives a
    /// continuous painterly trail with cheap accumulation.
    fn stamp_protect(&mut self, u: f32, v: f32, radius_px: f32, erase: bool) {
        if !self.has_source() {
            return;
        }
        let (w, h) = (self.source_w, self.source_h);
        let n = (w as usize) * (h as usize);
        // Erase on an unsized mask is a no-op (nothing to remove).
        if erase && self.protect_mask.len() != n {
            return;
        }
        // Lazy-size to the source resolution on first paint dab.
        if self.protect_mask.len() != n {
            self.protect_mask.clear();
            self.protect_mask.resize(n, 0);
            self.protect_mask_w = w;
            self.protect_mask_h = h;
        }
        let u = u.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);
        let r = radius_px.max(0.5);

        // Interpolate intermediate dabs between the previous (u, v)
        // and this one so a fast drag draws a continuous trail
        // instead of spaced discs. STAMP_SPACING_FRAC = 0.25 → ≥4
        // dabs per radius of cursor motion (Procreate-style default).
        // First dab of a stroke (no anchor yet) just stamps once.
        const STAMP_SPACING_FRAC: f32 = 0.25;
        let spacing_px = (r * STAMP_SPACING_FRAC).max(0.5);
        if let Some((lu, lv)) = self.last_protect_uv {
            let du_px = (u - lu) * (w as f32 - 1.0);
            let dv_px = (v - lv) * (h as f32 - 1.0);
            let dist_px = (du_px * du_px + dv_px * dv_px).sqrt();
            let n_steps = (dist_px / spacing_px).ceil().max(1.0) as u32;
            for i in 1..=n_steps {
                let t = i as f32 / n_steps as f32;
                let iu = lu + (u - lu) * t;
                let iv = lv + (v - lv) * t;
                self.stamp_single(iu, iv, r, erase);
            }
        } else {
            self.stamp_single(u, v, r, erase);
        }
        self.last_protect_uv = Some((u, v));

        // Protect dab mutates the mask (force-keep region for compose).
        // The matte itself only re-segments on pointer-up (shell drops
        // its cached preview there); but the on-canvas tint overlay
        // reads the mask each frame and the canvas preview gate uses
        // this flag — mark dirty so a follow-up render-loop tick sees
        // the new mask without waiting for an unrelated edit.
        self.params_dirty = true;
    }

    /// Stamp a single brush disc into `protect_mask` at UV `(u, v)`
    /// with `r` SOURCE-px radius. Assumes the mask has already been
    /// lazy-sized by the caller. Falloff strength accumulates via
    /// `max` (paint) or `saturating_sub` (erase).
    fn stamp_single(&mut self, u: f32, v: f32, r: f32, erase: bool) {
        let (w, h) = (self.source_w, self.source_h);
        let cx = u * (w as f32 - 1.0);
        let cy = v * (h as f32 - 1.0);
        let inv_r = 1.0 / r;
        let x0 = (cx - r).floor().max(0.0) as u32;
        let x1 = ((cx + r).ceil() as i64).clamp(0, w as i64 - 1) as u32;
        let y0 = (cy - r).floor().max(0.0) as u32;
        let y1 = ((cy + r).ceil() as i64).clamp(0, h as i64 - 1) as u32;
        let stride = w as usize;
        let falloff = self.falloff;
        for y in y0..=y1 {
            let dy = y as f32 - cy;
            for x in x0..=x1 {
                let dx = x as f32 - cx;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist > r {
                    continue;
                }
                let s = falloff.strength(dist * inv_r);
                let val = (s * 255.0 + 0.5) as u8;
                let i = (y as usize) * stride + x as usize;
                self.protect_mask[i] = if erase {
                    self.protect_mask[i].saturating_sub(val)
                } else {
                    self.protect_mask[i].max(val)
                };
            }
        }
    }

    /// Protection-brush radius in SOURCE pixels (the unit the shell passes
    /// to [`Self::paint_protect_at_uv`] and converts to a screen-space
    /// ring). Always ≥ a usable minimum.
    pub fn brush_radius_px(&self) -> f32 {
        self.brush_radius.max(0.5)
    }

    /// Active protection-brush falloff profile.
    pub fn falloff(&self) -> BrushFalloff {
        self.falloff
    }

    /// Whether the painted protection mask should be shown as an on-canvas
    /// tint overlay (shell gates its overlay on this).
    pub fn show_mask(&self) -> bool {
        self.show_mask
    }

    /// Whether the in-progress protection drag is an erase drag.
    pub fn is_protect_erasing(&self) -> bool {
        self.protect_erase_mode
    }

    /// Set the erase-drag flag (shell sets `true` on a secondary-button
    /// protection drag, `false` on a primary paint drag / drag-end).
    pub fn set_protect_erasing(&mut self, erasing: bool) {
        self.protect_erase_mode = erasing;
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
        // Canvas-preview cache must rebuild (the matte just dropped the
        // forced-keep region).
        self.params_dirty = true;
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
        // Eyedropper sampling mutates params.extra_bg_colors → canvas
        // preview must rebuild (previously this site did NOT invalidate
        // the shell-side cache because eyedropper dabs bypassed the bus;
        // refreshing it eagerly here closes a 1-frame staleness gap).
        self.params_dirty = true;
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
        // Same rationale as `add_extra_color`.
        self.params_dirty = true;
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
        // The protection mask is stored at source resolution, so it
        // aligns 1:1 with the full-res pipeline input.
        let protect: Option<&[u8]> = if self.protect_mask.len()
            == (self.source_w as usize) * (self.source_h as usize)
            && !self.protect_mask.is_empty()
        {
            Some(self.protect_mask.as_slice())
        } else {
            None
        };
        run_pipeline(
            &self.source_rgba,
            self.source_w,
            self.source_h,
            &self.params,
            protect,
            &mut self.scratch,
        );
        out.clear();
        out.extend_from_slice(&self.scratch.output_rgba);

        // Legacy parity: when "Separate Islands" is on, run CCL on the
        // freshly composed RGBA and stash one payload per surviving
        // component (filtered by `min_island_pixels`). The host drains
        // via `take_pending_islands` and spawns the rest as sibling
        // sprites — keeping the biggest one in the original. When the
        // toggle is off, ensure the slot is empty so a stale post-Apply
        // queue from a previous run doesn't leak.
        //
        // We read from `out` (just copied above) rather than
        // `self.scratch.output_rgba` so `&mut self.scratch` (for the
        // CCL label + queue buffers) doesn't clash with the source
        // borrow inside the same call.
        if self.params.separate_islands {
            islands::extract(
                out,
                self.source_w,
                self.source_h,
                self.params.min_island_pixels.max(1),
                &mut self.scratch,
                &mut self.pending_islands,
            );
        } else {
            self.pending_islands.clear();
        }

        (self.source_w, self.source_h)
    }

    /// Run the pipeline for the live on-canvas preview at a capped
    /// resolution (see [`PREVIEW_MAX_DIM`]) and write the result into
    /// `out`. The shell draws this scaled to the sprite footprint, so a
    /// slider drag re-segments a small image — keeping the drag smooth.
    /// Returns the `(w, h)` of the output (the capped preview dims, NOT
    /// the source dims).
    ///
    /// No-op (returns `(0, 0)`, clears `out`) when no source is loaded.
    pub fn run_canvas_preview(&mut self, out: &mut Vec<u8>) -> (u32, u32) {
        if self.canvas_src_rgba.is_empty() {
            out.clear();
            return (0, 0);
        }
        let (cw, ch) = (self.canvas_src_w, self.canvas_src_h);
        let protect: Option<&[u8]> = if self.protect_mask.is_empty() {
            None
        } else {
            self.resize_protect_into(cw, ch);
            Some(self.canvas_protect.as_slice())
        };
        run_pipeline(
            &self.canvas_src_rgba,
            cw,
            ch,
            &self.params,
            protect,
            &mut self.scratch,
        );
        out.clear();
        out.extend_from_slice(&self.scratch.output_rgba);
        (cw, ch)
    }

    /// Nearest-resample the source-resolution protection mask into
    /// `self.canvas_protect` at `(dw, dh)`. Reuses the allocation.
    fn resize_protect_into(&mut self, dw: u32, dh: u32) {
        let n = (dw as usize) * (dh as usize);
        self.canvas_protect.clear();
        self.canvas_protect.resize(n, 0);
        let (sw, sh) = (self.protect_mask_w, self.protect_mask_h);
        if self.protect_mask.is_empty() || sw == 0 || sh == 0 || dw == 0 || dh == 0 {
            return;
        }
        let (sw_u, sh_u) = (sw as u64, sh as u64);
        let (dw_u, dh_u) = (dw as u64, dh as u64);
        for y in 0..dh as usize {
            let sy = (((y as u64) * sh_u) / dh_u).min(sh_u - 1) as usize;
            for x in 0..dw as usize {
                let sx = (((x as u64) * sw_u) / dw_u).min(sw_u - 1) as usize;
                self.canvas_protect[y * dw as usize + x] = self.protect_mask[sy * sw as usize + sx];
            }
        }
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
        // The 160² thumbnail preview is letterboxed; threading the
        // protection mask through it would need the same letterbox
        // remap. It is not the user-facing preview (the on-canvas
        // overlay is), so it runs without protection.
        run_pipeline(
            &self.thumbnail_rgba,
            self.thumbnail_w,
            self.thumbnail_h,
            &self.params,
            None,
            &mut self.scratch,
        );
        self.preview_rgba.clear();
        self.preview_rgba
            .extend_from_slice(&self.scratch.output_rgba);
    }

    /// Rebuild [`Self::canvas_src_rgba`] — the source downscaled to fit
    /// [`PREVIEW_MAX_DIM`] (aspect preserved, no letterbox). Called once
    /// per source snapshot; the on-canvas preview re-segments this small
    /// buffer on every parameter change. No-op without a source.
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

    /// Project the current full-scale params into the normalized
    /// snapshot the typed `ph2d-panel-bgremoval` paints. Published by
    /// the host once per frame while the tool is active (forward of
    /// [`Self::apply_ui_edit`]).
    pub fn ui_snapshot(&self) -> BgRemovalUiSnapshot {
        BgRemovalUiSnapshot {
            tolerance01: (self.params.chroma.tolerance / TOLERANCE_FULL_SCALE).clamp(0.0, 1.0),
            feather01: (self.params.chroma.feather / FEATHER_FULL_SCALE).clamp(0.0, 1.0),
            refine01: (self.params.refinement.radius as f32 / REFINE_RADIUS_FULL_SCALE)
                .clamp(0.0, 1.0),
            grow01: (self.params.grow_px / (2.0 * GROW_FULL_SCALE) + 0.5).clamp(0.0, 1.0),
            extra_colors: self.params.extra_bg_colors.clone(),
            eyedropper_armed: self.eyedropper_armed,
            protect_brush_armed: self.protect_brush_armed,
            has_protect_mask: self.has_protect_mask(),
            brush_size01: (self.brush_radius / BRUSH_SIZE_FULL_SCALE).clamp(0.0, 1.0),
            falloff: self.falloff,
            show_mask: self.show_mask,
            separate_islands: self.params.separate_islands,
            min_island_pixels01: ((self.params.min_island_pixels.saturating_sub(1)) as f32
                / (MIN_ISLAND_PIXELS_FULL_SCALE - 1.0))
                .clamp(0.0, 1.0),
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
            BgRemovalUiEdit::BrushSize(v) => {
                // Brush radius only affects future dabs — no matte rerun.
                self.brush_radius = v.clamp(0.0, 1.0) * BRUSH_SIZE_FULL_SCALE;
            }
            BgRemovalUiEdit::SetFalloff(f) => {
                // Falloff only affects future dabs — no matte rerun.
                self.falloff = f;
            }
            BgRemovalUiEdit::ToggleShowMask => {
                // Overlay-visibility only — no params, no matte rerun.
                self.show_mask = !self.show_mask;
            }
            BgRemovalUiEdit::ToggleSeparateIslands => {
                // Post-process gate only — the matte itself is unchanged
                // by this toggle, so no preview rerun. Effect lands at
                // Apply time (run_full_resolution runs CCL on the baked
                // output when the flag is on).
                self.params.separate_islands = !self.params.separate_islands;
            }
            BgRemovalUiEdit::SetMinIslandPixels(v) => {
                // Linear normalized → integer pixel count in
                // [1, MIN_ISLAND_PIXELS_FULL_SCALE]. Like ToggleSeparateIslands,
                // this only matters at Apply time — no preview rerun.
                let v = v.clamp(0.0, 1.0);
                let scaled = (v * (MIN_ISLAND_PIXELS_FULL_SCALE - 1.0)).round() as u32 + 1;
                self.params.min_island_pixels =
                    scaled.clamp(1, MIN_ISLAND_PIXELS_FULL_SCALE as u32);
            }
            BgRemovalUiEdit::Apply => {
                self.pending_apply = true;
                // A commit ends the picking session — disarm so a stale
                // eyedropper / protect brush doesn't keep eating canvas
                // clicks on the freshly baked sprite.
                self.eyedropper_armed = false;
                self.protect_brush_armed = false;
                self.protect_painting = false;
                self.protect_erase_mode = false;
            }
            BgRemovalUiEdit::ResetAll => {
                // Snap params back to defaults AND wipe the painted
                // protection mask + extra-bg picks (those are part of
                // the per-session edit, not durable state). Disarm
                // every armed gesture so the next interaction starts
                // clean. The `on_activate` hook calls this too, so a
                // reopened panel always starts from a known clean
                // slate.
                self.params = crate::params::BgRemovalParams::default();
                self.protect_mask.fill(0);
                self.eyedropper_armed = false;
                self.protect_brush_armed = false;
                self.protect_painting = false;
                self.protect_erase_mode = false;
                // Stage the panel-store repopulation (shell drains via
                // `take_pending_panel_reset`).
                self.pending_panel_reset = true;
            }
        }
        if changed && self.has_source() {
            self.rerun_preview();
        }
        // Every UI edit marks the shell-side canvas-preview cache stale —
        // parity with the pre-ADR-0040-TG-B `!bgremoval_ui_edits.is_empty()`
        // gate. Some variants (`BrushSize` / `SetFalloff` / `Apply` /
        // toggles) don't change the matte itself, but the previous code
        // already invalidated the canvas preview for them; preserved here.
        self.params_dirty = true;
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
                label: "Bg Removal".to_string(),
                icon: None,
                active: true,
            }])
            .with_controls(vec![
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
        // Docked-panel NodeIds (`BGR_*` in `ph2d_editor_core::ids`) are
        // routed through `apply_ui_edit` so the semantic mapping
        // (normalized slider → full-scale param, clamps, projections) lives
        // exactly once. ADR-0040 TG-B replacement for the panel-side
        // semantic mapping that used to live in `panel-bgremoval/event.rs`.
        match event {
            // Sliders (normalized 0..1).
            PanelEvent::SetValue(id, v) if id == ids::BGR_TOLERANCE => {
                self.apply_ui_edit(BgRemovalUiEdit::Tolerance(v as f32));
                return;
            }
            PanelEvent::SetValue(id, v) if id == ids::BGR_FEATHER => {
                self.apply_ui_edit(BgRemovalUiEdit::Feather(v as f32));
                return;
            }
            PanelEvent::SetValue(id, v) if id == ids::BGR_REFINE => {
                self.apply_ui_edit(BgRemovalUiEdit::Refine(v as f32));
                return;
            }
            PanelEvent::SetValue(id, v) if id == ids::BGR_GROW => {
                self.apply_ui_edit(BgRemovalUiEdit::Grow(v as f32));
                return;
            }
            PanelEvent::SetValue(id, v) if id == ids::BGR_BRUSH_SIZE => {
                self.apply_ui_edit(BgRemovalUiEdit::BrushSize(v as f32));
                return;
            }
            // Number chips (same semantics as the matching slider).
            PanelEvent::SetValue(id, v) if id == ids::BGR_TOLERANCE_NUM => {
                self.apply_ui_edit(BgRemovalUiEdit::Tolerance(v as f32));
                return;
            }
            PanelEvent::SetValue(id, v) if id == ids::BGR_FEATHER_NUM => {
                self.apply_ui_edit(BgRemovalUiEdit::Feather(v as f32));
                return;
            }
            PanelEvent::SetValue(id, v) if id == ids::BGR_REFINE_NUM => {
                self.apply_ui_edit(BgRemovalUiEdit::Refine(v as f32));
                return;
            }
            PanelEvent::SetValue(id, v) if id == ids::BGR_GROW_NUM => {
                self.apply_ui_edit(BgRemovalUiEdit::Grow(v as f32));
                return;
            }
            PanelEvent::SetValue(id, v) if id == ids::BGR_BRUSH_SIZE_NUM => {
                self.apply_ui_edit(BgRemovalUiEdit::BrushSize(v as f32));
                return;
            }
            // 4-way falloff segmented.
            PanelEvent::Click(id) if id == ids::BGR_FALLOFF_SMOOTH => {
                self.apply_ui_edit(BgRemovalUiEdit::SetFalloff(BrushFalloff::Smooth));
                return;
            }
            PanelEvent::Click(id) if id == ids::BGR_FALLOFF_SPHERE => {
                self.apply_ui_edit(BgRemovalUiEdit::SetFalloff(BrushFalloff::Sphere));
                return;
            }
            PanelEvent::Click(id) if id == ids::BGR_FALLOFF_SHARP => {
                self.apply_ui_edit(BgRemovalUiEdit::SetFalloff(BrushFalloff::Sharp));
                return;
            }
            PanelEvent::Click(id) if id == ids::BGR_FALLOFF_CONSTANT => {
                self.apply_ui_edit(BgRemovalUiEdit::SetFalloff(BrushFalloff::Constant));
                return;
            }
            // Buttons.
            PanelEvent::Click(id) if id == ids::BGR_APPLY => {
                self.apply_ui_edit(BgRemovalUiEdit::Apply);
                return;
            }
            PanelEvent::Click(id) if id == ids::BGR_RESET => {
                self.apply_ui_edit(BgRemovalUiEdit::ResetAll);
                return;
            }
            PanelEvent::Click(id) if id == ids::BGR_EYEDROPPER => {
                self.apply_ui_edit(BgRemovalUiEdit::ToggleEyedropper);
                return;
            }
            PanelEvent::Click(id) if id == ids::BGR_PROTECT => {
                self.apply_ui_edit(BgRemovalUiEdit::ToggleProtectBrush);
                return;
            }
            PanelEvent::Click(id) if id == ids::BGR_PROTECT_CLEAR => {
                self.apply_ui_edit(BgRemovalUiEdit::ClearProtectMask);
                return;
            }
            PanelEvent::Click(id) if id == ids::BGR_SHOW_MASK => {
                self.apply_ui_edit(BgRemovalUiEdit::ToggleShowMask);
                return;
            }
            // "Separate Islands" toggle + its min-pixel slider/chip. IDs
            // owned by the tool crate (`crate::ids`) — declared next to
            // the semantic mapping below so a parallel agent adding a
            // peer feature doesn't collide on `editor-core/src/ids.rs`.
            PanelEvent::Click(id) if id == crate::ids::BGR_SEPARATE_ISLANDS => {
                self.apply_ui_edit(BgRemovalUiEdit::ToggleSeparateIslands);
                return;
            }
            PanelEvent::SetValue(id, v) if id == crate::ids::BGR_MIN_ISLAND_PX => {
                self.apply_ui_edit(BgRemovalUiEdit::SetMinIslandPixels(v as f32));
                return;
            }
            PanelEvent::SetValue(id, v) if id == crate::ids::BGR_MIN_ISLAND_PX_NUM => {
                self.apply_ui_edit(BgRemovalUiEdit::SetMinIslandPixels(v as f32));
                return;
            }
            _ => {}
        }
        // FloatingPanel built by `build_panel` — kept for parity with the
        // pre-docked-panel path. Distinct NodeIds (`TOLERANCE_NODE = 504`
        // etc.) so the two arm-sets never overlap.
        let mut matched = false;
        let mut changed = false;
        match event {
            PanelEvent::SetValue(id, v) if id == TOLERANCE_NODE => {
                self.params.chroma.tolerance = (v.clamp(0.0, 1.0) as f32) * 0.30;
                changed = true;
                matched = true;
            }
            PanelEvent::SetValue(id, v) if id == FEATHER_NODE => {
                self.params.chroma.feather = (v.clamp(0.0, 1.0) as f32) * 0.20;
                changed = true;
                matched = true;
            }
            PanelEvent::SetValue(id, v) if id == REFINE_NODE => {
                self.params.refinement.radius = (v.clamp(0.0, 1.0) * 100.0).round() as u32;
                changed = true;
                matched = true;
            }
            // One-shot trigger; the next build_panel emits a
            // fresh Toggle with on=false so the visual resets.
            PanelEvent::Toggle(id, on) if id == APPLY_NODE && on => {
                self.pending_apply = true;
                matched = true;
            }
            _ => {}
        }
        if changed && self.has_source() {
            self.rerun_preview();
        }
        // Parity with the docked-panel path (which routes through
        // `apply_ui_edit` and so picks up the same dirty mark): any matched
        // FloatingPanel edit invalidates the shell-side canvas-preview
        // cache. Without this, sliders on the legacy panel would not
        // refresh the on-canvas overlay until an unrelated event nudged
        // `take_params_dirty`.
        if matched {
            self.params_dirty = true;
        }
    }

    fn on_activate(&mut self) {
        // Defaults load on every fresh panel open — no carryover from a
        // previous session. Routed through `apply_ui_edit::ResetAll`
        // so the panel preview rebuilds against the cleaned params on
        // the next idle frame (same code path the Reset button uses).
        self.apply_ui_edit(crate::params::BgRemovalUiEdit::ResetAll);
    }

    fn on_deactivate(&mut self) {
        // Disarm the eyedropper + protect brush so reactivating later
        // starts clean and a stale armed state can't intercept canvas
        // clicks meant for another tool.
        self.eyedropper_armed = false;
        self.protect_brush_armed = false;
        self.protect_painting = false;
        self.protect_erase_mode = false;
        // Clear the one-shot drain flags so a Cancel-mid-Apply (or any
        // deactivation while the bridge hasn't yet drained the pending
        // apply / dirty bit) does not fire a phantom bake nor a spurious
        // canvas-preview rerun on the next activation.
        self.pending_apply = false;
        self.params_dirty = false;
        // Wave 10 / Etapa 1.B audit fix [A2]: align the invariant
        // "deactivate clears canvas preview cache" across BOTH paths —
        // `Tool::on_deactivate` (fired by ToolRegistry::set_active when
        // switching to ANY other tool, including non-Raster like Brush)
        // and `RasterEditTool::deactivate` (fired by tool-runtime's
        // drive_deactivate_cleanup when the next active tool is Raster).
        // Without this, BgR→Brush→BgR (without selection change) would
        // re-paint the stale cached frame on first re-activate.
        self.cached_canvas_preview = None;
    }

    fn as_raster_edit_mut(&mut self) -> Option<&mut dyn RasterEditTool> {
        // Wave 10 / Etapa 1.B (ADR-0041): BgRemoval is the first
        // production tool to implement the RasterEditTool generic
        // channel. The shell's `ph2d-tool-runtime` helpers compose
        // over this upcast — bridges no longer need
        // `downcast_mut::<BgRemovalTool>` for the generic raster
        // I/O lifecycle (set_source / current_preview /
        // take_pending_commit / run_full / deactivate). The
        // tool-specific concerns (eyedropper / protect-brush /
        // panel snapshot publish / brush ring) still go through
        // `as_any_mut` downcast — ADR-0040 §3 documented exception.
        Some(self)
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

/// Compute the `(w, h)` that fit inside a `max_dim × max_dim` box,
/// preserving aspect ratio and **never upscaling** — inputs already
/// within the box pass through unchanged. Used for the on-canvas
/// preview source (unlike [`aspect_fit`], which scales the longer side
/// up to `target` for the fixed-size thumbnail).
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

/// Wave 10 / Etapa 1.B (ADR-0041): BgRemoval drives its raster
/// I/O lifecycle through the generic `RasterEditTool` channel.
///
/// Mapping:
/// - `set_source` → wraps the existing `set_source_snapshot` inherent.
/// - `current_preview` → drains `params_dirty`; if dirty, runs
///   `run_canvas_preview` into `cached_canvas_preview` and returns a
///   slice into it.
/// - `take_pending_commit` → wraps `take_pending_apply`.
/// - `run_full` → wraps `run_full_resolution(&mut Vec)` into the
///   owned `(Vec, w, h)` shape the contract requires.
/// - `deactivate` → drops the canvas-preview cache + reuses
///   `on_deactivate` semantics (eyedropper/brush disarm,
///   pending_apply drop). Keeps `source_rgba` (so a re-activate
///   without selection change keeps state) — the cache being dropped
///   is what stops the shell overlay from painting after the tool
///   leaves.
impl RasterEditTool for BgRemovalTool {
    fn set_source(&mut self, rgba: Vec<u8>, width: u32, height: u32) {
        // Trait signature stays `Vec<u8>` (ADR-0041). Wave 11 typed
        // migration on the inherent `set_source_snapshot` — zero-copy
        // cast via bytemuck (SrgbRgba is repr-transparent + Pod).
        self.set_source_snapshot(bytemuck::allocation::cast_vec(rgba), width, height);
    }

    fn current_preview(&mut self) -> Option<(&[u8], u32, u32)> {
        // Drain dirty flag — this is the contract for current_preview
        // per ADR-0041. If nothing's dirty, last-good cache stays;
        // the bridge keeps painting it.
        if !self.take_params_dirty() {
            return None;
        }
        // No source pushed yet OR canvas-preview source not built.
        if !self.has_source() || self.canvas_src_rgba.is_empty() {
            return None;
        }
        let mut out = Vec::new();
        let (w, h) = self.run_canvas_preview(&mut out);
        if w == 0 || h == 0 {
            return None;
        }
        self.cached_canvas_preview = Some((out, w, h));
        // Borrow back the cache for the slice — guaranteed Some by the
        // line above; the assignment then unwrap pattern keeps Miri
        // happy (no overlapping &mut + &).
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
        // Mirror the existing `Tool::on_deactivate` semantics + drop
        // the canvas-preview cache (which the runtime-driven bridge
        // now reads from instead of holding it shell-side).
        self.eyedropper_armed = false;
        self.protect_brush_armed = false;
        self.protect_painting = false;
        self.pending_apply = false;
        self.params_dirty = false;
        self.cached_canvas_preview = None;
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
    fn panel_has_four_canonical_controls() {
        let p = BgRemovalTool::default().build_panel();
        assert_eq!(p.tool_id, ToolId::new("bgremoval"));
        assert_eq!(p.title, "Bg Removal");
        assert_eq!(p.tabs.len(), 1);
        assert_eq!(p.controls.len(), 4);
        assert!(matches!(p.controls[0], PanelControl::Slider(_)));
        assert!(matches!(p.controls[1], PanelControl::Slider(_)));
        assert!(matches!(p.controls[2], PanelControl::Slider(_)));
        assert!(matches!(p.controls[3], PanelControl::Toggle(_)));
        let labels: Vec<&str> = p.controls.iter().map(|c| c.label()).collect();
        assert_eq!(labels, vec!["Tolerance", "Feather", "Refine", "Apply"]);
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
        match &panel.controls[3] {
            PanelControl::Toggle(tg) => assert!(!tg.on, "Apply toggle must reset to off"),
            _ => panic!("expected Toggle at index 3"),
        }
    }

    #[test]
    fn set_source_snapshot_marks_has_source_true() {
        let mut t = BgRemovalTool::default();
        let buf = vec![255u8; 8 * 8 * 4];
        t.set_source_snapshot(bytemuck::allocation::cast_vec(buf), 8, 8);
        assert!(t.has_source());
    }

    #[test]
    fn set_source_snapshot_builds_thumbnail_and_preview() {
        // Push a 32×32 opaque-white source; the thumbnail must
        // letterbox to 160×160 and the preview pipeline must produce
        // a same-size buffer.
        let mut t = BgRemovalTool::default();
        let buf = vec![255u8; 32 * 32 * 4];
        t.set_source_snapshot(bytemuck::allocation::cast_vec(buf), 32, 32);
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
        t.set_source_snapshot(bytemuck::allocation::cast_vec(buf), 32, 32);
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
    fn ui_snapshot_reflects_default_params() {
        let t = BgRemovalTool::default();
        let s = t.ui_snapshot();
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
        let s = t.ui_snapshot();
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
        let buf: Vec<u8> = vec![
            255, 0, 0, 255, // (0,0) red
            0, 255, 0, 255, // (1,0) green
            0, 0, 255, 255, // (0,1) blue
            255, 255, 255, 255, // (1,1) white
        ];
        t.set_source_snapshot(bytemuck::allocation::cast_vec(buf), 2, 2);
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

    // ── Protection brush tests ─────────────────────────────────────

    #[test]
    fn paint_protect_stamps_disc_and_lazy_sizes_mask() {
        let mut t = BgRemovalTool::default();
        // No source → no-op (no panic, no mask).
        t.paint_protect_at_uv(0.5, 0.5, 4.0);
        assert!(!t.has_protect_mask());

        let buf = vec![255u8; 32 * 32 * 4];
        t.set_source_snapshot(bytemuck::allocation::cast_vec(buf), 32, 32);
        // Hard (Constant) falloff so the whole disc is full strength.
        t.apply_ui_edit(BgRemovalUiEdit::SetFalloff(BrushFalloff::Constant));
        t.paint_protect_at_uv(0.5, 0.5, 6.0);
        assert!(t.has_protect_mask());
        let (mask, w, h) = t.protect_mask_source();
        assert_eq!((w, h), (32, 32));
        // Centre pixel fully protected under Constant falloff.
        let c = 16 * 32 + 16;
        assert_eq!(mask[c], 255, "disc centre must be fully protected (hard)");
        // A far corner is untouched.
        assert_eq!(mask[0], 0, "corner outside the disc stays unprotected");
    }

    #[test]
    fn paint_protect_falloff_is_monotonic_center_to_rim() {
        // Smooth falloff: strength must be max at the centre and decay to
        // ~0 at the rim along a row through the dab.
        let mut t = BgRemovalTool::default();
        t.set_source_snapshot(
            bytemuck::allocation::cast_vec(vec![255u8; 64 * 64 * 4]),
            64,
            64,
        );
        t.apply_ui_edit(BgRemovalUiEdit::SetFalloff(BrushFalloff::Smooth));
        let r = 20.0;
        t.paint_protect_at_uv(0.5, 0.5, r);
        let (mask, _, _) = t.protect_mask_source();
        let cx = (0.5_f32 * 63.0).round() as usize;
        let row = cx * 64;
        let centre = mask[row + cx] as i32;
        let mid = mask[row + cx + 10] as i32; // ~half radius out
        let rim = mask[row + cx + 19] as i32; // near the rim
        assert!(centre >= mid && mid >= rim, "{centre} >= {mid} >= {rim}");
        assert!(centre > 200, "centre near-full, got {centre}");
        assert!(rim < 64, "rim near-zero, got {rim}");
    }

    #[test]
    fn erase_protect_subtracts_strength() {
        let mut t = BgRemovalTool::default();
        t.set_source_snapshot(
            bytemuck::allocation::cast_vec(vec![255u8; 32 * 32 * 4]),
            32,
            32,
        );
        t.apply_ui_edit(BgRemovalUiEdit::SetFalloff(BrushFalloff::Constant));
        // Paint a hard disc, then erase the centre with a hard dab.
        t.paint_protect_at_uv(0.5, 0.5, 8.0);
        let c = 16 * 32 + 16;
        assert_eq!(t.protect_mask_source().0[c], 255);
        t.erase_protect_at_uv(0.5, 0.5, 4.0);
        assert_eq!(
            t.protect_mask_source().0[c],
            0,
            "hard erase clears the painted centre"
        );
    }

    #[test]
    fn erase_on_empty_mask_is_noop() {
        let mut t = BgRemovalTool::default();
        t.set_source_snapshot(
            bytemuck::allocation::cast_vec(vec![255u8; 16 * 16 * 4]),
            16,
            16,
        );
        t.erase_protect_at_uv(0.5, 0.5, 4.0);
        assert!(
            !t.has_protect_mask(),
            "erase without a painted mask is inert"
        );
    }

    #[test]
    fn stroke_interpolation_fills_gap_between_distant_dabs() {
        // Two dabs spaced 30 px apart within one stroke with a small
        // radius (4 px) — before stroke interpolation, the midpoint
        // was untouched ("bolinhas" visible along the path). After
        // the fix, the segment is continuously covered.
        let mut t = BgRemovalTool::default();
        t.set_source_snapshot(
            bytemuck::allocation::cast_vec(vec![255u8; 64 * 64 * 4]),
            64,
            64,
        );
        t.apply_ui_edit(BgRemovalUiEdit::SetFalloff(BrushFalloff::Constant));
        let r = 4.0;
        // Stroke begins at (10, 32), ends at (40, 32) — 30 px horizontal.
        let w = 63.0_f32;
        let h = 63.0_f32;
        t.paint_protect_at_uv(10.0 / w, 32.0 / h, r);
        t.paint_protect_at_uv(40.0 / w, 32.0 / h, r);
        let (mask, _, _) = t.protect_mask_source();
        // Walk the midline: every pixel from (10..=40, 32) must be
        // touched. A single gap = the bug Enio reported.
        for x in 10..=40 {
            let i = 32 * 64 + x;
            assert!(
                mask[i] >= 200,
                "px {x} on the stroke path must be protected (got {})",
                mask[i]
            );
        }
        // Outside the stroke band stays untouched.
        let outside = 50 * 64 + 50;
        assert_eq!(mask[outside], 0);
    }

    #[test]
    fn stroke_anchor_resets_between_strokes() {
        // Stroke 1: paint at (10, 10). Pointer-up. Stroke 2: paint at
        // (50, 50). The line connecting them must NOT be filled — the
        // anchor reset on pointer-up prevents cross-stroke interpolation.
        let mut t = BgRemovalTool::default();
        t.set_source_snapshot(
            bytemuck::allocation::cast_vec(vec![255u8; 64 * 64 * 4]),
            64,
            64,
        );
        t.apply_ui_edit(BgRemovalUiEdit::SetFalloff(BrushFalloff::Constant));
        let r = 3.0;
        let scale = 63.0_f32;
        // First stroke.
        t.set_protect_painting(true);
        t.paint_protect_at_uv(10.0 / scale, 10.0 / scale, r);
        t.set_protect_painting(false); // pointer-up — resets anchor.
        // Second stroke, far away.
        t.set_protect_painting(true);
        t.paint_protect_at_uv(50.0 / scale, 50.0 / scale, r);
        t.set_protect_painting(false);
        let (mask, _, _) = t.protect_mask_source();
        // The mid-segment between the two strokes must remain untouched.
        let mid = 30 * 64 + 30;
        assert_eq!(
            mask[mid], 0,
            "no interpolation across pointer-up boundary (mid-segment must be clean)"
        );
        // Both stroke endpoints ARE painted.
        assert!(mask[10 * 64 + 10] > 200);
        assert!(mask[50 * 64 + 50] > 200);
    }

    #[test]
    fn brush_size_edit_maps_and_round_trips() {
        let mut t = BgRemovalTool::default();
        // Default snapshot reflects DEFAULT_BRUSH_SIZE01.
        assert!((t.ui_snapshot().brush_size01 - DEFAULT_BRUSH_SIZE01).abs() < 1e-5);
        t.apply_ui_edit(BgRemovalUiEdit::BrushSize(0.5));
        assert!((t.ui_snapshot().brush_size01 - 0.5).abs() < 1e-5);
        assert!((t.brush_radius_px() - 0.5 * BRUSH_SIZE_FULL_SCALE).abs() < 1e-3);
    }

    #[test]
    fn show_mask_and_falloff_edits() {
        let mut t = BgRemovalTool::default();
        assert!(t.show_mask(), "show-mask defaults on");
        t.apply_ui_edit(BgRemovalUiEdit::ToggleShowMask);
        assert!(!t.show_mask());
        assert_eq!(t.falloff(), BrushFalloff::Smooth);
        t.apply_ui_edit(BgRemovalUiEdit::SetFalloff(BrushFalloff::Sharp));
        assert_eq!(t.falloff(), BrushFalloff::Sharp);
        assert_eq!(t.ui_snapshot().falloff, BrushFalloff::Sharp);
    }

    #[test]
    fn clear_protect_mask_wipes_it() {
        let mut t = BgRemovalTool::default();
        let buf = vec![255u8; 16 * 16 * 4];
        t.set_source_snapshot(bytemuck::allocation::cast_vec(buf), 16, 16);
        t.paint_protect_at_uv(0.5, 0.5, 3.0);
        assert!(t.has_protect_mask());
        t.clear_protect_mask();
        assert!(!t.has_protect_mask());
        let (mask, w, h) = t.protect_mask_source();
        assert!(mask.is_empty());
        assert_eq!((w, h), (0, 0));
    }

    #[test]
    fn new_image_dims_clear_stale_protect_mask() {
        let mut t = BgRemovalTool::default();
        t.set_source_snapshot(
            bytemuck::allocation::cast_vec(vec![255u8; 16 * 16 * 4]),
            16,
            16,
        );
        t.paint_protect_at_uv(0.5, 0.5, 3.0);
        assert!(t.has_protect_mask());
        // Same dims → preserved (Apply re-feed case).
        t.set_source_snapshot(
            bytemuck::allocation::cast_vec(vec![128u8; 16 * 16 * 4]),
            16,
            16,
        );
        assert!(t.has_protect_mask(), "same-dims re-feed keeps the mask");
        // Different dims → cleared.
        t.set_source_snapshot(bytemuck::allocation::cast_vec(vec![255u8; 8 * 8 * 4]), 8, 8);
        assert!(!t.has_protect_mask(), "new dimensions drop the stale mask");
    }

    #[test]
    fn canvas_preview_caps_resolution_and_runs() {
        let mut t = BgRemovalTool::default();
        // 1024×512 source → capped to PREVIEW_MAX_DIM on the long axis.
        let buf = vec![255u8; 1024 * 512 * 4];
        t.set_source_snapshot(bytemuck::allocation::cast_vec(buf), 1024, 512);
        let mut out = Vec::new();
        let (cw, ch) = t.run_canvas_preview(&mut out);
        assert_eq!(cw, PREVIEW_MAX_DIM);
        assert_eq!(ch, PREVIEW_MAX_DIM / 2);
        assert_eq!(out.len(), (cw as usize) * (ch as usize) * 4);
    }

    #[test]
    fn canvas_preview_small_source_passes_through() {
        let mut t = BgRemovalTool::default();
        let buf = vec![255u8; 64 * 48 * 4];
        t.set_source_snapshot(bytemuck::allocation::cast_vec(buf), 64, 48);
        let mut out = Vec::new();
        let (cw, ch) = t.run_canvas_preview(&mut out);
        assert_eq!((cw, ch), (64, 48), "sub-cap source is not upscaled");
    }

    #[test]
    fn canvas_preview_no_source_is_noop() {
        let mut t = BgRemovalTool::default();
        let mut out = vec![1u8, 2, 3];
        let (cw, ch) = t.run_canvas_preview(&mut out);
        assert_eq!((cw, ch), (0, 0));
        assert!(out.is_empty());
    }

    #[test]
    fn aspect_fit_within_no_upscale() {
        assert_eq!(aspect_fit_within(100, 80, 512), (100, 80));
        assert_eq!(aspect_fit_within(1024, 512, 512), (512, 256));
        assert_eq!(aspect_fit_within(512, 1024, 512), (256, 512));
    }

    // ── Separate Islands tests ─────────────────────────────────────

    #[test]
    fn toggle_separate_islands_flips_param() {
        let mut t = BgRemovalTool::default();
        assert!(!t.params.separate_islands);
        assert!(!t.ui_snapshot().separate_islands);
        t.apply_ui_edit(BgRemovalUiEdit::ToggleSeparateIslands);
        assert!(t.params.separate_islands);
        assert!(t.ui_snapshot().separate_islands);
        t.apply_ui_edit(BgRemovalUiEdit::ToggleSeparateIslands);
        assert!(!t.params.separate_islands);
    }

    #[test]
    fn set_min_island_pixels_maps_normalized_to_full_scale() {
        let mut t = BgRemovalTool::default();
        // 0.0 → 1 pixel (the minimum useful filter).
        t.apply_ui_edit(BgRemovalUiEdit::SetMinIslandPixels(0.0));
        assert_eq!(t.params.min_island_pixels, 1);
        // 1.0 → full-scale ceiling.
        t.apply_ui_edit(BgRemovalUiEdit::SetMinIslandPixels(1.0));
        assert_eq!(
            t.params.min_island_pixels,
            MIN_ISLAND_PIXELS_FULL_SCALE as u32
        );
        // Out-of-range clamps.
        t.apply_ui_edit(BgRemovalUiEdit::SetMinIslandPixels(2.0));
        assert_eq!(
            t.params.min_island_pixels,
            MIN_ISLAND_PIXELS_FULL_SCALE as u32
        );
    }

    #[test]
    fn pending_islands_stays_empty_when_toggle_off() {
        let mut t = BgRemovalTool::default();
        t.set_source_snapshot(
            bytemuck::allocation::cast_vec(vec![255u8; 16 * 16 * 4]),
            16,
            16,
        );
        // Toggle is off by default.
        let mut out = Vec::new();
        let _ = t.run_full_resolution(&mut out);
        assert!(t.take_pending_islands().is_empty());
    }

    #[test]
    fn take_pending_islands_is_one_shot() {
        // Seed pending_islands by hand via the extraction algorithm —
        // we can't reach the field through public API except via take,
        // and an end-to-end test that exercises the pipeline depends
        // on what `chroma::segment` picks as background for a contrived
        // input. This test isolates the take semantics.
        let rgba: Vec<u8> = (0..16 * 16).flat_map(|_| [255u8, 255, 255, 255]).collect();
        let mut scratch = BgRemovalScratch::default();
        let mut islands_out = Vec::new();
        islands::extract(&rgba, 16, 16, 1, &mut scratch, &mut islands_out);
        // Sanity: single opaque block ⇒ exactly one island.
        assert_eq!(islands_out.len(), 1);

        // Splice the pre-computed islands in as if `run_full_resolution`
        // had populated them. (Test-only field access; the production
        // path is the `if self.params.separate_islands` branch in
        // `run_full_resolution`.)
        let mut t = BgRemovalTool {
            pending_islands: islands_out,
            ..BgRemovalTool::default()
        };
        let drained = t.take_pending_islands();
        assert_eq!(drained.len(), 1, "first drain returns the queue");
        assert!(
            t.take_pending_islands().is_empty(),
            "second drain is empty (one-shot)"
        );
    }

    #[test]
    fn run_full_resolution_works_after_per_sprite_source_swap() {
        // Mirrors the shell's multi-Apply drain pattern: one
        // BgRemovalTool instance bakes N sprites in sequence via
        // set_source_snapshot → run_full_resolution per entity. Each
        // bake must reflect the CURRENT snapshot, not leak state
        // (source_w/h, scratch buffer dims) from the prior sprite.
        // Regression cover (§12.6 / §12.9 UI_Bugs + Agent D gap).
        let mut t = BgRemovalTool::default();

        // Sprite 1: 8×8 red.
        let mut buf1: Vec<u8> = Vec::with_capacity(8 * 8 * 4);
        for _ in 0..(8 * 8) {
            buf1.extend_from_slice(&[200u8, 30, 30, 255]);
        }
        t.set_source_snapshot(bytemuck::allocation::cast_vec(buf1), 8, 8);
        let mut out1 = Vec::new();
        let (w1, h1) = t.run_full_resolution(&mut out1);
        assert_eq!((w1, h1), (8, 8));
        assert_eq!(out1.len(), 8 * 8 * 4);

        // Sprite 2: different dims + colour. Must re-bake against the
        // fresh snapshot, not reuse out1's dims.
        let mut buf2: Vec<u8> = Vec::with_capacity(12 * 5 * 4);
        for _ in 0..(12 * 5) {
            buf2.extend_from_slice(&[30u8, 200, 50, 255]);
        }
        t.set_source_snapshot(bytemuck::allocation::cast_vec(buf2), 12, 5);
        let mut out2 = Vec::new();
        let (w2, h2) = t.run_full_resolution(&mut out2);
        assert_eq!((w2, h2), (12, 5), "per-sprite source swap leaked dims");
        assert_eq!(out2.len(), 12 * 5 * 4);
    }

    #[test]
    fn on_activate_resets_params_and_arms_panel_repopulate() {
        // Regression cover (§12.3 / §12.4 UI_Bugs): `on_activate` must
        // route through `apply_ui_edit::ResetAll` so (a) params snap to
        // defaults AND (b) `pending_panel_reset` arms so the shell
        // bridge re-runs `Panel::populate(store)` and the slider knobs
        // visually snap back to defaults.
        let default_snap = BgRemovalUiSnapshot::default();
        let dirty_tolerance01 = if (default_snap.tolerance01 - 0.1).abs() < 1e-3 {
            0.9_f32
        } else {
            0.1_f32
        };
        let mut t = BgRemovalTool::default();
        t.apply_ui_edit(BgRemovalUiEdit::Tolerance(dirty_tolerance01));
        assert!(
            (t.ui_snapshot().tolerance01 - default_snap.tolerance01).abs() > 0.01,
            "test setup must actually dirty tolerance01"
        );
        // Drain any stray reset flag first.
        let _ = t.take_pending_panel_reset();

        t.on_activate();

        let after = t.ui_snapshot();
        assert!(
            (after.tolerance01 - default_snap.tolerance01).abs() < 1e-5,
            "on_activate must restore default tolerance01 (got {} expected {})",
            after.tolerance01,
            default_snap.tolerance01,
        );
        assert!(
            t.take_pending_panel_reset(),
            "on_activate must arm pending_panel_reset so the shell repopulates"
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // Wave 10 / Etapa 1.B — RasterEditTool impl tests (ADR-0041 follow-up)
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn as_raster_edit_mut_returns_some_for_bgremoval() {
        let mut t = BgRemovalTool::default();
        // Verify the upcast works — the runtime helpers depend on this.
        assert!(<dyn Tool as Tool>::as_raster_edit_mut(&mut t).is_some());
    }

    #[test]
    fn raster_edit_set_source_delegates_to_set_source_snapshot() {
        let mut t = BgRemovalTool::default();
        let rgba = vec![255u8; 16 * 16 * 4];
        RasterEditTool::set_source(&mut t, rgba, 16, 16);
        assert!(t.has_source());
        assert_eq!(t.source_size(), (16, 16));
        // params_dirty was set by set_source_snapshot (via rerun_preview)
        // — verify the dirty flag is up for current_preview to drain.
        // (peek without drain via cloning the bool — there's no peek API.)
    }

    #[test]
    fn raster_edit_current_preview_drains_dirty_and_caches() {
        let mut t = BgRemovalTool::default();
        // Push a source so canvas-preview has something to run on.
        let rgba = vec![128u8; 8 * 8 * 4];
        RasterEditTool::set_source(&mut t, rgba, 8, 8);
        // First call: dirty drains, cache populated, slice returned.
        let frame = RasterEditTool::current_preview(&mut t);
        assert!(
            frame.is_some(),
            "first call after set_source must return Some"
        );
        let (pixels, w, h) = frame.unwrap();
        assert!(w > 0 && h > 0);
        assert_eq!(pixels.len(), (w as usize) * (h as usize) * 4);
        // Second call: dirty was drained, no new frame.
        let frame2 = RasterEditTool::current_preview(&mut t);
        assert!(
            frame2.is_none(),
            "second call without new dirty must be None"
        );
        // Cache should still be there (last-good preview lives on in the tool).
        assert!(t.cached_canvas_preview.is_some());
    }

    #[test]
    fn raster_edit_current_preview_returns_none_without_source() {
        // No set_source — no canvas_src_rgba — must return None even
        // if dirty (defensive: drain dirty but don't fabricate a frame).
        let mut t = BgRemovalTool {
            params_dirty: true,
            ..BgRemovalTool::default()
        };
        let frame = RasterEditTool::current_preview(&mut t);
        assert!(frame.is_none());
    }

    #[test]
    fn raster_edit_take_pending_commit_drains_apply_flag() {
        let mut t = BgRemovalTool::default();
        t.apply_ui_edit(BgRemovalUiEdit::Apply);
        assert!(RasterEditTool::take_pending_commit(&mut t));
        // Drained.
        assert!(!RasterEditTool::take_pending_commit(&mut t));
    }

    #[test]
    fn raster_edit_run_full_returns_owned_buffer() {
        let mut t = BgRemovalTool::default();
        let rgba = vec![64u8; 8 * 8 * 4];
        RasterEditTool::set_source(&mut t, rgba, 8, 8);
        let (out, w, h) = RasterEditTool::run_full(&mut t);
        assert_eq!((w, h), (8, 8));
        assert_eq!(out.len(), 8 * 8 * 4);
    }

    // Wave 10 / Etapa 1.B audit fix [A1]: set_source must invalidate
    // the canvas-preview cache so a stale frame from the previous
    // selection can never paint over a new sprite.
    #[test]
    fn set_source_invalidates_cached_canvas_preview() {
        let mut t = BgRemovalTool::default();
        // Push source A → drain preview → cache populated.
        RasterEditTool::set_source(&mut t, vec![10u8; 4 * 4 * 4], 4, 4);
        let frame_a = RasterEditTool::current_preview(&mut t);
        assert!(frame_a.is_some());
        assert!(t.cached_canvas_preview.is_some());
        // Push source B (different selection in shell, same dim) →
        // cache must be invalidated BEFORE preview rebuilds.
        // The invariant: at no point can current_preview return a slice
        // that mixes pixels from A.
        RasterEditTool::set_source(&mut t, vec![200u8; 4 * 4 * 4], 4, 4);
        assert!(
            t.cached_canvas_preview.is_none(),
            "set_source MUST invalidate cached_canvas_preview (audit A1)"
        );
        // The next current_preview rebuilds — and the bytes are from B.
        let frame_b = RasterEditTool::current_preview(&mut t);
        assert!(frame_b.is_some());
    }

    // Wave 10 / Etapa 1.B audit fix [A2]: Tool::on_deactivate must
    // clear cached_canvas_preview so switching to a non-Raster tool
    // (like Brush) and back doesn't paint a stale frame.
    #[test]
    fn on_deactivate_clears_cached_canvas_preview() {
        let mut t = BgRemovalTool::default();
        RasterEditTool::set_source(&mut t, vec![50u8; 4 * 4 * 4], 4, 4);
        RasterEditTool::current_preview(&mut t); // populate cache
        assert!(t.cached_canvas_preview.is_some());
        // Tool::on_deactivate is what fires when the ToolRegistry switches
        // to a non-Raster tool (e.g. Brush) — different path from
        // RasterEditTool::deactivate. Both MUST clear the cache.
        Tool::on_deactivate(&mut t);
        assert!(
            t.cached_canvas_preview.is_none(),
            "on_deactivate MUST clear cached_canvas_preview (audit A2)"
        );
    }

    #[test]
    fn raster_edit_deactivate_clears_all_transient_state() {
        let mut t = BgRemovalTool::default();
        let rgba = vec![200u8; 4 * 4 * 4];
        RasterEditTool::set_source(&mut t, rgba, 4, 4);
        let _ = RasterEditTool::current_preview(&mut t); // populate cache
        t.set_eyedropper_armed(true);
        t.set_protect_armed(true);
        t.apply_ui_edit(BgRemovalUiEdit::Apply); // arms pending_apply
        // Now deactivate — every transient flag + cache must drop.
        RasterEditTool::deactivate(&mut t);
        assert!(!t.is_eyedropper_armed());
        assert!(!t.is_protect_armed());
        assert!(!t.is_protect_painting());
        assert!(t.cached_canvas_preview.is_none());
        // params_dirty drained.
        assert!(!t.take_params_dirty());
        // pending_apply drained.
        assert!(!t.take_pending_apply());
        // Source pixels NOT cleared — re-activate without new selection
        // keeps state (the shell drops cache externally; tool is just
        // dropping its OWN transient flags).
        assert!(t.has_source());
    }
}
