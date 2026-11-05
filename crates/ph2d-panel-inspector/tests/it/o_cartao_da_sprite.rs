//! ⭐ **A costura do cartão da Sprite** ([ficha](../../../../docs/GameEngineFuncional/componentes/01_sprite.md)): cada
//! gesto entra como ponteiro no que está pintado e sai como pedido no barramento.

use crate::o_inspector_em_cartoes_por_tipo::pinta;
use crate::seam_render_source::atlas_sprite;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::{
    InspectorOrderingInfo, InspectorOrderingMixed, InspectorSpriteInfo, InspectorTransformInfo,
    SpriteFieldEdit,
};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{ImageUseEdit, SpritePivot};
use ph2d_host::{PointerButton, PointerEvent, PointerKind};
use ph2d_panel_inspector::ids::*;
use ph2d_panel_inspector::slot_image::{self, Recibo, SpriteFacts};
use ph2d_panel_inspector::{
    CARD_ORDERING, InspectorPanel, InspectorState, set_cards_on, set_current_inspector_ordering,
    set_current_inspector_sprite, set_current_inspector_transform,
};
use ph2d_ui_testkit::MockPanelHost;

fn sprite() -> InspectorSpriteInfo {
    InspectorSpriteInfo {
        entity_bits: 7,
        ..atlas_sprite(true)
    }
}

fn factos() -> SpriteFacts {
    SpriteFacts {
        entity_bits: 7,
        pivot_ponto: Some([1, 1]),
        ..SpriteFacts::default()
    }
}

/// O painel aberto, com o cartão Look e o Advanced à vista.
fn cena(f: SpriteFacts) -> (MockPanelHost, InspectorState) {
    set_cards_on(true);
    set_current_inspector_sprite(Some(sprite()));
    set_current_inspector_transform(Some(InspectorTransformInfo {
        entity_bits: 7,
        translation: [0.0, 0.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
        skew_rad: [0.0, 0.0],
    }));
    set_current_inspector_ordering(Some(InspectorOrderingInfo {
        entity_bits: 7,
        z_index: None,
        z_as_relative: true,
        show_behind_parent: false,
        sorting_layer: 0,
        order_in_layer: 0,
        y_sort_enabled: false,
        y_sort_point: 0,
        y_sort_axis: [0.0, 0.0],
        sorting_group: false,
        sort_at_root: false,
        top_level: false,
        selected_count: 1,
        mixed: InspectorOrderingMixed::default(),
    }));
    slot_image::set_current_sprite_facts(Some(f));
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    for id in [
        core_ids::INSP_LIVE_COLOR_SECTION,
        core_ids::INSP_LIVE_RENDER_SECTION,
        CARD_ORDERING,
    ] {
        host.store_mut().set_collapsed(id, false);
    }
    (host, InspectorState::default())
}

fn rect_de(r: &[(NodeId, Rect)], id: NodeId) -> Rect {
    r.iter()
        .rev()
        .find(|(i, q)| *i == id && q.w > 0.0)
        .map(|(_, q)| *q)
        .unwrap_or_else(|| panic!("{id:?} não pintou"))
}

/// O clique real em `id`: os eventos do ponteiro vão ao painel; devolve os eventos e os pedidos.
fn clica(
    host: &mut MockPanelHost,
    st: &mut InspectorState,
    id: NodeId,
) -> (Vec<WidgetEvent>, Vec<EditorAction>) {
    let r = rect_de(&pinta(host, st), id);
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let _ = host.drained_actions();
    let mut eventos = Vec::new();
    for kind in [PointerKind::Move, PointerKind::Down, PointerKind::Up] {
        let ev = PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: ph2d_host::PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: 0,
        };
        eventos.extend(host.dispatch_pointer_event(ev));
    }
    for ev in &eventos {
        let _ = host.apply_panel_event::<InspectorPanel>(st, *ev);
    }
    (eventos, host.drained_actions())
}

fn edicoes(a: &[EditorAction]) -> Vec<ImageUseEdit> {
    a.iter()
        .filter_map(|x| match x {
            EditorAction::ImageUseEdit {
                entity_bits: 7,
                edit,
            } => Some(*edit),
            _ => None,
        })
        .collect()
}

