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
//!
//! ## Module layout
//!
//! Wave 11 refactor — the god-object `tool.rs` split by topic, mirroring
//! the sibling `algorithm/` directory. The `BgRemovalTool` type lives
//! here; its inherent methods + the `Tool` / `RasterEditTool` trait
//! impls are spread across submodules using Rust's ability to write
//! multiple `impl Type { … }` blocks across files in one crate:
//!
//! - [`source`] — source snapshot, thumbnail / canvas-src rebuild,
//!   preview rerun, pixel sampling, one-shot drain accessors.
//! - [`protect`] — the protection brush (arm/paint/erase/clear + dab
//!   kernel).
//! - [`add_area`] — the "Add area" flood-fill selector + extra-bg
//!   colour list.
//! - [`pipeline`] — full-res / canvas preview pipeline drivers +
//!   combined-protect preparation + mask resamplers.
//! - [`ui`] — `ui_snapshot` projection + `apply_ui_edit` dispatch.
//! - [`trait_impl`] — `impl Tool` + `impl RasterEditTool`.

use ph2d_a11y::NodeId;

use super::params::{BRUSH_SIZE_FULL_SCALE, BgRemovalParams, BrushFalloff, DEFAULT_BRUSH_SIZE01};
use super::scratch::BgRemovalScratch;

// Re-export `IslandPayload` so the field type on the struct below
// resolves without a fully-qualified path, matching the pre-split
// `use super::algorithm::islands::IslandPayload`.
use super::algorithm::islands::IslandPayload;

mod add_area;
mod pipeline;
mod protect;
mod source;
mod trait_impl;
mod ui;

/// Side length (px) of the square thumbnail used for the panel preview.
pub const THUMB_SIZE: u32 = 160;

/// Side cap (px) for the live on-canvas preview overlay. The overlay
/// re-runs the whole pipeline on every parameter change; doing that at
/// full source resolution makes each slider tick janky. The overlay is
/// drawn *scaled* to the sprite footprint anyway, so it re-segments a
/// copy of the source downscaled to fit this box (aspect preserved, no
/// letterbox) instead — keeping slider drags smooth. Apply still bakes
/// at full source resolution via [`BgRemovalTool::run_full_resolution`].
///
/// Enio 2026-05-26: "preciso da ferramenta atuando na imagem real em
/// tempo real" — bumped from 512 to `u32::MAX` so the preview pipeline
/// runs at the SAME resolution as Apply. Preview output is now
/// byte-identical to Apply (no downsample-induced anti-alias drift,
/// no silhouette resolution mismatch). Trade-off: slider drags on
/// large sources (≥ 2K²) become laggy because every tick re-runs the
/// full-res pipeline. Accepted by the user as the cost of "live on
/// the real image".
pub const PREVIEW_MAX_DIM: u32 = u32::MAX;

