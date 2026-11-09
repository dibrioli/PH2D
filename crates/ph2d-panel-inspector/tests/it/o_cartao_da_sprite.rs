//! ⭐ **A costura do cartão da Sprite** ([ficha](../../../../docs/GameEngineFuncional/componentes/01_sprite.md)): cada
//! gesto entra como ponteiro no que está pintado e sai como pedido no barramento.

use crate::o_inspector_em_cartoes_por_tipo::pinta;
use crate::seam_render_source::atlas_sprite;
use ph2d_a11y::NodeId;
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
    CARD_ORDERING, CabecaPintada, InspectorPanel, InspectorState, cabeca_pintada, set_cards_on,
    set_current_inspector_ordering, set_current_inspector_sprite, set_current_inspector_transform,
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

/// ⭐ Numa Animated Sprite o Pivot mora no editor de animações (GEF 29): no lugar da grade de nove pontos, um botão que
/// abre o editor com o Align ligado; nenhum ponto a clicar no cartão.
/// *Mutação: o botão a pedir o `OpenEditor` simples ⇒ abre sem o Align, vermelho.*
#[test]
fn numa_animated_o_pivot_e_um_botao_que_abre_o_editor_com_o_align() {
    let (mut host, mut st) = cena(SpriteFacts {
        pivot_no_editor: true,
        ..factos()
    });
    let r = pinta(&mut host, &mut st);
    assert!(
        !r.iter()
            .any(|(i, q)| INSP_SPRITE_PIVOT.contains(i) && q.w > 0.0),
        "sem a grade"
    );
    let (_, a) = clica(&mut host, &mut st, INSP_SPRITE_PIVOT_EDITOR);
    assert!(
        a.iter().any(|x| matches!(
            x,
            EditorAction::InspectorAnimEdit {
                edit: ph2d_editor_core::AnimFieldEdit::OpenEditorAlign,
                ..
            }
        )),
        "{a:?}"
    );
    assert!(edicoes(&a).is_empty(), "o Pivot não muda no cartão");
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
    assert!(
        tem(&pinta(&mut host, &mut st)),
        "o botão ao lado da grade abre-os"
    );
    let _ = clica(&mut host, &mut st, INSP_SPRITE_PIVOT_NOME);

    let (mut host, mut st) = cena(SpriteFacts {
        pivot_ponto: None,
        ..factos()
    });
    assert!(tem(&pinta(&mut host, &mut st)), "em Custom, abertos");
}

