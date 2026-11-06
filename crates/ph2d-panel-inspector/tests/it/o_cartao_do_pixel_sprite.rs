//! ⭐ **O cartão do Pixel Sprite** (dono 06/11: *«Criar um cartão com o botão Pixel: fica igual à imagem»*): por gesto
//! real, no que está pintado. A publicação pela família prova-se em `ph2d-app-pixel` (`cartao_tests`); o cartão preso,
//! em `ph2d-panel-registry-init` (`o_alfinete_de_todo_cartao`).

use crate::o_inspector_em_cartoes_por_tipo::pinta;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::object_mode::{ModeRequest, ObjectMode};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::pixel_panel::PixelSpriteCard;
use ph2d_editor_core::screens::hero::{InspectorNameInfo, InspectorTransformInfo};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind};
use ph2d_panel_inspector::ids::*;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, cabeca_pintada, imagens_pintadas, miniatura_pintada,
    set_cards_on, set_current_inspector_name, set_current_inspector_sprite,
    set_current_inspector_transform, slot_image, troca_pintada,
};
use ph2d_ui_testkit::MockPanelHost;
use std::sync::Arc;

const BITS: u64 = 11;

fn cartao(rgba: &Arc<Vec<u8>>, size: [u32; 2]) -> PixelSpriteCard {
    PixelSpriteCard {
        entity_bits: BITS,
        size,
        rgba: rgba.clone(),
        thumb: [2, 1],
    }
}

/// Um Pixel Sprite seleccionado: Transform e nome, sem `Sprite`, como a shell os publica.
fn cena(size: [u32; 2]) -> (MockPanelHost, InspectorState, Arc<Vec<u8>>) {
    set_cards_on(true);
    set_current_inspector_sprite(None);
    slot_image::set_current_slot_image(None);
    set_current_inspector_transform(Some(InspectorTransformInfo {
        entity_bits: BITS,
        translation: [0.0, 0.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
        skew_rad: [0.0, 0.0],
    }));
    set_current_inspector_name(Some(InspectorNameInfo {
        entity_bits: BITS,
        name: "Pixel Sprite".to_owned(),
    }));
    let rgba = Arc::new(vec![200; 8]);
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    host.store_mut()
        .set_pixel_sprite_cards(vec![cartao(&rgba, size)]);
    (host, InspectorState::default(), rgba)
}

fn rect_de(r: &[(NodeId, Rect)], id: NodeId) -> Option<Rect> {
    r.iter()
        .rev()
        .find(|(i, q)| *i == id && q.w > 0.0)
        .map(|(_, q)| *q)
}

fn evento(x: f32, y: f32, kind: PointerKind) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: ph2d_host::PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    }
}

/// O clique real no ponto: os eventos vão ao painel; devolve os pedidos.
fn clica_em(
    host: &mut MockPanelHost,
    st: &mut InspectorState,
    x: f32,
    y: f32,
) -> Vec<EditorAction> {
    let _ = host.drained_actions();
    let mut eventos = Vec::new();
    for kind in [PointerKind::Move, PointerKind::Down, PointerKind::Up] {
        eventos.extend(host.dispatch_pointer_event(evento(x, y, kind)));
    }
    for ev in &eventos {
        let _ = host.apply_panel_event::<InspectorPanel>(st, *ev);
    }
    host.drained_actions()
}

/// ⭐ *Mutação: tirar o ramo do `do_topo` ⇒ não há cartão.*
#[test]
fn o_pixel_sprite_selecionado_mostra_o_cartao_so_com_o_pixel() {
    let (mut host, mut st, _) = cena([96, 64]);
    let _ = imagens_pintadas();
    let r = pinta(&mut host, &mut st);
    assert!(rect_de(&r, INSP_IMAGE_EDIT_PIXEL).is_some());
    assert!(rect_de(&r, INSP_IMAGE_EDIT_PAINTER).is_none());
    assert!(
        imagens_pintadas()
            .iter()
            .any(|(b, n)| *b == BITS && n == "Pixel Sprite")
    );
    // Não mente: nada de Use, Look nem Pivot, que ele não tem.
    assert!(rect_de(&r, INSP_SPRITE_PIVOT_NOME).is_none());
    assert!(rect_de(&r, INSP_IMAGE_KEEP_SIZE).is_none());
}

/// A imagem continua só com o Painter, e um objecto sem cartão não mostra nada dele.
#[test]
fn a_imagem_nao_ganha_o_verbo_pixel_e_sem_cartao_nao_ha_moldura() {
    let (mut host, mut st, _) = cena([96, 64]);
    host.store_mut().set_pixel_sprite_cards(Vec::new());
    let r = pinta(&mut host, &mut st);
    assert!(rect_de(&r, INSP_IMAGE_EDIT_PIXEL).is_none());
    assert!(rect_de(&r, INSP_IMAGE_EDIT_PAINTER).is_none());

    set_current_inspector_sprite(Some(ph2d_editor_core::screens::hero::InspectorSpriteInfo {
        entity_bits: BITS,
        ..crate::seam_render_source::atlas_sprite(true)
    }));
    let r = pinta(&mut host, &mut st);
    assert!(rect_de(&r, INSP_IMAGE_EDIT_PAINTER).is_some());
    assert!(rect_de(&r, INSP_IMAGE_EDIT_PIXEL).is_none());
    set_current_inspector_sprite(None);
}