// NodeId range 500..599 reserved for bgremoval panel controls
// (clear of 100..199 brush/move and 1000..1099 grid_snap).
pub(crate) const TOLERANCE_NODE: NodeId = NodeId(504);
pub(crate) const FEATHER_NODE: NodeId = NodeId(505);
pub(crate) const REFINE_NODE: NodeId = NodeId(506);
pub(crate) const APPLY_NODE: NodeId = NodeId(507);

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
    pub(crate) source_rgba: Vec<u8>,
    pub(crate) source_w: u32,
    pub(crate) source_h: u32,

    /// Pre-scaled thumbnail derived from `source_rgba`. Always
    /// `THUMB_SIZE × THUMB_SIZE` RGBA8 (aspect-fit, letterboxed).
    /// Built once per `set_source_snapshot` call; re-used as the
    /// input of every preview pipeline run.
    pub(crate) thumbnail_rgba: Vec<u8>,
    pub(crate) thumbnail_w: u32,
    pub(crate) thumbnail_h: u32,

    /// Preview output — result of `run_pipeline` on `thumbnail_rgba`
    /// with the current `params`. The panel paint pass blits this.
    /// Length `THUMB_SIZE * THUMB_SIZE * 4`.
    pub(crate) preview_rgba: Vec<u8>,

    /// Reusable scratch for both the preview pipeline and the host's
    /// full-res Apply. Sized lazily.
    pub(crate) scratch: BgRemovalScratch,

    /// Set to `true` when the user activates the Apply toggle. Host
    /// polls via [`Self::take_pending_apply`] each frame; on `true`
    /// it runs the pipeline at full resolution against the active
    /// sprite and writes back a new Individual texture.
    pub(crate) pending_apply: bool,

    /// Set to `true` by any mutator that touches state the shell's
    /// on-canvas preview reflects (params, extra-bg colours, painted
    /// protection mask). Host polls via [`Self::take_params_dirty`]
    /// each frame as the gate for rerunning `run_canvas_preview` —
    /// replaces the old `!bgremoval_ui_edits.is_empty()` check the
    /// shell used before ADR-0040 TG-B routed panel events through
    /// `handle_panel_event` directly.
    pub(crate) params_dirty: bool,

    /// `true` when the params were just reset to defaults (Reset
    /// button OR `on_activate`). The shell bridge drains via
    /// [`Self::take_pending_panel_reset`] and re-runs
    /// `Panel::populate(store)` so the slider knob / chip text
    /// positions snap back to defaults — without this, only the
    /// params struct resets while the WidgetStore retains whatever
    /// drag position the user last left.
    pub(crate) pending_panel_reset: bool,

    /// Whether the panel eyedropper is armed. While `true`, the shell's
    /// canvas click-drag handler samples the source pixel under the
    /// cursor and feeds it to [`Self::add_extra_color`]. Reset on
    /// deactivate / Apply so a stale armed state can't keep eating
    /// canvas clicks after the tool is dismissed.
    pub(crate) eyedropper_armed: bool,

    /// Whether the protection brush is armed. While `true`, the shell's
    /// canvas click-drag handler paints into [`Self::protect_mask`] via
    /// [`Self::paint_protect_at_uv`] instead of running the normal pick /
    /// gizmo / selection logic. Reset on deactivate / Apply (mirrors
    /// `eyedropper_armed`).
    pub(crate) protect_brush_armed: bool,

    /// Whether a protection-brush dab-drag is in progress (set on
    /// pointer-down, cleared on pointer-up by the shell). Transient
    /// pointer state — it lives here rather than on the shell's `App`
    /// because the protection feature does not edit the `App` struct, and
    /// the tool is the natural per-tool home for the flag. Distinct from
    /// `protect_brush_armed` (whether the brush is *selected* at all).
    pub(crate) protect_painting: bool,

    /// Freehand protection mask at the SOURCE resolution
    /// (`protect_mask_w × protect_mask_h`, one byte/pixel; `255` =
    /// protected/forced-foreground, `0` = unprotected). Empty until the
    /// user paints. Threaded into `run_pipeline` as the compose
    /// force-keep mask (a painted region stays opaque).
    pub(crate) protect_mask: Vec<u8>,
    pub(crate) protect_mask_w: u32,
    pub(crate) protect_mask_h: u32,

    /// Source RGBA downscaled to fit [`PREVIEW_MAX_DIM`] (aspect kept,
    /// no letterbox) — the input of the on-canvas live preview. Rebuilt
    /// only when the source snapshot changes, so a slider drag
    /// re-segments this small image instead of the full-res source.
    /// Empty until the host pushes a source.
    pub(crate) canvas_src_rgba: Vec<u8>,
    pub(crate) canvas_src_w: u32,
    pub(crate) canvas_src_h: u32,
    /// Protection mask nearest-resampled to the canvas-preview dims.
    /// Re-filled each canvas-preview run; kept as a field so the
    /// allocation persists across runs (HR-3).
    pub(crate) canvas_protect: Vec<u8>,

    /// Protection-brush radius in SOURCE pixels (what `paint_protect_at_uv`
    /// consumes). Driven by the panel Brush Size slider; default ≈
    /// [`DEFAULT_BRUSH_SIZE01`] × [`BRUSH_SIZE_FULL_SCALE`].
    pub(crate) brush_radius: f32,
    /// Protection-brush dab falloff profile (Smooth / Sphere / Sharp /
    /// Hard) — applied to both paint and erase.
    pub(crate) falloff: BrushFalloff,
    /// Current drag is an ERASE drag (set by the shell on a secondary-
    /// button down). Transient, like `protect_painting`.
    pub(crate) protect_erase_mode: bool,
    /// Whether the painted protection mask is drawn as an on-canvas tint
    /// overlay (the shell gates the overlay on this). Default `true`.
    pub(crate) show_mask: bool,

    /// Per-island RGBA payloads stashed by `run_full_resolution` when
    /// `params.separate_islands` is on. The shell drains them via
    /// [`Self::take_pending_islands`] right after baking the main result
    /// (or alongside it) and spawns one new sprite per entry. Empty when
    /// the toggle is off, when an Apply hasn't run yet, or after the
    /// host has drained.
    pub(crate) pending_islands: Vec<IslandPayload>,

    /// Cached output of the most-recent `run_canvas_preview` invocation.
    /// `RasterEditTool::current_preview` returns a slice into this buffer
    /// when the tool's dirty flag has been drained — so the bridge
    /// doesn't need to maintain its own per-tool preview cache outside.
    ///
    /// Wave 10 / Etapa 1.B (ADR-0041 follow-up): the cache moved from
    /// `shells/desktop/src/app_state.rs::BgremovalPreview` to live inside
    /// the tool — this is what lets `ph2d-tool-runtime::drive_preview_cache`
    /// stay generic.
    pub(crate) cached_canvas_preview: Option<(Vec<u8>, u32, u32)>,

    /// Last (u, v) painted in the current stroke — anchor used by
    /// `stamp_protect` to interpolate intermediate dabs between the
    /// previous cursor position and the current one. Without this, a
    /// fast drag produces visibly spaced discs along the path (Enio
    /// 2026-05-26: "máscara apresenta pintura não regular, como se o
    /// espaço entre os pontos de pintura fossem muito grandes").
    /// `None` outside an active stroke; reset by
    /// [`Self::set_protect_painting`] on pointer-up.
    pub(crate) last_protect_uv: Option<(f32, f32)>,

    /// `max(user_protect_mask_resized, auto_protect_mask)` — what
    /// `algorithm::run_pipeline` actually receives as its `protect`
    /// argument. Lives on the tool (not `scratch`) so a `&` of this
    /// buffer coexists with the `&mut self.scratch` `run_pipeline`
    /// needs. Sized lazily to the current `(w * h)` and reused across
    /// runs (HR-3). Driven by [`Self::prepare_combined_protect`].
    pub(crate) combined_protect: Vec<u8>,

    /// Edge-aware silhouette mask computed at SOURCE resolution and
    /// cached across canvas-preview ticks (Enio 2026-05-26 "tente
    /// tornar o preview mais fiel"). Computing at source resolution
    /// guarantees the preview's silhouette outline + soft-falloff
    /// band match what Apply will produce — without the cache, the
    /// preview ran its own silhouette at the ~256² canvas resolution,
    /// so `DISTANCE_TO_FULL_LOCK = 8` covered ~3% of the image vs
    /// ~0.4% in full-res, producing visibly more edge transparency.
    /// Nearest-resampled into `scratch.auto_protect_mask` at canvas
    /// dims when the preview runs. `cached_auto_protect_for` holds
    /// the dims this buffer was computed for (`None` = not computed
    /// or stale).
    pub(crate) cached_auto_protect_source: Vec<u8>,
    pub(crate) cached_auto_protect_for: Option<(u32, u32)>,

    // ── "Add area" automatic selector (Enio 2026-05-26) ──
    /// Whether the "Add area" selector is armed. Symmetric to the
    /// eyedropper: while armed, a single click on the canvas runs a
    /// flood-fill from the clicked source pixel and writes the
    /// connected same-colour region into [`Self::force_remove_mask`]
    /// — no drag, no brush. Mutually exclusive with
    /// [`Self::protect_brush_armed`] + [`Self::eyedropper_armed`].
    /// Reset on deactivate / Apply / Detect Subject OFF.
    pub(crate) add_area_armed: bool,
    /// FORCE-REMOVE mask at the SOURCE resolution
    /// (`force_remove_mask_w × force_remove_mask_h`, one byte/pixel;
    /// `255` = forced alpha=0, `0` = no force). Threaded into
    /// `run_pipeline` as the compose `force_remove` argument; applied
    /// after `force_keep_protected` so an added area wins over a
    /// protect dab AND over the silhouette auto-protect (most-recent-
    /// intent semantics).
    pub(crate) force_remove_mask: Vec<u8>,
    pub(crate) force_remove_mask_w: u32,
    pub(crate) force_remove_mask_h: u32,
    /// Force-remove mask nearest-resampled to the canvas-preview dims
    /// (mirror of `canvas_protect`). Allocation persists across runs.
    pub(crate) canvas_remove: Vec<u8>,
    /// Source-pixel positions seeded by "Add area" clicks. Each entry
    /// is a `(x, y)` index into `source_rgba` (the user-clicked pixel
    /// at source resolution). The force-remove mask is REGENERATED
    /// from these seeds whenever the user clicks again OR moves the
    /// Tolerance / Feather sliders — so the destructive area tracks
    /// the same soft-band math the compose path uses for the auto-
    /// detected bg + extra picks. Cleared by Clear / ResetAll / source
    /// swap. Re-using the seed pattern (not just baked alpha) is what
    /// makes the destructive area honour slider changes after the
    /// click, exactly like `extra_bg_colors` does for the chroma
    /// backend (Enio 2026-05-27).
    pub(crate) add_area_seeds: Vec<(u32, u32)>,
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
            combined_protect: Vec::new(),
            cached_auto_protect_source: Vec::new(),
            cached_auto_protect_for: None,
            add_area_armed: false,
            force_remove_mask: Vec::new(),
            force_remove_mask_w: 0,
            force_remove_mask_h: 0,
            canvas_remove: Vec::new(),
            add_area_seeds: Vec::new(),
        }
    }
}

