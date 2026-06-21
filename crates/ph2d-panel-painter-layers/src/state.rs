//! Painter layers panel state.
//!
//! Espelha o padrão `PainterSidebarPanelState` (ADR-0029 §4.3 + ADR-0040
//! TG-B):
//! - `PainterLayersPanelState` é struct vazia — sem state autoritativo
//!   próprio. O `LayerStack` canônico vive no `PainterTool` (shell-side
//!   ToolRegistry); o panel renderiza um snapshot per-frame.
//! - O shell publica o snapshot das layers via [`set_current_layers`]
//!   ANTES de paint; o paint lê via [`current_layers`] e renderiza as rows.
//! - Eventos sairão via `EditorAction::ToolPanelEvent` (canal genérico
//!   ADR-0040 TG-B) — shell roteia pra `PainterTool::handle_panel_event`.
//!
//! **SCAFFOLD:** o snapshot é o próprio `LayerStack` (já tem `Clone`+`serde`
//! na fundação T3.1). Quando o Implementador preencher as layer rows, ele
//! lê `current_layers()` em `paint`. Se um snapshot mais enxuto (só os
//! campos exibidos: id/name/visible/opacity/blend/thumb-handle) for melhor
//! pra perf, troque o tipo aqui — `set_current_layers`/`current_layers` são
//! os únicos pontos de contato com o shell.

use ph2d_editor_core::zones::Rect;
use ph2d_tool_painter::{BrushSettings, LayerId, LayerStack};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

