//! The param-panel **event handlers** — what happens when a widget fires (a slider drag,
//! a checkbox flip, a segmented click, a formula commit) turned into a `MotionParamIntent`
//! the bridge applies. Split from `lib.rs` for the 600-LOC panel-file cap; `apply_event`
//! routes each `WidgetEvent` to the one here. `use super::*` pulls the pooled-id helpers,
//! the intent funnel and the snapshot types the handlers read.

use super::*;

/// A slider drag / chip commit → emit the scalar row value. A chip fires its own
/// ValueChanged mirrored from the slider, so it is swallowed to avoid a double
/// notify. Only Scalar rows own a pooled slider (Color reports via the picker).
pub(crate) fn on_value_changed(
    id: NodeId,
    host: &dyn PanelHostInternal,
    snap: &ParamsSnapshot,
) -> EventOutcome {
    for slot in 0..snap.rows.len().min(MAX_PARAM_ROWS) {
        if id == param_chip_id(slot) {
            // Normally the chip is a MIRROR of the slider: the affine drives it,
            // it re-fires ValueChanged, and swallowing that is what keeps one
            // gesture from notifying twice.
            //
            // Above the slider's soft `max` there is nothing to mirror. The track
            // is 0..1 over the soft span, so it saturates at 1.0 and the slider
            // would report `max` — turning a typed 4.000.000 into 12.000 without
            // a word. Up there the box is the only widget that can hold the
            // value, so it speaks for itself.
            let ParamRow::Scalar(row) = &snap.rows[slot] else {
                return EventOutcome::Consumed;
            };
            let typed = number_value(host.store(), id);
            if row.driven || typed <= row.max {
                return EventOutcome::Consumed;
            }
            push_param_intent(MotionParamIntent::SetParam {
                node: snap.node,
                param: row.name,
                value: if row.integer { typed.round() } else { typed },
            });
            return EventOutcome::Consumed;
        }
        // The standalone number box of an Angle / Seed row. An Angle param IS
        // degrees (the app's authored-angle unit), so the box's value is the
        // param's value — nothing to convert. Seed emits the whole number typed.
        if id == param_number_id(slot) {
            let committed = number_value(host.store(), id);
            let (param, value) = match &snap.rows[slot] {
                ParamRow::Angle(row) => (row.name, committed),
                ParamRow::Seed(row) => (row.name, committed.round()),
                _ => return EventOutcome::Ignored,
            };
            push_param_intent(MotionParamIntent::SetParam {
                node: snap.node,
                param,
                value,
            });
            return EventOutcome::Consumed;
        }
        if id == param_slider_id(slot) {
            let ParamRow::Scalar(row) = &snap.rows[slot] else {
                return EventOutcome::Ignored;
            };
            // Second barrier (the paint registers nothing for a driven row, so this should be
            // unreachable — but a stale id from the frame the wire landed must not write a
            // number the wire is about to overwrite anyway).
            if row.driven {
                return EventOutcome::Ignored;
            }
            let track = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.5);
            push_param_intent(MotionParamIntent::SetParam {
                node: snap.node,
                param: row.name,
                value: row_value(track, row.min, row.max, row.integer),
            });
            return EventOutcome::Consumed;
        }
    }
    EventOutcome::Ignored
}

/// A checkbox flip → emit 1.0 / 0.0 for the Toggle row. The dispatch already
/// flipped the stored value; read it back rather than tracking the old one.
pub(crate) fn on_toggled(
    id: NodeId,
    host: &dyn PanelHostInternal,
    snap: &ParamsSnapshot,
) -> EventOutcome {
    for slot in 0..snap.rows.len().min(MAX_PARAM_ROWS) {
        if id == param_checkbox_id(slot) {
            let ParamRow::Toggle(row) = &snap.rows[slot] else {
                return EventOutcome::Ignored;
            };
            let on = matches!(
                host.store().checkbox(id).map(|(_, v)| v),
                Some(CheckboxValue::Checked)
            );
            push_param_intent(MotionParamIntent::SetParam {
                node: snap.node,
                param: row.name,
                value: if on { 1.0 } else { 0.0 },
            });
            return EventOutcome::Consumed;
        }
    }
    EventOutcome::Ignored
}

