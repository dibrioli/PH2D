//! ⭐ **O renomear guarda de quem é** ([ficha](../../../../docs/GameEngineFuncional/componentes/02_animated_sprite.md)):
//! dois cliques na linha de um cartão Animation preso, digitar, largar o foco pela mão. O nome vai ao objecto do
//! cartão, com a seleção noutro objecto, com ela sem tocador, e com a pressão a cair no próprio cartão.
#![cfg(all(feature = "panel-inspector", feature = "panel-hierarchy"))]

use super::cartoes_mao::{clica, mao, pinta};
use super::o_bloco5_abertos::escolhe;
use super::o_bloco8_player_solto::{hero_com_aberto, solto_de};
use super::o_inspector_armado::{arma_tudo, fisica_armada};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids::{self as i, INSP_PANEL};
use ph2d_editor_core::interaction::{dispatch_text_input, loose_pin_of};
use ph2d_editor_core::screens::hero::{AnimFieldEdit, HeroScreen};
use ph2d_host::PointerKind;
use ph2d_panel_inspector as insp;
use ph2d_panel_inspector::ids as pids;

const OUTRO: u64 = 0xB;

/// O tocador da fixtura, em nome de `bits`, com a linha 0 chamada `nome`.
fn tocador(bits: u64, nome: &str) -> ph2d_editor_core::screens::hero::InspectorAnimInfo {
    arma_tudo();
    let mut a = insp::current_inspector_anim().expect("a fixtura arma o tocador");
    a.entity_bits = bits;
    a.player_present = true;
    a.rows[0].name = nome.to_owned();
    a
}

/// A seleção é `OUTRO`: com um tocador próprio, ou sem nenhum.
fn quadros(h: &mut HeroScreen, com_tocador: bool, n: usize) {
    let a = fisica_armada().entity_bits;
    for _ in 0..n {
        let pins = h.store.loose_pins_of(INSP_PANEL);
        insp::com_cada_preso(&pins, |_| {
            let t = tocador(a, "idle");
            insp::set_current_inspector_anim(Some(t));
        });
        let sel = tocador(OUTRO, "walk");
        insp::set_current_inspector_anim(com_tocador.then_some(sel));
        escolhe(OUTRO, "Other", 1.0);
        h.store
            .set_card_subject(INSP_PANEL, Some((OUTRO, "Other".to_owned())));
        pinta(h, 1);
    }
}

fn preso(com_tocador: bool) -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let sec = i::INSP_LIVE_ANIM_SECTION;
    let a = fisica_armada().entity_bits;
    insp::set_current_inspector_anim(Some(tocador(a, "idle")));
    let mut h = hero_com_aberto(sec);
    solto_de(&mut h, sec, 300.0);
    clica(&mut h, loose_pin_of(sec));
    assert_eq!(h.store.loose_card(sec).and_then(|c| c.pin), Some(a));
    quadros(&mut h, com_tocador, 4);
    h
}

fn digita_o_nome(h: &mut HeroScreen, novo: &str) {
    clica(h, pids::INSP_ANIM_ROW[0]);
    h.bus.drain().for_each(drop);
    clica(h, pids::INSP_ANIM_ROW[0]);
    assert_eq!(
        h.store.focus_id(),
        Some(pids::INSP_ANIM_NAME),
        "abriu o campo"
    );
    let arena = bumpalo::Bump::new();
    for ch in novo.chars() {
        let evs = dispatch_text_input(&mut h.store, ch, &arena).to_vec();
        for e in evs {
            h.apply_event(e);
        }
    }
}

fn fora(h: &mut HeroScreen) {
    let c = h.last_layout.expect("houve um quadro").canvas;
    let p = (c.x + c.w * 0.9, c.y + c.h * 0.9);
    mao(h, PointerKind::Down, p);
    mao(h, PointerKind::Up, p);
}

fn renomeios(h: &mut HeroScreen) -> Vec<(u64, String)> {
    h.bus
        .drain()
        .filter_map(|a| match a {
            EditorAction::InspectorAnimEdit {
                entity_bits,
                edit: AnimFieldEdit::Rename(0, n),
            } => Some((entity_bits, n)),
            _ => None,
        })
        .collect()
}

/// Com a seleção noutro objecto, com ou sem tocador: clicar fora grava no objecto do cartão, uma vez, e o estado morre.
/// *Mutação: gravar com o `entity_bits` da seleção ⇒ o nome vai a `OUTRO`, ou a lugar nenhum.*
#[test]
fn clicar_fora_grava_no_objecto_do_cartao_preso() {
    let a = fisica_armada().entity_bits;
    for com_tocador in [true, false] {
        let mut h = preso(com_tocador);
        digita_o_nome(&mut h, "ataque");
        fora(&mut h);
        quadros(&mut h, com_tocador, 2);
        assert_eq!(
            renomeios(&mut h),
            [(a, "idleataque".to_owned())],
            "seleção com tocador: {com_tocador}"
        );
        // O próximo objecto animado não herda nada.
        quadros(&mut h, true, 2);
        assert!(renomeios(&mut h).is_empty(), "com_tocador {com_tocador}");
    }
}

/// A pressão que tira o foco cai no próprio cartão preso: o que se digitou não volta ao nome de antes.
/// *Mutação: `poe_estados_presos` sem poupar o campo em foco ⇒ o renomear perde-se.*
#[test]
fn a_pressao_no_proprio_cartao_nao_apaga_o_que_se_digitou() {
    let a = fisica_armada().entity_bits;
    let mut h = preso(true);
    digita_o_nome(&mut h, "ataque");
    let r = h.hit_index.rect_for(pids::INSP_ANIM_SPEED).expect("Speed");
    let p = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    mao(&mut h, PointerKind::Down, p);
    mao(&mut h, PointerKind::Up, p);
    quadros(&mut h, true, 2);
    assert_eq!(renomeios(&mut h), [(a, "idleataque".to_owned())]);
}
