//! The Gradient-editor seam gate (doc 85), split from `lib_tests.rs` for the 600-LOC
//! panel-file cap — it was the test that pushed the file over. `use super::*` is the
//! same scope `lib_tests.rs` uses (the testkit + the pooled-id helpers + the snapshot).

use super::*;

/// The Gradient editor (doc 85) is **reachable and wired**: painting it registers each
/// stop swatch as a live picker swatch (seeded with the stop's sRGB) and the position
/// markers as `CurvePoint` handles, and a REAL pointer click on the `+` button (over the
/// rect the paint registered — never a synthetic event) pushes a `SetTextParam` that adds a
/// stop. Proving the two halves the "green-but-dead" family teaches: painted-and-hittable
/// (the rect) AND live-under-the-mouse (the store state the dispatcher needs).
#[test]
fn the_gradient_editor_is_reachable_and_wired() {
    use crate::snapshot::{param_grad_add_id, param_grad_preset_id, param_grad_stop_id};
    let _ = drain_param_intents();
    // A two-stop black→white gradient in the "ramp" text param.
    set_current_params(Some(ParamsSnapshot {
        node: 7,
        title: "Color Ramp".into(),
        rows: vec![ParamRow::Gradient(GradientRow {
            name: "ramp",
            label: "Gradient".into(),
            value: "g1 2 0:0,0,0 1:1,1,1".into(),
        })],
    }));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    let mut state = MotionParamsPanelState;
    let viewport = ph2d_editor_core::zones::Rect {
        x: 0.0,
        y: 0.0,
        w: 1200.0,
        h: 800.0,
    };
    let rects = host.paint::<MotionParamsPanel>(&mut state, viewport);

    // Both stop swatches are LIVE picker swatches (a Down opens the OKLCH picker) + seeded
    // with the stop's colour (black at 0, white at 1) — not dead under the mouse.
    for (i, want) in [(0usize, [0, 0, 0, 255]), (1usize, [255, 255, 255, 255])] {
        let sid = param_grad_swatch_id("ramp", i);
        assert!(
            host.store().is_picker_swatch(sid),
            "stop {i} swatch must be a picker swatch"
        );
        assert_eq!(
            host.store().widget_color(sid),
            Some(want),
            "stop {i} swatch seeded with its colour"
        );
    }
    // Each position marker is a registered `CurvePoint` (the dispatcher only makes a
    // focusable id active — an unregistered marker is stone dead).
    for i in 0..2 {
        assert!(
            matches!(
                host.store().get(param_grad_stop_id(0, i)),
                Some(InteractiveState::CurvePoint { .. })
            ),
            "marker {i} must be a live CurvePoint"
        );
    }

    // Click the REAL `+` button rect the paint registered → the dispatch's event, fed to
    // the panel, adds a stop.
    let add = rects
        .iter()
        .find(|(id, r)| *id == param_grad_add_id(0) && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
        .expect("the + button paints a hittable rect");
    for ev in host.click_at(add.x + add.w * 0.5, add.y + add.h * 0.5) {
        host.apply_panel_event::<MotionParamsPanel>(&mut state, ev);
    }
    let intents = drain_param_intents();
    let added = intents.iter().find_map(|it| match it {
        MotionParamIntent::SetTextParam { param, value, .. } if *param == "ramp" => Some(value),
        _ => None,
    });
    let value = added.expect("clicking + emits a SetTextParam on the ramp");
    assert_eq!(
        ph2d_color::parse_gradient(value)
            .expect("valid gradient")
            .len(),
        3,
        "the + button added a stop (2 -> 3)"
    );

    // A preset chip LOADS that preset into the editable ramp (doc 85) — the presets appear
    // in the editor and become draggable stops. Click Rainbow (index 0) → the ramp string
    // becomes the 7-stop rainbow.
    let rainbow = rects
        .iter()
        .find(|(id, r)| *id == param_grad_preset_id(0, 0) && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
        .expect("the Rainbow preset chip paints a hittable rect");
    for ev in host.click_at(rainbow.x + rainbow.w * 0.5, rainbow.y + rainbow.h * 0.5) {
        host.apply_panel_event::<MotionParamsPanel>(&mut state, ev);
    }
    let seeded = drain_param_intents()
        .into_iter()
        .find_map(|it| match it {
            MotionParamIntent::SetTextParam {
                param: "ramp",
                value,
                ..
            } => Some(value),
            _ => None,
        })
        .expect("clicking Rainbow seeds the ramp");
    assert_eq!(
        ph2d_color::parse_gradient(&seeded)
            .expect("valid gradient")
            .len(),
        ph2d_color::GradientPreset::Rainbow.ramp().len(),
        "the Rainbow chip loaded the 7-stop rainbow into the editable ramp"
    );
    set_current_params(None);
}
