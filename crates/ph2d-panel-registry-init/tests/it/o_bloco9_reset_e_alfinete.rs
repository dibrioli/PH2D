//! ⭐ **CartoesUI T60–T61 (smoke do dono 02/11: «nem o botão reset», «porque o pin não aparece nos
//! vários cards novos?»)** — no hero montado: todo cartão de componente tem o ↺ no cabeçalho, e o
//! clique pede à app que reponha o componente principal dele no objecto do cartão.
#![cfg(feature = "panel-inspector")]

use super::cartoes_mao::{InspectorPrivado, centro, clica, mao, pinta};
use super::o_bloco8_player_solto::{hero_com_aberto, solto_de};
use super::o_inspector_armado::{arma_tudo, fisica_armada, player_armado};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, loose_pin_of, loose_rect_of};
use ph2d_editor_core::screens::hero::{HeroScreen, InspectorNameInfo};
use ph2d_host::PointerKind;
use ph2d_panel_inspector as insp;
use ph2d_panel_inspector::card_reset_of;

const PLAYER: NodeId = ids::INSP_LIVE_PLAYER_SECTION;

/// T60: cada cartão de componente pintado tem o ↺ (com a dica), por cima do título; o do Player
/// pede a reposição do `PlatformPlayer` do objecto armado.
#[test]
fn every_component_card_has_a_reset_that_asks_for_its_component() {
    super::o_inspector_armado::arma_tudo();
    let mut h = hero_com_aberto(PLAYER);
    let pintados: Vec<NodeId> = ids::LIVE_SECTION_IDS
        .into_iter()
        .filter(|s| h.hit_index.rect_for(ids::grip_de(*s)).is_some() && h.store.is_card_section(*s))
        .collect();
    assert!(
        pintados.contains(&PLAYER),
        "o cartão do Player não se pintou"
    );
    for s in &pintados {
        let r = h
            .hit_index
            .rect_for(card_reset_of(*s))
            .unwrap_or_else(|| panic!("o cartão {s:?} não tem o ↺ no cabeçalho"));
        let c = centro(r);
        assert_eq!(
            h.hit_index.hit(c.0, c.1),
            Some(card_reset_of(*s)),
            "o título cobre o ↺ de {s:?}"
        );
        assert_eq!(
            h.store.tooltip_for(card_reset_of(*s)),
            Some("Reset to defaults")
        );
    }
    let _ = h.store.take_component_resets();
    let float = ph2d_panel_inspector::ids::INSP_PLAYER_FLOAT;
    let de_fabrica = h.store.number_value(float).expect("a caixa Float Height");
    h.store.set_number_value(float, de_fabrica + 70.0); // o artista mudou-a
    clica(&mut h, card_reset_of(PLAYER));
    let bits = super::o_inspector_armado::fisica_armada().entity_bits;
    assert_eq!(
        h.store.take_component_resets(),
        vec![(bits, "ph2d::physics::PlatformPlayer")],
        "o ↺ do Player não pediu a reposição do Player do objecto"
    );
    // A shell repôs (o pedido foi atendido acima); a ficha que chega é a de fábrica e a caixa
    // re-semeia-se, embora o objecto seja o mesmo (as caixas do Player só se semeavam ao trocar).
    super::cartoes_mao::pinta(&mut h, 3);
    assert_eq!(
        h.store.number_value(float),
        Some(de_fabrica),
        "depois do ↺ a caixa Float Height continua com o número antigo"
    );
    super::o_inspector_armado::desarma_tudo();
}

const FLOAT: NodeId = insp::ids::INSP_PLAYER_FLOAT;

/// O objecto `bits` escolhido: o mesmo arnês do [`arma_tudo`], com outro nome e outro Float Height.
fn escolhe(bits: u64, nome: &str, float: f32) {
    arma_tudo();
    insp::set_current_inspector_sprite(None); // com sprite o Inspector não é de cartões
    insp::set_current_inspector_transform(Some(super::o_bloco5_abertos::objeto(bits, 2.0)));
    insp::set_current_inspector_name(Some(InspectorNameInfo {
        entity_bits: bits,
        name: nome.to_owned(),
    }));
    insp::set_current_inspector_physics(Some(ph2d_editor_core::InspectorPhysicsInfo {
        entity_bits: bits,
        ..fisica_armada()
    }));
    insp::set_current_inspector_player(Some(ph2d_editor_core::InspectorPlayerInfo {
        entity_bits: bits,
        float_height: float,
        ..player_armado()
    }));
}

fn caixa(h: &HeroScreen) -> f64 {
    h.store.number_value(FLOAT).expect("a caixa Float Height")
}

