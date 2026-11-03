//! ⭐ D25 (CartoesUI, 02/11): o «+» de Add component aparece também no TOPO da lista do Inspector,
//! à esquerda do «…», e pede o mesmo que o pé.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::screens::hero::InspectorTransformInfo;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, set_cards_on, set_current_inspector_transform};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

#[test]
fn the_plus_at_the_top_of_the_list_adds_a_component() {
    set_cards_on(true);
    set_current_inspector_transform(Some(InspectorTransformInfo {
        entity_bits: 0xABCD_1234,
        translation: [0.0, 0.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
        skew_rad: [0.0, 0.0],
    }));
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = Default::default();
    let pintados = host.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let menu = ph2d_editor_core::panel::cards::list_menu_of(ids::INSP_PANEL);
    let tres_pontos = pintados
        .iter()
        .find(|(id, _)| *id == menu)
        .map(|(_, r)| *r)
        .expect("o «…» da lista foi pintado");
    let mais: Vec<Rect> = pintados
        .iter()
        .filter(|(id, _)| *id == ph2d_panel_inspector::ids::INSP_ADD_COMPONENT)
        .map(|(_, r)| *r)
        .collect();
    let topo = mais
        .iter()
        .find(|r| {
            (r.y - tres_pontos.y).abs() < 0.5
                && r.x + r.w <= tres_pontos.x + 0.5
                && (r.w, r.h) == (tres_pontos.w, tres_pontos.h)
        })
        .unwrap_or_else(|| {
            panic!("sem «+» no topo, à esquerda do «…» ({tres_pontos:?}); +: {mais:?}")
        });
    let evs = host.click_at(topo.x + topo.w * 0.5, topo.y + topo.h * 0.5);
    for ev in evs {
        host.apply_panel_event::<InspectorPanel>(&mut st, ev);
    }
    let pediu = host.drained_actions().into_iter().any(|a| {
        matches!(
            a,
            EditorAction::InspectorAddComponentRequested {
                entity_bits: 0xABCD_1234
            }
        )
    });
    set_current_inspector_transform(None);
    assert!(pediu, "o «+» do topo não pediu Add component");
}

/// ⭐ D25b (02/11): o «+» do topo é do Inspector GERAL (modo Object) — em Draw não aparece; o pé
/// «+ Add component» continua.
#[test]
fn the_plus_at_the_top_is_only_in_object_mode() {
    use ph2d_editor_core::object_mode::ObjectMode;
    use ph2d_editor_core::panel::PanelHostInternal;
    set_cards_on(true);
    set_current_inspector_transform(Some(InspectorTransformInfo {
        entity_bits: 0xABCD_1234,
        translation: [0.0, 0.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
        skew_rad: [0.0, 0.0],
    }));
    let menu = ph2d_editor_core::panel::cards::list_menu_of(ids::INSP_PANEL);
    let mut contagem = Vec::new();
    for modo in [ObjectMode::Object, ObjectMode::Draw] {
        let mut host = MockPanelHost::with_panel::<InspectorPanel>();
        host.store_mut()
            .publish_add_at_top(modo == ObjectMode::Object);
        let mut st = Default::default();
        let pintados = host.paint::<InspectorPanel>(&mut st, VIEWPORT);
        let tres = pintados
            .iter()
            .find(|(id, _)| *id == menu)
            .map(|(_, r)| *r)
            .expect("«…»");
        let mais = pintados
            .iter()
            .filter(|(id, _)| *id == ph2d_panel_inspector::ids::INSP_ADD_COMPONENT)
            .map(|(_, r)| *r)
            .collect::<Vec<_>>();
        let no_topo = mais.iter().filter(|r| (r.y - tres.y).abs() < 0.5).count();
        contagem.push((modo, no_topo, mais.len()));
    }
    set_current_inspector_transform(None);
    assert_eq!(
        contagem,
        vec![(ObjectMode::Object, 1, 2), (ObjectMode::Draw, 0, 1)],
        "(modo, «+» no topo, «+» ao todo)"
    );
}
