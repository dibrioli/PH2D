//! A corrente do Scale (T18): fechada, mudar um eixo leva o outro na proporção de ANTES.

use super::*;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::InspectorTransformInfo;
use ph2d_tool_vector::appearance::PaintRow;
use ph2d_ui_testkit::MockPanelHost;

/// Escala de partida `2 × 1` (proporção 2:1, não 1:1 — com 1:1 «copiar o valor» passaria).
fn info(scale: [f32; 2]) -> InspectorTransformInfo {
    InspectorTransformInfo {
        entity_bits: 1,
        translation: [0.0, 0.0],
        rotation_rad: 0.0,
        scale,
        skew_rad: [0.0, 0.0],
    }
}

fn host() -> MockPanelHost {
    let mut h = MockPanelHost::new();
    crate::populate::populate(h.store_mut());
    h
}

fn escala(h: &MockPanelHost) -> (f64, f64) {
    let s = h.store();
    (
        s.number_value(ids::INSP_TRANSFORM_SCALE_X).unwrap(),
        s.number_value(ids::INSP_TRANSFORM_SCALE_Y).unwrap(),
    )
}

#[test]
fn keep_proportion_carries_the_other_axis_only_when_locked() {
    let antes = info([2.0, 1.0]);
    // Aberta: X muda sozinho.
    let mut h = host();
    h.store_mut()
        .set_number_value(ids::INSP_TRANSFORM_SCALE_Y, 1.0);
    h.store_mut()
        .set_number_value(ids::INSP_TRANSFORM_SCALE_X, 4.0);
    assert!(!scale_locked());
    keep_proportion(&mut h, ids::INSP_TRANSFORM_SCALE_X, &antes);
    assert_eq!(escala(&h), (4.0, 1.0));

    toggle_scale_lock();
    // Fechada: X 2 → 4 leva Y 1 → 2.
    keep_proportion(&mut h, ids::INSP_TRANSFORM_SCALE_X, &antes);
    assert_eq!(escala(&h), (4.0, 2.0));
    // E pelo outro eixo: Y 1 → 3 leva X 2 → 6.
    h.store_mut()
        .set_number_value(ids::INSP_TRANSFORM_SCALE_Y, 3.0);
    keep_proportion(&mut h, ids::INSP_TRANSFORM_SCALE_Y, &antes);
    assert_eq!(escala(&h), (6.0, 3.0));
    // Um eixo a zero não tem proporção: nada muda (nem NaN).
    h.store_mut()
        .set_number_value(ids::INSP_TRANSFORM_SCALE_X, 5.0);
    keep_proportion(&mut h, ids::INSP_TRANSFORM_SCALE_X, &info([0.0, 1.0]));
    assert_eq!(escala(&h), (5.0, 3.0));
    // Outro campo não é do Scale: nada muda.
    keep_proportion(&mut h, ids::INSP_TRANSFORM_POS_X, &antes);
    assert_eq!(escala(&h), (5.0, 3.0));
    toggle_scale_lock();
}

/// Uma forma com UMA camada de preenchimento `cor` (a alfa conta: a do hex de 6 não a muda).
fn uma_camada(cor: [u8; 4]) {
    appearance::set_current_appearance(Some(appearance::Appearance {
        opacity: 1.0,
        blend: ph2d_vec_scene::BlendMode::Normal,
        layers: vec![PaintRow {
            is_fill: true,
            ramp: None,
            align: ph2d_vec_scene::StrokeAlign::Centre,
            color: cor,
            width: 0.0,
            enabled: true,
            opacity: 1.0,
            blend: ph2d_vec_scene::BlendMode::Normal,
            offset: [0.0, 0.0],
            dilate: 0.0,
            dilate_join: 0,
            mixed: Default::default(),
        }],
        order: Vec::new(),
        mixed_opacity: false,
        mixed_blend: false,
        layers_differ: false,
    }));
}

fn host_com_seletor() -> MockPanelHost {
    let mut h = host();
    let picker = ph2d_editor_core::ids::INSP_BLENDER_PICKER;
    let hero = ph2d_editor_core::screens::hero::HeroScreen::new(NodeId(1));
    let st = hero
        .store
        .get(picker)
        .cloned()
        .expect("o hero regista o seletor");
    h.store_mut().register(picker, st);
    h
}

fn digita(h: &mut MockPanelHost, id: NodeId, s: &str) {
    if let Some(ph2d_editor_core::interaction::InteractiveState::TextInput { text, .. }) =
        h.store_mut().get_mut(id)
    {
        *text = s.into();
    }
}

