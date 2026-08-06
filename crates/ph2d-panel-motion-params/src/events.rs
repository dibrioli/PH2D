//! The param-panel **event handlers** — what happens when a widget fires (a slider drag,
//! a checkbox flip, a segmented click, a formula commit) turned into a `MotionParamIntent`
//! the bridge applies. Split from `lib.rs` for the 600-LOC panel-file cap; `apply_event`
//! routes each `WidgetEvent` to the one here. `use super::*` pulls the pooled-id helpers,
//! the intent funnel and the snapshot types the handlers read.

use super::*;

/// **The one door from a scalar row back to the document** — round in the face
/// the artist sees, then convert to what the document stores.
///
/// Both emit sites of [`on_value_changed`] (the typed chip and the slider's
/// affine) go through it, so a number cannot reach `Graph::set_param` still
/// wearing the artist's face. Two sites doing this by hand is one site away from
/// a `gap_x` written in pixels into a param the cook reads as metres.
///
/// ⚠️ The rounding happens in DISPLAY space, because that is the face the chip
/// snaps in. It never actually mixes with a conversion — `unit_of` refuses a
/// converting unit on a whole-number widget precisely because scaling does not
/// commute with rounding — but the order is written down so the pair cannot
/// drift apart later.
fn scalar_intent(snap: &ParamsSnapshot, row: &ScalarRow, displayed: f64) -> MotionParamIntent {
    let shown = if row.integer {
        displayed.round()
    } else {
        displayed
    };
    MotionParamIntent::SetParam {
        node: snap.node,
        param: row.name,
        value: row.display.to_stored(shown),
    }
}

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
            // OUTSIDE the slider's soft `[min, max]` there is nothing to mirror.
            // The track is 0..1 over the soft span, so it saturates at either end
            // and the slider would report the bound — turning a typed 4.000.000
            // into 12.000, or a typed 0.0001 into 0.01, without a word. Out there
            // the box is the only widget that can hold the value, so it speaks for
            // itself.
            //
            // ⚠️ BOTH ends, since doc 88. This read `typed <= row.max` — the
            // ceiling alone — and the asymmetry was invisible because no node had
            // declared a hard FLOOR for it to swallow. It lives in two places (the
            // range handed to the box, and this rule about who reports), and
            // fixing only one leaves the box able to hold a number it may not
            // report: the artist types it, sees it, and the doc never hears it.
            let ParamRow::Scalar(row) = &snap.rows[slot] else {
                return EventOutcome::Consumed;
            };
            let typed = number_value(host.store(), id);
            if row.driven || (typed >= row.min && typed <= row.max) {
                return EventOutcome::Consumed;
            }
            push_param_intent(scalar_intent(snap, row, typed));
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
            // `row.min` / `row.max` are the DISPLAY face, so the affine lands in
            // it too and `scalar_intent` is the only thing that converts.
            push_param_intent(scalar_intent(
                snap,
                row,
                row_value(track, row.min, row.max, row.integer),
            ));
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
        // A seta de reverter vem ANTES do `match` por-tipo, e não é ordem arbitrária: ela
        // existe em TODA row, então enumerar quais variantes a têm seria a lista que apodrece
        // quando a décima terceira chegar. Uma cor emite quatro intents (o swatch dobra RGBA)
        // e um picker de canal dois — `ParamRow::params` é quem sabe, uma vez.
        if id == crate::snapshot::param_reset_id(slot) {
            for param in snap.rows[slot].params() {
                push_param_intent(MotionParamIntent::ResetParam {
                    node: snap.node,
                    param: param.to_string(),
                });
            }
            return EventOutcome::Consumed;
        }
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
            ParamRow::Palette(row) => {
                // `+` appends a colour, `−` drops the LAST. Both re-serialize the whole
                // list into the text param — the same channel the swatch pick writes, so
                // there is one answer to *what is this palette*.
                //
                // ⚠️ **No cap on `+`, and that is the wave** (Enio: *"tire os limites"*).
                // `−` stops at one: an empty palette would leave the node with nothing to
                // cycle, and the strip with nothing to click back from.
                let mut colors = ph2d_color::parse_palette(&row.value)
                    .filter(|p| !p.is_empty())
                    .unwrap_or_else(|| ph2d_color::DEFAULT_PALETTE_FALLBACK.to_vec());
                let add = id == crate::snapshot::param_pal_add_id(slot);
                let rem = id == crate::snapshot::param_pal_remove_id(slot);
                if add {
                    // The new colour copies the last one — an artist adds a swatch to
                    // then EDIT it, and a copy is visible where a black hole is a gap.
                    let last = *colors.last().unwrap_or(&[1.0, 1.0, 1.0, 1.0]);
                    colors.push(last);
                } else if rem && colors.len() > 1 {
                    colors.pop();
                }
                if add || rem {
                    push_param_intent(MotionParamIntent::SetTextParam {
                        node: snap.node,
                        param: row.name,
                        value: ph2d_color::serialize_palette(&colors),
                    });
                    return EventOutcome::Consumed;
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