#[test]
fn cada_um_dos_nove_pontos_pede_o_seu_pivo() {
    let (mut host, mut st) = cena(factos());
    for (i, id) in INSP_SPRITE_PIVOT.into_iter().enumerate() {
        let (_, a) = clica(&mut host, &mut st, id);
        let ponto = [(i % 3) as u8, (i / 3) as u8];
        assert_eq!(
            edicoes(&a),
            [ImageUseEdit::Pivot(SpritePivot::Ponto(ponto))]
        );
    }
}

#[test]
fn os_campos_custom_so_aparecem_em_custom_ou_pela_mao() {
    let (mut host, mut st) = cena(factos());
    let tem = |r: &[(NodeId, Rect)]| {
        r.iter()
            .any(|(i, q)| *i == INSP_SPRITE_PIVOT_XY[0] && q.w > 0.0)
    };
    assert!(
        !tem(&pinta(&mut host, &mut st)),
        "num dos nove pontos, fechados"
    );
    let _ = clica(&mut host, &mut st, INSP_SPRITE_PIVOT_NOME);
    assert!(tem(&pinta(&mut host, &mut st)), "o nome do ponto abre-os");
    let _ = clica(&mut host, &mut st, INSP_SPRITE_PIVOT_NOME);

    let (mut host, mut st) = cena(SpriteFacts {
        pivot_ponto: None,
        ..factos()
    });
    assert!(tem(&pinta(&mut host, &mut st)), "em Custom, abertos");
}

#[test]
fn edit_in_pixel_pede_so_a_ida_e_edit_in_painter_entra_no_draw() {
    use ph2d_editor_core::object_mode::{ModeRequest, ObjectMode};
    let entra = |a: &[EditorAction]| {
        a.iter().any(|x| {
            matches!(
                x,
                EditorAction::ObjectMode(ModeRequest::Enter(ObjectMode::Draw))
            )
        })
    };
    let (mut host, mut st) = cena(factos());
    let (_, a) = clica(&mut host, &mut st, INSP_IMAGE_EDIT_PIXEL);
    assert_eq!(edicoes(&a), [ImageUseEdit::EditInPixel]);
    assert!(
        !entra(&a),
        "o Pixel entra no Draw do objecto de trabalho, não neste"
    );
    let (_, a) = clica(&mut host, &mut st, INSP_IMAGE_EDIT_PAINTER);
    assert_eq!(edicoes(&a), [ImageUseEdit::EditInPainter]);
    assert!(entra(&a));
}

#[test]
fn a_miniatura_abre_o_seletor() {
    let abre = |a: &[EditorAction]| {
        a.iter()
            .any(|x| matches!(x, EditorAction::OpenImagePicker { entity_bits: 7 }))
    };
    let (mut host, mut st) = cena(factos());
    assert!(abre(&clica(&mut host, &mut st, INSP_RENDER_TEXTURE_SLOT).1));
}

#[test]
fn keep_old_size_devolve_o_tamanho_e_o_recibo_some() {
    let (mut host, mut st) = cena(factos());
    slot_image::set_recibo(Some(Recibo {
        entity_bits: 7,
        de: [256, 128],
        para: [64, 64],
        tamanho: [2.5, 1.25],
    }));
    let (_, a) = clica(&mut host, &mut st, INSP_IMAGE_KEEP_SIZE);
    assert_eq!(edicoes(&a), [ImageUseEdit::KeepSize([2.5, 1.25])]);
    let r = pinta(&mut host, &mut st);
    assert!(
        !r.iter()
            .any(|(i, q)| *i == INSP_IMAGE_KEEP_SIZE && q.w > 0.0)
    );
}

#[test]
fn o_aviso_de_folha_da_as_duas_saidas() {
    use ph2d_editor_core::InspectorImageUse as Uso;
    let (mut host, mut st) = cena(SpriteFacts {
        folha: Some([4, 2]),
        ..factos()
    });
    let (_, a) = clica(&mut host, &mut st, INSP_IMAGE_SHEET_ANIMATED);
    assert_eq!(edicoes(&a), [ImageUseEdit::Use(Uso::Animated)]);
    let (_, a) = clica(&mut host, &mut st, INSP_IMAGE_SHEET_REGION);
    assert_eq!(edicoes(&a), [ImageUseEdit::Use(Uso::Region)]);
}

