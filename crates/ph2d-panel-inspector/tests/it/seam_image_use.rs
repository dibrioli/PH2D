//! ⭐ **O uso da imagem à vista** (plano GEF 15) — popular → publicar → PINTAR → clicar, o caminho da shell: sob
//! a linha Image, o seletor «Use» com os três usos; por baixo só os campos do escolhido (parada: a folha para
//! clicar o quadro · repetida: Width/Height · animada: a folha com o corte e a secção Animation ali).

use crate::seam_anim::anim;
use crate::seam_render_source::atlas_sprite;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::Panel;
use ph2d_editor_core::screens::hero::{
    InspectorSpriteInfo, InspectorTransformInfo, InspectorVisibilityInfo,
};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{ImageUseEdit, InspectorDraw, InspectorImageUse, SpriteFieldEdit};
use ph2d_panel_inspector::ids::{
    INSP_ANIM_ROW, INSP_IMAGE_TILED_SIZE, INSP_IMAGE_USE, INSP_SPRITE_FRAME, INSP_SPRITE_HFRAMES,
    INSP_SPRITE_VFRAMES, insp_image_cell,
};
use ph2d_panel_inspector::slot_image::{SlotImage, set_current_slot_image};
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_current_inspector_anim, set_current_inspector_sprite,
    set_current_inspector_transform, set_current_inspector_visibility,
};
use ph2d_ui_testkit::MockPanelHost;
use std::sync::Arc;

fn com(uso: InspectorImageUse, grelha: [u32; 2], draw: InspectorDraw) -> InspectorSpriteInfo {
    InspectorSpriteInfo {
        image_use: uso,
        image_cells: grelha[0] * grelha[1],
        hframes: grelha[0],
        vframes: grelha[1],
        frame: 0,
        draw,
        ..atlas_sprite(true)
    }
}

/// A Visibility de `bits` — é nela que a linha Image e o seletor «Use» moram.
pub(crate) fn publish_visibility(bits: u64) {
    set_current_inspector_visibility(Some(InspectorVisibilityInfo {
        entity_bits: bits,
        visible: true,
        mixed: false,
    }));
}

