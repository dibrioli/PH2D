//! Vector Style panel state + the shell→panel Style snapshot.
//!
//! Per-instance state is empty (ADR-0029 §4.3, mirror of Padding): the
//! authoritative Style lives on the shell-side `VectorTool`. Each frame the
//! shell publishes a [`VectorStyleSnapshot`] via [`set_current_vector_style`]
//! BEFORE the panel paints; `paint` reads it to seed the Width chip + the two
//! colour swatches. Edits flow back out over `EditorAction::ToolPanelEvent`
//! (Width slider, Fill-None) and the colour-picker read-back (Stroke / Fill
//! swatches), so the panel holds no authoritative state.

use ph2d_tool_vector::{VectorStyleSnapshot, VertexType};
use std::cell::{Cell, RefCell};

thread_local! {
    /// Live snapshot published by the host before each `paint`. `None` until
    /// the first push (panel paints defaults).
    static CURRENT_SNAPSHOT: RefCell<Option<VectorStyleSnapshot>> = const { RefCell::new(None) };
    /// Type of the currently-selected vertex (published by the shell each frame
    /// from the Pen). `None` = no vertex selected → the Vertex section hides.
    static CURRENT_VERTEX_TYPE: RefCell<Option<VertexType>> = const { RefCell::new(None) };
    /// Selected path's anchor bbox `[x, y, w, h]` (world), published each frame.
    /// `None` = no path selected → the Transform section hides.
    static CURRENT_TRANSFORM: Cell<Option<[f64; 4]>> = const { Cell::new(None) };
    /// Selected path's `closed` flag, published each frame (`None` = no selection).
    /// Drives the Close/Open toggle button's label.
    static CURRENT_PATH_CLOSED: Cell<Option<bool>> = const { Cell::new(None) };
    /// Selected path's fill kind (`None` = no path selected / no fill). Drives the
    /// Fill-type selector highlight + whether the gradient controls show.
    static CURRENT_FILL_KIND: Cell<Option<FillKind>> = const { Cell::new(None) };
    /// Selected path's linear-gradient angle in degrees (`None` unless Linear).
    static CURRENT_GRAD_ANGLE: Cell<Option<f64>> = const { Cell::new(None) };
    /// Selected multi-point gradient point's influence (`None` unless a point is
    /// selected) — drives the Influence slider's visibility + value.
    static CURRENT_GRAD_INFLUENCE: Cell<Option<f64>> = const { Cell::new(None) };
    /// Selected multi-point gradient point's jitter (`None` unless a point is
    /// selected) — drives the Jitter slider's visibility + value.
    static CURRENT_GRAD_JITTER: Cell<Option<f64>> = const { Cell::new(None) };
    /// Number of paths in the object selection — drives the Align (≥2) / Distribute
    /// (≥3) section visibility.
    static CURRENT_SELECTION_COUNT: Cell<usize> = const { Cell::new(0) };
    /// Fill rule of the selected path, `Some` only when it is a COMPOUND path —
    /// the two rules agree on a single contour, so the row would be a no-op there.
    static CURRENT_FILL_RULE: Cell<Option<PathFillRule>> = const { Cell::new(None) };
    /// Whether shape-snapping is on (mirrored from the shell). The GRID toggle
    /// lives in the editor's universal Grid Snap panel, not here.
    static CURRENT_SNAP: Cell<bool> = const { Cell::new(true) };
    /// "Set Center" armado: a próxima pressão no canvas reposiciona a origem.
    static CURRENT_PIVOT_EDIT: Cell<bool> = const { Cell::new(false) };
    /// Texto da sessão de edição ativa (modo Text). `None` = sem sessão. Só
    /// LEITURA no painel (display); a digitação segue no canvas (A2). Publicado
    /// pela shell a cada frame.
    static CURRENT_TEXT: RefCell<Option<String>> = const { RefCell::new(None) };
    /// Rotation-field accumulator: the angle (degrees) the Angle chip last
    /// reported THIS gesture. `event` emits the DELTA `(current − this)` so the
    /// shell rotates incrementally; reset to 0 by `paint` whenever the field is
    /// unfocused (gesture ended), so the shell stays stateless.
    static ROT_LAST: Cell<f64> = const { Cell::new(0.0) };
    /// Last measured scrollable content height (set by `paint`).
    static LAST_CONTENT_H: Cell<f32> = const { Cell::new(0.0) };
    /// Last visible body height (panel rect minus title + paddings).
    static LAST_VISIBLE_H: Cell<f32> = const { Cell::new(0.0) };
}