#[test]
fn o_flip_em_icones_emite_o_mesmo_toggled() {
    let (mut host, mut st) = cena(factos());
    let (ev, a) = clica(&mut host, &mut st, INSP_SPRITE_FLIP_X);
    assert!(
        ev.contains(&WidgetEvent::Toggled(INSP_SPRITE_FLIP_X)),
        "{ev:?}"
    );
    assert!(
        a.iter().any(|x| matches!(
            x,
            EditorAction::InspectorSpriteEdit {
                entity_bits: 7,
                edit: SpriteFieldEdit::FlipX(_)
            }
        )),
        "{a:?}"
    );
}

#[test]
fn escolher_uma_opcao_revela_a_linha_dela_no_ordering() {
    let (mut host, mut st) = cena(factos());
    let tem = |r: &[(NodeId, Rect)], id| r.iter().any(|(i, q)| *i == id && q.w > 0.0);
    assert!(!tem(&pinta(&mut host, &mut st), INSP_ORDER_TOP_LEVEL));
    let lista = ph2d_tool_registry::hash_node_id("insp_order_options.dd");
    assert!(
        rect_de(&pinta(&mut host, &mut st), lista).w > 0.0,
        "a lista está à vista"
    );
    // As opções pinta-as o hero por cima de tudo (`o_alfinete` e o smoke cobrem esse trecho): aqui entra o clique dela.
    let opcao = ph2d_tool_registry::hash_node_id("insp_order_opt_top_level");
    let _ = host.apply_panel_event::<InspectorPanel>(&mut st, WidgetEvent::Click(opcao));
    assert!(
        tem(&pinta(&mut host, &mut st), INSP_ORDER_TOP_LEVEL),
        "a opção revela"
    );
}

#[test]
fn desligar_tint_each_corner_devolve_o_branco() {
    let (mut host, mut st) = cena(factos());
    set_current_inspector_sprite(Some(InspectorSpriteInfo {
        per_corner_tint: [[0.5, 0.5, 0.5, 1.0]; 4],
        ..sprite()
    }));
    let (_, a) = clica(&mut host, &mut st, INSP_SPRITE_CORNERS_ON);
    let brancos = a
        .iter()
        .filter(|x| {
            matches!(
                x,
                EditorAction::InspectorSpriteEdit {
                    entity_bits: 7,
                    edit: SpriteFieldEdit::PerCornerTintAt(_, [1.0, 1.0, 1.0, 1.0])
                }
            )
        })
        .count();
    assert_eq!(brancos, 4, "{a:?}");
}

#[test]
fn o_convite_so_existe_sem_imagem() {
    let tem = |r: &[(NodeId, Rect)], id| r.iter().any(|(i, q)| *i == id && q.w > 0.0);
    let (mut host, mut st) = cena(factos());
    let r = pinta(&mut host, &mut st);
    assert!(!tem(&r, INSP_IMAGE_PAINT_NEW) && tem(&r, INSP_IMAGE_EDIT_PIXEL));

    let (mut host, mut st) = cena(SpriteFacts {
        sem_imagem: true,
        ..factos()
    });
    let r = pinta(&mut host, &mut st);
    assert!(tem(&r, INSP_IMAGE_PAINT_NEW) && !tem(&r, INSP_IMAGE_EDIT_PIXEL));
    let (_, a) = clica(&mut host, &mut st, INSP_RENDER_TEXTURE_SLOT);
    assert!(
        a.iter()
            .any(|x| matches!(x, EditorAction::OpenImagePicker { entity_bits: 7 })),
        "o cartão vazio inteiro é o alvo: {a:?}"
    );
}

