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
use ph2d_painter_brush::material::MaterialBytes;
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

/// A live **protection session** — the state that lets the mask/selection gate apply its keep factor
/// exactly **ONCE per texel** instead of once per pointer event (`docs/Painter/25_avaliacao_gpu.md`
/// §13.12; the paint crossing a feathered protection came out crackled because the old gate lerped
/// against a per-BATCH snapshot, so the surviving paint was `(1−keep)^N` with `N` = the mouse's polling
/// rate — a law that was a fact about the sampling, not about the gesture).
///
/// It lives as long as the **PROTECTION DECLARATION** does, not as long as a stroke — and that is
/// Enio's call (2026-07-25, §13.13): a protection that scales each stroke separately lets
/// `1 − (1−keep)^N` through, so **eight passes and the mask stops protecting anything** (measured: 0.522
/// → 0.949 at four → 1.000 at eight). A ceiling never erodes, and its boundary is exactly the `keep`
/// contour — the residual comb goes from 1.68 px to the keep field's own 0.07 px.
///
/// ⚠️ **This IS the épocha of §13.7** (`38c1f725b`, reverted in `569149dfc`) — the semantics was never what
/// was wrong with it. What killed it was the **lifecycle**: it had to be committed by hand at **22 foreign
/// canvas writers**, and a writer nobody listed silently had its pixels projected away on the next gated
/// batch (*"o teto capava a tinta comum"*, §13.8). Here the 22 sites collapse into ONE question asked at
/// the top of every batch — *did anything change under me?* — answered by three witnesses ([`Self::witness`],
/// [`Self::scratch_gen`], [`Self::layer`]). Enumeration rots; a witness does not.
pub(crate) struct GateSession {
    /// The canvas as the protection found it — what a fully-protected texel keeps forever. An `Arc` clone,
    /// so seeding costs a refcount, not a copy (the first `make_mut` on the canvas forks it once and the
    /// session keeps the pristine side).
    pub(crate) base: Arc<Vec<u8>>,
    /// What **unrestricted** painting would have produced, accumulated across every stroke of the epoch.
    /// Swapped into `canvas_rgba` for the stamp, so every route (colour, smear, blur, clone, composite)
    /// paints exactly what it would paint with no gate at all — the gate has no say in WHAT is painted,
    /// only in what SHOWS. ⚠️ It accumulates **freely**, which is what makes the ceiling a ceiling and not
    /// a wall: the brush still builds up toward full opacity, the RESULT just converges on `keep` instead
    /// of past it. A cap applied to the brush instead would make the second stroke a no-op.
    pub(crate) free: Arc<Vec<u8>>,
    /// The free plane's pixels under the last batch's region, saved before that batch stamped — the free
    /// plane's own `DragPreview`. A re-stamp method undoes its last preview every frame, and with an epoch
    /// that outlives strokes the free plane cannot simply be reset to `base`: everything the earlier
    /// strokes put there is still owed ([`PainterTool::restore_region`]).
    pub(crate) preview_patch: Option<(Region, Vec<u8>)>,
    /// The layer the epoch belongs to. A layer switch is a different canvas and a dormant scratch.
    pub(crate) layer: Option<RtLayerId>,
    /// The mask scratch's generation when the epoch began. Editing the scratch ENDS the epoch — else
    /// RAISING the protection would retroactively reveal stroke history that was already buried under it
    /// (the §13.7 gate `lowering_protection_starts_a_new_epoch_not_a_replay_of_history`).
    pub(crate) scratch_gen: u64,
    /// `pixel_clock` right after the epoch's own last write. Anything else that touches canvas pixels goes
    /// through `mark_dirty` → `bump_layer_pixels` and moves the clock, so a mismatch means a **foreign
    /// write** — Fill, an adjustment bake, a Deform, a paste — and the epoch commits rather than projecting
    /// over work it does not own.
    pub(crate) witness: u64,
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
    images: BTreeMap<RtLayerId, Arc<LayerImage>>,
    /// **Impasto** relief per layer (canvas-sized `f32`, signed: `+` lifts paint off the canvas, `-`
    /// carves into it). A sibling of [`Self::images`], and LAZY — a layer with no entry has no relief
    /// and costs nothing, which is every layer until someone ticks Impasto. Unlike the colour, the
    /// ACTIVE layer's height lives here too (there is no `canvas_rgba` equivalent to special-case);
    /// the in-progress stroke's contribution is the separate `PaintState::stroke_height` envelope.
    /// Captured by the structural-undo snapshot, so undoing a stroke restores the relief with it.
    heights: BTreeMap<RtLayerId, Arc<Vec<f32>>>,
    /// **Impasto** paint-coverage per layer (canvas-sized, `0..=255`) — how much of each pixel is PAINT
    /// rather than the paper showing through it. The light needs this and the height is not a substitute:
    /// height is `Depth × coverage`, so it cannot say whether a low value means thin paint or a lot of
    /// paint laid thinly. Without it, the pass shades the paper at the stroke's translucent edge, and on
    /// a white canvas ×1.65 bleaches a pale pink straight to white — the halo Enio photographed
    /// (2026-07-12), which measured HARDER at the edge than at the core.
    covers: BTreeMap<RtLayerId, Arc<Vec<u8>>>,
    /// **Impasto** MATERIAL per layer (canvas-sized `[shine, roughness, metallic, wax]` bytes) — what
    /// the paint at each pixel IS, as opposed to what shape it has ([`Self::heights`]) or how much of it
    /// there is ([`Self::covers`]).
    ///
    /// Per-PIXEL, and that is the whole point: the material is a property of the paint, so it belongs to
    /// the brush that laid it and is baked in with the stroke. A canvas-global Shine (which is what this
    /// replaced) can only ever describe ONE material per painting — glossy oil beside matte gouache was
    /// unrepresentable. Deposited by the SAME stroke that deposits the relief, merged with the paint's
    /// own alpha (`over`, not `max`: material follows whoever is on top), and lazy exactly like its two
    /// siblings — a layer nobody sculpted has no entry and costs nothing.
    ///
    /// A layer with relief but NO material entry (a document from before this existed) reads as
    /// [`ph2d_painter_brush::material::Material::NEUTRAL`], which is the pass as it shaded then.
    mats: BTreeMap<RtLayerId, Arc<Vec<MaterialBytes>>>,
    /// The live **protection session** — see [`GateSession`]. Canvas-shaped, so it lives here beside
    /// [`Self::canvas_rgba`] and the three relief planes rather than in `PaintState` (which holds
    /// settings and per-stroke bookkeeping). `None` whenever nothing is painting through a gate,
    /// which is almost always: an ungated document never allocates it and is byte-identical to before.
    gate: Option<GateSession>,
    /// The mask scratch's content generation — bumped by **every** write to `paint.mask_scratch_rgba`, and
    /// the witness that ends a protection epoch when the artist edits the protection itself. A counter and
    /// not the buffer's pointer, because `Arc::make_mut` on a uniquely-owned scratch writes IN PLACE and
    /// leaves the pointer exactly where it was — the address cannot see the edit (the lesson the audio
    /// editor paid for in ADR-0124, `SampleData::version`).
    mask_scratch_gen: u64,
    /// Cached composite output (non-trivial stacks only), behind an `Arc` so the
    /// bridge drain is zero-copy. Invalidated (`None`) on any layer edit.
    composited: Option<Arc<Vec<u8>>>,
    /// The dirty bbox the LAST `take_preview_arc` recomposed (`Some` for a
    /// partial fast-lane update the bridge may upload as a sub-rect).
    preview_upload_bbox: Option<Region>,
    /// Which arm the LAST `take_preview_arc` took — a `PH2D_PAINT_PERF` diagnostic so the shell can
    /// name the exact cost (the trivial fast lane vs the full O(W×H) composite+light), which the
    /// coarse `is_trivial_stack` flag cannot (a trivial stack still falls to the full arm under
    /// impasto or a live mask scratch). Purely observational.
    last_drain_branch: DrainBranch,
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
    /// Monotonic CONTENT version of the drained preview — bumped once per dirty
    /// `take_preview_arc`, so the shell can detect "the preview changed" without
    /// holding a clone of the canvas `Arc` and comparing pointers. Holding that
    /// clone is what forced `stamp_dabs`' `Arc::make_mut` to copy the WHOLE canvas
    /// every move (16 MiB @ 2048², 64 MiB @ 4096², regardless of a 0.5px dab —
    /// the CPU-bound FPS drop Enio measured). The shell now owns its preview
    /// buffer and keys the upload on this version (ADR-0124's lesson: ask the
    /// version, never the pointer), so the tool stays the sole owner of
    /// `canvas_rgba` and the write is in-place. Never reset — a monotonic counter
    /// so a value seen once is never re-seen.
    preview_version: u64,
    pending_commit: bool,
    /// `true` once any edit (stroke, layer op, adjustment) touched the working canvas/composite since
    /// the last `set_source` bind. The shell uses it to auto-bake the painting back into the sprite
    /// when the selection leaves the bound sprite or the tool deactivates, so work never silently
    /// vanishes (Enio 2026-06-24). Cleared on `set_source` / `mark_baked` / `deactivate`.
    edited_since_bind: bool,
    /// Set by `on_deactivate` when it KEEPS the working canvas (there were unbaked edits) instead of
    /// tearing it down, so the shell can bake it back to the sprite before finishing the teardown.
    deferred_bake: bool,
    /// Structural (layer-model) undo/redo. Driven by the layer ops in
    /// [`crate::tool`] via `commit_structural_edit`.
    undo: crate::undo::UndoController,
    /// Which panel occupies the shared right-dock slot (the layers panel header
    /// toggle). Read by the shell `painter_bridge` to drive `panel_visibility`.
    dock_shows_layers: bool,
    /// Accumulated dirty bbox for the partial-recompose fast lane (`take_preview_arc`).
    dirty_rect: Option<Region>,
    /// **Só em teste:** os rects que o [`Self::mark_dirty`] recebeu, na ordem, sem união.
    ///
    /// A sonda do over-claim (`measure_dirty_overclaim.rs`) precisa da reivindicação REAL — o que os
    /// chamadores de fato marcam — e ela não é recuperável do `dirty_rect`, que já os uniu. Foi
    /// exatamente esta lista que mostrou que a caixa não é o problema: um pincel de 24 px marca rects
    /// de **90×54** (o bbox de um SEGMENTO), então a grade de tiles não pode ser mais apertada do que
    /// aquilo que lhe contam. Zero custo no produto (o campo não existe fora de `cfg(test)`).
    #[cfg(test)]
    pub(crate) marks: Vec<Region>,
    /// The region the LAST GPU-lane drain (`take_preview_dirty`) took out of `dirty_rect` — stashed
    /// so the GPU compositor's per-layer provider (`preview_layer_pixels`) can hand it back as
    /// `LayerPixels.dirty`, and the compositor re-uploads only that sub-rect of the changed (active)
    /// layer instead of the whole slice per move. The CPU lane's equivalent is `preview_upload_bbox`;
    /// this is its GPU-lane twin (the GPU lane bypasses `take_preview_arc`).
    preview_dirty_region: Option<Region>,
    /// **A pista GPU perdeu edições que nunca viu** — o bit que obriga a próxima drenagem dela a
    /// declarar *não-confinado*.
    ///
    /// `dirty_rect` é COMPARTILHADO pelas duas pistas e **consumido por quem drena**. Enquanto a CPU
    /// é dona ela o leva para o `preview_upload_bbox` dela, então no frame em que a GPU retoma o
    /// `dirty_rect` descreve só o último frame da CPU — não a era inteira que a GPU não viu. Sem este
    /// bit a retomada confina o trabalho a esse retângulo minúsculo e **os dois** consumidores do
    /// `preview_dirty_region` servem estado velho: o compositor remenda um sub-rect numa fatia
    /// cacheada de antes da era CPU, e o fold do impasto dobra a mesma janela sobre planos da mesma
    /// era. É o retângulo que o Enio fotografou (`PH2D_IMPASTO_SMOKE=2`, 2026-07-25).
    ///
    /// É o ESPELHO EXATO da cura que a Fase D deu ao sentido oposto: lá `take_preview_dirty` derruba
    /// o `composited` (o cache da CPU) para a próxima `take_preview_arc` recompor inteiro. A pista
    /// CPU não tinha como dizer o mesmo, porque o cache da GPU não mora aqui — mora nas texturas do
    /// shell. Este bool é essa frase, e é a única forma dela que não pode ser esquecida por
    /// enumeração: **toda** drenagem da CPU o levanta, **toda** drenagem da GPU o consome.
    gpu_lane_stale: bool,
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
    /// The sprite the working document is currently bound to (`bind_document`), so switching sprites can
    /// stash THIS document's layers by id before binding the next. `None` until the first bind.
    bound_doc: Option<u64>,
    /// Stashed multi-layer documents by sprite id — switching sprites preserves each sprite's layer stack
    /// instead of flattening it. See [`crate::tool::documents`].
    doc_cache: BTreeMap<u64, documents::StashedDoc>,
    /// The sprite id a multi-layer **Shape** was captured from ("Use Document Layers"), so editing that
    /// same sprite re-captures the Shape automatically (preserving the per-layer colours) instead of
    /// needing a manual re-capture. `None` until a multi-layer Shape is captured.
    shape_source_doc: Option<u64>,
    /// The Shape source's content revision (`layers_revision` + `pixel_clock`) at capture time. Any edit
    /// to the active source — paint, opacity, visibility, structure, undo/redo — bumps one of those, so a
    /// per-frame compare detects the change and re-captures (covering edits that aren't paint strokes).
    shape_source_revision: u64,
    /// `true` when the STASHED Shape source sprite needs its live-preview composite re-rendered (it was
    /// just stashed, or a brush opacity/blend remote-control edited its layers) — so the bridge re-uploads
    /// its preview slot. Lets a non-selected shape-source sprite reflect brush edits in real time.
    shape_source_preview_dirty: bool,
    /// The layer-system mask whose GRAYSCALE the canvas shows (its row's "view" eye is open); `None` (the
    /// default) shows the masked effect. View-only, transient (not saved). See `layers::mask_view`.
    mask_view_grayscale: Option<RtLayerId>,
}

