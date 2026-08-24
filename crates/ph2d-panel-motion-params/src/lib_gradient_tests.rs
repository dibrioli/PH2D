//! The Gradient-editor seam gate (doc 85), split from `lib_tests.rs` for the 600-LOC
//! panel-file cap — it was the test that pushed the file over. `use super::*` is the
//! same scope `lib_tests.rs` uses (the testkit + the pooled-id helpers + the snapshot).

use super::*;

/// Pinta uma row de Gradiente com `value` e devolve o host (vivo, para clicar) + os rects
/// que o paint registrou. Existe porque os gates do ESPAÇO precisam pintar a MESMA row com
/// duas rampas diferentes, e a metade que importa neles é *o que aparece*.
fn painted_ramp(
    value: &str,
) -> (
    ph2d_ui_testkit::MockPanelHost,
    MotionParamsPanelState,
    Vec<(ph2d_a11y::NodeId, ph2d_editor_core::zones::Rect)>,
) {
    set_current_params(Some(ParamsSnapshot {
        node: 7,
        title: "Color Ramp".into(),
        modified: Default::default(),
        sections: Vec::new(),
        folded_by_default: std::collections::BTreeSet::new(),
        rows: vec![ParamRow::Gradient(GradientRow {
            name: "ramp",
            label: "Gradient".into(),
            value: value.into(),
        })],
    }));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    let mut state = MotionParamsPanelState;
    let rects = host.paint::<MotionParamsPanel>(
        &mut state,
        ph2d_editor_core::zones::Rect {
            x: 0.0,
            y: 0.0,
            w: 1200.0,
            h: 800.0,
        },
    );
    (host, state, rects)
}

/// **O ESPAÇO DE INTERPOLAÇÃO É ALCANÇÁVEL, E O MATIZ SÓ EXISTE ONDE DECIDE ALGUMA COISA.**
///
/// As duas metades que a família *"verde e morto"* ensina, mais a que este botão exige:
/// presença (o espaço pinta um rect vivo e um clique REAL sobre ele muda a string) **e
/// AUSÊNCIA** (em RGB o matiz não pinta, porque o braço `Rgb` do `mix2` nunca chama
/// `lerp_hue` — ele seria um botão que gira e não muda um pixel).
#[test]
fn the_space_button_cycles_and_the_hue_button_only_exists_outside_rgb() {
    use crate::snapshot::{param_grad_hue_id, param_grad_space_id};
    let _ = drain_param_intents();

    // ── RGB: o espaço aparece, o matiz NÃO ──
    let (mut host, mut state, rects) = painted_ramp("g1 2 0:0,0,1 1:1,1,0");
    let live = |rects: &Vec<(ph2d_a11y::NodeId, ph2d_editor_core::zones::Rect)>,
                id: ph2d_a11y::NodeId| {
        rects
            .iter()
            .find(|(rid, r)| *rid == id && r.w > 0.0 && r.h > 0.0)
            .map(|(_, r)| *r)
    };
    let space = live(&rects, param_grad_space_id(0)).expect("o botão de espaço pinta um rect vivo");
    assert!(
        live(&rects, param_grad_hue_id(0)).is_none(),
        "em RGB o caminho de matiz não decide nada, então o botão não é oferecido"
    );

    // Um clique REAL sobre o rect que o paint registrou leva a rampa para HSV — e o `g3`
    // no começo da string é a prova de que a escolha tem onde ser guardada.
    for ev in host.click_at(space.x + space.w * 0.5, space.y + space.h * 0.5) {
        host.apply_panel_event::<MotionParamsPanel>(&mut state, ev);
    }
    let out = drain_param_intents()
        .into_iter()
        .find_map(|it| match it {
            MotionParamIntent::SetTextParam {
                param: "ramp",
                value,
                ..
            } => Some(value),
            _ => None,
        })
        .expect("clicar o espaço emite um SetTextParam");
    assert!(
        out.starts_with("g3 "),
        "sair do RGB pede o header novo: {out}"
    );
    assert_eq!(
        ph2d_color::parse_gradient(&out).expect("válido").color_mode,
        ph2d_color::RampColorMode::Hsv,
        "RGB cicla para HSV"
    );

    // ── HSV: agora o matiz APARECE, e um clique real o cicla ──
    let (mut host, mut state, rects) = painted_ramp(&out);
    let hue = live(&rects, param_grad_hue_id(0)).expect("fora do RGB o matiz é oferecido");
    for ev in host.click_at(hue.x + hue.w * 0.5, hue.y + hue.h * 0.5) {
        host.apply_panel_event::<MotionParamsPanel>(&mut state, ev);
    }
    let cycled = drain_param_intents()
        .into_iter()
        .find_map(|it| match it {
            MotionParamIntent::SetTextParam {
                param: "ramp",
                value,
                ..
            } => Some(value),
            _ => None,
        })
        .expect("clicar o matiz emite um SetTextParam");
    let back = ph2d_color::parse_gradient(&cycled).expect("válido");
    assert_eq!(back.hue, ph2d_color::RampHue::Far, "Near cicla para Far");
    assert_eq!(
        back.color_mode,
        ph2d_color::RampColorMode::Hsv,
        "ciclar o matiz não mexe no espaço"
    );
    set_current_params(None);
}

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
        modified: Default::default(),
        sections: Vec::new(),
        folded_by_default: std::collections::BTreeSet::new(),
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