/// ⭐ C4: o código hex do Fill/Stroke edita-se. Enter = uma ESCOLHA no seletor da amostra (a porta
/// que a `ph2d-app-vec` aplica, com a lei da seleção mista); a alfa da camada fica; o seletor fecha
/// quando a forma mostra a cor; um hex inválido não escolhe nada.
#[test]
fn typing_a_hex_code_picks_that_colour_for_the_layer() {
    let picker = ph2d_editor_core::ids::INSP_BLENDER_PICKER;
    let amostra = vids::vector_paint_swatch_id(0);
    uma_camada([0x4C, 0x8D, 0xF0, 128]);
    let mut h = host_com_seletor();
    prepare(h.store_mut());
    match h.store().get(hex_id(0)) {
        Some(ph2d_editor_core::interaction::InteractiveState::TextInput { text, .. }) => {
            assert_eq!(text, "4C8DF0", "a caixa não nasce com o código da camada");
        }
        outro => panic!("a caixa do hex não foi registada: {outro:?}"),
    }
    let mut estado = crate::state::InspectorState::default();
    digita(&mut h, hex_id(0), "#ff0000");
    crate::event::apply_event(
        &mut estado,
        &mut h,
        ph2d_editor_core::interaction::WidgetEvent::Submit(hex_id(0)),
    );
    assert_eq!(
        h.store().picker_target(),
        Some(amostra),
        "o hex não foi à amostra"
    );
    assert_eq!(
        h.store().blender_picker(picker).map(|v| v.0.rgba),
        Some([255, 0, 0, 128]),
        "a cor escolhida não é a do hex com a alfa da camada"
    );
    assert_eq!(h.store().colour_picks(picker), 1, "não contou como escolha");
    // A forma ainda não mudou: o seletor fica aberto.
    prepare(h.store_mut());
    assert_eq!(h.store().picker_target(), Some(amostra));
    // A forma mostra a cor: o seletor fecha.
    uma_camada([255, 0, 0, 128]);
    prepare(h.store_mut());
    assert_eq!(
        h.store().picker_target(),
        None,
        "o seletor do hex não fechou"
    );
    // Inválido: nada.
    digita(&mut h, hex_id(0), "zz");
    crate::event::apply_event(
        &mut estado,
        &mut h,
        ph2d_editor_core::interaction::WidgetEvent::Submit(hex_id(0)),
    );
    assert_eq!(h.store().picker_target(), None);
    assert_eq!(h.store().colour_picks(picker), 1);
    appearance::set_current_appearance(None);
}

/// C1: o olho do cartão é o do objecto publicado, não a caixa da seleção (um preso escondido com a
/// seleção visível). *Mutação: `visivel` a ler a caixa ⇒ o ícone do preso é o da seleção.*
#[test]
fn the_eye_of_the_card_is_of_the_published_object_not_of_the_selection_box() {
    use ph2d_editor_core::IconId;
    use ph2d_editor_core::screens::hero::{InspectorTransformInfo, InspectorVisibilityInfo};
    let store = ph2d_editor_core::interaction::WidgetStore::with_capacity(8);
    crate::state::set_current_inspector_transform(Some(InspectorTransformInfo {
        entity_bits: 3,
        translation: [0.0, 0.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
        skew_rad: [0.0, 0.0],
    }));
    crate::state::set_current_inspector_visibility(Some(InspectorVisibilityInfo {
        entity_bits: 3,
        visible: false,
        mixed: false,
    }));
    ph2d_tool_vector::appearance::set_current_appearance(Some(
        ph2d_tool_vector::appearance::Appearance {
            opacity: 1.0,
            blend: ph2d_vec_scene::BlendMode::Normal,
            layers: Vec::new(),
            mixed_opacity: false,
            mixed_blend: false,
            layers_differ: false,
            order: Vec::new(),
        },
    ));
    let defs = crate::cards::cards(&store);
    let olho = defs
        .iter()
        .find(|c| c.section == super::CARD_APPEARANCE)
        .and_then(|c| c.actions.first())
        .map(|h| h.icon);
    assert_eq!(olho, Some(IconId::EyeClosed));
    ph2d_tool_vector::appearance::set_current_appearance(None);
    crate::state::set_current_inspector_visibility(None);
    crate::state::set_current_inspector_transform(None);
}
