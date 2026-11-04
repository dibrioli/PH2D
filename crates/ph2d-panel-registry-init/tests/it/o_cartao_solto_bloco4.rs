//! ⭐ **Bloco 4 do smoke do dono, 02/11** (`docs/CartoesUI/01_decisoes_do_dono.md` T39–T43), no
//! hero montado: a largura do solto, a secção que fica na lista com o cartão preso a outro osso, o
//! regresso com mola, a visibilidade do solto e a barra de rolagem dentro da lista.
#![cfg(feature = "panel-skeleton")]

use super::o_cartao_solto_no_canvas::{
    arrasta, bones, centro, clica, hero_com_o_bones, mao, osso_em_foco, pinta, solta,
};
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::ids::VECTOR_BONE_IK_MIX;
use ph2d_editor_core::interaction::{
    LooseEdge, loose_edge_of, loose_ghost_of, loose_pin_of, loose_rect_of, loose_return_of,
};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerKind;
use ph2d_panel_skeleton::cards::{CARD_BONE, CARD_IK, CARD_TOOL};
use ph2d_text::TextSystem;

fn borda(h: &HeroScreen, e: LooseEdge) -> (f32, f32) {
    centro(
        h.hit_index
            .rect_for(loose_edge_of(CARD_TOOL, e))
            .expect("a borda do solto está pintada"),
    )
}

/// T39: as duas bordas do solto mudam-lhe a largura (a oposta não se mexe), com o mínimo legível; e
/// apanhar a borda sem a mexer não dobra o cartão.
/// *Mutação: o despacho sem a porta da borda ⇒ a mão apanha o cabeçalho e o cartão anda.*
#[test]
fn a_loose_card_changes_width_from_either_edge() {
    let mut h = hero_com_o_bones();
    solta(&mut h, CARD_TOOL);
    let antes = h.store.loose_card(CARD_TOOL).expect("solto");
    let d = borda(&h, LooseEdge::Right);
    arrasta(&mut h, d, (d.0 + 90.0, d.1));
    let c = h.store.loose_card(CARD_TOOL).expect("solto");
    assert!(
        (c.w - (antes.w + 90.0)).abs() < 0.5 && (c.at.0 - antes.at.0).abs() < 0.5,
        "a borda direita: {antes:?} → {c:?}"
    );
    let e = borda(&h, LooseEdge::Left);
    arrasta(&mut h, e, (e.0 - 40.0, e.1));
    let c2 = h.store.loose_card(CARD_TOOL).expect("solto");
    assert!(
        (c2.w - (c.w + 40.0)).abs() < 0.5 && (c2.at.0 - (c.at.0 - 40.0)).abs() < 0.5,
        "a borda esquerda: {c:?} → {c2:?}"
    );
    let d = borda(&h, LooseEdge::Right);
    arrasta(&mut h, d, (d.0 - 2000.0, d.1));
    let c3 = h.store.loose_card(CARD_TOOL).expect("solto");
    assert!(
        (c3.w - m::LOOSE_MIN_W).abs() < 0.5,
        "abaixo do mínimo: {c3:?}"
    );
    let d = borda(&h, LooseEdge::Right);
    mao(&mut h, PointerKind::Down, d);
    mao(&mut h, PointerKind::Up, d);
    pinta(&mut h, 2);
    assert!(
        !h.store.is_collapsed(CARD_TOOL),
        "apanhar a borda dobrou o cartão"
    );
}

/// T40: o IK preso ao Bone 2 fica no canvas e a lista do Bone 3 mostra o IK dele — dois campos «Mix»,
/// cada um com o seu osso; a pressão no preso põe os números dele no store (o gesto parte daí).
/// *Mutação: os soltos presos a outro osso de volta à linha-fantasma ⇒ a lista não tem o IK.*
#[test]
fn a_card_pinned_to_one_bone_leaves_its_section_in_the_list_for_another() {
    let mut h = hero_com_o_bones();
    osso_em_foco(2, "Bone 2", 0.9);
    pinta(&mut h, 20);
    solta(&mut h, CARD_IK);
    clica(&mut h, loose_pin_of(CARD_IK));
    osso_em_foco(3, "Bone 3", 0.2);
    ph2d_panel_skeleton::publica_presos(&h.store.loose_pins_of(bones()), |b| {
        osso_em_foco(b, "Bone 2", 0.9);
    });
    pinta(&mut h, 5);
    assert_eq!(
        h.hit_index.rect_for(loose_ghost_of(CARD_IK)),
        None,
        "a lista do Bone 3 mostra a linha-fantasma do IK do Bone 2"
    );
    let lista = h.store.panel_rect(bones()).expect("a lista");
    let solto = h
        .store
        .panel_rect(loose_rect_of(CARD_IK))
        .expect("o preso no canvas");
    let mix: Vec<Rect> = h
        .hit_index
        .iter_registrations()
        .filter(|(id, _)| *id == VECTOR_BONE_IK_MIX)
        .map(|(_, r)| r)
        .collect();
    let dentro = |a: Rect, r: Rect| a.contains(r.x + 1.0, r.y + 1.0);
    assert!(
        mix.iter().any(|r| dentro(lista, *r)) && mix.iter().any(|r| dentro(solto, *r)),
        "o «Mix» não está na lista E no preso: {mix:?} (lista {lista:?}, preso {solto:?})"
    );
    assert_eq!(h.store.number_value(VECTOR_BONE_IK_MIX), Some(0.2));
    assert!(
        h.store
            .pinned_values(CARD_IK)
            .contains(&(VECTOR_BONE_IK_MIX, 0.9)),
        "os números do preso não são os do Bone 2"
    );
    let canto = (solto.x + solto.w - 12.0, solto.y + solto.h - 4.0);
    mao(&mut h, PointerKind::Down, canto);
    assert_eq!(
        h.store.number_value(VECTOR_BONE_IK_MIX),
        Some(0.9),
        "a pressão no preso não partiu dos números dele"
    );
    mao(&mut h, PointerKind::Up, canto);
}

