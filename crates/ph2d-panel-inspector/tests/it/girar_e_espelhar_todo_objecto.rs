//! ⭐ Girar 90° e espelhar em todo objecto que não é vetor (dono 04/11): cada clique publica UM
//! `InspectorTransformEdit`, a soma de um quarto de volta ou o sinal da escala, e nada mais muda.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{PanelHost, PanelHostInternal};
use ph2d_editor_core::screens::hero::InspectorTransformInfo;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, ids, set_current_inspector_transform,
};
use ph2d_ui_testkit::MockPanelHost;

fn objecto() -> InspectorTransformInfo {
    InspectorTransformInfo {
        entity_bits: 7,
        translation: [1.0, 2.0],
        rotation_rad: 0.0,
        scale: [2.0, 3.0],
        skew_rad: [0.0, 0.0],
    }
}

/// O painel com as caixas semeadas pelo objecto, como o `sync` as deixa no app.
fn mesa(i: &InspectorTransformInfo) -> (MockPanelHost, InspectorState) {
    set_current_inspector_transform(Some(*i));
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    let (unidade, ppm) = (host.project().display_unit, host.project().pixels_per_meter);
    for (id, v) in [
        (ids::INSP_TRANSFORM_POS_X, unidade.from_meters(i.translation[0], ppm)),
        (ids::INSP_TRANSFORM_POS_Y, unidade.from_meters(i.translation[1], ppm)),
        (ids::INSP_TRANSFORM_ROT, i.rotation_rad.to_degrees()),
        (ids::INSP_TRANSFORM_SCALE_X, i.scale[0]),
        (ids::INSP_TRANSFORM_SCALE_Y, i.scale[1]),
        (ids::INSP_TRANSFORM_SKEW_X, 0.0),
        (ids::INSP_TRANSFORM_SKEW_Y, 0.0),
    ] {
        host.store_mut().set_number_value(id, f64::from(v));
    }
    let _ = (&mut state, host.drained_actions());
    (host, state)
}

/// Clica no id e devolve as edições publicadas.
fn clica(id: NodeId, info: InspectorTransformInfo) -> Vec<InspectorTransformInfo> {
    let (mut host, mut state) = mesa(&info);
    host.apply_panel_event::<InspectorPanel>(&mut state, WidgetEvent::Click(id));
    let out = host
        .drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::InspectorTransformEdit(i) => Some(i),
            _ => None,
        })
        .collect();
    set_current_inspector_transform(None);
    out
}

/// *Mutação: tirar o `FRAC_PI_2`, ou trocar o sinal do eixo errado.*
#[test]
fn each_click_publishes_one_edit_with_only_its_channel_changed() {
    let base = objecto();
    let g = clica(ids::INSP_TRANSFORM_ROTATE_CW, base);
    assert_eq!(g.len(), 1, "um gesto, um passo de desfazer");
    assert!((g[0].rotation_rad - std::f32::consts::FRAC_PI_2).abs() < 1e-3, "{g:?}");
    assert_eq!(g[0].scale, base.scale);

    let h = clica(ids::INSP_TRANSFORM_FLIP_H, base);
    assert_eq!(h.len(), 1);
    assert_eq!(h[0].scale, [-2.0, 3.0]);
    assert_eq!(h[0].translation, base.translation);

    let v = clica(ids::INSP_TRANSFORM_FLIP_V, base);
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].scale, [2.0, -3.0]);
}

/// Espelhar duas vezes devolve o objecto; quatro quartos de volta também (fica em −180°…180°).
#[test]
fn the_gestures_return() {
    let mut i = objecto();
    for _ in 0..2 {
        i = clica(ids::INSP_TRANSFORM_FLIP_H, i)[0];
    }
    assert_eq!(i.scale, objecto().scale);
    let mut i = objecto();
    for _ in 0..4 {
        i = clica(ids::INSP_TRANSFORM_ROTATE_CW, i)[0];
        assert!(i.rotation_rad.abs() <= std::f32::consts::PI + 1e-3);
    }
    assert!(i.rotation_rad.abs() < 1e-3, "{}", i.rotation_rad);
}

/// Sem objecto, os verbos calam-se.
#[test]
fn without_an_object_the_verbs_are_silent() {
    let (mut host, mut state) = mesa(&objecto());
    set_current_inspector_transform(None);
    host.apply_panel_event::<InspectorPanel>(
        &mut state,
        WidgetEvent::Click(ids::INSP_TRANSFORM_FLIP_H),
    );
    assert!(host.drained_actions().is_empty());
}
