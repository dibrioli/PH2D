//! ⭐ **Todo cartão do Inspector que mostra dados de UM objecto tem alfinete** (dono 05/11, «vários cards
//! ainda sem pin»): o censo solta cada cartão da lista, por gesto real, com o objecto que tem todas as
//! secções, e exige o alfinete, salvo [`SEM_ALFINETE`]. A lista é derivada da pintura, nunca escrita à mão.
#![cfg(all(feature = "panel-inspector", feature = "panel-hierarchy"))]

use super::cartoes_mao::{clica, pinta, solta};
use super::o_alfinete_da_sprite::{A, B, no_solto, preso, quadros};
use super::o_inspector_armado::{BITS, arma_tudo, desarma_tudo};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::{loose_pin_of, loose_return_of};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_panel_inspector as insp;

/// O que fica sem alfinete, e porquê.
const SEM_ALFINETE: &[(NodeId, &str)] = &[(insp::CARD_EMPTY, "no object selected: nothing to pin")];

fn hero() -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    insp::set_cards_on(true);
    let mut h = HeroScreen::new(NodeId(1));
    let ids: Vec<&'static str> = ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels().iter().map(|p| p.manifest.id).collect()
    });
    for n in ids {
        h.panel_visibility.insert(n, n == "inspector");
    }
    h
}

fn quadro(h: &mut HeroScreen, n: usize) {
    for _ in 0..n {
        insp::com_cada_preso(&h.store.loose_pins_of(core_ids::INSP_PANEL), |_| {
            arma_tudo()
        });
        arma_tudo();
        pinta(h, 1);
    }
}

fn cartoes(h: &mut HeroScreen) -> Vec<NodeId> {
    h.store
        .card_list_mut(core_ids::INSP_PANEL)
        .items
        .iter()
        .map(|i| i.key)
        .collect()
}

/// Rola o painel até a pega de `sec` estar dentro dele: a mão só apanha o que está à vista.
fn traz_a_vista(h: &mut HeroScreen, sec: NodeId) {
    let p = core_ids::INSP_PANEL;
    let painel = h.store.panel_rect(p).expect("o Inspector pintou");
    let total = h.store.panel_content_h(p).unwrap_or(0.0);
    let mut alvo = 0.0;
    loop {
        h.store.set_panel_scroll(p, alvo);
        h.store.set_panel_scroll_live(p, alvo);
        quadro(h, 2);
        let grip = h.hit_index.rect_for(core_ids::grip_de(sec));
        if grip.is_some_and(|g| g.h > 0.0 && g.y >= painel.y && g.y + g.h <= painel.y + painel.h) {
            return;
        }
        alvo += painel.h * 0.5;
        assert!(alvo <= total + painel.h, "{sec:?} não aparece ao rolar");
    }
}

/// ⭐ O censo. *Mutações: tirar `CARD_IMAGE` de `DO_OBJECTO` ⇒ o Image cai sem alfinete · tirar `CARD_EMPTY` de [`SEM_ALFINETE`] ⇒ a parte vazia reprova.*
#[test]
fn todo_cartao_de_um_objecto_tem_alfinete_salvo_a_lista() {
    let mut h = hero();
    desarma_tudo();
    pinta(&mut h, 5);
    for sec in cartoes(&mut h) {
        assert!(
            SEM_ALFINETE.iter().any(|(s, _)| *s == sec),
            "{sec:?} aparece sem objecto e não está na lista"
        );
        assert!(
            !h.store.is_card_pinnable(sec),
            "{sec:?} é da lista e tem alfinete"
        );
    }
    assert!(
        !cartoes(&mut h).is_empty(),
        "o estado vazio não pintou: a lista não prova nada"
    );

    arma_tudo();
    h.gizmo.replace_selection(Some(BITS));
    h.store
        .set_card_subject(core_ids::INSP_PANEL, Some((BITS, "Everything".to_owned())));
    quadro(&mut h, 30);
    let todos = cartoes(&mut h);
    assert!(
        todos.len() >= 30,
        "só {} cartões: o censo está cego",
        todos.len()
    );
    for s in &todos {
        h.store.set_collapsed(*s, true);
    }
    quadro(&mut h, 30);
    let mut sem = Vec::new();
    for sec in &todos {
        traz_a_vista(&mut h, *sec);
        solta(&mut h, *sec);
        quadro(&mut h, 2);
        if !h.store.is_card_pinnable(*sec) || h.hit_index.rect_for(loose_pin_of(*sec)).is_none() {
            sem.push(*sec);
        }
        clica(&mut h, loose_return_of(*sec));
        quadro(&mut h, 10);
    }
    assert!(
        sem.is_empty(),
        "{} cartões sem alfinete: {sem:?}",
        sem.len()
    );
    desarma_tudo();
}