/// Stamp a single brush disc into `mask` (one byte per pixel, sized
/// to `w × h`) at UV `(u, v)` with `r` SOURCE-px radius. Falloff
/// strength accumulates via `max` (paint) or `saturating_sub` (erase).
///
/// Free function so Protect + Acrescentar-Área brushes share the
/// kernel without contesting a `&mut self` borrow over the mask field.
/// 8 primitive args; the wrapper-struct refactor would force the
/// optimizer to unwrap on each call (HR-3 hot-path concern), so we
/// allow the lint here.
#[allow(clippy::too_many_arguments)]
pub(crate) fn stamp_disc_into(
    mask: &mut [u8],
    w: u32,
    h: u32,
    falloff: BrushFalloff,
    u: f32,
    v: f32,
    r: f32,
    erase: bool,
) {
    if mask.is_empty() || w == 0 || h == 0 {
        return;
    }
    let cx = u * (w as f32 - 1.0);
    let cy = v * (h as f32 - 1.0);
    let inv_r = 1.0 / r;
    let x0 = (cx - r).floor().max(0.0) as u32;
    let x1 = ((cx + r).ceil() as i64).clamp(0, w as i64 - 1) as u32;
    let y0 = (cy - r).floor().max(0.0) as u32;
    let y1 = ((cy + r).ceil() as i64).clamp(0, h as i64 - 1) as u32;
    let stride = w as usize;
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
            mask[i] = if erase {
                mask[i].saturating_sub(val)
            } else {
                mask[i].max(val)
            };
        }
    }
}

/// Compute the (w, h) that fit inside a `target × target` square,
/// preserving the input aspect ratio. The longer side lands on
/// `target`; the shorter side scales proportionally. Outputs are
/// clamped to at least 1 px so the resize call never sees a 0
/// dimension.
pub(crate) fn aspect_fit(sw: u32, sh: u32, target: u32) -> (u32, u32) {
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
pub(crate) fn aspect_fit_within(sw: u32, sh: u32, max_dim: u32) -> (u32, u32) {
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

    #[test]
    fn default_tool_has_no_source_and_no_pending() {
        let t = BgRemovalTool::default();
        assert!(!t.has_source());
        assert!(t.preview_rgba().is_empty());
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
    fn aspect_fit_within_no_upscale() {
        assert_eq!(aspect_fit_within(100, 80, 512), (100, 80));
        assert_eq!(aspect_fit_within(1024, 512, 512), (512, 256));
        assert_eq!(aspect_fit_within(512, 1024, 512), (256, 512));
    }
}