/// Which arm the last preview drain ([`PainterTool::take_preview_arc`]) took. A `PH2D_PAINT_PERF`
/// diagnostic (see [`PainterTool::last_drain_branch`]) — the cost profile is entirely different per
/// arm, and the shell's `trivial`/`mask` flags cannot distinguish them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DrainBranch {
    /// Not dirty (or empty canvas) — the drain returned `None`. The cheapest frame.
    #[default]
    Idle,
    /// A mask row's grayscale-view eye is open — full grayscale buffer.
    Gray,
    /// The zero-copy trivial fast lane (`Arc::clone(canvas_rgba)`) — O(1).
    TrivialFast,
    /// Partial recompose of a cached composite over the dirty bbox — O(N×bbox), partial upload.
    PartialComposite,
    /// Full recompose + full-canvas impasto light — O(N×W×H), full upload. The expensive arm.
    FullComposite,
}

impl DrainBranch {
    /// A short label for the perf summary line.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            DrainBranch::Idle => "idle",
            DrainBranch::Gray => "gray",
            DrainBranch::TrivialFast => "trivial-fast",
            DrainBranch::PartialComposite => "partial-composite",
            DrainBranch::FullComposite => "FULL-composite",
        }
    }
}

impl Default for PainterTool {
    fn default() -> Self {
        Self {
            params: PainterParams::default(),
            canvas_rgba: Arc::new(Vec::new()),
            layers: LayerStack::new(),
            images: BTreeMap::new(),
            heights: BTreeMap::new(),
            covers: BTreeMap::new(),
            mats: BTreeMap::new(),
            gate: None,
            mask_scratch_gen: 0,
            composited: None,
            preview_upload_bbox: None,
            last_drain_branch: DrainBranch::Idle,
            layers_revision: 0,
            layer_pixel_versions: BTreeMap::new(),
            pixel_clock: 0,
            source_size: (0, 0),
            preview_dirty: false,
            preview_version: 0,
            pending_commit: false,
            edited_since_bind: false,
            deferred_bake: false,
            undo: crate::undo::UndoController::default(),
            dirty_rect: None,
            #[cfg(test)]
            marks: Vec::new(),
            preview_dirty_region: None,
            gpu_lane_stale: false,
            selection: BTreeSet::new(),
            compositor_cache: CompositorCache::new(),
            adjustment_cache_pending: false,
            // Dock opens on the BRUSH-properties view (Enio 2026-07-04); the header toggle flips to the
            // Layers/Effects view (`dock_shows_layers == true`).
            dock_shows_layers: false,
            paint: paint::PaintState::default(),
            bound_doc: None,
            doc_cache: BTreeMap::new(),
            shape_source_doc: None,
            shape_source_revision: 0,
            shape_source_preview_dirty: false,
            mask_view_grayscale: None,
        }
    }
}

