//! ⭐ Girar 90° e espelhar em todo objecto que não é vetor (dono 04/11): cada clique publica UM
//! `InspectorTransformEdit`, a soma de um quarto de volta ou o sinal da escala, e nada mais muda.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{PanelHost, PanelHostInternal};
use ph2d_editor_core::screens::hero::InspectorTransformInfo;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, ids, set_current_inspector_transform};
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
        (
            ids::INSP_TRANSFORM_POS_X,
            unidade.from_meters(i.translation[0], ppm),
        ),
        (
            ids::INSP_TRANSFORM_POS_Y,
            unidade.from_meters(i.translation[1], ppm),
        ),
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
    assert!(
        (g[0].rotation_rad - std::f32::consts::FRAC_PI_2).abs() < 1e-3,
        "{g:?}"
    );
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

/// Publica `edit` pelo mesmo caminho do commit da shell (`inspector_commits.rs`): codifica o `Transform`
/// da edição, põe-no na fila de comandos e aplica-a ao mundo.
fn commit(sim: &mut ph2d_ecs::SimWorld, edit: &InspectorTransformInfo) {
    use ph2d_ecs::scene::{
        ComponentRegistry, EditorCommand, EditorCommandQueue, apply_editor_commands,
        register_ecs_components, stable_type_id,
    };
    let mut registo = ComponentRegistry::new();
    register_ecs_components(&mut registo);
    let fila = EditorCommandQueue::new();
    let t = ph2d_ecs::Transform::de_autoria(
        edit.translation,
        edit.rotation_rad,
        edit.scale,
        edit.skew_rad,
    );
    fila.push(EditorCommand::SetComponent {
        entity: edit.entity_bits,
        type_id: stable_type_id("ph2d::ecs::Transform"),
        data: postcard::to_allocvec(&t).unwrap(),
    })
    .unwrap();
    apply_editor_commands(sim.world_mut(), &fila, &registo).unwrap();
}

/// ⭐ Do clique ao objecto: a rotação e o sinal da escala chegam ao `Transform` da entidade, a posição
/// fica, e dois espelhos devolvem-no. Quem o publica não é o teste, é o painel.
/// *Mutação: `Transform::de_autoria` com `abs` na escala; ou sem a rotação da edição.*
#[test]
fn each_click_reaches_the_transform_of_the_object() {
    use ph2d_ecs::{SimWorld, Transform};
    let mut sim = SimWorld::new();
    let autorado = Transform {
        translation: ph2d_core::Vec2::new(1.0, 2.0),
        scale: ph2d_core::Vec2::new(2.0, 3.0),
        ..Transform::default()
    };
    let e = sim.world_mut().spawn(autorado).id();
    let info = InspectorTransformInfo {
        entity_bits: e.to_bits(),
        ..objecto()
    };
    let no_mundo = |sim: &SimWorld| *sim.world().get::<Transform>(e).unwrap();

    let h = clica(ids::INSP_TRANSFORM_FLIP_H, info);
    assert_eq!(h.len(), 1);
    commit(&mut sim, &h[0]);
    assert_eq!(no_mundo(&sim).scale, ph2d_core::Vec2::new(-2.0, 3.0));
    assert_eq!(no_mundo(&sim).translation, autorado.translation);

    let v = clica(ids::INSP_TRANSFORM_FLIP_V, h[0]);
    commit(&mut sim, &v[0]);
    assert_eq!(no_mundo(&sim).scale, ph2d_core::Vec2::new(-2.0, -3.0));

    let g = clica(ids::INSP_TRANSFORM_ROTATE_CW, v[0]);
    commit(&mut sim, &g[0]);
    assert!(
        (no_mundo(&sim).rotation - std::f32::consts::FRAC_PI_2).abs() < 1e-3,
        "{}",
        no_mundo(&sim).rotation
    );
    assert_eq!(no_mundo(&sim).scale, ph2d_core::Vec2::new(-2.0, -3.0));

    let volta = clica(ids::INSP_TRANSFORM_FLIP_V, g[0]);
    let volta = clica(ids::INSP_TRANSFORM_FLIP_H, volta[0]);
    commit(&mut sim, &volta[0]);
    assert_eq!(no_mundo(&sim).scale, autorado.scale);
    assert_eq!(no_mundo(&sim).translation, autorado.translation);
}

/// ⭐ O gesto de verdade: num objecto sem vetor os três ícones pintam-se no cartão Transform, e o clique
/// no rect pintado publica a edição. *Mutação: voltar a esconder os giros fora do vetor.*
#[test]
fn the_painted_icons_answer_a_real_click_on_a_non_vector_object() {
    let base = objecto();
    let (mut host, mut state) = mesa(&base);
    ph2d_panel_inspector::set_cards_on(true);
    let janela = ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1366.0, 2000.0);
    host.settle_section_folds();
    let mut pintados = Vec::new();
    for _ in 0..60 {
        pintados = ph2d_editor_core::aquecimento::com_passo_fixo(
            ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
            || host.paint::<InspectorPanel>(&mut state, janela),
        );
    }
    let de = |id| {
        pintados
            .iter()
            .rev()
            .find(|(i, q)| *i == id && q.w > 0.0)
            .map(|(_, q)| *q)
    };
    let r = de(ids::INSP_TRANSFORM_FLIP_H).expect("o espelho não se pintou");
    assert!(
        de(ids::INSP_TRANSFORM_ROTATE_CW).is_some() && de(ids::INSP_TRANSFORM_FLIP_V).is_some(),
        "os outros dois verbos não se pintaram"
    );
    let _ = host.drained_actions();
    let evs = host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert!(
        evs.contains(&WidgetEvent::Click(ids::INSP_TRANSFORM_FLIP_H)),
        "{evs:?}"
    );
    for e in evs {
        host.apply_panel_event::<InspectorPanel>(&mut state, e);
    }
    let edits: Vec<_> = host
        .drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::InspectorTransformEdit(i) => Some(i),
            _ => None,
        })
        .collect();
    ph2d_panel_inspector::set_cards_on(false);
    set_current_inspector_transform(None);
    assert_eq!(edits.len(), 1, "{edits:?}");
    assert_eq!(edits[0].scale, [-2.0, 3.0]);
}