thread_local! {
    /// Snapshot do `LayerStack` publicado pelo host antes de cada `paint`.
    /// `None` até o primeiro push (panel pinta o placeholder "No layers").
    static CURRENT_LAYERS: RefCell<Option<LayerStack>> = const { RefCell::new(None) };

    /// Snapshot do brush ativo publicado pelo host antes de cada `paint` (mirror
    /// de [`CURRENT_LAYERS`]). `None` até o primeiro push (Brush section usa o
    /// default). `Copy`, então o clone é trivial.
    static CURRENT_BRUSH: Cell<Option<BrushSettings>> = const { Cell::new(None) };

    /// Dock-view mode published per-frame: `true` = Layers/Effects body, `false`
    /// = Brush-properties body. The header toggle flips the tool's
    /// `dock_shows_layers`; the bridge publishes it here and `paint` branches.
    /// Defaults to Layers so a fresh frame (pre-publish) shows the main view.
    static CURRENT_DOCK_SHOWS_LAYERS: Cell<bool> = const { Cell::new(true) };

    /// The open brush blend-mode dropdown to render as a deferred popover on top
    /// of the body (after the clip pops): `(chip_rect, current_mode_u8)`. Set
    /// during the Brush section paint, drained at the end of `paint`. Mirror of
    /// [`PENDING_BLEND_DD`] but for the single fixed brush chip.
    static PENDING_BRUSH_BLEND_DD: Cell<Option<(Rect, u8)>> = const { Cell::new(None) };

    /// The open brush Falloff dropdown popover: `(chip_rect, current_preset_u8)`.
    /// Mirror of [`PENDING_BRUSH_BLEND_DD`] for the Falloff section chip.
    static PENDING_BRUSH_FALLOFF_DD: Cell<Option<(Rect, u8)>> = const { Cell::new(None) };

    /// Multi-selection set published by the bridge each frame (W3 multi-select):
    /// the layer rows the panel highlights. Always includes the active layer
    /// (the tool folds it in via `selection()`); a single-element set means just
    /// the active row, so no extra "also-selected" wash is drawn for it.
    static CURRENT_SELECTION: RefCell<BTreeSet<LayerId>> =
        const { RefCell::new(BTreeSet::new()) };

    /// Última altura de conteúdo scrollable medida (set por paint, lido
    /// pelo orchestrator content_h publish). Paridade com o sidebar.
    static LAST_CONTENT_H: Cell<f32> = const { Cell::new(0.0) };
    /// Última altura visível do body (panel rect minus title + paddings).
    static LAST_VISIBLE_H: Cell<f32> = const { Cell::new(0.0) };

    /// The single open blend-mode dropdown to render as a deferred popover on
    /// top of the rows (after the body clip pops): `(layer_id, chip_rect,
    /// current_mode_u8)`. Set during row paint, drained at the end of `paint`.
    /// Enforces one-open-at-a-time. `(u64, Rect, u8)` is `Copy`.
    static PENDING_BLEND_DD: Cell<Option<(u64, Rect, u8)>> = const { Cell::new(None) };

    /// The open "+ Adjustment" kind-picker menu, as the synthetic chip `Rect`
    /// the deferred popover anchors to (full content width, at the action-toolbar
    /// row). Set when the `+ Adj` dropdown's store state is open; drained at the
    /// end of `paint` to render the 24-kind list on top of everything (mirror of
    /// [`PENDING_BLEND_DD`]). W4 T4.15.
    static PENDING_ADJ_MENU: Cell<Option<Rect>> = const { Cell::new(None) };

    /// Active curve-editor channel TAB per Curves layer (W4 §3): `0` = master
    /// (RGB), `1` = R, `2` = G, `3` = B. Pure VIEW state (which channel the curve
    /// canvas shows + edits) — it never goes to the tool; `set_curve_point` carries
    /// the channel. Keyed by layer id so multiple expanded Curves layers each keep
    /// their own tab. Default (absent) = master.
    static ACTIVE_CURVE_CHANNEL: RefCell<BTreeMap<u64, u8>> = const { RefCell::new(BTreeMap::new()) };

    /// Last-interacted curve control point `(layer, channel, index)` — set when a
    /// handle is dragged (W4 §3), read by the "−" (remove) button so it knows which
    /// point to drop. `None` until a handle is touched.
    static SELECTED_CURVE_POINT: Cell<Option<(u64, u8, usize)>> = const { Cell::new(None) };

    /// Last-interacted brush Custom-falloff control point — its STABLE id (not a
    /// sorted index, so it survives a drag-past-neighbour re-sort). Set on drag /
    /// right-click / click-add; read by the Falloff "−" button + the Delete key +
    /// the highlight. Tool-global (the brush is not per-layer).
    static SELECTED_FALLOFF_POINT: Cell<Option<u8>> = const { Cell::new(None) };

    /// Screen geometry of the Falloff curve graph, published each frame by
    /// [`crate::paint_falloff`] so the shell can hit-test a right-click (open the
    /// handle menu) / left-click (add a point) against it — the panel knows the
    /// canvas rect + handle positions; the shell only has the cursor.
    static FALLOFF_GEOM: Cell<Option<FalloffGeom>> = const { Cell::new(None) };

    /// Active output-channel TAB per Channel-Mixer layer (W4 BATCH-1): `0` = Red
    /// (or Gray when monochrome), `1` = Green, `2` = Blue. Pure VIEW state (which
    /// output row the 4 weight sliders edit) — it never goes to the tool; the
    /// slider edit carries the channel in its `PAINTER_MIXER_EDIT` payload. Keyed
    /// by layer id. Default (absent) = Red.
    static ACTIVE_MIXER_CHANNEL: RefCell<BTreeMap<u64, u8>> = const { RefCell::new(BTreeMap::new()) };

    /// Active color-group bucket TAB per Selective-Color layer (W4 BATCH-2):
    /// `0..9` = Reds … Blacks. Pure VIEW state (which group the 4 CMYK sliders
    /// edit) — the slider edit carries the bucket in its `PAINTER_SELCOLOR_EDIT`
    /// payload. Keyed by layer id. Default (absent) = Reds.
    static ACTIVE_SELECTIVE_BUCKET: RefCell<BTreeMap<u64, u8>> = const { RefCell::new(BTreeMap::new()) };

    /// Selected gradient stop per Gradient-Map layer (W4 BATCH-2): which stop the
    /// RGB color sliders edit + the "−" button removes. Set when a stop handle is
    /// dragged/clicked. Keyed by layer id. Default (absent) = the first stop.
    static SELECTED_GRADIENT_STOP: RefCell<BTreeMap<u64, usize>> = const { RefCell::new(BTreeMap::new()) };
}