/// Which kind of fill the selected path has (published by the shell each frame so
/// the panel's Fill-type selector reflects + drives it). Mirror of the scene
/// `Paint` variants (kept panel-local so the panel needn't depend on the scene).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FillKind {
    Solid,
    Linear,
    Radial,
    MultiPoint,
}

/// Fill rule of a compound path — which nested contours are holes. Mirror of the
/// scene `FillRule` (kept panel-local so the panel needn't depend on the scene).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PathFillRule {
    NonZero,
    EvenOdd,
}

/// Retained per-instance state slot for `VectorPanel`. Intentionally empty —
/// the authoritative Style lives on the shell-side `VectorTool`; the panel
/// renders the per-frame snapshot. `Default` is required by the
/// `Panel::State: Default` bound.
#[derive(Clone, Debug, Default)]
pub struct VectorPanelState;

/// Publish the current Style snapshot. Called by the shell once per frame while
/// the `vector` tool is active; pass `None` to clear (tool inactive).
pub fn set_current_vector_style(snapshot: Option<VectorStyleSnapshot>) {
    CURRENT_SNAPSHOT.with(|c| *c.borrow_mut() = snapshot);
}

/// Read the snapshot the host published this frame, falling back to
/// [`VectorStyleSnapshot::default`] when none was pushed.
pub(crate) fn current_snapshot() -> VectorStyleSnapshot {
    CURRENT_SNAPSHOT.with(|c| c.borrow().unwrap_or_default())
}

/// Publish the selected vertex's type (or `None` when no vertex is selected).
/// Called by the shell each frame while the `vector` tool is active.
pub fn set_selected_vertex_type(kind: Option<VertexType>) {
    CURRENT_VERTEX_TYPE.with(|c| *c.borrow_mut() = kind);
}

/// The selected vertex's type this frame (`None` ⇒ hide the Vertex section).
pub(crate) fn current_vertex_type() -> Option<VertexType> {
    CURRENT_VERTEX_TYPE.with(|c| *c.borrow())
}

/// Publish the selected path's anchor bbox `[x, y, w, h]` (world), or `None`.
/// Called by the shell each frame while the `vector` tool is active.
pub fn set_current_transform(bbox: Option<[f64; 4]>) {
    CURRENT_TRANSFORM.with(|c| c.set(bbox));
}

/// The selected path's bbox this frame (`None` ⇒ hide the Transform section).
pub(crate) fn current_transform() -> Option<[f64; 4]> {
    CURRENT_TRANSFORM.with(|c| c.get())
}

/// Publish the selected path's `closed` flag (or `None` when no path is selected).
pub fn set_current_path_closed(closed: Option<bool>) {
    CURRENT_PATH_CLOSED.with(|c| c.set(closed));
}

/// The selected path's `closed` flag this frame (drives the toggle button label).
pub(crate) fn current_path_closed() -> Option<bool> {
    CURRENT_PATH_CLOSED.with(|c| c.get())
}

/// Publish the selected path's fill kind + linear angle (both `None` when no path
/// is selected or it has no fill / isn't linear).
pub fn set_current_fill(kind: Option<FillKind>, angle_deg: Option<f64>) {
    CURRENT_FILL_KIND.with(|c| c.set(kind));
    CURRENT_GRAD_ANGLE.with(|c| c.set(angle_deg));
}

/// The selected path's fill kind this frame (`None` ⇒ hide the Fill-type selector).
pub(crate) fn current_fill_kind() -> Option<FillKind> {
    CURRENT_FILL_KIND.with(|c| c.get())
}

