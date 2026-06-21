//! `PainterTool` — the layers + effects host (impl `Tool` + `RasterEditTool`).
//!
//! Stateful tool in the TopBar `image_tools` cluster. When activated it installs
//! a takeover UI (ADR-0043 §1.1) and surfaces the docked **Layers** panel
//! (`ph2d-panel-painter-layers`): a layer stack with masks, groups, blend modes
//! and non-destructive **adjustment layers** (`ph2d-painter-effects`). The CPU
//! reference compositor lives in [`crate::compositor`]; the GPU compositor is in
//! `ph2d-render` and is the real-time path the shell uses.
//!
//! The brush/painting engine was deleted; this tool no longer takes pointer
//! input or rasterizes strokes. `canvas_rgba` holds the active layer's working
//! pixels (seeded from the sprite source / imports), composited with the rest of
//! the stack for preview and for the Apply bake (`RasterEditTool::run_full`).

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use ph2d_editor_core::floating_panel::{FloatingPanel, ToolId};
use ph2d_editor_core::tool::{CanvasPaintTool, RasterEditTool, Tool};
use ph2d_painter_effects::BlendMode;

use crate::compositor::{
    CompositorCache, LayerImage, LayerPixelSource, Region, composite, composite_region,
    composite_with_cache,
};
use crate::layers::{LayerId as RtLayerId, LayerKind, LayerStack};
use crate::params::PainterParams;

thread_local! {
    // Pending Cmd/Shift modifiers for layer-row selects, KEYED BY the row's
    // widget NodeId. Written by the layers panel apply_event (the only place
    // that can read host.store().cmd_held / shift_held) right before it forwards
    // a row Click, and read + removed by handle_panel_event keyed by the SAME
    // click NodeId. The frozen PanelEvent (4 variants) cannot carry the bits and
    // handle_panel_event gets no store, so this side channel bridges the gap.
    //
    // Keyed by NodeId (not a single slot) on purpose: the panel writes
    // SYNCHRONOUSLY during apply_event, but the matching Click is queued on the
    // action bus and drained later. Keying by the click's own NodeId keeps each
    // click's modifiers with it; entries are removed on consume.
    static PENDING_SELECT_MODS:
        std::cell::RefCell<std::collections::BTreeMap<ph2d_a11y::NodeId, (bool, bool)>> =
        const { std::cell::RefCell::new(std::collections::BTreeMap::new()) };
}

/// Stash the Cmd/Ctrl + Shift state for the row-select click on `row_id`.
/// Called by the layers panel `apply_event` immediately before it forwards that
/// row's `Click` (it is the only side with `host.store()` access to the live
/// modifier state). Consumed by [`PainterTool::handle_panel_event`] on the
/// matching click. See `PENDING_SELECT_MODS` for why this is keyed by id.
pub fn set_pending_select_mods(row_id: ph2d_a11y::NodeId, cmd: bool, shift: bool) {
    PENDING_SELECT_MODS.with(|m| {
        m.borrow_mut().insert(row_id, (cmd, shift));
    });
}

/// Take + remove the pending modifiers for `row_id` (default = no modifiers).
fn take_pending_select_mods(row_id: ph2d_a11y::NodeId) -> (bool, bool) {
    PENDING_SELECT_MODS.with(|m| m.borrow_mut().remove(&row_id).unwrap_or((false, false)))
}