/// Set the active output-channel tab for `layer`'s Channel Mixer (W4 BATCH-1).
pub(crate) fn set_active_mixer_channel(layer: u64, channel: u8) {
    ACTIVE_MIXER_CHANNEL.with(|c| {
        c.borrow_mut().insert(layer, channel);
    });
}

/// The active output-channel tab for `layer`'s Channel Mixer (default `0` = Red).
pub(crate) fn active_mixer_channel(layer: u64) -> u8 {
    ACTIVE_MIXER_CHANNEL.with(|c| c.borrow().get(&layer).copied().unwrap_or(0))
}

/// Set the active color-group bucket tab for `layer`'s Selective Color (W4 BATCH-2).
pub(crate) fn set_active_selective_bucket(layer: u64, bucket: u8) {
    ACTIVE_SELECTIVE_BUCKET.with(|c| {
        c.borrow_mut().insert(layer, bucket);
    });
}

/// The active color-group bucket for `layer`'s Selective Color (default `0` = Reds).
pub(crate) fn active_selective_bucket(layer: u64) -> u8 {
    ACTIVE_SELECTIVE_BUCKET.with(|c| c.borrow().get(&layer).copied().unwrap_or(0))
}

/// Set the selected gradient stop for `layer`'s Gradient Map editor (W4 BATCH-2).
pub(crate) fn set_selected_gradient_stop(layer: u64, stop: usize) {
    SELECTED_GRADIENT_STOP.with(|c| {
        c.borrow_mut().insert(layer, stop);
    });
}

/// The selected gradient stop for `layer`'s Gradient Map (default `0` = first stop).
pub(crate) fn selected_gradient_stop(layer: u64) -> usize {
    SELECTED_GRADIENT_STOP.with(|c| c.borrow().get(&layer).copied().unwrap_or(0))
}

/// Set the active channel tab for `layer`'s curve editor (W4 §3).
pub(crate) fn set_active_curve_channel(layer: u64, channel: u8) {
    ACTIVE_CURVE_CHANNEL.with(|c| {
        c.borrow_mut().insert(layer, channel);
    });
}

/// The active channel tab for `layer`'s curve editor (default `0` = master).
pub(crate) fn active_curve_channel(layer: u64) -> u8 {
    ACTIVE_CURVE_CHANNEL.with(|c| c.borrow().get(&layer).copied().unwrap_or(0))
}

/// Record the last-interacted curve point (for the remove button).
pub(crate) fn set_selected_curve_point(v: Option<(u64, u8, usize)>) {
    SELECTED_CURVE_POINT.with(|c| c.set(v));
}

/// The last-interacted curve point `(layer, channel, index)`, if any.
pub(crate) fn selected_curve_point() -> Option<(u64, u8, usize)> {
    SELECTED_CURVE_POINT.with(|c| c.get())
}

/// Record the selected brush Custom-falloff point by its stable id (for the "−"
/// button, the Delete key, and the highlight). `pub` so the shell can set it on
/// a right-click / click-add.
pub fn set_selected_falloff_point(v: Option<u8>) {
    SELECTED_FALLOFF_POINT.with(|c| c.set(v));
}

/// The selected brush Custom-falloff point's stable id, if any. `pub` so the
/// shell's Delete key can drop it.
#[must_use]
pub fn selected_falloff_point() -> Option<u8> {
    SELECTED_FALLOFF_POINT.with(|c| c.get())
}

/// Max control points the Falloff geometry snapshot carries (mirrors
/// `ph2d_painter_brush::MAX_FALLOFF_POINTS`).
const FALLOFF_GEOM_MAX: usize = 8;

/// Screen geometry of the Falloff curve graph for the shell's hit-test: the
/// plotting canvas rect, each visible control point's stable id + screen centre,
/// and the grab radius. Only meaningful while the Custom preset is shown
/// ([`Self::active`]).
#[derive(Clone, Copy, Debug)]
pub struct FalloffGeom {
    canvas: Rect,
    points: [(u8, f32, f32); FALLOFF_GEOM_MAX],
    len: usize,
    grab_r: f32,
    active: bool,
}

