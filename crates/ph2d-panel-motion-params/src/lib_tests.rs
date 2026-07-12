//! Unit tests for the params panel (split out of `lib.rs` for the HR-18 LOC
//! cap). Declared by the parent as a `#[path]` sibling, so `super` is the
//! crate root and the pooled-id / row-mapping helpers are all in scope.

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
        rows: vec![ParamRow::Scalar(ScalarRow {
            name: "rows",
            label: "Rows".into(),
            value: 3.0,
            min: 1.0,
            max: 20.0,
            step: 1.0,
            integer: true,
        })],
    }));
    let got = current_params().expect("published");
    assert_eq!(got.node, 7);
    let ParamRow::Scalar(r0) = &got.rows[0] else {
        panic!("scalar row");
    };
    assert_eq!(r0.name, "rows");

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

#[test]
fn text_row_and_set_text_param_intent_round_trip() {
    let _ = drain_param_intents();
    // A Text (formula) row publishes + reads back (the string-valued row).
    set_current_params(Some(ParamsSnapshot {
        node: 3,
        title: "Expression".into(),
        rows: vec![ParamRow::Text(TextRow {
            name: "expr",
            label: "Formula".into(),
            value: "sin(t)".into(),
        })],
    }));
    let got = current_params().expect("published");
    let ParamRow::Text(r) = &got.rows[0] else {
        panic!("text row");
    };
    assert_eq!((r.name, r.value.as_str()), ("expr", "sin(t)"));

    // A formula edit rides the String-carrying SetTextParam intent (not the f64 SetParam).
    push_param_intent(MotionParamIntent::SetTextParam {
        node: 3,
        param: "expr",
        value: "cos(i * a)".into(),
    });
    assert_eq!(
        drain_param_intents(),
        vec![MotionParamIntent::SetTextParam {
            node: 3,
            param: "expr",
            value: "cos(i * a)".into(),
        }]
    );
    set_current_params(None);
}

#[test]
fn color_row_publishes_and_swatch_id_is_anchor_keyed() {
    // A colour row round-trips through the publish channel, and its swatch id
    // is derived from the anchor channel name (so the shell bridge computes
    // the same id) — distinct from other anchors + from the slider pool.
    set_current_params(Some(ParamsSnapshot {
        node: 3,
        title: "Tint".into(),
        rows: vec![ParamRow::Color(ColorRow {
            label: "Color".into(),
            channels: ["r", "g", "b", "a"],
            srgb: [255, 255, 255, 255],
        })],
    }));
    let got = current_params().expect("published");
    let ParamRow::Color(c) = &got.rows[0] else {
        panic!("color row");
    };
    assert_eq!(c.channels, ["r", "g", "b", "a"]);
    assert_eq!(param_swatch_id("r"), param_swatch_id("r"));
    assert_ne!(param_swatch_id("r"), param_swatch_id("g"));
    assert_ne!(param_swatch_id("r"), param_slider_id(0));
    set_current_params(None);
}

/// The Angle row publishes degrees end to end — no conversion factor, because the
/// param already stores degrees (the app's one authored-angle unit). What the box
/// shows is what a `ValueChanged` pushes back.
#[test]
fn angle_row_publishes_degrees_verbatim() {
    let _ = drain_param_intents();
    set_current_params(Some(ParamsSnapshot {
        node: 4,
        title: "Orbit".into(),
        rows: vec![ParamRow::Angle(AngleRow {
            name: "angle",
            label: "Angle".into(),
            deg: 90.0,
            min_deg: -360.0,
            max_deg: 360.0,
            step_deg: 1.0,
        })],
    }));
    let got = current_params().expect("published");
    let ParamRow::Angle(a) = &got.rows[0] else {
        panic!("angle row");
    };
    assert_eq!(a.deg, 90.0);
    assert_eq!((a.min_deg, a.max_deg), (-360.0, 360.0));
    set_current_params(None);
}

/// The Seed row's pooled widgets are distinct from every other row's pool —
/// a re-roll click can never be mistaken for a slider drag or an enum option.
#[test]
fn seed_row_widget_ids_do_not_collide_with_other_pools() {
    assert_ne!(param_number_id(0), param_reroll_id(0));
    assert_ne!(param_number_id(0), param_chip_id(0));
    assert_ne!(param_number_id(0), param_slider_id(0));
    assert_ne!(param_reroll_id(0), param_enum_id(0, 0));
    assert_ne!(param_number_id(0), param_number_id(1)); // positional pool
}
