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
        modified: Default::default(),
        sections: Vec::new(),
        folded_by_default: std::collections::BTreeSet::new(),
        rows: vec![ParamRow::Scalar(ScalarRow {
            name: "rows",
            label: "Rows".into(),
            value: 3.0,
            min: 1.0,
            max: 20.0,
            hard_min: 0.0,
            hard_max: 20.0,
            step: 1.0,
            integer: true,
            driven_by: None,
            display: RowDisplay::default(),
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
        modified: Default::default(),
        sections: Vec::new(),
        folded_by_default: std::collections::BTreeSet::new(),
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
        modified: Default::default(),
        sections: Vec::new(),
        folded_by_default: std::collections::BTreeSet::new(),
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
        modified: Default::default(),
        sections: Vec::new(),
        folded_by_default: std::collections::BTreeSet::new(),
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

/// A `value.attribute`-style channel picker snapshot: two channels (Speed, Opacity),
/// `selected` the current segment (2 = Custom), `custom` the live column text.
fn channels_snapshot(selected: usize, custom: &str) -> ParamsSnapshot {
    ParamsSnapshot {
        node: 7,
        title: "Attribute".into(),
        modified: Default::default(),
        sections: Vec::new(),
        folded_by_default: std::collections::BTreeSet::new(),
        rows: vec![ParamRow::Channels(ChannelsRow {
            label: "Read".into(),
            text_param: "attr",
            mode_param: "mode",
            channels: vec![("Speed", "vel", 1), ("Opacity", "tint", 5)],
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
    set_current_params(Some(channels_snapshot(1, "tint"))); // Opacity currently
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
        modified: Default::default(),
        sections: Vec::new(),
        folded_by_default: std::collections::BTreeSet::new(),
        rows: vec![ParamRow::Channels(ChannelsRow {
            label: "Read".into(),
            text_param: "attr",
            mode_param: "mode",
            channels: vec![("Speed", "vel", 1), ("Opacity", "tint", 5)],
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

/// **The live-column chips must be REGISTERED by `populate`** — or they paint and
/// hit-register yet stay DEAD under the mouse, because the dispatch only routes a
/// click to a widget the store knows (the exact seam the synthetic-Click gate above
/// cannot see: a synthetic event skips the store's focus check). FALSIFIED by
/// dropping the extra-chip registration loop.
#[test]
fn the_live_column_chips_are_registered_so_a_click_reaches_them() {
    let host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    assert!(
        host.store()
            .button_state(param_enum_id(0, CHANNELS_EXTRA_BASE))
            .is_some(),
        "the first live-column chip is a registered (clickable) button"
    );
    assert!(
        host.store()
            .button_state(param_enum_id(0, CHANNELS_EXTRA_BASE + 1))
            .is_some(),
        "the second live-column chip is registered too"
    );
}

/// **A source-picker chip writes the published NAME to the text param** (doc 65) — the
/// artist clicks "Ring" (a shape they drew) instead of typing its exact name. The chip
/// ids reuse the enum-option pool `populate` registers, so a real click reaches them (a
/// synthetic Click here proves the decode; the registration is the enum pool, gated
/// elsewhere). FALSIFIED by dropping the `ParamRow::Source` arm in the click handler.
#[test]
fn picking_a_source_chip_writes_the_published_name() {
    let _ = drain_param_intents();
    set_current_params(Some(ParamsSnapshot {
        node: 4,
        title: "Path".into(),
        modified: Default::default(),
        sections: Vec::new(),
        folded_by_default: std::collections::BTreeSet::new(),
        rows: vec![ParamRow::Source(SourceRow {
            label: "Shape".into(),
            param: "path",
            options: vec!["Track".into(), "Ring".into()],
            current: String::new(),
        })],
    }));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    let mut state = MotionParamsPanelState;
    // The chip ids are registered (the shared enum pool) → a real click would land.
    assert!(
        host.store().button_state(param_enum_id(0, 1)).is_some(),
        "the source chips are registered (clickable) buttons"
    );
    // The 2nd chip is "Ring".
    host.apply_panel_event::<MotionParamsPanel>(
        &mut state,
        WidgetEvent::Click(param_enum_id(0, 1)),
    );
    assert_eq!(
        drain_param_intents(),
        vec![MotionParamIntent::SetTextParam {
            node: 4,
            param: "path",
            value: "Ring".into(),
        }],
        "clicking a source chip writes its published name to the text param"
    );
    set_current_params(None);
}

fn curve_snapshot(value: &str) -> ParamsSnapshot {
    ParamsSnapshot {
        node: 9,
        title: "Remap".into(),
        modified: Default::default(),
        sections: Vec::new(),
        folded_by_default: std::collections::BTreeSet::new(),
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

/// **Um chip de coluna viva nunca reusa o id de um segmento curado.**
///
/// As duas fileiras de um picker em Custom saem do MESMO pool (`param_enum_id(slot, opt)`),
/// separadas só por `CHANNELS_EXTRA_BASE` começar onde os segmentos acabam. Se a base ficar
/// ABAIXO do teto, um chip e um segmento pedem a mesma string ⇒ o mesmo `NodeId` ⇒ **um
/// widget**, com dois desenhos e um roteamento: o clique no chip arma o segmento, ou o
/// contrário, sem erro em lugar nenhum.
///
/// ⚠️ O gate é escrito sobre os **IDS QUE O PAINEL PEDE**, não sobre a desigualdade entre as
/// duas constantes — comparar `CHANNELS_EXTRA_BASE >= MAX_ENUM_OPTIONS` seria repetir a
/// derivação e nunca poderia falhar. A mutação que ele existe para pegar é a base voltar a
/// ser um literal (`32`) enquanto o teto sobe.
#[test]
fn the_live_column_chips_never_land_on_a_curated_segments_id() {
    for slot in 0..MAX_PARAM_ROWS {
        let curated: Vec<_> = (0..MAX_ENUM_OPTIONS)
            .map(|opt| param_enum_id(slot, opt))
            .collect();
        for j in 0..MAX_ENUM_OPTIONS {
            let chip = param_enum_id(slot, CHANNELS_EXTRA_BASE + j);
            assert!(
                !curated.contains(&chip),
                "chip {j} do slot {slot} caiu em cima de um segmento curado \
                 (base {CHANNELS_EXTRA_BASE}, teto {MAX_ENUM_OPTIONS})"
            );
        }
    }
}
