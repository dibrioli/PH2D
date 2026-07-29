//! Event dispatch for the two **text-channel shaper editors** — the Curve (A1) and the
//! Gradient (doc 85). Both author a `String` in a text param through draggable handles +
//! `+`/`−`/interp buttons, so their `apply_event` routing is the same shape; split out of
//! `lib.rs` for the 700-LOC panel-file cap. `super` is the crate root (the pooled-id helpers
//! and the row types are in scope).

use crate::snapshot::{
    MAX_PARAM_ROWS, MotionParamIntent, ParamRow, ParamsSnapshot, param_curve_add_id,
    param_curve_editor_id, param_curve_interp_id, param_curve_remove_id, param_grad_add_id,
    param_grad_editor_id, param_grad_interp_id, param_grad_remove_id, push_param_intent,
};
use crate::{curve_row, gradient_row};
use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};

/// A Curve handle drag: the dispatch stashed the normalized `(index, x, y)` in the store's
/// `curve_point_drag` slot; fold it into the row's curve and emit the new serialized value.
/// `Some` = the id WAS a Curve editor (handled), `None` = try the scalar path.
pub(crate) fn on_curve_drag(
    id: NodeId,
    host: &mut dyn PanelHostInternal,
    snap: &ParamsSnapshot,
) -> Option<EventOutcome> {
    for slot in 0..snap.rows.len().min(MAX_PARAM_ROWS) {
        if id == param_curve_editor_id(slot) {
            let ParamRow::Curve(row) = &snap.rows[slot] else {
                return Some(EventOutcome::Ignored);
            };
            if let Some(value) = curve_row::drain_drag(host.store_mut(), slot, &row.value) {
                push_param_intent(MotionParamIntent::SetTextParam {
                    node: snap.node,
                    param: row.name,
                    value,
                });
            }
            return Some(EventOutcome::Consumed);
        }
    }
    None
}

/// A Curve `+` / `−` / interp button → the new serialized curve. `None` = not a Curve
/// button (fall through to the enum / seed-reroll handler).
pub(crate) fn on_curve_click(id: NodeId, snap: &ParamsSnapshot) -> Option<EventOutcome> {
    for slot in 0..snap.rows.len().min(MAX_PARAM_ROWS) {
        let ParamRow::Curve(row) = &snap.rows[slot] else {
            continue;
        };
        let value = if id == param_curve_add_id(slot) {
            curve_row::add_point(&row.value)
        } else if id == param_curve_remove_id(slot) {
            curve_row::remove_point(&row.value, slot)
        } else if id == param_curve_interp_id(slot) {
            curve_row::cycle_interp(&row.value, slot)
        } else {
            continue;
        };
        push_param_intent(MotionParamIntent::SetTextParam {
            node: snap.node,
            param: row.name,
            value,
        });
        return Some(EventOutcome::Consumed);
    }
    None
}

/// A Gradient position-marker drag: the dispatch stashed the normalized `(index, x, y)` in
/// the store's `curve_point_drag` slot; fold the x into the stop's position and emit the new
/// serialized gradient. `Some` = the id WAS a Gradient editor (handled), `None` = try the
/// Curve / scalar path. (The COLOUR edit rides the shell's picker read-back, not here.)
pub(crate) fn on_gradient_drag(
    id: NodeId,
    host: &mut dyn PanelHostInternal,
    snap: &ParamsSnapshot,
) -> Option<EventOutcome> {
    for slot in 0..snap.rows.len().min(MAX_PARAM_ROWS) {
        if id == param_grad_editor_id(slot) {
            let ParamRow::Gradient(row) = &snap.rows[slot] else {
                return Some(EventOutcome::Ignored);
            };
            if let Some(value) = gradient_row::drain_drag(host.store_mut(), slot, &row.value) {
                push_param_intent(MotionParamIntent::SetTextParam {
                    node: snap.node,
                    param: row.name,
                    value,
                });
            }
            return Some(EventOutcome::Consumed);
        }
    }
    None
}

/// A Gradient `+` / `−` / interp button → the new serialized gradient. `None` = not a
/// Gradient button (fall through to the Curve / enum / seed-reroll handler).
pub(crate) fn on_gradient_click(id: NodeId, snap: &ParamsSnapshot) -> Option<EventOutcome> {
    for slot in 0..snap.rows.len().min(MAX_PARAM_ROWS) {
        let ParamRow::Gradient(row) = &snap.rows[slot] else {
            continue;
        };
        let value = if id == param_grad_add_id(slot) {
            gradient_row::add_stop(&row.value)
        } else if id == param_grad_remove_id(slot) {
            gradient_row::remove_stop(&row.value, slot)
        } else if id == param_grad_interp_id(slot) {
            gradient_row::cycle_interp(&row.value, slot)
        } else {
            continue;
        };
        push_param_intent(MotionParamIntent::SetTextParam {
            node: snap.node,
            param: row.name,
            value,
        });
        return Some(EventOutcome::Consumed);
    }
    None
}
