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
            hard_max: 20.0,
            step: 1.0,
            integer: true,
            driven: false,
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

/// A scalar row whose typed ceiling is far above its slider (`motion.emitter`'s
/// `rate`: the slider drags to 12.000, the box types to 4.000.000).
fn soft_hard_row(value: f64) -> ParamsSnapshot {
    ParamsSnapshot {
        node: 7,
        title: "Emitter".into(),
        rows: vec![ParamRow::Scalar(ScalarRow {
            name: "rate",
            label: "Rate".into(),
            value,
            min: 0.0,
            max: 12_000.0,
            hard_max: 4_000_000.0,
            step: 1.0,
            integer: false,
            driven: false,
        })],
    }
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

/// A `value.attribute`-style channel picker snapshot: two channels (Speed, Opacity),
/// `selected` the current segment (2 = Custom), `custom` the live column text.
fn channels_snapshot(selected: usize, custom: &str) -> ParamsSnapshot {
    ParamsSnapshot {
        node: 7,
        title: "Attribute".into(),
        rows: vec![ParamRow::Channels(ChannelsRow {
            label: "Read".into(),
            text_param: "attr",
            mode_param: "mode",
            channels: vec![("Speed", "vel", 1), ("Opacity", "opacity", 0)],
            selected,
            custom: custom.into(),
            extra: Vec::new(),
        })],
    }
}

/// **Picking a named channel writes BOTH the column and its mode** — the whole point:
/// the artist reads "Speed", the editor sets `attr = vel` AND `mode = 1` in one
/// gesture. FALSIFIED by a click that emits only one of the two (the magnitude would
/// be lost, and `vel` would read as zeros).
#[test]
fn picking_a_channel_writes_the_column_and_its_mode() {
    let _ = drain_param_intents();
    set_current_params(Some(channels_snapshot(1, "opacity"))); // Opacity currently
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    let mut state = MotionParamsPanelState;
    // Segment 0 is "Speed".
    host.apply_panel_event::<MotionParamsPanel>(
        &mut state,
        WidgetEvent::Click(param_enum_id(0, 0)),
    );
    assert_eq!(
        drain_param_intents(),
        vec![
            MotionParamIntent::SetTextParam {
                node: 7,
                param: "attr",
                value: "vel".into(),
            },
            MotionParamIntent::SetParam {
                node: 7,
                param: "mode",
                value: 1.0,
            },
        ],
        "a named channel writes both the column and its mode"
    );
    set_current_params(None);
}

/// **Custom clears the column so the raw field opens** — but only when a channel is
/// currently selected. Clicking Custom while already Custom keeps the typed value
/// (no-op), so a power user's column survives a stray tap.
#[test]
fn the_custom_segment_switches_in_but_never_stomps_a_typed_column() {
    // From a channel (Speed) → Custom clears the column.
    let _ = drain_param_intents();
    set_current_params(Some(channels_snapshot(0, "vel")));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    let mut state = MotionParamsPanelState;
    // Custom is segment n = channels.len() = 2.
    host.apply_panel_event::<MotionParamsPanel>(
        &mut state,
        WidgetEvent::Click(param_enum_id(0, 2)),
    );
    assert_eq!(
        drain_param_intents(),
        vec![MotionParamIntent::SetTextParam {
            node: 7,
            param: "attr",
            value: String::new(),
        }],
        "switching to Custom clears the column so the field opens empty"
    );

    // Already Custom (a typed column) → clicking Custom does nothing.
    set_current_params(Some(channels_snapshot(2, "id")));
    host.apply_panel_event::<MotionParamsPanel>(
        &mut state,
        WidgetEvent::Click(param_enum_id(0, 2)),
    );
    assert!(
        drain_param_intents().is_empty(),
        "clicking Custom while already Custom keeps the typed column"
    );
    set_current_params(None);
}

/// **The Custom picker offers the LIVE upstream columns as clickable chips** (the
/// roadmap's dropdown populated at runtime): clicking a chip writes that real
/// column name + the scalar mode (0), so the artist never guesses a name. The chip
/// ids live above the curated segments (`CHANNELS_EXTRA_BASE`), so this cannot be a
/// curated segment misfire. FALSIFIED by a click that emits nothing (dead chip) or
/// the wrong column.
#[test]
fn clicking_a_live_column_chip_writes_that_column_with_scalar_mode() {
    let _ = drain_param_intents();
    // Custom is active (selected = channels.len() = 2), with two live columns the
    // upstream stream carries.
    set_current_params(Some(ParamsSnapshot {
        node: 7,
        title: "Attribute".into(),
        rows: vec![ParamRow::Channels(ChannelsRow {
            label: "Read".into(),
            text_param: "attr",
            mode_param: "mode",
            channels: vec![("Speed", "vel", 1), ("Opacity", "opacity", 0)],
            selected: 2, // Custom
            custom: String::new(),
            extra: vec!["id".into(), "inv_mass".into()],
        })],
    }));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    let mut state = MotionParamsPanelState;
    // The 2nd live chip is `inv_mass` (base + 1).
    host.apply_panel_event::<MotionParamsPanel>(
        &mut state,
        WidgetEvent::Click(param_enum_id(0, CHANNELS_EXTRA_BASE + 1)),
    );
    assert_eq!(
        drain_param_intents(),
        vec![
            MotionParamIntent::SetTextParam {
                node: 7,
                param: "attr",
                value: "inv_mass".into(),
            },
            MotionParamIntent::SetParam {
                node: 7,
                param: "mode",
                value: 0.0,
            },
        ],
        "a live-column chip writes its column name and the scalar mode"
    );
    set_current_params(None);
}

fn curve_snapshot(value: &str) -> ParamsSnapshot {
    ParamsSnapshot {
        node: 9,
        title: "Remap".into(),
        rows: vec![ParamRow::Curve(CurveRow {
            name: "curve",
            label: "Curve".into(),
            value: value.into(),
        })],
    }
}

/// The seam: a Curve row's handles must be REGISTERED, not merely drawn — a painted
/// handle the store does not know is dead under the mouse. After a paint the three
/// control points are `CurvePoint` widgets, so the dispatch can grab them.
#[test]
fn a_curve_row_paints_registered_draggable_handles() {
    set_current_params(Some(curve_snapshot("c1 0:0:L 0.5:1:S 1:0:L")));
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
    for i in 0..3 {
        assert!(
            matches!(
                host.store()
                    .get(crate::snapshot::param_curve_point_id(0, i)),
                Some(ph2d_editor_core::interaction::InteractiveState::CurvePoint { .. })
            ),
            "handle {i} must be a registered CurvePoint (painted but unregistered = dead)"
        );
    }
    set_current_params(None);
}

/// The `+` button click routes through `apply_event` to a `SetTextParam` whose curve has
/// one more point — the seam from a real button id to the document edit.
#[test]
fn the_add_button_emits_a_curve_with_one_more_point() {
    let _ = drain_param_intents();
    set_current_params(Some(curve_snapshot("c1 0:0:L 1:1:L")));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    let mut state = MotionParamsPanelState;
    host.apply_panel_event::<MotionParamsPanel>(
        &mut state,
        WidgetEvent::Click(crate::snapshot::param_curve_add_id(0)),
    );
    let intents = drain_param_intents();
    assert_eq!(intents.len(), 1, "one edit");
    let MotionParamIntent::SetTextParam { node, param, value } = &intents[0] else {
        panic!("a curve edit rides SetTextParam");
    };
    assert_eq!((*node, *param), (9, "curve"));
    assert_eq!(
        ph2d_curve::parse(value).unwrap().points.len(),
        3,
        "+ grew the curve from 2 to 3 points"
    );
    set_current_params(None);
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
}