/// A button click: a segmented option selects its index for the Enum row; the
/// Seed row's re-roll advances the seed deterministically ([`next_seed`]).
pub(crate) fn on_click(id: NodeId, snap: &ParamsSnapshot) -> EventOutcome {
    for slot in 0..snap.rows.len().min(MAX_PARAM_ROWS) {
        match &snap.rows[slot] {
            ParamRow::Seed(row) if id == param_reroll_id(slot) => {
                push_param_intent(MotionParamIntent::SetParam {
                    node: snap.node,
                    param: row.name,
                    value: next_seed(row.value, row.min, row.max),
                });
                return EventOutcome::Consumed;
            }
            ParamRow::Enum(row) => {
                for opt in 0..row.labels.len().min(MAX_ENUM_OPTIONS) {
                    if id == param_enum_id(slot, opt) {
                        push_param_intent(MotionParamIntent::SetParam {
                            node: snap.node,
                            param: row.name,
                            value: opt as f64,
                        });
                        return EventOutcome::Consumed;
                    }
                }
            }
            ParamRow::Channels(row) => {
                // Channel segments reuse the enum-option ids; Custom is the n-th.
                let n = row.channels.len();
                for opt in 0..(n + 1).min(MAX_ENUM_OPTIONS) {
                    if id != param_enum_id(slot, opt) {
                        continue;
                    }
                    if opt < n {
                        // A named channel writes BOTH the column and its mode.
                        let (_, column, mode) = row.channels[opt];
                        push_param_intent(MotionParamIntent::SetTextParam {
                            node: snap.node,
                            param: row.text_param,
                            value: column.to_string(),
                        });
                        push_param_intent(MotionParamIntent::SetParam {
                            node: snap.node,
                            param: row.mode_param,
                            value: mode as f64,
                        });
                    } else if row.selected < n {
                        // Switch INTO Custom: clear the column so the raw field opens
                        // empty. (Already Custom → no-op, so a typed value survives.)
                        push_param_intent(MotionParamIntent::SetTextParam {
                            node: snap.node,
                            param: row.text_param,
                            value: String::new(),
                        });
                    }
                    return EventOutcome::Consumed;
                }
                // Live-column chips (the Custom picker): clicking a real upstream
                // column writes its name + the scalar mode (0).
                for j in 0..row.extra.len().min(MAX_ENUM_OPTIONS) {
                    if id != param_enum_id(slot, CHANNELS_EXTRA_BASE + j) {
                        continue;
                    }
                    push_param_intent(MotionParamIntent::SetTextParam {
                        node: snap.node,
                        param: row.text_param,
                        value: row.extra[j].clone(),
                    });
                    push_param_intent(MotionParamIntent::SetParam {
                        node: snap.node,
                        param: row.mode_param,
                        value: 0.0,
                    });
                    return EventOutcome::Consumed;
                }
            }
            ParamRow::Source(row) => {
                // A source chip (doc 65): clicking a published name writes it to the text
                // param — the same channel the raw field below writes, one source of truth.
                for j in 0..row.options.len().min(MAX_ENUM_OPTIONS) {
                    if id == param_enum_id(slot, j) {
                        push_param_intent(MotionParamIntent::SetTextParam {
                            node: snap.node,
                            param: row.param,
                            value: row.options[j].clone(),
                        });
                        return EventOutcome::Consumed;
                    }
                }
            }
            _ => {}
        }
    }
    EventOutcome::Ignored
}

/// A formula field commit (Enter → Submit, or focus-loss → Blur) → emit the text-param
/// edit. The store-global dispatch already wrote the buffer; read it back and push a
/// [`MotionParamIntent::SetTextParam`] the bridge applies via `Graph::set_text_param`.
pub(crate) fn on_text_commit(
    id: NodeId,
    host: &dyn PanelHostInternal,
    snap: &ParamsSnapshot,
) -> EventOutcome {
    for slot in 0..snap.rows.len().min(MAX_PARAM_ROWS) {
        if id == param_text_id(slot) {
            // Both a plain Text row and a Channels row's Custom field write the same
            // text channel — only the param name differs.
            let param = match &snap.rows[slot] {
                ParamRow::Text(row) => row.name,
                ParamRow::Channels(row) => row.text_param,
                ParamRow::Source(row) => row.param,
                _ => return EventOutcome::Ignored,
            };
            push_param_intent(MotionParamIntent::SetTextParam {
                node: snap.node,
                param,
                value: text_value(host.store(), id),
            });
            return EventOutcome::Consumed;
        }
    }
    EventOutcome::Ignored
}