#[test]
fn a_imagem_so_tem_o_verbo_do_painter_e_ele_entra_no_draw() {
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
    assert!(!tem(&r, INSP_IMAGE_PAINT_NEW) && tem(&r, INSP_IMAGE_EDIT_PAINTER));

    let (mut host, mut st) = cena(SpriteFacts {
        sem_imagem: true,
        ..factos()
    });
    let r = pinta(&mut host, &mut st);
    assert!(tem(&r, INSP_IMAGE_PAINT_NEW) && !tem(&r, INSP_IMAGE_EDIT_PAINTER));
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

/// A coluna do painel com `largura` px, numa janela larga, assente.
fn pinta_a_largura(
    host: &mut MockPanelHost,
    st: &mut InspectorState,
    largura: f32,
) -> Vec<(NodeId, Rect)> {
    pinta_na_janela(host, st, largura, 900.0)
}

pub(crate) fn pinta_na_janela(
    host: &mut MockPanelHost,
    st: &mut InspectorState,
    largura: f32,
    altura: f32,
) -> Vec<(NodeId, Rect)> {
    use ph2d_editor_core::screens::layout::{CenterSplit, ChromeBands, DockSides, HeroLayout};
    let janela = Rect::new(0.0, 0.0, 1600.0, altura);
    let bandas = ChromeBands {
        right_dock_w: largura,
        ..ChromeBands::DEFAULT
    };
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
}

/// O painel a `largura`: o botão, a miniatura e o que a moldura pintou.
fn a_largura(largura: f32) -> (Rect, Rect, CabecaPintada) {
    let (mut host, mut st) = cena(factos());
    let r = pinta_a_largura(&mut host, &mut st, largura);
    (
        rect_de(&r, INSP_IMAGE_EDIT_PAINTER),
        rect_de(&r, INSP_RENDER_TEXTURE_SLOT),
        cabeca_pintada().expect("a moldura pintou"),
    )
}

fn dentro(q: Rect, m: Rect) -> bool {
    q.x >= m.x && q.y >= m.y && q.x + q.w <= m.x + m.w + 0.01 && q.y + q.h <= m.y + m.h + 0.01
}

/// As larguras do painel que o cartão tem de aguentar: do piso real a bem largo.
fn larguras() -> [f32; 6] {
    [
        ph2d_editor_core::interaction::WidgetStore::DOCK_W_MIN,
        ph2d_tokens::chrome::PANEL_MIN_W_PX,
        240.0,
        270.0,
        300.0,
        420.0,
    ]
}

/// ⭐ Dono 06/11: com um botão só, o tamanho não sobe ao título e a miniatura não muda ao estreitar o
/// painel. Nome, tamanho e botão ficam ao lado dela, dentro da moldura, sem sobrepor.
/// *Mutações: a miniatura a ceder com a largura · o tamanho fora da moldura.*
#[test]
fn a_miniatura_e_o_tamanho_nao_mudam_com_a_largura() {
    let lado = a_largura(300.0).1.w;
    for largura in larguras() {
        let (a, foto, p) = a_largura(largura);
        let m = p.moldura;
        assert!((foto.w - lado).abs() < 0.01, "a {largura}: {foto:?}");
        assert!((foto.h - lado).abs() < 0.01, "a {largura}: {foto:?}");
        assert!(foto.x + foto.w < a.x, "a {largura}: {foto:?} {a:?}");
        assert!(a.y >= p.nome.y + p.nome.h, "a {largura}: nome sob o botão");
        let t = p.tamanho.expect("o tamanho na moldura");
        assert!(
            t.x >= foto.x + foto.w,
            "a {largura}: {t:?} sobre a miniatura"
        );
        assert!(t.y + t.h <= a.y, "a {largura}: tamanho sob o botão");
        for q in [a, foto, t] {
            assert!(dentro(q, m), "a {largura}: {q:?} fora de {m:?}");
        }
    }
}

/// ⭐ Dono 06/11: na moldura só a miniatura e UM botão, o da ferramenta da imagem.
/// *Mutação: pôr um segundo `Verbo` em `cabeca` ⇒ RED.*
#[test]
fn a_moldura_tem_um_so_botao() {
    let (mut host, mut st) = cena(factos());
    let r = pinta(&mut host, &mut st);
    let m = cabeca_pintada().expect("a moldura pintou").moldura;
    let alvos: Vec<NodeId> = r
        .iter()
        .filter(|(i, q)| {
            q.w > 0.0
                && dentro(*q, m)
                && *i != INSP_RENDER_TEXTURE_SLOT
                && *i != INSP_IMAGE_EDIT_PAINTER
        })
        .map(|(i, _)| *i)
        .collect();
    assert!(alvos.is_empty(), "{alvos:?}");
}

/// O tamanho comprido do Repeated (`1024 × 1024 px (…)`) vem inteiro onde cabe; no estreito, sem os parênteses, que
/// os campos Width/Height já mostram, e no piso sem a unidade. Nunca passa da coluna.
/// *Mutação: sem o recurso à linha curta ⇒ no piso o tamanho passa da coluna.*
#[test]
fn o_tamanho_comprido_do_repeated_vem_inteiro_ou_sem_os_parenteses() {
    let medida = |largura: f32| {
        let (mut host, mut st) = cena(factos());
        set_current_inspector_sprite(Some(InspectorSpriteInfo {
            image_use: ph2d_editor_core::InspectorImageUse::Repeated,
            draw: ph2d_editor_core::InspectorDraw::Tiled { copies: [3.0, 1.0] },
            source_pixels: Some((1024, 1024)),
            ..sprite()
        }));
        let _ = pinta_a_largura(&mut host, &mut st, largura);
        let p = cabeca_pintada().expect("a moldura pintou");
        let t = p.tamanho.expect("o tamanho na moldura");
        assert!(
            t.x + t.w <= p.nome.x + p.nome.w,
            "a {largura}: {t:?} passa da coluna {:?}",
            p.nome
        );
        t
    };
    let curta = medida(ph2d_editor_core::interaction::WidgetStore::DOCK_W_MIN);
    let inteira = medida(420.0);
    assert!(inteira.w > curta.w, "{inteira:?} × {curta:?}");
}

/// ⭐ A moldura não muda de altura com a largura, e o botão clica em todas.
/// *Mutação: a altura a depender da largura.*
#[test]
fn a_moldura_tem_a_mesma_altura_em_toda_largura_e_o_botao_clica() {
    let altura = a_largura(300.0).2.moldura.h;
    for largura in larguras() {
        let d = a_largura(largura).2.moldura.h - altura;
        assert!(d.abs() < 0.01, "a {largura}: a moldura mudou {d}");
        let (mut host, mut st) = cena(factos());
        let r = pinta_a_largura(&mut host, &mut st, largura);
        for (id, pedido, draw) in [(INSP_IMAGE_EDIT_PAINTER, ImageUseEdit::EditInPainter, true)] {
            let q = rect_de(&r, id);
            let _ = host.drained_actions();
            let mut eventos = Vec::new();
            for kind in [PointerKind::Move, PointerKind::Down, PointerKind::Up] {
                eventos.extend(host.dispatch_pointer_event(PointerEvent {
                    x: q.x + q.w * 0.5,
                    y: q.y + q.h * 0.5,
                    pressure: 1.0,
                    kind,
                    source: ph2d_host::PointerSource::Mouse,
                    button: PointerButton::Primary,
                    timestamp_ns: 0,
                }));
            }
            for ev in eventos {
                let _ = host.apply_panel_event::<InspectorPanel>(&mut st, ev);
            }
            let a = host.drained_actions();
            assert_eq!(edicoes(&a), [pedido], "a {largura}");
            let entrou = a.iter().any(|x| {
                matches!(
                    x,
                    EditorAction::ObjectMode(ph2d_editor_core::object_mode::ModeRequest::Enter(
                        ph2d_editor_core::object_mode::ObjectMode::Draw
                    ))
                )
            });
            assert_eq!(entrou, draw, "a {largura}");
        }
    }
}

/// ⭐ Dono 05/11: em Repeated nada se pinta abaixo de Width/Height: a distância até ao cartão seguinte é a do Still.
/// *Mutação: devolver uma tira abaixo dos campos ⇒ o vão cresce.*
#[test]
fn em_repeated_o_cartao_termina_em_width_e_height() {
    use ph2d_editor_core::InspectorImageUse as Uso;
    let vao = |uso, draw, id: NodeId| {
        let (mut host, mut st) = cena(factos());
        set_current_inspector_sprite(Some(InspectorSpriteInfo {
            image_use: uso,
            draw,
            ..sprite()
        }));
        slot_image::set_current_slot_image(Some(slot_image::SlotImage {
            entity_bits: 7,
            name: "Platform".to_owned(),
            rgba: std::sync::Arc::new(vec![255; 16]),
            w: 2,
            h: 2,
        }));
        let r = pinta(&mut host, &mut st);
        slot_image::set_current_slot_image(None);
        let campo = rect_de(&r, id);
        let abaixo = r
            .iter()
            .filter(|(_, q)| q.w > 0.0 && q.y >= campo.y + campo.h)
            .map(|(_, q)| q.y)
            .fold(f32::MAX, f32::min);
        abaixo - (campo.y + campo.h)
    };
    let still = vao(
        Uso::Still,
        ph2d_editor_core::InspectorDraw::Fixed,
        INSP_IMAGE_USE[0],
    );
    let repetida = vao(
        Uso::Repeated,
        ph2d_editor_core::InspectorDraw::Tiled { copies: [3.0, 1.0] },
        INSP_IMAGE_TILED_SIZE[0],
    );
    // O segmento do Use fica `SEG_PAD` dentro da caixa dele; o campo de Height é a própria caixa.
    assert_eq!(
        repetida,
        still - ph2d_card_stack::metrics::SEG_PAD,
        "sobra um vão abaixo de Width/Height"
    );
}

/// ⭐ Dono 05/11: Width e Height na mesma linha, em qualquer largura; o rótulo não cobre o vizinho nem o campo.
/// *Mutações: `tudo_cabe` sem o par de eixos ⇒ empilham. `faixa` sem tirar a unidade ⇒ o rótulo quebra e a linha cresce.*
#[test]
fn width_e_height_ficam_lado_a_lado_em_qualquer_largura() {
    use ph2d_editor_core::panel::cards::censo;
    for largura in [
        ph2d_tokens::chrome::PANEL_MIN_W_PX,
        200.0,
        240.0,
        293.0,
        400.0,
    ] {
        let (mut host, mut st) = cena(factos());
        set_current_inspector_sprite(Some(InspectorSpriteInfo {
            image_use: ph2d_editor_core::InspectorImageUse::Repeated,
            draw: ph2d_editor_core::InspectorDraw::Tiled { copies: [3.0, 1.0] },
            ..sprite()
        }));
        let _ = pinta_a_largura(&mut host, &mut st, largura);
        let (r, c) = censo::grava(|| pinta_a_largura(&mut host, &mut st, largura));
        let (a, b) = (
            rect_de(&r, INSP_IMAGE_TILED_SIZE[0]),
            rect_de(&r, INSP_IMAGE_TILED_SIZE[1]),
        );
        assert_eq!(a.y, b.y, "a {largura}: {a:?} {b:?}");
        assert_eq!(a.w, b.w, "a {largura}: colunas iguais");
        assert!(a.x + a.w <= b.x, "a {largura}: sobrepostos {a:?} {b:?}");
        assert!(
            a.w >= ph2d_card_stack::metrics::FIELD_VALUE_MIN_W,
            "a {largura}: o número não cabe: {a:?}"
        );
        assert!(b.x + b.w <= 1600.0, "a {largura}: {b:?} fora");
        let dica = ph2d_editor_core::panel::cards::dicas::field_hint(
            host.store_mut(),
            INSP_IMAGE_TILED_SIZE[0],
        );
        assert!(
            dica.is_some_and(|d| d.contains("Width (tiles)")),
            "a {largura}: a dica perdeu a unidade"
        );
        let cortados = c.textos_cortados();
        assert!(
            !cortados
                .iter()
                .any(|t| t.contains("Width") || t.contains("Height")),
            "a {largura}: {cortados:?}"
        );
        let linha = c
            .linhas
            .iter()
            .find(|l| l.celulas.iter().any(|m| m.nome.starts_with("Width")))
            .expect("a linha de Width");
        assert_eq!(linha.celulas.len(), 2, "a {largura}: {linha:?}");
        assert_eq!(linha.fundo, a.y + a.h, "a {largura}: {linha:?}");
        // Uma linha de rótulo: a mesma altura de cima a baixo, estreito ou largo.
        let m = ph2d_card_stack::metrics::LABEL_PX
            + ph2d_card_stack::metrics::LABEL_BOTTOM
            + ph2d_card_stack::metrics::FIELD_H;
        assert!(
            (linha.fundo - linha.topo - m).abs() < 0.01,
            "a {largura}: a faixa cresceu {linha:?}"
        );
    }
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

/// ⭐ **Dono 05/11:** no mínimo real do painel (`DOCK_W_MIN`, 210 px; a barra de rolar come mais) nenhum texto do cartão
/// da Sprite sai cortado: nem botão (`Reimport`: 157 > 146) nem nome de ponto do Pivot, em todos os pontos e em Custom.
#[test]
fn nada_do_cartao_da_sprite_e_cortado_no_minimo_do_painel() {
    use ph2d_editor_core::text_elide::elisao;
    use ph2d_editor_core::widget::card_censo;
    let piso = ph2d_editor_core::interaction::WidgetStore::DOCK_W_MIN;
    let pontos = (0..3_u8)
        .flat_map(|l| (0..3_u8).map(move |c| Some([c, l])))
        .chain([None]);
    for ponto in pontos {
        for largura in [piso, ph2d_tokens::PANEL_MIN_W_PX, 294.0] {
            let (mut host, mut st) = cena(SpriteFacts {
                pivot_ponto: ponto,
                ..factos()
            });
            let _ = pinta_na_janela(&mut host, &mut st, largura, 2400.0);
            let ((_, medidos), censo) = card_censo::grava(|| {
                elisao::medindo(|| pinta_na_janela(&mut host, &mut st, largura, 2400.0))
            });
            let cortados: std::collections::BTreeSet<_> = medidos
                .iter()
                .filter(|m| !m.coube())
                .map(|m| m.texto.clone())
                .collect();
            assert!(cortados.is_empty(), "{ponto:?} a {largura}: {cortados:?}");
            assert_eq!(
                censo.textos_cortados(),
                Vec::<String>::new(),
                "{ponto:?} a {largura}"
            );
        }
    }
    // Com a barra de rolar o corpo perde 8 px, e o Advanced fica fora da janela do censo: a 200 px sobram 146.
    let (mut host, mut st) = cena(factos());
    let (_, censo) = card_censo::grava(|| pinta_na_janela(&mut host, &mut st, 200.0, 2400.0));
    let r = censo
        .textos
        .iter()
        .find(|t| t.texto.starts_with("Reimport"));
    assert!(r.expect("o censo viu o Reimport").pede <= 146.0, "{r:?}");
}

/// Dono, 06/11: no painel estreito o resumo do cartão dobrado caía em cima do título. O título manda;
/// o resumo vem inteiro ou não vem, e o controlo é que, largo, ele aparece.
#[test]
fn o_resumo_do_cartao_dobrado_nunca_cai_em_cima_do_titulo() {
    use ph2d_editor_core::widget::card_censo;
    let piso = ph2d_editor_core::interaction::WidgetStore::DOCK_W_MIN;
    let mede = |largura: f32| {
        let (mut host, mut st) = cena(factos());
        for id in [core_ids::INSP_LIVE_RENDER_SECTION, CARD_ORDERING] {
            host.store_mut().set_collapsed(id, true);
        }
        let (_, censo) = card_censo::grava(|| pinta_na_janela(&mut host, &mut st, largura, 2400.0));
        censo.titulos
    };
    for largura in [piso, 230.0, 250.0, 270.0, 294.0, 340.0, 420.0] {
        for t in mede(largura) {
            if let Some(x) = t.resumo_x {
                assert!(x >= t.fim, "{largura}: {t:?}");
            }
        }
    }
    let tem_resumo = |titulos: &[card_censo::TituloMedido], nome: &str| {
        titulos
            .iter()
            .rev()
            .find(|t| t.titulo == nome)
            .is_some_and(|t| t.resumo_x.is_some())
    };
    let largo = mede(420.0);
    assert!(tem_resumo(&largo, "Transform"), "largo mostra: {largo:?}");
    assert!(tem_resumo(&largo, "Advanced"), "largo mostra: {largo:?}");
    let estreito = mede(piso);
    assert!(!tem_resumo(&estreito, "Transform"), "{estreito:?}");
}

/// ⭐ *Editar o X do Pivot não arredonda nem grava o Y: o outro eixo vem do valor exacto, não da caixa.*
/// *Mutação: ler as duas caixas ⇒ o Y sai 21.*
#[test]
fn editar_um_eixo_do_pivot_nao_mexe_no_outro() {
    let f = SpriteFacts {
        pivot_ponto: None,
        pivot_px: [10.4, 20.6],
        ..factos()
    };
    let (mut host, mut st) = cena(f);
    let _ = pinta(&mut host, &mut st);
    let _ = host.drained_actions();
    host.store_mut()
        .set_number_value(INSP_SPRITE_PIVOT_XY[0], 5.0);
    let _ = host.apply_panel_event::<InspectorPanel>(
        &mut st,
        WidgetEvent::ValueChanged(INSP_SPRITE_PIVOT_XY[0]),
    );
    let a = host.drained_actions();
    assert_eq!(
        edicoes(&a),
        [ImageUseEdit::Pivot(SpritePivot::Px([5.0, 20.6]))]
    );
    let _ = pinta(&mut host, &mut st);
    let _ = host.drained_actions();
    host.store_mut()
        .set_number_value(INSP_SPRITE_PIVOT_XY[1], 6.0);
    let _ = host.apply_panel_event::<InspectorPanel>(
        &mut st,
        WidgetEvent::ValueChanged(INSP_SPRITE_PIVOT_XY[1]),
    );
    let a = host.drained_actions();
    assert_eq!(
        edicoes(&a),
        [ImageUseEdit::Pivot(SpritePivot::Px([10.4, 6.0]))]
    );
}

/// *«Tint each corner» aberto à mão é de um objecto: o outro continua fechado.*
/// *Mutação: um interruptor global ⇒ o objecto 8 abre os quatro cantos.*
#[test]
fn tint_each_corner_aberto_e_de_um_objecto_so() {
    let (mut host, mut st) = cena(factos());
    let _ = clica(&mut host, &mut st, INSP_SPRITE_CORNERS_ON);
    let pintado = |r: &[(NodeId, Rect)]| {
        r.iter()
            .any(|(i, q)| *i == INSP_SPRITE_CORNER_TL && q.w > 0.0)
    };
    assert!(pintado(&pinta(&mut host, &mut st)), "o objecto 7 abriu");
    set_current_inspector_sprite(Some(InspectorSpriteInfo {
        entity_bits: 8,
        ..sprite()
    }));
    assert!(!pintado(&pinta(&mut host, &mut st)), "o 8 continua fechado");
    set_current_inspector_sprite(Some(sprite()));
    assert!(pintado(&pinta(&mut host, &mut st)), "o 7 lembra-se");
}

/// Os campos do cartão: um grupo é um só campo (o seletor Use, a grade do Pivot, o par Strategy).
pub(crate) fn campos_da_sprite() -> Vec<Vec<NodeId>> {
    let um = |v: &[NodeId]| v.iter().map(|id| vec![*id]).collect::<Vec<_>>();
    let mut c = um(&[
        core_ids::INSP_ENTITY_NAME,
        INSP_VISIBILITY_CHECK,
        INSP_RENDER_TEXTURE_SLOT,
        INSP_IMAGE_EDIT_PAINTER,
        INSP_SPRITE_TINT_SWATCH,
        INSP_SPRITE_OPACITY_CHIP,
        INSP_SPRITE_FLIP_X,
        INSP_SPRITE_FLIP_Y,
        INSP_TRANSFORM_POS_X,
        INSP_TRANSFORM_POS_Y,
        INSP_TRANSFORM_ROT,
        INSP_TRANSFORM_ROTATE_CW,
        INSP_TRANSFORM_FLIP_H,
        INSP_TRANSFORM_FLIP_V,
        ph2d_panel_inspector::ids::INSP_SPRITE_MIRROR_KEEP,
        INSP_TRANSFORM_SCALE_X,
        INSP_TRANSFORM_SCALE_Y,
        ph2d_panel_inspector::SCALE_LOCK,
        INSP_TRANSFORM_SKEW_X,
        INSP_TRANSFORM_SKEW_Y,
        INSP_ORDER_Z_INDEX,
        INSP_ORDER_YSORT_ENABLED,
        ph2d_tool_registry::hash_node_id("insp_order_options.dd"),
        INSP_SPRITE_EMISSIVE_CHIP,
        INSP_SPRITE_TINT_FILL,
        INSP_SPRITE_CORNERS_ON,
        INSP_RENDER_SOURCE_REIMPORT,
    ]);
    c.push(INSP_IMAGE_USE.to_vec());
    c.push(
        INSP_SPRITE_PIVOT
            .iter()
            .copied()
            .chain([INSP_SPRITE_PIVOT_NOME])
            .collect(),
    );
    c.push(vec![
        core_ids::INSP_RENDER_STRATEGY_ATLAS,
        INSP_RENDER_STRATEGY_INDIVIDUAL,
    ]);
    c.push(vec![INSP_RENDER_FORMAT_RGBA8, INSP_RENDER_FORMAT_RGBA16]);
    // P14: a linha Normal Map do cartão Image.
    c.push(vec![INSP_IMAGE_NORMAL_SLOT, INSP_IMAGE_NORMAL_NONE]);
    c
}

/// A aceitação da ficha, contada no que está pintado: em uso, os campos à vista com o Look aberto e os outros dobrados, e
/// o total com tudo aberto. O que pinta e não é campo é a moldura: quem acrescenta um campo muda o número da moldura.
#[test]
fn o_censo_dos_campos_da_sprite_em_uso() {
    let (mut host, mut st) = cena(factos());
    let conta = |r: &[(NodeId, Rect)]| {
        let tem = |id: &NodeId| r.iter().any(|(i, q)| i == id && q.w > 0.0);
        let campos = campos_da_sprite();
        let vistos = campos.iter().filter(|g| g.iter().any(tem)).count();
        let moldura = r
            .iter()
            .filter(|(i, q)| q.w > 0.0 && !campos.iter().flatten().any(|c| c == i))
            .count();
        (vistos, moldura)
    };
    for id in [core_ids::INSP_LIVE_RENDER_SECTION, CARD_ORDERING] {
        host.store_mut().set_collapsed(id, true);
    }
    assert_eq!(conta(&pinta(&mut host, &mut st)), (14, 19), "à vista");
    for id in [
        core_ids::INSP_LIVE_RENDER_SECTION,
        CARD_ORDERING,
        ph2d_panel_inspector::CARD_TRANSFORM_IMAGE,
    ] {
        host.store_mut().set_collapsed(id, false);
    }
    assert_eq!(conta(&pinta(&mut host, &mut st)), (32, 19), "no total");
}

/// Na Sprite o `Mirror object` mora no Look, e clicá-lo troca o sinal da escala (dono 06/11).
/// *Mutação: o Transform pintar os três verbos ⇒ o último botão pintado fica abaixo do Position.*
#[test]
fn o_mirror_do_look_espelha_o_objeto_e_saiu_do_transform() {
    let (mut host, mut st) = cena(factos());
    host.store_mut()
        .set_collapsed(ph2d_panel_inspector::CARD_TRANSFORM_IMAGE, false);
    let r = pinta(&mut host, &mut st);
    let (botao, posicao) = (
        rect_de(&r, INSP_TRANSFORM_FLIP_H),
        rect_de(&r, INSP_TRANSFORM_POS_X),
    );
    assert!(botao.y < posicao.y, "{botao:?} × {posicao:?}");
    let (_, a) = clica(&mut host, &mut st, INSP_TRANSFORM_FLIP_H);
    assert!(
        a.iter().any(|x| matches!(
            x,
            EditorAction::InspectorTransformEdit(t) if t.scale == [-1.0, 1.0]
        )),
        "{a:?}"
    );
}

/// Keep in place marcado: o Mirror passa a pedir o espelho que compensa o pivô (dono 06/11).
/// *Mutação: tirar o desvio de `giro_click` ⇒ sai o espelho de sempre, e o desenho salta.*
#[test]
fn com_keep_in_place_o_mirror_pede_o_espelho_no_lugar() {
    let (mut host, mut st) = cena(factos());
    let keep = ph2d_panel_inspector::ids::INSP_SPRITE_MIRROR_KEEP;
    let _ = clica(&mut host, &mut st, keep);
    let (_, a) = clica(&mut host, &mut st, INSP_TRANSFORM_FLIP_H);
    let _ = clica(&mut host, &mut st, keep);
    assert!(
        a.iter().any(|x| matches!(
            x,
            EditorAction::ImageUseEdit {
                edit: ph2d_editor_core::ImageUseEdit::MirrorInPlace(0),
                ..
            }
        )),
        "{a:?}"
    );
}

/// Virar e tingir com os outros cartões dobrados: Flip, a amostra do Tint e a cor no seletor, 3 gestos.
#[test]
fn virar_e_tingir_com_os_cartoes_fechados_sao_tres_gestos() {
    let (mut host, mut st) = cena(factos());
    for id in [core_ids::INSP_LIVE_RENDER_SECTION, CARD_ORDERING] {
        host.store_mut().set_collapsed(id, true);
    }
    let (_, a) = clica(&mut host, &mut st, INSP_SPRITE_FLIP_X);
    assert!(
        a.iter().any(|x| matches!(
            x,
            EditorAction::InspectorSpriteEdit {
                edit: SpriteFieldEdit::FlipX(_),
                ..
            }
        )),
        "{a:?}"
    );
    let _ = clica(&mut host, &mut st, INSP_SPRITE_TINT_SWATCH);
    assert_eq!(
        host.store().picker_target(),
        Some(INSP_SPRITE_TINT_SWATCH),
        "o 2.º gesto abre o seletor, e o 3.º é escolher a cor"
    );
}

/// ⭐ P14: a linha **Normal Map** do cartão Image — pintada e registada, o clique REAL na ranhura pede o seletor
/// (`LoadNormalMap`) e o **None** (só com normal) tira-a. Mutação que sangra: tirar o `cards_sprite_normal::evento`
/// do `evento` do cartão.
#[test]
fn a_linha_normal_map_responde_a_um_ponteiro_real() {
    slot_image::set_current_normal_map(None);
    let (mut host, mut st) = cena(factos());
    assert!(
        !pinta(&mut host, &mut st)
            .iter()
            .any(|(i, _)| *i == INSP_IMAGE_NORMAL_NONE),
        "sem normal não há None"
    );
    let (_, a) = clica(&mut host, &mut st, INSP_IMAGE_NORMAL_SLOT);
    assert_eq!(edicoes(&a), [ImageUseEdit::LoadNormalMap]);
    slot_image::set_current_normal_map(Some((7, "rock_n".into())));
    let (_, a) = clica(&mut host, &mut st, INSP_IMAGE_NORMAL_NONE);
    assert_eq!(edicoes(&a), [ImageUseEdit::SetNormalMap(None)]);
    slot_image::set_current_normal_map(None);
}
