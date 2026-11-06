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

/// A coluna do painel com `largura` px, numa janela larga, assente.
fn pinta_a_largura(
    host: &mut MockPanelHost,
    st: &mut InspectorState,
    largura: f32,
) -> Vec<(NodeId, Rect)> {
    use ph2d_editor_core::screens::layout::{CenterSplit, ChromeBands, DockSides, HeroLayout};
    let janela = Rect::new(0.0, 0.0, 1600.0, 900.0);
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

/// O painel a `largura`: os dois botões, a miniatura e o que a moldura pintou.
fn a_largura(largura: f32) -> (Rect, Rect, Rect, CabecaPintada) {
    let (mut host, mut st) = cena(factos());
    let r = pinta_a_largura(&mut host, &mut st, largura);
    (
        rect_de(&r, INSP_IMAGE_EDIT_PIXEL),
        rect_de(&r, INSP_IMAGE_EDIT_PAINTER),
        rect_de(&r, INSP_RENDER_TEXTURE_SLOT),
        cabeca_pintada().expect("a moldura pintou"),
    )
}

/// Os dois lados do degrau.
const FOLGA: f32 = 1.0;

fn lado_a_lado(largura: f32) -> bool {
    let (a, b, _, _) = a_largura(largura);
    a.y == b.y
}

/// O limite do degrau, em px de painel, medido: a menor largura com os botões lado a lado.
fn limite_do_degrau() -> f32 {
    (200..=400)
        .map(|w| w as f32)
        .find(|w| lado_a_lado(*w))
        .expect("o degrau largo existe")
}

fn dentro(q: Rect, m: Rect) -> bool {
    q.x >= m.x && q.y >= m.y && q.x + q.w <= m.x + m.w + 0.01 && q.y + q.h <= m.y + m.h + 0.01
}

/// ⭐ Dono 05/11: largo = nome, tamanho e os dois botões lado a lado; estreito = o tamanho sobe ao título, os
/// botões empilham-se na largura da coluna, tudo dentro da moldura, sem sobrepor.
/// *Mutações: o degrau ao contrário · a pilha com a largura de meia coluna · o nome sem subir · o tamanho na moldura.*
#[test]
fn os_dois_lados_do_degrau_arrumam_o_tamanho_e_os_botoes() {
    let limite = limite_do_degrau();
    eprintln!("o degrau largo começa em {limite} px de painel");
    for (largura, largo) in [
        (300.0, true),
        (limite + FOLGA, true),
        (limite, true),
        (limite - FOLGA, false),
        (ph2d_tokens::chrome::PANEL_MIN_W_PX, false),
    ] {
        let (a, b, foto, p) = a_largura(largura);
        let m = p.moldura;
        for q in [a, b, foto] {
            assert!(dentro(q, m), "a {largura}: {q:?} fora de {m:?}");
        }
        assert_eq!(a.y == b.y, largo, "a {largura}: {a:?} {b:?}");
        assert!((foto.w - foto.h).abs() < 0.01, "a {largura}: {foto:?}");
        assert!(foto.x + foto.w < a.x, "a {largura}: {foto:?} {a:?}");
        assert!(a.y >= p.nome.y + p.nome.h, "a {largura}: nome sob o botão");
        assert!(foto.x > 1600.0 - largura, "a {largura}: {foto:?} fora");
        if largo {
            assert!((a.w - b.w).abs() < 0.01, "a {largura}: {a:?} {b:?}");
            assert!(a.x + a.w <= b.x, "a {largura}: sobrepostos {a:?} {b:?}");
            let t = p.tamanho.expect("o tamanho na moldura");
            assert!(p.tamanho_no_titulo.is_none(), "a {largura}");
            assert!(dentro(t, m), "a {largura}: {t:?}");
            assert!(t.y + t.h <= a.y, "a {largura}: tamanho sob o botão");
        } else {
            assert_eq!((a.x, a.w), (b.x, b.w), "a {largura}: a mesma coluna");
            assert!(
                a.w >= p.nome.w - 0.01,
                "a {largura}: na largura da coluna {a:?} {:?}",
                p.nome
            );
            assert!(a.y + a.h <= b.y, "a {largura}: sobrepostos {a:?} {b:?}");
            assert!(
                p.tamanho.is_none(),
                "a {largura}: o tamanho ficou na moldura"
            );
            let t = p.tamanho_no_titulo.expect("o tamanho no título, inteiro");
            assert!(
                t.y + t.h <= m.y,
                "a {largura}: {t:?} não está acima de {m:?}"
            );
            assert!(t.x + t.w <= m.x + m.w, "a {largura}: {t:?} sai da moldura");
        }
    }
    assert!(
        limite > 270.0 && limite <= 300.0,
        "o painel de fábrica fica no largo: {limite}"
    );
}

/// O tamanho comprido do Repeated (`1024 × 1024 px (…)`) sobe inteiro onde cabe; onde nem o título o leva, sobe sem
/// os parênteses, que os campos Width/Height já mostram. Nunca cortado.
/// *Mutação: sem o recurso à linha curta ⇒ a 220 o tamanho some.*
#[test]
fn o_tamanho_comprido_do_repeated_sobe_inteiro_ou_sem_os_parenteses() {
    let medida = |largura: f32| {
        let (mut host, mut st) = cena(factos());
        set_current_inspector_sprite(Some(InspectorSpriteInfo {
            image_use: ph2d_editor_core::InspectorImageUse::Repeated,
            draw: ph2d_editor_core::InspectorDraw::Tiled { copies: [3.0, 1.0] },
            source_pixels: Some((1024, 1024)),
            ..sprite()
        }));
        let _ = pinta_a_largura(&mut host, &mut st, largura);
        cabeca_pintada()
            .and_then(|p| p.tamanho_no_titulo)
            .expect("o tamanho subiu ao título")
    };
    let curta = medida(ph2d_tokens::chrome::PANEL_MIN_W_PX);
    let inteira = medida(260.0);
    assert!(inteira.w > curta.w, "{inteira:?} × {curta:?}");
}

/// ⭐ A moldura não muda de altura com o degrau: o controlo abaixo dela fica no mesmo sítio.
/// *Mutação: a altura a somar o degrau.*
#[test]
fn a_moldura_tem_a_mesma_altura_nos_dois_degraus_e_os_dois_clicam() {
    let limite = limite_do_degrau();
    let altura = |w: f32| a_largura(w).3.moldura.h;
    let d = altura(limite) - altura(limite - FOLGA);
    assert!(d.abs() < 0.01, "a moldura mudou com o degrau: {d}");
    for largura in [limite, limite - FOLGA] {
        let (mut host, mut st) = cena(factos());
        let r = pinta_a_largura(&mut host, &mut st, largura);
        for (id, pedido, draw) in [
            (INSP_IMAGE_EDIT_PIXEL, ImageUseEdit::EditInPixel, false),
            (INSP_IMAGE_EDIT_PAINTER, ImageUseEdit::EditInPainter, true),
        ] {
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