/// ⭐ Dono 05/11: o hover real na miniatura acende a borda e o ícone; sem ele, nada.
/// *Mutação: tirar o `if` de estado de `troca_em_hover` ⇒ o ícone pinta sempre.*
#[test]
fn o_hover_da_miniatura_acende_a_borda_e_o_icone_de_troca() {
    let (mut host, mut st) = cena(factos());
    let foto = rect_de(&pinta(&mut host, &mut st), INSP_RENDER_TEXTURE_SLOT);
    let sinal = |host: &mut MockPanelHost, st: &mut InspectorState| {
        let _ = pinta(host, st);
        ph2d_panel_inspector::troca_pintada()
    };
    assert!(!sinal(&mut host, &mut st), "sem hover não acende");
    let mao = |x: f32, y: f32| PointerEvent {
        x,
        y,
        pressure: 0.0,
        kind: PointerKind::Move,
        source: ph2d_host::PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    };
    let _ = host.dispatch_pointer_event(mao(foto.x + 5.0, foto.y + 5.0));
    assert!(sinal(&mut host, &mut st), "o hover real não acendeu");
    let _ = host.dispatch_pointer_event(mao(foto.x + foto.w + 80.0, foto.y + foto.h + 300.0));
    assert!(!sinal(&mut host, &mut st), "fora dela volta a apagar");
}

/// ⭐ Dono 05/11: a 270 px de painel os dois botões mostram o texto inteiro, um por linha se não couberem lado a lado,
/// e a miniatura continua maior que um botão. *Mutação: fixar a coluna em largura zero.*
#[test]
fn a_270_px_os_botoes_mostram_o_texto_inteiro_e_a_miniatura_e_maior() {
    let (mut host, mut st) = cena(factos());
    let janela = Rect::new(0.0, 0.0, 270.0, 900.0);
    host.settle_section_folds();
    let mut r = Vec::new();
    for _ in 0..30 {
        r = host.paint::<InspectorPanel>(&mut st, janela);
    }
    let (a, b, foto) = (
        rect_de(&r, INSP_IMAGE_EDIT_PIXEL),
        rect_de(&r, INSP_IMAGE_EDIT_PAINTER),
        rect_de(&r, INSP_RENDER_TEXTURE_SLOT),
    );
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let pede = |ts: &mut ph2d_text::TextSystem, k: &str| {
        ts.prefix_width(ph2d_i18n::tr(k), m::VALUE_PX).ceil() + m::BUTTON_TEXT_PAD * 2.0
    };
    let (pa, pb) = (
        pede(&mut ts, "card.insp.edit_in_pixel"),
        pede(&mut ts, "card.insp.edit_in_painter"),
    );
    eprintln!("medido a 270: Pixel {a:?} pede {pa} · Painter {b:?} pede {pb} · miniatura {foto:?}");
    assert!(a.w >= pa && b.w >= pb, "cortou: {a:?}/{pa} {b:?}/{pb}");
    assert!(foto.h > a.h && foto.x + foto.w < a.x, "{foto:?} × {a:?}");
}

/// ⭐ Dono 05/11: em Repeated o tamanho do bloco vai entre parênteses na linha da miniatura; «Size (px)» já não existe.
#[test]
fn o_tamanho_do_repeated_vai_entre_parenteses() {
    let t = ph2d_i18n::tr_with(
        "card.insp.image_size_repeated",
        &[("pw", &256), ("ph", &128), ("size", &"300 × 50")],
    );
    assert_eq!(t, "256 × 128 px (300 × 50)");
}

/// ⭐ Dono 05/11: Flip e Pivot pintam a bolinha do canal. *Mutação: tirar `.key(..)` da linha ⇒ o alvo some.*
#[test]
fn o_flip_e_o_pivot_pintam_a_bolinha_de_chave() {
    use ph2d_editor_core::widget::key_mark::{KeyMark, key_mark_id, set_key_marks};
    let (mut host, mut st) = cena(factos());
    set_key_marks(
        [
            (INSP_SPRITE_FLIP_X, KeyMark::Livre),
            (INSP_SPRITE_PIVOT_XY[0], KeyMark::Livre),
        ]
        .into(),
    );
    let r = pinta(&mut host, &mut st);
    set_key_marks(Default::default());
    let _ = rect_de(&r, key_mark_id(INSP_SPRITE_FLIP_X));
    let _ = rect_de(&r, key_mark_id(INSP_SPRITE_PIVOT_XY[0]));
}
