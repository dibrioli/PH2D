//! `ph2d-panel-motion-params` — the Motion Nodes node-params panel (M1.P1).
//!
//! Right-docked in the Inspector slot (takeover, mirror of `ph2d-panel-vector`),
//! visible only while the `motion` tool is active — the shell's `motion_bridge`
//! drives `panel_visible("motion_params")` and hides the real Inspector.
//!
//! Reads the selected node's [`ParamsSnapshot`] (published by the bridge) and
//! paints one canonical **label + slider + numeric chip** row per param, using a
//! fixed pool of positional slider/chip widgets (`param_slider_id(slot)`). A row
//! edit returns as a [`MotionParamIntent::SetParam`] the bridge applies to the
//! graph. The slider tracks the doc value every frame **except** while the user
//! is dragging the slider or typing in the chip (so undo / external edits
//! live-update the knob without fighting an in-progress interaction).

#![forbid(unsafe_code)]

mod curve_row;
mod number_rows;
mod rows_paint;
mod snapshot;
mod text_rows;

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

use number_rows::{
    ANGLE_DECIMALS, SEED_DECIMALS, mirror_number, next_seed, number_is_typing, number_value,
    paint_angle_row, paint_seed_row,
};
pub use snapshot::{
    AngleRow, ColorRow, CurveRow, EnumRow, MotionParamIntent, ParamRow, ParamsSnapshot, ScalarRow,
    SeedRow, TextRow, ToggleRow, drain_param_intents, param_swatch_id, set_current_params,
};
use snapshot::{
    MAX_ENUM_OPTIONS, MAX_PARAM_ROWS, current_params, param_checkbox_id, param_chip_id,
    param_curve_add_id, param_curve_editor_id, param_curve_interp_id, param_curve_remove_id,
    param_enum_id, param_number_id, param_reroll_id, param_slider_id, param_text_id,
    push_param_intent,
};
use text_rows::{mirror_text, paint_text_row, text_is_typing, text_value};

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, WidgetStore, format_number};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_TITLE_BASELINE, paint_panel_surface, paint_panel_title,
};
use ph2d_editor_core::widget::{
    ButtonState, CheckboxState, CheckboxValue, NUMBER_INPUT_MIN_W_PX, SliderOrientation,
    SliderState, TextInputState,
};
use ph2d_tokens::{Spacing, TypeToken};

/// Retained panel state — none needed: the selected node + its params live
/// shell-side (`MotionState`) and the pooled widgets re-seed from the published
/// snapshot each frame (interaction-gated). Unit struct so the typed registry can
/// default-construct it.
#[derive(Default)]
pub struct MotionParamsPanelState;

/// Zero-size marker implementing the typed node-params panel contract.
pub struct MotionParamsPanel;

impl Panel for MotionParamsPanel {
    type State = MotionParamsPanelState;

    const ID: &'static str = "motion_params";
    const NODE_ID: NodeId = ids::MOTION_PARAMS_PANEL;
    const DEFAULT_VISIBLE: bool = false;