/// ⭐ O cartão Image preso a A, com B selecionado: a imagem é a de A, e o Edit in Painter pede A.
/// *Mutações: tirar `slot` do `Retrato` · o `com_retrato` do gesto.*
#[test]
fn o_cartao_image_preso_mostra_a_imagem_dele_e_pede_para_ele() {
    let sec = insp::CARD_IMAGE;
    let mut h = preso(sec, &[B]);
    // O que veio antes da troca de seleção não conta.
    let _ = insp::imagens_pintadas();
    quadros(&mut h, &[B], 2);
    let pintadas = insp::imagens_pintadas();
    assert!(
        pintadas.iter().any(|(b, n)| *b == A && n == "Hero sheet"),
        "{pintadas:?}"
    );
    assert!(
        !pintadas.iter().any(|(b, n)| *b == A && n != "Hero sheet"),
        "{pintadas:?}"
    );
    let feitos = no_solto(&mut h, sec, insp::ids::INSP_IMAGE_EDIT_PAINTER, &[B]);
    assert!(
        feitos.iter().any(|a| matches!(
            a,
            EditorAction::ImageUseEdit {
                entity_bits: A,
                edit: ph2d_editor_core::ImageUseEdit::EditInPainter
            }
        )),
        "{feitos:?}"
    );
    assert!(
        !feitos.iter().any(|a| matches!(
            a,
            EditorAction::ImageUseEdit { entity_bits, .. } if *entity_bits != A
        )),
        "{feitos:?}"
    );
}

/// O Pixel Sprite A (sem `Sprite`, como a shell o publica) ou a imagem B.
fn arma_pixel(b: u64) {
    super::o_alfinete_da_sprite::arma(b);
    if b == A {
        insp::set_current_inspector_sprite(None);
        insp::slot_image::set_current_slot_image(None);
    }
}

/// ⭐ O cartão do Pixel Sprite preso a A, com B selecionado: o facto é por objecto, e o verbo entra no Draw de A.
/// *Mutação: o `Enter` no lugar do `OpenContent` do preso ⇒ o Draw caía na seleção.*
#[test]
fn o_cartao_do_pixel_sprite_preso_e_o_dele_e_pede_o_draw_dele() {
    let sec = insp::CARD_IMAGE;
    let mut h = hero();
    h.store
        .set_pixel_sprite_cards(vec![ph2d_editor_core::pixel_panel::PixelSpriteCard {
            entity_bits: A,
            size: [96, 64],
            rgba: std::sync::Arc::new(vec![255; 16]),
            thumb: [2, 2],
            doc: 1,
            pivots: Vec::new(),
            pivot: None,
            luz_na_paleta: false,
        }]);
    h.gizmo.replace_selection(Some(A));
    h.store
        .set_card_subject(core_ids::INSP_PANEL, Some((A, "Hero".to_owned())));
    let quadros_pixel = |h: &mut HeroScreen, sel: u64, n: usize| {
        for _ in 0..n {
            insp::com_cada_preso(&h.store.loose_pins_of(core_ids::INSP_PANEL), arma_pixel);
            arma_pixel(sel);
            insp::set_current_inspector_selecionados(1);
            pinta(h, 1);
        }
    };
    quadros_pixel(&mut h, A, 30);
    solta(&mut h, sec);
    clica(&mut h, loose_pin_of(sec));
    assert_eq!(h.store.loose_card(sec).and_then(|c| c.pin), Some(A));
    h.gizmo.replace_selection(Some(B));
    quadros_pixel(&mut h, B, 6);
    let _ = insp::imagens_pintadas();
    quadros_pixel(&mut h, B, 2);
    let pintadas = insp::imagens_pintadas();
    assert!(
        pintadas.iter().any(|(b, n)| *b == A && n == "Hero"),
        "{pintadas:?}"
    );
    let feitos = no_solto(&mut h, sec, insp::ids::INSP_IMAGE_EDIT_PIXEL, &[B]);
    assert!(
        feitos.iter().any(|a| matches!(
            a,
            EditorAction::ObjectMode(ph2d_editor_core::object_mode::ModeRequest::OpenContent {
                entity: A,
                mode: ph2d_editor_core::object_mode::ObjectMode::Draw,
                ..
            })
        )),
        "{feitos:?}"
    );
}