/// ⭐ *Mutação: tirar o ramo do verbo em `cards_sprite::evento` ⇒ não entra.*
#[test]
fn o_verbo_pixel_entra_no_draw() {
    let (mut host, mut st, _) = cena([96, 64]);
    let v = rect_de(&pinta(&mut host, &mut st), INSP_IMAGE_EDIT_PIXEL).expect("o verbo");
    let a = clica_em(&mut host, &mut st, v.x + v.w * 0.5, v.y + v.h * 0.5);
    assert!(
        a.iter().any(|x| matches!(
            x,
            EditorAction::ObjectMode(ModeRequest::Enter(ObjectMode::Draw))
        )),
        "{a:?}"
    );
    assert!(
        !a.iter()
            .any(|x| matches!(x, EditorAction::ImageUseEdit { .. }))
    );
}

/// ⭐ Dono: a miniatura do Pixel Sprite não troca nada. *Mutação: `trocavel: true` ⇒ o clique abre o seletor.*
#[test]
fn clicar_na_miniatura_do_pixel_nao_abre_o_seletor() {
    let (mut host, mut st, _) = cena([96, 64]);
    let r = pinta(&mut host, &mut st);
    assert!(rect_de(&r, INSP_RENDER_TEXTURE_SLOT).is_none());
    let f = cabeca_pintada().expect("a moldura").foto;
    let (x, y) = (f.x + f.w * 0.5, f.y + f.h * 0.5);
    let a = clica_em(&mut host, &mut st, x, y);
    assert!(
        !a.iter()
            .any(|x| matches!(x, EditorAction::OpenImagePicker { .. })),
        "{a:?}"
    );
    let _ = host.dispatch_pointer_event(evento(x, y, PointerKind::Move));
    let _ = pinta(&mut host, &mut st);
    assert!(
        !troca_pintada(),
        "o ícone de troca mente: não há o que trocar"
    );
}

/// O cartão mostra o que a família publicou por último: depois de pintar, os píxeis novos.
#[test]
fn a_miniatura_do_cartao_e_a_que_a_familia_publicou_por_ultimo() {
    let (mut host, mut st, antes) = cena([96, 64]);
    let _ = pinta(&mut host, &mut st);
    assert!(Arc::ptr_eq(&miniatura_pintada().expect("pintou"), &antes));
    let depois = Arc::new(vec![50; 8]);
    host.store_mut()
        .set_pixel_sprite_cards(vec![cartao(&depois, [96, 64])]);
    let _ = pinta(&mut host, &mut st);
    assert!(Arc::ptr_eq(&miniatura_pintada().expect("pintou"), &depois));
}

/// A mesma lei da moldura da imagem, do piso real do painel a bem largo, com o maior tamanho que o documento aceita.
#[test]
fn nada_do_cartao_do_pixel_e_cortado_no_minimo_do_painel() {
    use ph2d_editor_core::screens::layout::{CenterSplit, ChromeBands, DockSides, HeroLayout};
    use ph2d_editor_core::text_elide::elisao;
    use ph2d_editor_core::widget::card_censo;
    let mut lado = None;
    for largura in [
        WidgetStore::DOCK_W_MIN,
        ph2d_tokens::chrome::PANEL_MIN_W_PX,
        300.0,
        420.0,
    ] {
        let (mut host, mut st, _) = cena([65_535, 65_535]);
        let janela = Rect::new(0.0, 0.0, 1600.0, 2400.0);
        let bandas = ChromeBands {
            right_dock_w: largura,
            ..ChromeBands::DEFAULT
        };
        let pinta_a = |host: &mut MockPanelHost, st: &mut InspectorState| {
            host.settle_section_folds();
            let mut r = Vec::new();
            for _ in 0..60 {
                let layout = HeroLayout::for_viewport_bands(
                    janela,
                    false,
                    bandas,
                    CenterSplit::None,
                    DockSides::BOTH,
                );
                r = ph2d_editor_core::aquecimento::com_passo_fixo(
                    ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
                    || host.paint_with_layout::<InspectorPanel>(st, layout, janela),
                );
            }
            r
        };
        let _ = pinta_a(&mut host, &mut st);
        let ((r, medidos), censo) =
            card_censo::grava(|| elisao::medindo(|| pinta_a(&mut host, &mut st)));
        let cortados: Vec<_> = medidos
            .iter()
            .filter(|m| !m.coube())
            .map(|m| m.texto.clone())
            .collect();
        assert!(cortados.is_empty(), "a {largura}: {cortados:?}");
        assert_eq!(censo.textos_cortados(), Vec::<String>::new(), "a {largura}");
        let p = cabeca_pintada().expect("a moldura");
        let v = rect_de(&r, INSP_IMAGE_EDIT_PIXEL).expect("o verbo");
        let dentro = |q: Rect| {
            q.x >= p.moldura.x
                && q.y >= p.moldura.y
                && q.x + q.w <= p.moldura.x + p.moldura.w + 0.01
                && q.y + q.h <= p.moldura.y + p.moldura.h + 0.01
        };
        let t = p.tamanho.expect("o tamanho");
        for q in [v, p.foto, t] {
            assert!(dentro(q), "a {largura}: {q:?} fora de {:?}", p.moldura);
        }
        assert!(p.foto.x + p.foto.w <= t.x, "a {largura}");
        let l = *lado.get_or_insert(p.foto.w);
        assert!((p.foto.w - l).abs() < 0.01, "a miniatura mudou a {largura}");
    }
}
