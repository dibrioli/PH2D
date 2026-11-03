//! ⭐ **CartoesUI C3/C4: todo ícone SEM texto dentro de um cartão tem dica** (o tooltip «por cima
//! dos ícones»). Um ícone mudo só se explica pelo desenho — a regra do rótulo (D10) estendida ao
//! ícone. O censo lê os ícones que a lista DECLARA ao pintar (`panel::cards::dicas`), no Inspector
//! com uma forma com Fill e Stroke; controlo: ≥ 1 ícone. A parte do Bones vive na linha do Bones.
#![cfg(feature = "panel-inspector")]

use ph2d_editor_core::NodeId;
use ph2d_editor_core::panel::cards::dicas;
use ph2d_editor_core::screens::hero::{HeroScreen, InspectorTransformInfo};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tool_vector::appearance::{self, Appearance, BasePaint, PaintRow};

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 1400.0,
};

fn hero_so_com(id: &'static str) -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_panel_inspector::set_cards_on(true);
    let mut h = HeroScreen::new(NodeId(1));
    let ids: Vec<&'static str> = ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels().iter().map(|p| p.manifest.id).collect()
    });
    for n in ids {
        h.panel_visibility.insert(n, n == id);
    }
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    for _ in 0..12 {
        scene.reset();
        ph2d_editor_core::screens::hero::paint_hero_screen(&mut h, VIEWPORT, &mut scene, &mut text);
    }
    h
}

fn camada(is_fill: bool, cor: [u8; 4]) -> PaintRow {
    PaintRow {
        is_fill,
        color: cor,
        width: 3.0,
        enabled: true,
        opacity: 1.0,
        blend: ph2d_vec_scene::BlendMode::Normal,
        offset: [0.0, 0.0],
        dilate: 0.0,
        dilate_join: 0,
        mixed: Default::default(),
        ramp: None,
        align: ph2d_vec_scene::StrokeAlign::Centre,
    }
}

fn sem_dica(h: &HeroScreen, panel: NodeId) -> (usize, Vec<NodeId>) {
    let todos = dicas::icon_only_ids(panel);
    let mudos = todos
        .iter()
        .copied()
        .filter(|id| h.store.tooltip_for(*id).is_none())
        .collect();
    (todos.len(), mudos)
}

#[test]
fn every_icon_in_a_card_has_a_tooltip() {
    ph2d_panel_inspector::set_current_inspector_transform(Some(InspectorTransformInfo {
        entity_bits: 7,
        translation: [0.0, 0.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
        skew_rad: [0.0, 0.0],
    }));
    appearance::set_base_paints(Some([BasePaint::Absent, BasePaint::Absent]));
    appearance::set_current_appearance(Some(Appearance {
        opacity: 1.0,
        blend: ph2d_vec_scene::BlendMode::Normal,
        layers: vec![camada(true, [76, 141, 240, 255]), camada(false, [255; 4])],
        mixed_opacity: false,
        mixed_blend: false,
        layers_differ: false,
        order: Vec::new(),
    }));
    let h = hero_so_com("inspector");
    let (n, mudos) = sem_dica(&h, ph2d_editor_core::ids::INSP_PANEL);
    assert!(n > 0, "controlo: o Inspector não declarou nenhum ícone");
    assert!(
        mudos.is_empty(),
        "Inspector: {} de {n} ícones sem dica: {mudos:?}",
        mudos.len()
    );
    appearance::set_current_appearance(None);
    ph2d_panel_inspector::set_current_inspector_transform(None);
}

/// O campo numérico sem dica autorada diz como se arrasta, com o rótulo dele (maquete B).
#[test]
fn a_card_field_hints_how_to_drag_it() {
    ph2d_panel_inspector::set_current_inspector_transform(Some(InspectorTransformInfo {
        entity_bits: 7,
        translation: [0.0, 0.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
        skew_rad: [0.0, 0.0],
    }));
    let h = hero_so_com("inspector");
    let dica = dicas::field_hint(&h.store, ph2d_panel_inspector::ids::INSP_TRANSFORM_ROT)
        .expect("o Rotation não tem dica de arrasto");
    assert!(
        dica.starts_with("Rotation"),
        "a dica não traz o rótulo: {dica}"
    );
    assert!(dica.contains("Shift"), "a dica não ensina o Shift: {dica}");
    ph2d_panel_inspector::set_current_inspector_transform(None);
}