/// Painter — the layers + effects host. Stateful tool.
///
/// `PainterTool` is a state holder: `canvas_rgba` (the active layer's RGBA8
/// straight working buffer) + the `LayerStack` model + per-layer pixel store +
/// the composite caches. It is testable headless.
pub struct PainterTool {
    pub params: PainterParams,
    /// Working pixels of the ACTIVE layer — RGBA8 straight (compatible with
    /// `wgpu::TextureFormat::Rgba8Unorm`). `Arc<Vec<u8>>` so the bridge's
    /// `painter_preview` cache holds an Arc clone (1 atomic increment per drain
    /// instead of a full memcpy). Seeded by `set_source` from the sprite.
    canvas_rgba: Arc<Vec<u8>>,
    /// The runtime layer model — source of truth for layer structure + per-layer
    /// blend/opacity/visibility/flags. The ACTIVE layer's pixels live in
    /// `canvas_rgba`; non-active layers' pixels live in `images`.
    layers: LayerStack,
    /// Per-(non-active-)layer pixel buffers (canvas-sized straight sRGB8).
    /// `BTreeMap` per HR-5 (deterministic iteration).
    images: BTreeMap<RtLayerId, LayerImage>,
    /// Cached composite output (non-trivial stacks only), behind an `Arc` so the
    /// bridge drain is zero-copy. Invalidated (`None`) on any layer edit.
    composited: Option<Arc<Vec<u8>>>,
    /// The dirty bbox the LAST `take_preview_arc` recomposed (`Some` for a
    /// partial fast-lane update the bridge may upload as a sub-rect).
    preview_upload_bbox: Option<Region>,
    /// Monotonic counter bumped on every change to the PUBLISHED layer structure
    /// (add / select / reorder / visibility / opacity / blend / source reset).
    /// The bridge publishes the `LayerStack` snapshot only when this changes.
    layers_revision: u64,
    /// Per-layer pixel CONTENT version for the GPU preview compositor — bumped
    /// when a layer's PIXELS change (undo/redo, mask flatten, fresh source), so
    /// metadata edits (opacity / blend / adjustment params) don't re-upload.
    layer_pixel_versions: BTreeMap<RtLayerId, u64>,
    /// Monotonic source for [`Self::layer_pixel_versions`] bumps. Never reset, so
    /// a reused layer key always gets a strictly-greater version than cached.
    pixel_clock: u64,
    source_size: (u32, u32),
    preview_dirty: bool,
    pending_commit: bool,
    /// Structural (layer-model) undo/redo. Driven by the layer ops in
    /// [`crate::tool`] via `commit_structural_edit`.
    undo: crate::undo::UndoController,
    /// Which panel occupies the shared right-dock slot (the layers panel header
    /// toggle). Read by the shell `painter_bridge` to drive `panel_visibility`.
    dock_shows_layers: bool,
    /// Accumulated dirty bbox for the partial-recompose fast lane (`take_preview_arc`).
    dirty_rect: Option<Region>,
    /// Multi-selection — the set of layer rows highlighted in the panel. Plain
    /// click collapses to one; Cmd/Ctrl-click toggles; Shift-click selects a run.
    selection: BTreeSet<RtLayerId>,
    /// Adjustment cut-point cache (ADR-0045 §2.7): caches the composite-below each
    /// root adjustment so a slider-drag restarts from the cut instead of
    /// recomposing the whole stack every frame.
    compositor_cache: CompositorCache,
    /// Set by `set_adjustment_param` so the next `take_preview_arc` full recompose
    /// routes through the cut-point cache. Taken on consume.
    adjustment_cache_pending: bool,
    /// Brush + in-progress stroke state for canvas painting (the clean-room
    /// Blender brush engine). See [`crate::tool::paint`].
    paint: paint::PaintState,
}

impl Default for PainterTool {
    fn default() -> Self {
        Self {
            params: PainterParams::default(),
            canvas_rgba: Arc::new(Vec::new()),
            layers: LayerStack::new(),
            images: BTreeMap::new(),
            composited: None,
            preview_upload_bbox: None,
            layers_revision: 0,
            layer_pixel_versions: BTreeMap::new(),
            pixel_clock: 0,
            source_size: (0, 0),
            preview_dirty: false,
            pending_commit: false,
            undo: crate::undo::UndoController::default(),
            dirty_rect: None,
            selection: BTreeSet::new(),
            compositor_cache: CompositorCache::new(),
            adjustment_cache_pending: false,
            // Dock opens on the Layers/Effects view; the header toggle flips to
            // the Brush-properties view (`dock_shows_layers == false`).
            dock_shows_layers: true,
            paint: paint::PaintState::default(),
        }
    }
}

// ── Submodules (god-object split, pure mechanical move) ──
mod internal;
pub(crate) use internal::*;
mod layers;
mod paint;
pub use paint::{
    BRUSH_COUNT_SLIDER_MAX, BRUSH_JITTER_ABS_MAX_PX, BRUSH_SIZE_MAX_PX, BRUSH_SIZE_MIN_PX,
    BRUSH_SPACING_MAX, BrushSettings, brush_falloff_weight_at,
};
mod runtime;
mod trait_impls;