    fn paint(_state: &mut MotionParamsPanelState, ctx: &mut PaintCtx) {
        if !ctx.host.panel_visible(MotionParamsPanel::ID) {
            ctx.host
                .store_mut()
                .clear_panel_rect(ids::MOTION_PARAMS_PANEL);
            return;
        }
        let rect = ctx.layout.inspector;
        let theme = ctx.host.theme();
        ctx.host
            .store_mut()
            .set_panel_rect(ids::MOTION_PARAMS_PANEL, rect);
        paint_panel_surface(rect, ctx.scene, theme);

        let snap = current_params();
        let title = snap.as_ref().map(|s| s.title.as_str()).unwrap_or("Motion");
        let title_size = paint_panel_title(rect, title, 0.0, ctx.scene, ctx.text_system, theme);

        let Some(snap) = snap else {
            return;
        };

        // Layout (mirror of the Vector panel body metrics).
        let inner_x = rect.x + PANEL_HEAD_PAD;
        let inner_w = (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0);
        let chip_w = NUMBER_INPUT_MIN_W_PX;
        let row_gap = Spacing::Sm.px();
        let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();

        // Phase A — seed the pooled widgets from the doc values (skipping any row
        // being interacted with) + refresh each chip's range + slider↔chip link.
        seed_rows(ctx.host.store_mut(), &snap.rows);

        // Phase A — paint each row: a scalar slider-with-chip composite, or a
        // colour swatch (label + right-aligned swatch, mirror of the Vector
        // Stroke/Fill rows). Both use the SHARED source-of-truth painters.
        let label_font = TypeToken::Base.px();
        // `paint_rows` draws + registers hit rects (it holds `hit_index`); a Curve row's
        // `CurvePoint`/`Button` STORE states cannot be registered through the immutable
        // store here, so they ride back in `curve_widgets` for Phase C below.
        let curve_widgets = {
            let scene = &mut *ctx.scene;
            let text_system = &mut *ctx.text_system;
            let (store, hit_index) = ctx.host.store_and_hit_index_mut();
            rows_paint::paint_rows(
                &snap.rows,
                inner_x,
                inner_w,
                chip_w,
                row_gap,
                body_top,
                label_font,
                store,
                hit_index,
                scene,
                text_system,
                theme,
            )
        };

        // Phase B (mutable store) — mark each colour swatch so a Down opens the
        // shared OKLCH picker (generic `is_picker_swatch` dispatch). Idempotent.
        // Phase C — register the Curve editor's per-frame `CurvePoint` handles (the
        // dispatch reads `canvas`/`index` off these to normalize a drag) + its buttons.
        {
            let store = ctx.host.store_mut();
            for row in &snap.rows {
                if let ParamRow::Color(c) = row {
                    store.register_picker_swatch(param_swatch_id(c.channels[0]));
                }
            }
            for &(id, parent, index, canvas) in &curve_widgets.points {
                store.register(
                    id,
                    InteractiveState::CurvePoint {
                        parent,
                        channel: 0,
                        index,
                        canvas,
                    },
                );
            }
            for &id in &curve_widgets.buttons {
                store.register(
                    id,
                    InteractiveState::Button {
                        state: ButtonState::Normal,
                    },
                );
            }
        }
    }

    fn apply_event(
        _state: &mut MotionParamsPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        let Some(snap) = current_params() else {
            return EventOutcome::Ignored;
        };
        match ev {
            // A Curve handle drag arrives as ValueChanged(editor) — drain it FIRST (the
            // dispatch stashed the normalized point); else it is a scalar slider / chip.
            WidgetEvent::ValueChanged(id) => {
                on_curve_drag(id, host, &snap).unwrap_or_else(|| on_value_changed(id, host, &snap))
            }
            WidgetEvent::Toggled(id) => on_toggled(id, host, &snap),
            // A Curve +/−/interp button, else a segmented option / seed re-roll.
            WidgetEvent::Click(id) => {
                on_curve_click(id, &snap).unwrap_or_else(|| on_click(id, &snap))
            }
            // A formula field commits on Enter (Submit) or focus-loss (Blur).
            WidgetEvent::Submit(id) | WidgetEvent::Blur(id) => on_text_commit(id, host, &snap),
            _ => EventOutcome::Ignored,
        }
    }