/// Publica a sprite como a shell: a Visibility (onde a linha Image mora), a miniatura da ranhura e, numa
/// animada, a secção Animation.
fn publica(info: &InspectorSpriteInfo) {
    let bits = info.entity_bits;
    publish_visibility(bits);
    set_current_slot_image(Some(SlotImage {
        entity_bits: bits,
        name: "folha".into(),
        rgba: Arc::new(vec![255; 64 * 32 * 4]),
        w: 64,
        h: 32,
    }));
    set_current_inspector_anim((info.image_use == InspectorImageUse::Animated).then(|| {
        let mut a = anim(true, true);
        a.entity_bits = bits;
        a
    }));
    set_current_inspector_transform(Some(InspectorTransformInfo {
        entity_bits: bits,
        translation: [0.0, 0.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
        skew_rad: [0.0, 0.0],
    }));
    set_current_inspector_sprite(Some(info.clone()));
}

fn pintado(
    info: &InspectorSpriteInfo,
) -> (
    MockPanelHost,
    InspectorState,
    Vec<(ph2d_a11y::NodeId, Rect)>,
) {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    publica(info);
    host.settle_section_folds();
    let rects = host.paint::<InspectorPanel>(&mut state, Rect::new(0.0, 0.0, 320.0, 3000.0));
    (host, state, rects)
}

fn tem(rects: &[(ph2d_a11y::NodeId, Rect)], id: ph2d_a11y::NodeId) -> bool {
    rects.iter().any(|(i, _)| *i == id)
}

fn y_de(rects: &[(ph2d_a11y::NodeId, Rect)], id: ph2d_a11y::NodeId) -> f32 {
    rects
        .iter()
        .find(|(i, _)| *i == id)
        .map(|(_, r)| r.y)
        .unwrap_or_else(|| panic!("{id:?} não pintado"))
}

/// Gate 1: o seletor pinta os três; cada uso pinta SÓ os seus campos.
#[test]
fn cada_uso_pinta_so_os_seus_campos() {
    let tiled = InspectorDraw::Tiled { size: [2.4, 0.5] };
    // (uso, grelha, draw, células clicáveis, Width/Height, corte, Animation)
    for (uso, grelha, draw, celulas, tamanho, corte, animacao) in [
        (
            InspectorImageUse::Still,
            [1, 1],
            InspectorDraw::Simple,
            false,
            false,
            false,
            false,
        ),
        (
            InspectorImageUse::Still,
            [4, 1],
            InspectorDraw::Simple,
            true,
            false,
            false,
            false,
        ),
        (
            InspectorImageUse::Repeated,
            [1, 1],
            tiled,
            false,
            true,
            false,
            false,
        ),
        (
            InspectorImageUse::Animated,
            [4, 1],
            InspectorDraw::Fixed,
            false,
            false,
            true,
            true,
        ),
    ] {
        let (_, _, rects) = pintado(&com(uso, grelha, draw));
        let quem = format!("{uso:?} {grelha:?}");
        for id in INSP_IMAGE_USE {
            assert!(tem(&rects, id), "{quem}: o seletor");
        }
        assert_eq!(tem(&rects, insp_image_cell(0)), celulas, "{quem}: células");
        for id in INSP_IMAGE_TILED_SIZE {
            assert_eq!(tem(&rects, id), tamanho, "{quem}: Width/Height");
        }
        for id in [INSP_SPRITE_HFRAMES, INSP_SPRITE_VFRAMES] {
            assert_eq!(tem(&rects, id), corte, "{quem}: o corte");
        }
        assert!(
            !tem(&rects, INSP_SPRITE_FRAME),
            "{quem}: o campo Frame saiu"
        );
        assert_eq!(tem(&rects, INSP_ANIM_ROW[0]), animacao, "{quem}: Animation");
    }
}

/// ⭐ D2: a secção Animation abre ALI — logo abaixo do seletor, antes do Transform e de tudo o resto.
#[test]
fn a_animation_abre_sob_o_seletor() {
    let (_, _, rects) = pintado(&com(
        InspectorImageUse::Animated,
        [4, 1],
        InspectorDraw::Fixed,
    ));
    let seletor = y_de(&rects, INSP_IMAGE_USE[2]);
    let lista = y_de(&rects, INSP_ANIM_ROW[0]);
    let corte = y_de(&rects, INSP_SPRITE_HFRAMES);
    assert!(
        seletor < corte && corte < lista,
        "{seletor} {corte} {lista}"
    );
    let transform = y_de(&rects, ph2d_panel_inspector::ids::INSP_TRANSFORM_POS_X);
    assert!(lista < transform, "a Animation ficou abaixo do Transform");
}

/// Gate 2: cada clique do seletor chega como `ImageUseEdit::Use`; o uso que já é não pede nada.
#[test]
fn cada_clique_do_seletor_pede_o_uso() {
    for de in InspectorImageUse::ALL {
        let info = com(de, [4, 1], InspectorDraw::Simple);
        for (i, para) in InspectorImageUse::ALL.into_iter().enumerate() {
            let (mut host, mut state, _) = pintado(&info);
            InspectorPanel::apply_event(
                &mut state,
                &mut host,
                WidgetEvent::Click(INSP_IMAGE_USE[i]),
            );
            let quer = if para == de {
                vec![]
            } else {
                vec![EditorAction::ImageUseEdit {
                    entity_bits: info.entity_bits,
                    edit: ImageUseEdit::Use(para),
                }]
            };
            assert_eq!(host.drained_actions(), quer, "{de:?} → {para:?}");
        }
    }
}

/// Uma camada de fundo repete sempre: nada de seletor, e os ids não agem.
#[test]
fn a_camada_de_fundo_nao_oferece_o_seletor() {
    let info = com(InspectorImageUse::Repeated, [1, 1], InspectorDraw::Fixed);
    let (mut host, mut state, rects) = pintado(&info);
    assert!(!tem(&rects, INSP_IMAGE_USE[0]));
    InspectorPanel::apply_event(&mut state, &mut host, WidgetEvent::Click(INSP_IMAGE_USE[2]));
    assert!(host.drained_actions().is_empty());
}

/// Gate 4: numa parada, clicar a célula 2 da folha mostra o quadro 2; numa animada a folha não escolhe.
#[test]
fn clicar_uma_celula_escolhe_o_quadro() {
    let info = com(InspectorImageUse::Still, [4, 1], InspectorDraw::Simple);
    let (mut host, mut state, rects) = pintado(&info);
    assert!(tem(&rects, insp_image_cell(3)) && !tem(&rects, insp_image_cell(4)));
    InspectorPanel::apply_event(
        &mut state,
        &mut host,
        WidgetEvent::Click(insp_image_cell(2)),
    );
    assert_eq!(
        host.drained_actions(),
        vec![EditorAction::InspectorSpriteEdit {
            entity_bits: info.entity_bits,
            edit: SpriteFieldEdit::Frame(2),
        }]
    );
    // CONTROLO: o mesmo id numa animada não age.
    let info = com(InspectorImageUse::Animated, [4, 1], InspectorDraw::Fixed);
    let (mut host, mut state, _) = pintado(&info);
    InspectorPanel::apply_event(
        &mut state,
        &mut host,
        WidgetEvent::Click(insp_image_cell(2)),
    );
    assert!(host.drained_actions().is_empty());
}

/// ⭐ A Largura de uma repetida chega ao barramento como `ImageUseEdit`.
#[test]
fn largura_pede_a_accao() {
    let info = com(
        InspectorImageUse::Repeated,
        [1, 1],
        InspectorDraw::Tiled { size: [2.4, 0.5] },
    );
    let (mut host, mut state, _) = pintado(&info);
    host.set_number_value(INSP_IMAGE_TILED_SIZE[0], 4.0);
    InspectorPanel::apply_event(
        &mut state,
        &mut host,
        WidgetEvent::ValueChanged(INSP_IMAGE_TILED_SIZE[0]),
    );
    assert_eq!(
        host.drained_actions(),
        vec![EditorAction::ImageUseEdit {
            entity_bits: info.entity_bits,
            edit: ImageUseEdit::Width(4.0)
        }]
    );
}
