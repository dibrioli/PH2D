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

mod number_rows;
mod snapshot;

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

use number_rows::{
    ANGLE_DECIMALS, SEED_DECIMALS, mirror_number, next_seed, number_is_typing, number_value,
    paint_angle_row, paint_seed_row,
};
pub use snapshot::{
    AngleRow, ColorRow, EnumRow, MotionParamIntent, ParamRow, ParamsSnapshot, ScalarRow, SeedRow,
    ToggleRow, drain_param_intents, param_swatch_id, set_current_params,
};
use snapshot::{
    MAX_ENUM_OPTIONS, MAX_PARAM_ROWS, current_params, param_checkbox_id, param_chip_id,
    param_enum_id, param_number_id, param_reroll_id, param_slider_id, push_param_intent,
};

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, WidgetStore, format_number};
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_TITLE_BASELINE, paint_panel_surface, paint_panel_title,
    paint_segmented_button,
};
use ph2d_editor_core::widget::{
    ButtonState, Checkbox, CheckboxState, CheckboxValue, ColorSwatch, DEFAULT_LABEL_W,
    NUMBER_INPUT_MIN_W_PX, SliderOrientation, SliderState, SwatchSize, TextInputState,
    paint_checkbox, paint_color_swatch, paint_slider_with_chip_layout_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, TypeToken};

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
        {
            let scene = &mut *ctx.scene;
            let text_system = &mut *ctx.text_system;
            let (store, hit_index) = ctx.host.store_and_hit_index_mut();
            let mut y = body_top;
            for (i, row) in snap.rows.iter().enumerate().take(MAX_PARAM_ROWS) {
                match row {
                    ParamRow::Scalar(row) => {
                        let slider_id = param_slider_id(i);
                        let chip_id = param_chip_id(i);
                        let span = (row.max - row.min).max(f64::EPSILON);
                        let track = store
                            .slider(slider_id)
                            .map(|(_, v)| v)
                            .unwrap_or(normalized_track(row.value, row.min, span));
                        let display = row_value(track, row.min, row.max, row.integer);
                        let used = paint_slider_with_chip_layout_adaptive(
                            Rect::new(inner_x, y, inner_w, ROW_H_PX),
                            &row.label,
                            track,
                            display,
                            None,
                            slider_id,
                            chip_id,
                            DEFAULT_LABEL_W,
                            chip_w,
                            store,
                            hit_index,
                            scene,
                            text_system,
                            theme,
                        );
                        y += used + row_gap;
                    }
                    ParamRow::Color(row) => {
                        let swatch_w = SwatchSize::Md.px();
                        paint_text(
                            text_system,
                            scene,
                            &row.label,
                            inner_x,
                            y + (ROW_H_PX - label_font) * 0.5,
                            label_font,
                            DEFAULT_LABEL_W,
                            resolve(ColorToken::Text1, theme),
                        );
                        let swatch_id = param_swatch_id(row.channels[0]);
                        let srect = Rect::new(inner_x + inner_w - swatch_w, y, swatch_w, ROW_H_PX);
                        let sw =
                            ColorSwatch::new(swatch_id, "Color", row.srgb).size(SwatchSize::Md);
                        paint_color_swatch(&sw, srect, scene, theme);
                        hit_index.register(swatch_id, srect);
                        y += ROW_H_PX + row_gap;
                    }
                    ParamRow::Toggle(row) => {
                        // A real checkbox — label + box on the left (the box owns
                        // the click; the dispatch flips its value + fires Toggled).
                        let cb_id = param_checkbox_id(i);
                        let (cb_state, _) = store
                            .checkbox(cb_id)
                            .unwrap_or((CheckboxState::Normal, CheckboxValue::Unchecked));
                        let value = if row.on {
                            CheckboxValue::Checked
                        } else {
                            CheckboxValue::Unchecked
                        };
                        let cb = Checkbox::new(cb_id, row.label.clone())
                            .state(cb_state)
                            .value(value);
                        let crect = Rect::new(inner_x, y, inner_w, ROW_H_PX);
                        paint_checkbox(&cb, crect, scene, text_system, theme);
                        hit_index.register(cb_id, crect);
                        y += ROW_H_PX + row_gap;
                    }
                    ParamRow::Enum(row) => {
                        // Named segmented selector (label line + option buttons),
                        // mirror of the Vector panel's Cap / Join / Draw rows.
                        paint_text(
                            text_system,
                            scene,
                            &row.label,
                            inner_x,
                            y,
                            TypeToken::Sm.px(),
                            inner_w,
                            resolve(ColorToken::Text2, theme),
                        );
                        y += TypeToken::Sm.px() + Spacing::Xs.px();
                        let k = row.labels.len().min(MAX_ENUM_OPTIONS);
                        // Up to 4 buttons across, then wrap; a single option → 1.
                        let cols = k.clamp(1, 4); // CLAMP-OK: segmented column count (option-count layout, not a UI metric)
                        let gap = Spacing::Sm.px();
                        let seg_w = ((inner_w - gap * (cols as f32 - 1.0)) / cols as f32).max(1.0);
                        for (opt, caption) in row.labels.iter().enumerate().take(k) {
                            let bid = param_enum_id(i, opt);
                            let rx = inner_x + (opt % cols) as f32 * (seg_w + gap);
                            let ry = y + (opt / cols) as f32 * (ROW_H_PX + gap);
                            let brect = Rect::new(rx, ry, seg_w, ROW_H_PX);
                            let bstate = store.button_state(bid).unwrap_or(ButtonState::Normal);
                            paint_segmented_button(
                                brect,
                                caption,
                                opt == row.selected,
                                bstate,
                                scene,
                                text_system,
                                theme,
                            );
                            hit_index.register(bid, brect);
                        }
                        let seg_rows = k.div_ceil(cols) as f32;
                        y += seg_rows * ROW_H_PX + (seg_rows - 1.0) * gap + row_gap;
                    }
                    ParamRow::Angle(row) => {
                        // A `deg` number box — never a raw turns/radians slider.
                        let used = paint_angle_row(
                            Rect::new(inner_x, y, inner_w, ROW_H_PX),
                            &row.label,
                            param_number_id(i),
                            row.step_deg,
                            store,
                            hit_index,
                            scene,
                            text_system,
                            theme,
                        );
                        y += used + row_gap;
                    }
                    ParamRow::Seed(row) => {
                        // A whole-number box + a re-roll button (never a slider).
                        let used = paint_seed_row(
                            Rect::new(inner_x, y, inner_w, ROW_H_PX),
                            &row.label,
                            param_number_id(i),
                            param_reroll_id(i),
                            store,
                            hit_index,
                            scene,
                            text_system,
                            theme,
                        );
                        y += used + row_gap;
                    }
                }
            }
        }

        // Phase B (mutable store) — mark each colour swatch so a Down opens the
        // shared OKLCH picker (generic `is_picker_swatch` dispatch). Idempotent.
        {
            let store = ctx.host.store_mut();
            for row in &snap.rows {
                if let ParamRow::Color(c) = row {
                    store.register_picker_swatch(param_swatch_id(c.channels[0]));
                }
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
            WidgetEvent::ValueChanged(id) => on_value_changed(id, host, &snap),
            WidgetEvent::Toggled(id) => on_toggled(id, host, &snap),
            WidgetEvent::Click(id) => on_click(id, &snap),
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
            return EventOutcome::Consumed;
        }
        // The standalone number box of an Angle / Seed row. Angle emits in the
        // param's NATIVE unit (the box shows degrees), so the bridge stays
        // unit-agnostic; Seed emits the whole number as typed.
        if id == param_number_id(slot) {
            let committed = number_value(host.store(), id);
            let (param, value) = match &snap.rows[slot] {
                ParamRow::Angle(row) => (row.name, committed * row.deg_to_native),
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
    })
}

/// `[0,1]` slider track for a value in `[min, min+span]`.
fn normalized_track(value: f64, min: f64, span: f64) -> f32 {
    (((value - min) / span) as f32).clamp(0.0, 1.0)
}

/// The param value a slider `track` (`0..1`) maps to over `[min, max]`, rounded
/// to a whole number for integer params. Shared by `paint` (chip display) and
/// `apply_event` (the emitted `SetParam` value) so the knob and the doc agree.
fn row_value(track: f32, min: f64, max: f64, integer: bool) -> f64 {
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
        store.set_number_range(chip_id, row.min, row.max, row.step);
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