/// Result of hit-testing the cursor against the published Falloff geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FalloffHit {
    /// Over a control point (within the grab radius) — its stable id.
    Point(u8),
    /// Over the empty canvas at normalized `(distance, strength)` in `[0,1]²`.
    Canvas(f32, f32),
    /// Not over the editable Falloff graph at all.
    Outside,
}

/// Publish the Falloff graph's screen geometry for the shell (called once per
/// frame by [`crate::paint_falloff`]). `points` is `(stable_id, cx, cy)` per
/// visible handle; `active` is `true` only while the Custom preset is shown.
pub(crate) fn set_falloff_geom(canvas: Rect, points: &[(u8, f32, f32)], grab_r: f32, active: bool) {
    let mut arr = [(0u8, 0.0_f32, 0.0_f32); FALLOFF_GEOM_MAX];
    let len = points.len().min(FALLOFF_GEOM_MAX);
    arr[..len].copy_from_slice(&points[..len]);
    FALLOFF_GEOM.with(|c| {
        c.set(Some(FalloffGeom {
            canvas,
            points: arr,
            len,
            grab_r,
            active,
        }))
    });
}

/// Hit-test a screen point against the published Falloff geometry: a control
/// point (within grab radius), the empty canvas (normalized position), or
/// outside. `Outside` whenever the Custom preset is not currently shown. `pub`
/// so the shell's right-click / left-click handlers can route to the tool.
#[must_use]
pub fn falloff_hit_test(px: f32, py: f32) -> FalloffHit {
    // The graph only exists in the Brush-properties view; in Layers mode the
    // published geometry is stale, so never hit-test it there.
    if current_dock_shows_layers() {
        return FalloffHit::Outside;
    }
    let Some(g) = FALLOFF_GEOM.with(Cell::get) else {
        return FalloffHit::Outside;
    };
    if !g.active {
        return FalloffHit::Outside;
    }
    // Nearest handle within the grab radius wins (handles sit on top of the canvas).
    let r2 = g.grab_r * g.grab_r;
    let mut best: Option<(u8, f32)> = None;
    for &(id, cx, cy) in &g.points[..g.len] {
        let d2 = (px - cx) * (px - cx) + (py - cy) * (py - cy);
        if d2 <= r2 && best.is_none_or(|(_, bd)| d2 < bd) {
            best = Some((id, d2));
        }
    }
    if let Some((id, _)) = best {
        return FalloffHit::Point(id);
    }
    let c = g.canvas;
    if px >= c.x && px <= c.x + c.w && py >= c.y && py <= c.y + c.h && c.w > 0.0 && c.h > 0.0 {
        let nx = ((px - c.x) / c.w).clamp(0.0, 1.0);
        let ny = (1.0 - (py - c.y) / c.h).clamp(0.0, 1.0);
        return FalloffHit::Canvas(nx, ny);
    }
    FalloffHit::Outside
}

/// Stash the open "+ Adjustment" kind menu for the deferred popover pass.
pub(crate) fn set_pending_adj_menu(v: Option<Rect>) {
    PENDING_ADJ_MENU.with(|c| c.set(v));
}

/// Take (and clear) the pending "+ Adjustment" kind menu for the deferred paint.
pub(crate) fn take_pending_adj_menu() -> Option<Rect> {
    PENDING_ADJ_MENU.with(|c| c.take())
}

/// Stash the open blend dropdown for the deferred popover pass.
pub(crate) fn set_pending_blend_dd(v: Option<(u64, Rect, u8)>) {
    PENDING_BLEND_DD.with(|c| c.set(v));
}

/// Peek the pending blend dropdown (non-consuming) — used to enforce a single
/// open popover when more than one dropdown's store state says "open".
pub(crate) fn pending_blend_dd() -> Option<(u64, Rect, u8)> {
    PENDING_BLEND_DD.with(|c| c.get())
}

/// Take (and clear) the pending blend dropdown for the deferred popover paint.
pub(crate) fn take_pending_blend_dd() -> Option<(u64, Rect, u8)> {
    PENDING_BLEND_DD.with(|c| c.take())
}