/// T41: «Return to panel» e «Reset Positions» trazem o solto de volta a VOAR do canvas (a mola
/// parte de onde ele estava) e ele assenta na coluna.
/// *Mutação: sem o `arrive` ⇒ o cartão aparece no lugar no mesmo quadro (salto).*
#[test]
fn a_returning_card_flies_back_from_the_canvas() {
    let mut h = hero_com_o_bones();
    solta(&mut h, CARD_TOOL);
    let volta = h
        .hit_index
        .rect_for(loose_return_of(CARD_TOOL))
        .expect("o «×»");
    mao(&mut h, PointerKind::Down, centro(volta));
    mao(&mut h, PointerKind::Up, centro(volta));
    pinta(&mut h, 1);
    let dx = h.store.card_list_mut(bones()).stack.arriving_dx(CARD_TOOL);
    assert!(
        dx.is_some_and(|d| d.abs() > 50.0),
        "o «×» devolveu sem voo: {dx:?}"
    );
    pinta(&mut h, 200);
    assert_eq!(
        h.store.card_list_mut(bones()).stack.arriving_dx(CARD_TOOL),
        None,
        "não assentou"
    );
    solta(&mut h, CARD_TOOL);
    h.store.reset_card_list(bones(), true, false, false);
    assert_eq!(
        h.store.loose_card(CARD_TOOL),
        None,
        "o Reset não o devolveu"
    );
    pinta(&mut h, 1);
    let dx = h.store.card_list_mut(bones()).stack.arriving_dx(CARD_TOOL);
    assert!(
        dx.is_some_and(|d| d.abs() > 50.0),
        "o Reset devolveu sem voo: {dx:?}"
    );
}

/// T42 (revê a T37): o solto NÃO preso some quando a seleção é de outra classe e volta com a mesma;
/// o PRESO fica com qualquer seleção.
/// *Mutação: o «Not available» de volta ⇒ o não-preso continua no canvas.*
#[test]
fn an_unpinned_loose_card_hides_for_another_class_and_a_pinned_one_stays() {
    let mut h = hero_com_o_bones();
    osso_em_foco(7, "Bone 7", 0.5);
    pinta(&mut h, 20);
    solta(&mut h, CARD_BONE);
    let sem_osso = || {
        ph2d_panel_skeleton::state::set_current_bone(None);
        ph2d_panel_skeleton::set_current_bone_subject(None);
    };
    sem_osso();
    pinta(&mut h, 10);
    assert_eq!(
        h.store.panel_rect(loose_rect_of(CARD_BONE)),
        None,
        "o solto não preso ficou no canvas com outra classe escolhida"
    );
    assert!(
        h.store.loose_card(CARD_BONE).is_some(),
        "deixou de estar solto"
    );
    osso_em_foco(8, "Bone 8", 0.5);
    pinta(&mut h, 10);
    assert!(
        h.store.panel_rect(loose_rect_of(CARD_BONE)).is_some(),
        "não voltou com um osso escolhido"
    );
    clica(&mut h, loose_pin_of(CARD_BONE));
    sem_osso();
    ph2d_panel_skeleton::publica_presos(&h.store.loose_pins_of(bones()), |b| {
        osso_em_foco(b, "Bone 8", 0.5);
    });
    sem_osso();
    pinta(&mut h, 10);
    assert!(
        h.store.panel_rect(loose_rect_of(CARD_BONE)).is_some(),
        "o preso sumiu com outra classe escolhida"
    );
}