/// The selected path's linear-gradient angle this frame (`None` unless Linear).
pub(crate) fn current_grad_angle() -> Option<f64> {
    CURRENT_GRAD_ANGLE.with(|c| c.get())
}

/// Publish the selected multi-point gradient point's influence (`None` = no point).
pub fn set_current_grad_influence(v: Option<f64>) {
    CURRENT_GRAD_INFLUENCE.with(|c| c.set(v));
}

/// The selected multi-point point's influence this frame (drives the slider).
pub(crate) fn current_grad_influence() -> Option<f64> {
    CURRENT_GRAD_INFLUENCE.with(|c| c.get())
}

/// Publish the selected multi-point gradient point's jitter (`None` = no point).
pub fn set_current_grad_jitter(v: Option<f64>) {
    CURRENT_GRAD_JITTER.with(|c| c.set(v));
}

/// The selected multi-point point's jitter this frame (drives the slider).
pub(crate) fn current_grad_jitter() -> Option<f64> {
    CURRENT_GRAD_JITTER.with(|c| c.get())
}

/// Publish the number of paths in the object selection (drives Align/Distribute).
pub fn set_current_selection_count(n: usize) {
    CURRENT_SELECTION_COUNT.with(|c| c.set(n));
}

/// The object-selection path count this frame (≥2 shows Align, ≥3 shows Distribute).
pub(crate) fn current_selection_count() -> usize {
    CURRENT_SELECTION_COUNT.with(|c| c.get())
}

/// Publish the selected path's fill rule — `None` unless it is a compound path
/// (the Fill Rule row hides otherwise, since both rules would paint the same).
pub fn set_current_fill_rule(rule: Option<PathFillRule>) {
    CURRENT_FILL_RULE.with(|c| c.set(rule));
}

/// The selected compound path's fill rule this frame (`None` = not compound).
pub(crate) fn current_fill_rule() -> Option<PathFillRule> {
    CURRENT_FILL_RULE.with(|c| c.get())
}

/// Publish whether shape-snapping is on, so the Snap section reflects it.
pub fn set_current_snap(on: bool) {
    CURRENT_SNAP.with(|c| c.set(on));
}

/// Whether shape-snapping is on this frame.
pub(crate) fn current_snap() -> bool {
    CURRENT_SNAP.with(|c| c.get())
}

/// The angle the Angle chip last reported this gesture (for the delta emit).
pub(crate) fn rot_last() -> f64 {
    ROT_LAST.with(Cell::get)
}

/// Record the Angle chip's current value as the gesture baseline (or reset to 0
/// between gestures).
pub(crate) fn set_rot_last(v: f64) {
    ROT_LAST.with(|c| c.set(v));
}

#[must_use]
pub fn last_content_h() -> f32 {
    LAST_CONTENT_H.with(|c| c.get())
}

#[must_use]
pub fn last_visible_h() -> f32 {
    LAST_VISIBLE_H.with(|c| c.get())
}

pub(crate) fn set_last_content_h(v: f32) {
    LAST_CONTENT_H.with(|c| c.set(v));
}

pub(crate) fn set_last_visible_h(v: f32) {
    LAST_VISIBLE_H.with(|c| c.set(v));
}

/// Publica se "Set Center" está armado (a shell manda; o painel só reflete).
pub fn set_current_pivot_edit(armed: bool) {
    CURRENT_PIVOT_EDIT.with(|c| c.set(armed));
}

pub(crate) fn pivot_edit_armed() -> bool {
    CURRENT_PIVOT_EDIT.with(|c| c.get())
}

/// Publica o texto da sessão de edição ativa (`None` = sem sessão de texto).
/// A shell chama a cada frame; o painel só o exibe (read-only na A2).
pub fn set_current_text(text: Option<String>) {
    CURRENT_TEXT.with(|c| *c.borrow_mut() = text);
}

/// O texto da sessão ativa este frame (`None` ⇒ nada a exibir).
pub(crate) fn current_text() -> Option<String> {
    CURRENT_TEXT.with(|c| c.borrow().clone())
}