    fn populate(store: &mut WidgetStore) {
        // Register the pooled widgets so the dispatch can route drags / clicks /
        // toggles even before the first paint seeds their values: a slider + chip,
        // a checkbox, and a row of segmented option buttons per slot.
        for slot in 0..MAX_PARAM_ROWS {
            store.register(
                param_slider_id(slot),
                InteractiveState::Slider {
                    state: SliderState::Normal,
                    value: 0.5,
                    orientation: SliderOrientation::Horizontal,
                },
            );
            store.register(param_chip_id(slot), number_input(0.0));
            // Standalone number box (Angle / Seed rows) + the Seed re-roll button.
            store.register(param_number_id(slot), number_input(0.0));
            store.register(
                param_reroll_id(slot),
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
            store.register(
                param_text_id(slot),
                InteractiveState::TextInput {
                    state: TextInputState::Normal,
                    text: String::new(),
                    caret: 0,
                    selection_anchor: None,
                },
            );
            store.register(
                param_checkbox_id(slot),
                InteractiveState::Checkbox {
                    state: CheckboxState::Normal,
                    value: CheckboxValue::Unchecked,
                },
            );
            for opt in 0..MAX_ENUM_OPTIONS {
                store.register(
                    param_enum_id(slot, opt),
                    InteractiveState::Button {
                        state: ButtonState::Normal,
                    },
                );
            }
        }
    }
}

/// A Curve handle drag: the dispatch stashed the normalized `(index, x, y)` in the
/// store's `curve_point_drag` slot; fold it into the row's curve and emit the new
/// serialized value. `Some` = the id WAS a Curve editor (handled), `None` = try the
/// scalar path.
fn on_curve_drag(
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
fn on_curve_click(id: NodeId, snap: &ParamsSnapshot) -> Option<EventOutcome> {
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

/// A slider drag / chip commit → emit the scalar row value. A chip fires its own
/// ValueChanged mirrored from the slider, so it is swallowed to avoid a double
/// notify. Only Scalar rows own a pooled slider (Color reports via the picker).
fn on_value_changed(
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
fn on_toggled(id: NodeId, host: &dyn PanelHostInternal, snap: &ParamsSnapshot) -> EventOutcome {
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
fn on_click(id: NodeId, snap: &ParamsSnapshot) -> EventOutcome {
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
            _ => {}
        }
    }
    EventOutcome::Ignored
}

/// A formula field commit (Enter → Submit, or focus-loss → Blur) → emit the text-param
/// edit. The store-global dispatch already wrote the buffer; read it back and push a
/// [`MotionParamIntent::SetTextParam`] the bridge applies via `Graph::set_text_param`.
fn on_text_commit(id: NodeId, host: &dyn PanelHostInternal, snap: &ParamsSnapshot) -> EventOutcome {
    for slot in 0..snap.rows.len().min(MAX_PARAM_ROWS) {
        if id == param_text_id(slot) {
            let ParamRow::Text(row) = &snap.rows[slot] else {
                return EventOutcome::Ignored;
            };
            push_param_intent(MotionParamIntent::SetTextParam {
                node: snap.node,
                param: row.name,
                value: text_value(host.store(), id),
            });
            return EventOutcome::Consumed;
        }
    }
    EventOutcome::Ignored
}

/// True while any pooled param row is being interacted with — a slider dragged /
/// focused, a chip focused, or a standalone number box (Angle / Seed) focused.
/// The shell reads this as the undo-bracket edge (open on the false→true
/// transition, commit one step on true→false), so a whole slider drag — or a
/// whole type-into-the-angle-box — is a single undo step (M1.P1).
#[must_use]
pub fn any_param_editing(store: &WidgetStore) -> bool {
    (0..MAX_PARAM_ROWS).any(|slot| {
        matches!(
            store.slider(param_slider_id(slot)),
            Some((SliderState::Dragging | SliderState::Focused, _))
        ) || matches!(
            store.get(param_chip_id(slot)),
            Some(InteractiveState::NumberInput {
                state: TextInputState::Focused,
                ..
            })
        ) || number_is_typing(store, param_number_id(slot))
            || text_is_typing(store, param_text_id(slot))
    })
}

/// `[0,1]` slider track for a value in `[min, min+span]`.
pub(crate) fn normalized_track(value: f64, min: f64, span: f64) -> f32 {
    (((value - min) / span) as f32).clamp(0.0, 1.0)
}

/// The param value a slider `track` (`0..1`) maps to over `[min, max]`, rounded
/// to a whole number for integer params. Shared by `paint` (chip display) and
/// `apply_event` (the emitted `SetParam` value) so the knob and the doc agree.
pub(crate) fn row_value(track: f32, min: f64, max: f64, integer: bool) -> f64 {
    let span = (max - min).max(f64::EPSILON);
    let v = min + f64::from(track) * span;
    if integer { v.round() } else { v }
}

/// A `NumberInput` state seeded at `value` (buffer = its canonical formatting).
fn number_input(value: f64) -> InteractiveState {
    InteractiveState::NumberInput {
        state: TextInputState::Normal,
        value,
        buffer: format_number(value),
        caret: 0,
        last_committed: value,
        selection_anchor: None,
    }
}

/// Seed each pooled row from its doc value + refresh its range/link, EXCEPT for a
/// row currently being dragged (slider) or typed (chip) — so an in-progress
/// interaction owns the widget, but idle rows always mirror the doc (undo /
/// external edits live-update the knob).
fn seed_rows(store: &mut WidgetStore, rows: &[ParamRow]) {
    for (i, row) in rows.iter().enumerate().take(MAX_PARAM_ROWS) {
        // Toggle rows: sync the checkbox value to the doc while PRESERVING its
        // transient hover/press state (the dispatch owns that + flips the value
        // on click). Colour swatches seed from the bridge's `widget_color`; Enum
        // buttons are stateless (selection comes from the snapshot at paint).
        if let ParamRow::Toggle(t) = row {
            let cb_id = param_checkbox_id(i);
            let state = store
                .checkbox(cb_id)
                .map(|(s, _)| s)
                .unwrap_or(CheckboxState::Normal);
            store.register(
                cb_id,
                InteractiveState::Checkbox {
                    state,
                    value: if t.on {
                        CheckboxValue::Checked
                    } else {
                        CheckboxValue::Unchecked
                    },
                },
            );
            continue;
        }
        // Standalone number boxes (Angle in degrees, Seed as a whole number):
        // mirror the doc value when unfocused + register the range so the
        // drag-scrub is range-proportional and the stepper uses `step`.
        if let ParamRow::Angle(a) = row {
            let id = param_number_id(i);
            mirror_number(store, id, a.deg, ANGLE_DECIMALS);
            store.set_number_range(id, a.min_deg, a.max_deg, a.step_deg);
            continue;
        }
        if let ParamRow::Seed(s) = row {
            let id = param_number_id(i);
            mirror_number(store, id, s.value, SEED_DECIMALS);
            store.set_number_range(id, s.min, s.max, 1.0);
            continue;
        }
        // Text (formula) rows: mirror the doc formula into the field when unfocused.
        if let ParamRow::Text(t) = row {
            mirror_text(store, param_text_id(i), &t.value);
            continue;
        }
        let ParamRow::Scalar(row) = row else { continue };
        let slider_id = param_slider_id(i);
        let chip_id = param_chip_id(i);
        let span = (row.max - row.min).max(f64::EPSILON);

        let dragging = matches!(
            store.slider(slider_id),
            Some((SliderState::Dragging | SliderState::Focused, _))
        );
        let typing = matches!(
            store.get(chip_id),
            Some(InteractiveState::NumberInput {
                state: TextInputState::Focused,
                ..
            })
        );
        if !dragging && !typing {
            store.register(
                slider_id,
                InteractiveState::Slider {
                    state: SliderState::Normal,
                    value: normalized_track(row.value, row.min, span),
                    orientation: SliderOrientation::Horizontal,
                },
            );
            store.register(chip_id, number_input(row.value));
        }
        // Range (chip typed-value clamp/step) + slider↔chip affine (track 0..1 →
        // value): display = track * span + min. Integer rows snap the chip.
        //
        // The CHIP gets the HARD ceiling and the slider keeps the soft one: the
        // drag range and the legal range are different questions (Blender's soft
        // vs hard limits). Above `row.max` the affine below saturates the track
        // at 1.0, so such a value cannot come back through the slider — which is
        // exactly why `on_value_changed` lets the chip speak for itself up there.
        store.set_number_range(chip_id, row.min, row.hard_max, row.step);
        if row.integer {
            store.link_slider_number_mapped_integer(
                slider_id,
                chip_id,
                span as f32,
                row.min as f32,
            );
        } else {
            store.link_slider_number_mapped(slider_id, chip_id, span as f32, row.min as f32);
        }
    }
}