// ── Submodules (god-object split, pure mechanical move) ──
mod documents;
mod internal;
pub(crate) use internal::*;
mod layers;
mod paint;
pub use paint::impasto_gpu::{ImpastoLamp, ImpastoPlanes};
pub mod persist;
// The parametric selection shape list is part of the model snapshot (undo/redo), so `crate::undo` needs
// to name its entry type — re-export it crate-wide without opening the rest of `paint`.
pub use paint::media::PaintMedia;
pub(crate) use paint::selection_shapes::SelectionEntry;
pub use paint::{
    BRUSH_AIRBRUSH_RATE_MAX_S, BRUSH_AIRBRUSH_RATE_MIN_S, BRUSH_COUNT_SLIDER_MAX,
    BRUSH_JITTER_ABS_MAX_PX, BRUSH_SIZE_MAX_PX, BRUSH_SIZE_MIN_PX, BRUSH_SPACING_MAX,
    BrushSettings, CurveOverlay, DEFORM_TEMPERAMENT_NONE, DEFORM_TEMPERAMENT_RESHAPE,
    DEFORM_TEMPERAMENT_TRANSFORM, DeformGizmoView, EllipseOverlay, FilterScope, ImpastoLight,
    LightRig, LineCornerGizmo, LineDimensions, LineOverlay, MAX_IMPASTO_LIGHTS, MAX_SHAPE_LAYERS,
    PANEL_RAMP_STOPS, PolygonOverlay, SelectionGizmoView, StencilOverlay, StencilPreview,
    StrokeOpBadge, TangentHandles, TransformGizmo, WetKnobs, WetTool, brush_falloff_weight_at,
};
mod runtime;
mod trait_impls;
mod trait_impls_raster; // `impl RasterEditTool` split from `trait_impls` (workspace file-LOC cap)
