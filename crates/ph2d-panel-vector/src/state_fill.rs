//! **Com que TINTA a forma aparece** — a metade do estado do painel que fala de preenchimento,
//! irmã de [`super::state`] pelo teto de 600 LOC (HR-18).
//!
//! O corte é por ASSUNTO, e é o mesmo que a `ph2d-vec-scene` já fez no `lib.rs` desta linha:
//! *com que tinta a forma aparece* × *o que a forma É*. O pai fica com **onde** a forma está e
//! **o que** ela é (a bbox, os vértices, a unidade, o pivô); aqui mora o tipo de preenchimento,
//! o ângulo do gradiente linear, os dois números do ponto de gradiente selecionado e a regra de
//! preenchimento do caminho composto.
//!
//! ⚠️ **Irmão e não filho**, ao contrário do `state_text`: estas seis portas são `pub` para a
//! shell as chamar, então elas re-exportam pela raiz do crate como sempre fizeram — quem as
//! chamava não muda uma linha.

use super::{FillKind, PathFillRule};
use std::cell::Cell;

thread_local! {
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
    /// Fill rule of the selected path, `Some` only when it is a COMPOUND path —
    /// the two rules agree on a single contour, so the row would be a no-op there.
    static CURRENT_FILL_RULE: Cell<Option<PathFillRule>> = const { Cell::new(None) };
}

/// Publish the selected path's fill kind + linear angle (both `None` when no path
/// is selected or it has no fill / isn't linear).
pub fn set_current_fill(kind: Option<FillKind>, angle_deg: Option<f64>) {
    CURRENT_FILL_KIND.with(|c| c.set(kind));
    CURRENT_GRAD_ANGLE.with(|c| c.set(angle_deg));
}

/// The selected path's fill kind this frame (`None` ⇒ hide the Fill-type selector).
pub(crate) fn current_fill_kind() -> Option<FillKind> {
    CURRENT_FILL_KIND.with(Cell::get)
}

/// The selected path's linear-gradient angle this frame (`None` unless Linear).
pub(crate) fn current_grad_angle() -> Option<f64> {
    CURRENT_GRAD_ANGLE.with(Cell::get)
}

/// Publish the selected multi-point gradient point's influence (`None` = no point).
pub fn set_current_grad_influence(v: Option<f64>) {
    CURRENT_GRAD_INFLUENCE.with(|c| c.set(v));
}

/// The selected multi-point point's influence this frame (drives the slider).
pub(crate) fn current_grad_influence() -> Option<f64> {
    CURRENT_GRAD_INFLUENCE.with(Cell::get)
}

/// Publish the selected multi-point gradient point's jitter (`None` = no point).
pub fn set_current_grad_jitter(v: Option<f64>) {
    CURRENT_GRAD_JITTER.with(|c| c.set(v));
}

/// The selected multi-point point's jitter this frame (drives the slider).
pub(crate) fn current_grad_jitter() -> Option<f64> {
    CURRENT_GRAD_JITTER.with(Cell::get)
}

/// Publish the selected path's fill rule — `None` unless it is a compound path
/// (the Fill Rule row hides otherwise, since both rules would paint the same).
pub fn set_current_fill_rule(rule: Option<PathFillRule>) {
    CURRENT_FILL_RULE.with(|c| c.set(rule));
}

/// The selected compound path's fill rule this frame (`None` = not compound).
pub(crate) fn current_fill_rule() -> Option<PathFillRule> {
    CURRENT_FILL_RULE.with(Cell::get)
}