/// State per-instance retido do `PainterLayersPanel`. Vazio
/// intencionalmente — o `LayerStack` canônico vive no PainterTool
/// shell-side; o panel renderiza o snapshot per-frame. `Default` exigido
/// pelo bound `Panel::State: Default`.
#[derive(Clone, Debug, Default)]
pub struct PainterLayersPanelState;

/// Publica o snapshot atual do `LayerStack`. Chamado pelo shell uma vez
/// por frame quando o `painter` tool é ativo; pass `None` pra limpar
/// (panel volta ao placeholder).
pub fn set_current_layers(stack: Option<LayerStack>) {
    CURRENT_LAYERS.with(|c| *c.borrow_mut() = stack);
}

/// Lê o snapshot publicado pelo host neste frame. `None` quando o host
/// ainda não pushou (boot, ou Painter inativo) — `paint` pinta o
/// placeholder "No layers".
pub(crate) fn current_layers() -> Option<LayerStack> {
    CURRENT_LAYERS.with(|c| c.borrow().clone())
}

/// Publica o snapshot do brush ativo. Chamado pelo shell uma vez por frame
/// quando o `painter` tool é ativo (mirror de [`set_current_layers`]); pass
/// `None` pra limpar.
pub fn set_current_brush(brush: Option<BrushSettings>) {
    CURRENT_BRUSH.with(|c| c.set(brush));
}

/// Lê o snapshot do brush publicado neste frame (`None` antes do 1º push — a
/// Brush section então usa o `BrushSettings` default).
pub(crate) fn current_brush() -> Option<BrushSettings> {
    CURRENT_BRUSH.with(Cell::get)
}

/// Publica o modo do dock (`true` = Layers/Effects, `false` = Brush). Chamado
/// pelo shell por frame quando o `painter` é ativo.
pub fn set_current_dock_shows_layers(shows_layers: bool) {
    CURRENT_DOCK_SHOWS_LAYERS.with(|c| c.set(shows_layers));
}

/// Lê o modo do dock publicado neste frame (default `true` = Layers).
pub(crate) fn current_dock_shows_layers() -> bool {
    CURRENT_DOCK_SHOWS_LAYERS.with(Cell::get)
}

/// Stash the open brush blend dropdown for the deferred popover pass.
pub(crate) fn set_pending_brush_blend_dd(v: Option<(Rect, u8)>) {
    PENDING_BRUSH_BLEND_DD.with(|c| c.set(v));
}

/// Take (and clear) the pending brush blend dropdown for the deferred popover paint.
pub(crate) fn take_pending_brush_blend_dd() -> Option<(Rect, u8)> {
    PENDING_BRUSH_BLEND_DD.with(|c| c.take())
}

/// Stash the open brush Falloff dropdown for the deferred popover pass.
pub(crate) fn set_pending_brush_falloff_dd(v: Option<(Rect, u8)>) {
    PENDING_BRUSH_FALLOFF_DD.with(|c| c.set(v));
}

/// Take (and clear) the pending brush Falloff dropdown for the deferred popover.
pub(crate) fn take_pending_brush_falloff_dd() -> Option<(Rect, u8)> {
    PENDING_BRUSH_FALLOFF_DD.with(|c| c.take())
}

/// Publica o set de multi-seleção atual (W3). Chamado pelo shell uma vez por
/// frame quando o `painter` tool é ativo (mirror de [`set_current_layers`]).
pub fn set_current_selection(sel: BTreeSet<LayerId>) {
    CURRENT_SELECTION.with(|c| *c.borrow_mut() = sel);
}

/// Lê o set de multi-seleção publicado neste frame — as rows a destacar.
pub(crate) fn current_selection() -> BTreeSet<LayerId> {
    CURRENT_SELECTION.with(|c| c.borrow().clone())
}

pub fn last_content_h() -> f32 {
    LAST_CONTENT_H.with(|c| c.get())
}

pub fn last_visible_h() -> f32 {
    LAST_VISIBLE_H.with(|c| c.get())
}

pub(crate) fn set_last_content_h(v: f32) {
    LAST_CONTENT_H.with(|c| c.set(v));
}

pub(crate) fn set_last_visible_h(v: f32) {
    LAST_VISIBLE_H.with(|c| c.set(v));
}