/// T61 (dono 02/11: «porque o pin não aparece nos vários cards novos?»): o Player solto preso ao
/// Hero, com o Enemy escolhido, mostra o Float Height do Hero (a lista, o do Enemy) e o gesto nele
/// parte do número do Hero e edita o Hero.
/// *Mutação: o ensaio desligado ⇒ o preso não tem número próprio; `com_retrato` com a seleção ⇒ o
/// pedido vai ao Enemy.*
#[test]
fn a_pinned_component_card_shows_and_edits_its_own_object() {
    let (hero, inimigo) = (super::o_inspector_armado::BITS, 77_u64);
    escolhe(hero, "Hero", 0.1);
    let mut h = hero_com_aberto(PLAYER);
    let do_hero = caixa(&h);
    solto_de(&mut h, PLAYER, 300.0);
    // Abaixo da barra de modos do canvas (ela fica por cima do cabeçalho de um solto colado ao topo).
    if let Some(c) = h.store.loose_card_mut(PLAYER) {
        c.at.1 += 150.0;
    }
    pinta(&mut h, 3);
    clica(&mut h, loose_pin_of(PLAYER));
    assert_eq!(h.store.loose_card(PLAYER).and_then(|c| c.pin), Some(hero));
    let quadros = |h: &mut HeroScreen, n: usize| {
        for _ in 0..n {
            insp::com_cada_preso(&h.store.loose_pins_of(ids::INSP_PANEL), |_| {
                escolhe(hero, "Hero", 0.1);
            });
            pinta(h, 1);
        }
    };
    escolhe(inimigo, "Enemy", 0.7);
    quadros(&mut h, 5);
    let r = h
        .store
        .panel_rect(loose_rect_of(PLAYER))
        .expect("o Player preso fica no canvas");
    assert_eq!(h.store.card_subject_name(hero), Some("Hero"));
    let do_inimigo = caixa(&h);
    assert!(
        (do_inimigo - do_hero).abs() > 1e-6,
        "a lista não mostra o Enemy"
    );
    let preso = match h.store.pinned_states(PLAYER).and_then(|e| e.estado(FLOAT)) {
        Some(InteractiveState::NumberInput { value, .. }) => Some(*value),
        _ => None,
    };
    assert_eq!(
        preso,
        Some(do_hero),
        "o cartão preso não mostra o Float do Hero"
    );
    let campo = h
        .hit_index
        .rect_for(FLOAT)
        .expect("o Float no cartão preso");
    let dentro = centro(campo);
    assert!(
        r.contains(dentro.0, dentro.1),
        "o Float pintado não é o do preso"
    );
    mao(&mut h, PointerKind::Down, dentro);
    assert_eq!(h.store.card_press_subject(), Some(hero));
    assert_eq!(
        caixa(&h),
        do_hero,
        "o gesto no preso não parte do número do Hero"
    );
    mao(&mut h, PointerKind::Up, dentro);
    let _ = h.bus.drain().count();
    h.apply_event(WidgetEvent::ValueChanged(FLOAT));
    let alvo = h.bus.drain().find_map(|a| match a {
        EditorAction::InspectorPlayerEdit { entity_bits, .. } => Some(entity_bits),
        _ => None,
    });
    assert_eq!(
        alvo,
        Some(hero),
        "o gesto no Player preso editou outro objecto"
    );
    super::o_inspector_armado::desarma_tudo();
}

/// Bloco 10, o ABERTO 2 do bloco 9: um Ctrl+Z (o mundo muda, o objecto é o mesmo) re-semeia as
/// caixas dos componentes — elas só se semeavam ao trocar de objecto; a caixa em foco fica.
/// Num registo privado: no do processo, um vizinho que pinte outro objecto larga o foco.
/// *Mutação: `semeia = entity_changed` ⇒ a caixa fica com o número antigo.*
#[test]
fn an_undo_on_a_component_field_reseeds_its_box() {
    use ph2d_editor_core::panel::PanelHostInternal;
    let hero = super::o_inspector_armado::BITS;
    let caixa = |p: &InspectorPrivado| {
        p.host
            .store()
            .number_value(FLOAT)
            .expect("a caixa Float Height")
    };
    escolhe(hero, "Hero", 0.1);
    let mut p = InspectorPrivado::novo();
    p.pinta();
    let antes = caixa(&p);
    escolhe(hero, "Hero", 0.4); // o Ctrl+Z devolveu outro Float ao mesmo objecto
    p.pinta();
    let depois = caixa(&p);
    assert!(
        (depois - antes).abs() > 1e-6,
        "depois do Ctrl+Z a caixa ficou com {antes}"
    );
    p.host.store_mut().set_focus(Some(FLOAT));
    escolhe(hero, "Hero", 0.9);
    p.pinta();
    assert_eq!(
        p.host.store().focus_id(),
        Some(FLOAT),
        "o mesmo objecto largou o foco"
    );
    assert_eq!(
        caixa(&p),
        depois,
        "a caixa em foco foi reescrita por baixo da mão"
    );
    p.host.store_mut().set_focus(None);
    p.pinta();
    assert!(
        (caixa(&p) - depois).abs() > 1e-6,
        "sem foco a caixa não voltou ao mundo"
    );
    super::o_inspector_armado::desarma_tudo();
}
