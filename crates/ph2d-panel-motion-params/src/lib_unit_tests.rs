//! **The way BACK** (doc 88, Wave A) — a `#[path]` sibling of `lib_range_tests.rs`
//! split along the same subject line: that one answers *how far does the box
//! reach*, this one answers *what does the document hear when it gets there*.
//!
//! The bridge converts a stored quantity into the artist's face once, on the way
//! out. Everything the panel then does — the affine, the track, the number range,
//! the painted text — happens in that face, which is right, because it is the
//! face the artist is looking at. The one place the face must be undone is the
//! **emit**, and a row has TWO emit sites (the typed chip and the slider's
//! affine). Two sites is exactly one site away from a `gap_x` written to the
//! document in PIXELS and read by the cook as METRES — a hundredfold error with
//! no error message ([[feedback_layered_defenses_need_per_layer_gates]]).

use super::*;

/// A row wearing the pixel face of a world length: the document stores metres,
/// the artist reads `× 100`. The numbers are already converted, exactly as the
/// bridge hands them over — `±10 m` of slider is `±1000 px` of row.
fn pixel_row(value_px: f64) -> ParamsSnapshot {
    ParamsSnapshot {
        node: 7,
        title: "Emitter".into(),
        rows: vec![ParamRow::Scalar(ScalarRow {
            name: "x",
            label: "Origin X".into(),
            value: value_px,
            min: -1000.0,
            hard_min: -1000.0,
            max: 1000.0,
            hard_max: 1000.0,
            step: 10.0,
            integer: false,
            driven: false,
            display: RowDisplay::new(100.0, "px"),
        })],
    }
}

fn one_intent() -> MotionParamIntent {
    let mut intents = drain_param_intents();
    assert_eq!(intents.len(), 1, "expected exactly one param edit");
    intents.pop().expect("checked")
}

/// **The typed box: the artist types pixels, the document hears metres.**
///
/// A value outside the slider's soft span is the box's to report (the dual-range
/// rule), so this drives the site that speaks out there. `250 px ÷ 100 = 2.5 m` —
/// and if the conversion were missing the document would receive `250`, which the
/// cook would place a hundred metres off-screen without complaining once.
#[test]
fn a_typed_pixel_value_reaches_the_document_as_metres() {
    let _ = drain_param_intents();
    // Below the slider's floor so the chip owns the report, not the affine.
    set_current_params(Some(pixel_row(-1500.0)));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    let mut state = MotionParamsPanelState;
    let chip = param_chip_id(0);
    host.set_number_value(chip, -1500.0);
    host.apply_panel_event::<MotionParamsPanel>(&mut state, WidgetEvent::ValueChanged(chip));
    assert_eq!(
        one_intent(),
        MotionParamIntent::SetParam {
            node: 7,
            param: "x",
            value: -15.0,
        },
        "-1500 px is -15 m; the document must never hear the artist's face"
    );
}

/// **The SLIDER is the second emit site, and it needs its own gate.**
///
/// The two sites are unrelated code paths: the chip reports a typed number, the
/// slider reports `track → value` through the row's affine. Converting one and
/// forgetting the other is the exact shape of defect that ships green — the
/// artist types and it works, then drags and the value silently jumps by a
/// hundred. Track `0.75` over `[-1000, 1000] px` is `500 px`, i.e. `5 m`.
#[test]
fn a_dragged_slider_reaches_the_document_as_metres_too() {
    let _ = drain_param_intents();
    set_current_params(Some(pixel_row(0.0)));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    let mut state = MotionParamsPanelState;
    let slider = param_slider_id(0);
    host.set_slider_value(slider, 0.75);
    host.apply_panel_event::<MotionParamsPanel>(&mut state, WidgetEvent::ValueChanged(slider));
    assert_eq!(
        one_intent(),
        MotionParamIntent::SetParam {
            node: 7,
            param: "x",
            value: 5.0,
        },
        "the affine lands in the DISPLAY face, so the same door must undo it"
    );
}

/// **The control** — a row that declared no unit reports the number verbatim.
/// A neutral face has scale exactly `1.0`, and `x / 1.0` is `x` bit for bit in
/// IEEE-754, so every param in the app that has not opted in is untouched.
#[test]
fn an_unitless_row_reports_the_number_verbatim() {
    let _ = drain_param_intents();
    let mut snap = pixel_row(0.0);
    if let ParamRow::Scalar(row) = &mut snap.rows[0] {
        row.display = RowDisplay::default();
    }
    set_current_params(Some(snap));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    let mut state = MotionParamsPanelState;
    let slider = param_slider_id(0);
    host.set_slider_value(slider, 0.75);
    host.apply_panel_event::<MotionParamsPanel>(&mut state, WidgetEvent::ValueChanged(slider));
    assert_eq!(
        one_intent(),
        MotionParamIntent::SetParam {
            node: 7,
            param: "x",
            value: 500.0,
        }
    );
}

/// **A face with a broken scale must not poison the document.**
///
/// `RowDisplay::new` refuses a non-finite or non-positive scale and falls back to
/// the neutral `1.0`. Without that, `to_stored` is a division that hands the
/// document `inf` or `NaN` — and a param the cook reads as NaN takes the whole
/// stream with it. The boundary is allowed to be useless; it is not allowed to be
/// the thing that corrupts.
#[test]
fn a_broken_scale_falls_back_to_neutral_instead_of_dividing_by_it() {
    for bad in [0.0, -100.0, f64::NAN, f64::INFINITY] {
        let face = RowDisplay::new(bad, "px");
        assert_eq!(face.scale, 1.0, "a scale of {bad} must not reach to_stored");
        assert_eq!(face.to_stored(42.0), 42.0);
    }
}

/// **One formatter, so a value never wears two faces.**
///
/// The chip's `display_override` and the wired row's accent text both go through
/// [`scalar_text`]. If they did not, plugging a wire into a Gap would change the
/// reading from `100 px` to `100` — which looks exactly like the value changed
/// when only its author did.
#[test]
fn the_one_formatter_carries_the_suffix_and_omits_it_when_there_is_none() {
    let ParamRow::Scalar(row) = &pixel_row(0.0).rows[0] else {
        unreachable!("the fixture is a scalar row")
    };
    assert_eq!(scalar_text(row, 100.0), "100 px");

    let mut bare = row.clone();
    bare.display = RowDisplay::default();
    assert_eq!(
        scalar_text(&bare, 100.0),
        "100",
        "no suffix means no trailing space either"
    );
}

/// **And BOTH painters go through it** — the half the test above cannot see.
///
/// A gate over the formatter proves the formatter; it says nothing about who
/// calls it, and the divergence this prevents lives in the caller. So this reads
/// the painters' own source: the scalar row and the wire-driven row must each
/// name `scalar_text`, and neither may reach for the raw number formatter behind
/// its back. The positive control is that the scan found the file at all.
#[test]
fn both_row_painters_read_through_the_one_formatter() {
    let src = include_str!("rows_paint_kinds.rs");
    assert!(
        src.contains("fn paint_driven_row") && src.contains("fn paint_scalar_row"),
        "positive control: the painters moved, so this gate is scanning nothing"
    );
    assert_eq!(
        src.matches("scalar_text(row").count(),
        2,
        "both painters must format through the one door — the wired readout and \
         the chip cannot disagree about what a value reads as"
    );
    assert!(
        !src.contains("format_number("),
        "a painter formatting the number itself is the second face this gate \
         exists to prevent"
    );
}