/// T43: numa janela baixa a lista transborda e a barra de rolagem acaba dentro dela, acima da
/// margem do fundo ([foto](docs/CartoesUI/maquete/smoke_0211_barra_atravessa_o_fundo.png)).
/// *Mutação: o corpo até à beirada da lista ⇒ a pista atravessa a margem do fundo.*
#[test]
fn the_scroll_bar_stays_inside_the_list() {
    const BAIXA: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 1600.0,
        h: 500.0,
    };
    let mut h = hero_com_o_bones();
    osso_em_foco(4, "Bone 4", 0.5);
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    for _ in 0..40 {
        scene.reset();
        ph2d_editor_core::aquecimento::com_passo_fixo(
            ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
            || {
                ph2d_editor_core::screens::hero::paint_hero_screen(
                    &mut h, BAIXA, &mut scene, &mut text,
                )
            },
        );
    }
    let lista = h.store.panel_rect(bones()).expect("a lista");
    let pista = h
        .hit_index
        .rect_for(ph2d_editor_core::panel::cards::scroll_bar_of(bones()))
        .expect("a lista transborda e tem barra");
    let fundo = lista.y + lista.h;
    assert!(fundo <= BAIXA.h, "a lista {lista:?} passa da janela");
    assert!(
        pista.y + pista.h <= fundo - m::LIST_PAD_BOTTOM + 0.5,
        "a pista {pista:?} atravessa o fundo da lista {lista:?}"
    );
}

/// T41: arrastado de volta à coluna, o solto é a MÃO que o traz — não ganha também o voo de regresso
/// do «×» (dois movimentos sobrepostos, e o cartão sem clique enquanto voa).
/// *Mutação: o regresso pela mão a guardar a chegada ⇒ o cartão volta a voar do canvas.*
#[test]
fn a_card_dragged_back_does_not_fly_twice() {
    let mut h = hero_com_o_bones();
    let pega = h
        .hit_index
        .rect_for(ph2d_editor_core::ids::grip_de(CARD_TOOL))
        .expect("o título do Tool");
    let de = (pega.x + 12.0, pega.y + pega.h * 0.5);
    solta(&mut h, CARD_TOOL);
    let solto = h
        .hit_index
        .rect_for(ph2d_editor_core::ids::grip_de(CARD_TOOL))
        .expect("a pega do solto");
    mao(
        &mut h,
        PointerKind::Down,
        (solto.x + 12.0, solto.y + solto.h * 0.5),
    );
    pinta(&mut h, 2);
    mao(&mut h, PointerKind::Move, de);
    pinta(&mut h, 2);
    assert_eq!(h.store.loose_card(CARD_TOOL), None, "não reentrou");
    assert_eq!(
        h.store.card_list_mut(bones()).stack.arriving_dx(CARD_TOOL),
        None,
        "a mão e o voo de regresso ao mesmo tempo"
    );
    mao(&mut h, PointerKind::Up, de);
}

/// Smoke do dono 02/11 («Look at»): uma lista que CABE nunca mostra a barra — nem no quadro em que o
/// cartão cresce, nem no quadro em que uma nota nova chega à lista antes de a lista a medir.
/// *Mutação: `rolagem::conteudo` a devolver a altura real ⇒ a barra aparece num quadro.*
#[test]
fn a_list_that_fits_never_flashes_its_scroll_bar() {
    use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
    let mut h = hero_com_o_bones();
    ph2d_panel_skeleton::state::set_current_bone(Some(ph2d_skeleton::bend::BoneSpec::straight(
        20.0, 1.0,
    )));
    ph2d_panel_skeleton::set_current_bone_ik(None);
    pinta(&mut h, 60);
    let barra = ph2d_editor_core::panel::cards::scroll_bar_of(bones());
    let mut vista = String::new();
    osso_em_foco(5, "Bone 5", 1.0);
    for _ in 0..40 {
        pinta(&mut h, 1);
        vista.push(if h.hit_index.rect_for(barra).is_some() {
            '#'
        } else {
            '.'
        });
    }
    h.store.open_context_menu(ContextMenuRequest {
        x: 0.0,
        y: 0.0,
        kind: ContextMenuKind::CreateNote {
            panel: bones(),
            section: None,
        },
    });
    h.store.close_context_menu();
    assert!(ph2d_editor_core::widget::showcase::apply_showcase_event(
        &mut h.store,
        WidgetEvent::Click(ph2d_editor_core::ids::CTX_MENU_CREATE_NOTE)
    ));
    for _ in 0..40 {
        pinta(&mut h, 1);
        vista.push(if h.hit_index.rect_for(barra).is_some() {
            '#'
        } else {
            '.'
        });
    }
    assert!(
        !vista.contains('#'),
        "a barra apareceu numa lista que cabe: {vista}"
    );
}
