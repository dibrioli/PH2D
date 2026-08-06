//! **The dual range** — soft vs hard limits (doc 88 / Blender's soft-vs-hard),
//! a `#[path]` sibling of `lib_tests.rs` split off at the 600-LOC panel cap along
//! the subject line: everything here answers *how far the SLIDER drags versus how
//! far the BOX types*, and nothing else does.
//!
//! Two layers, and they need a gate each
//! ([[feedback_layered_defenses_need_per_layer_gates]]): `set_number_range`
//! decides what the box can HOLD, `on_value_changed` decides what it may REPORT.
//! Fixing one and not the other leaves the artist typing a number, seeing it, and
//! the document never hearing it.

use super::*;

/// A scalar row whose typed ceiling is far above its slider (`motion.emitter`'s
/// `rate`: the slider drags to 12.000, the box types to 4.000.000).
fn soft_hard_row(value: f64) -> ParamsSnapshot {
    ParamsSnapshot {
        node: 7,
        title: "Emitter".into(),
        modified: Default::default(),
        rows: vec![ParamRow::Scalar(ScalarRow {
            name: "rate",
            label: "Rate".into(),
            value,
            min: 0.0,
            max: 12_000.0,
            hard_min: 0.0,
            hard_max: 4_000_000.0,
            step: 1.0,
            integer: false,
            driven: false,
            display: RowDisplay::default(),
        })],
    }
}

/// A scalar row whose typed FLOOR is below its slider — the mirror of
/// [`soft_hard_row`]. The drag starts at `0.01`; the box types to `0.0001`.
fn soft_hard_floor_row(value: f64) -> ParamsSnapshot {
    ParamsSnapshot {
        node: 7,
        title: "Noise".into(),
        modified: Default::default(),
        rows: vec![ParamRow::Scalar(ScalarRow {
            name: "frequency",
            label: "Frequency".into(),
            value,
            min: 0.01,
            hard_min: 0.0001,
            max: 4.0,
            hard_max: 4.0,
            step: 0.01,
            integer: false,
            driven: false,
            display: RowDisplay::default(),
        })],
    }
}

/// The floor half of the dual range — the one that did not exist until doc 88.
///
/// The ceiling shipped alone, so `set_number_range` was handed `row.min` as the
/// typed floor and a param whose useful drag starts at `0.01` could not be typed
/// to `0.0001`: the box clamped it back up to the slider's start, in silence, and
/// the artist's number was gone. The slider's track is `0..1` over the SOFT span,
/// so it saturates at 0.0 down there — a value this small can only be the box's to
/// report, exactly as a value above `max` is.
#[test]
fn a_typed_value_below_the_sliders_range_reaches_the_param() {
    let _ = drain_param_intents();
    set_current_params(Some(soft_hard_floor_row(1.0)));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    let mut state = MotionParamsPanelState;
    let chip = param_chip_id(0);
    host.set_number_value(chip, 0.0001);
    host.apply_panel_event::<MotionParamsPanel>(&mut state, WidgetEvent::ValueChanged(chip));
    let intents = drain_param_intents();
    assert_eq!(
        intents,
        vec![MotionParamIntent::SetParam {
            node: 7,
            param: "frequency",
            value: 0.0001,
        }],
        "a value only the box can hold must be the box's to report — the floor is \
         the mirror of the ceiling, not a special case"
    );
}

#[test]
fn a_typed_value_above_the_sliders_range_reaches_the_param() {
    // The whole point of a hard limit. The slider's track is 0..1 over the SOFT
    // span, so it saturates at 1.0 and would report 12.000 — a typed 4.000.000
    // that came back through the slider would be silently divided by 333.
    let _ = drain_param_intents();
    set_current_params(Some(soft_hard_row(200.0)));
    // `populate` registers the pooled slider/chip widgets, so the chip is already
    // there — the same pool the real dispatch routes through.
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    let mut state = MotionParamsPanelState;
    let chip = param_chip_id(0);
    host.set_number_value(chip, 4_000_000.0);
    host.apply_panel_event::<MotionParamsPanel>(&mut state, WidgetEvent::ValueChanged(chip));
    let intents = drain_param_intents();
    assert_eq!(
        intents,
        vec![MotionParamIntent::SetParam {
            node: 7,
            param: "rate",
            value: 4_000_000.0,
        }],
        "a value only the box can hold must be the box's to report"
    );
}

#[test]
fn a_typed_value_inside_the_sliders_range_is_still_the_sliders_to_report() {
    // The other half, and the reason the chip was swallowed in the first place:
    // below the soft max the affine mirrors the chip onto the slider, the slider
    // fires too, and reporting from BOTH would notify twice per gesture. This is
    // the gate that keeps the fix from becoming a double-notify regression.
    let _ = drain_param_intents();
    set_current_params(Some(soft_hard_row(200.0)));
    // `populate` registers the pooled slider/chip widgets, so the chip is already
    // there — the same pool the real dispatch routes through.
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    let mut state = MotionParamsPanelState;
    let chip = param_chip_id(0);
    host.set_number_value(chip, 5_000.0);
    host.apply_panel_event::<MotionParamsPanel>(&mut state, WidgetEvent::ValueChanged(chip));
    assert!(
        drain_param_intents().is_empty(),
        "inside the track the slider speaks; the chip must stay silent"
    );
}

#[test]
fn the_box_is_ranged_to_the_hard_limit_and_the_slider_to_the_soft_one() {
    // The other half of the split, and the one the two gates above CANNOT see:
    // they write the chip's value directly, so they would stay green with the
    // box still clamped to the slider's 12.000 and the artist unable to type
    // past it. The range is what makes 4.000.000 enterable at all.
    set_current_params(Some(soft_hard_row(200.0)));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    let mut state = MotionParamsPanelState;
    host.paint::<MotionParamsPanel>(
        &mut state,
        ph2d_editor_core::zones::Rect {
            x: 0.0,
            y: 0.0,
            w: 320.0,
            h: 800.0,
        },
    );
    assert_eq!(
        host.store().number_range(param_chip_id(0)),
        Some((0.0, 4_000_000.0, 1.0)),
        "the box types to the HARD limit"
    );

    // And the FLOOR, the half that did not exist until doc 88: the ceiling
    // shipped alone, so the box was handed the slider's own `min` and a param
    // whose useful drag starts at `0.01` could not be typed to `0.0001` — the
    // box clamped it back up, in silence, and the artist's number was gone.
    set_current_params(Some(soft_hard_floor_row(1.0)));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    host.paint::<MotionParamsPanel>(
        &mut state,
        ph2d_editor_core::zones::Rect {
            x: 0.0,
            y: 0.0,
            w: 320.0,
            h: 800.0,
        },
    );
    assert_eq!(
        host.store().number_range(param_chip_id(0)),
        Some((0.0001, 4.0, 0.01)),
        "and DOWN to the hard floor — the range is [hard_min, hard_max], not the \
         slider's own bounds"
    );
}
