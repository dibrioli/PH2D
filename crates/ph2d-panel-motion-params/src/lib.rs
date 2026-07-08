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

mod snapshot;

use snapshot::{MAX_PARAM_ROWS, current_params, param_chip_id, param_slider_id, push_param_intent};
pub use snapshot::{
    MotionParamIntent, ParamRow, ParamsSnapshot, drain_param_intents, set_current_params,
};

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, WidgetStore, format_number};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_TITLE_BASELINE, paint_panel_surface, paint_panel_title,
};
use ph2d_editor_core::widget::{
    DEFAULT_LABEL_W, NUMBER_INPUT_MIN_W_PX, SliderOrientation, SliderState, TextInputState,
    paint_slider_with_chip_layout_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing};

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

        // Phase B — paint each row via the canonical slider-with-chip composite.
        let scene = &mut *ctx.scene;
        let text_system = &mut *ctx.text_system;
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        let mut y = body_top;
        for (i, row) in snap.rows.iter().enumerate().take(MAX_PARAM_ROWS) {
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
    }

    fn apply_event(
        _state: &mut MotionParamsPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        let WidgetEvent::ValueChanged(id) = ev else {
            return EventOutcome::Ignored;
        };
        let Some(snap) = current_params() else {
            return EventOutcome::Ignored;
        };
        for slot in 0..snap.rows.len().min(MAX_PARAM_ROWS) {
            // A chip edit is already mirrored to its slider (which fires its own
            // ValueChanged, handled below) — swallow to avoid a double notify.
            if id == param_chip_id(slot) {
                return EventOutcome::Consumed;
            }
            if id == param_slider_id(slot) {
                let row = &snap.rows[slot];
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

    fn populate(store: &mut WidgetStore) {
        // Register the pooled slider + chip widgets so the dispatch can route
        // drags/typing even before the first paint seeds their values.
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
        }
    }
}

/// True while any pooled param row is being interacted with — a slider dragged /
/// focused, or a chip focused. The shell reads this as the undo-bracket edge
/// (open on the false→true transition, commit one step on true→false), so a
/// whole slider drag is a single undo step (M1.P1).
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
        )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_value_maps_over_the_range_and_inverts() {
        // Continuous: track 0 → min, 0.5 → midpoint, 1 → max.
        assert!((row_value(0.0, -10.0, 10.0, false) + 10.0).abs() < 1e-6);
        assert!(row_value(0.5, -10.0, 10.0, false).abs() < 1e-6);
        assert!((row_value(1.0, -10.0, 10.0, false) - 10.0).abs() < 1e-6);
        // Integer rows snap the endpoints to whole numbers.
        assert_eq!(row_value(0.0, 1.0, 20.0, true), 1.0);
        assert_eq!(row_value(1.0, 1.0, 20.0, true), 20.0);
        // `normalized_track` is the inverse used to seed the knob.
        assert!((normalized_track(0.0, -10.0, 20.0) - 0.5).abs() < 1e-6);
        // Out-of-range values clamp into the track.
        assert_eq!(normalized_track(100.0, 0.0, 10.0), 1.0);
        assert_eq!(normalized_track(-5.0, 0.0, 10.0), 0.0);
    }

    #[test]
    fn params_and_intent_channels_round_trip() {
        let _ = drain_param_intents();
        set_current_params(Some(ParamsSnapshot {
            node: 7,
            title: "Grid".into(),
            rows: vec![ParamRow {
                name: "rows",
                label: "Rows".into(),
                value: 3.0,
                min: 1.0,
                max: 20.0,
                step: 1.0,
                integer: true,
            }],
        }));
        let got = current_params().expect("published");
        assert_eq!(got.node, 7);
        assert_eq!(got.rows[0].name, "rows");

        push_param_intent(MotionParamIntent::SetParam {
            node: 7,
            param: "rows",
            value: 5.0,
        });
        assert_eq!(
            drain_param_intents(),
            vec![MotionParamIntent::SetParam {
                node: 7,
                param: "rows",
                value: 5.0,
            }]
        );
        assert!(drain_param_intents().is_empty()); // capacity-retaining drain
        set_current_params(None);
        assert!(current_params().is_none());
    }
}
