//! ⭐ **Os abertos dos blocos 3–5** (`docs/CartoesUI/04_plano_de_construcao.md`), no hero montado: a
//! secção que entra ou sai com a seleção abre-se/fecha-se com mola (T41), o solto que a T42 esconde
//! esmaece, e o cartão Transform do Inspector prende-se ao objecto dele (T36 no Inspector).
#![cfg(feature = "panel-inspector")]

use super::cartoes_mao::{clica, mao, pinta, solta};
use super::o_bloco8_player_solto::{hero_com_aberto, solto_de};
use super::o_inspector_armado::{fisica_armada, player_armado};
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids::{INSP_LIVE_PHYSICS_SECTION as FISICA, INSP_PANEL};
use ph2d_editor_core::interaction::{WidgetEvent, loose_pin_of, loose_rect_of};
use ph2d_editor_core::screens::hero::{HeroScreen, InspectorNameInfo, InspectorTransformInfo};
use ph2d_host::PointerKind;
use ph2d_panel_inspector as insp;

fn altura_da_lista(h: &HeroScreen) -> f32 {
    h.store.panel_rect(INSP_PANEL).expect("a lista publicou").h
}

/// Um objecto com corpo e player: dois cartões de componente além dos do topo.
fn com_componentes(bits: u64) {
    escolhe(bits, "Hero", 1.0);
    insp::set_current_inspector_physics(Some(fisica_armada()));
    insp::set_current_inspector_player(Some(player_armado()));
}

fn sem_componentes() {
    insp::set_current_inspector_physics(None);
    insp::set_current_inspector_player(None);
}

/// T41: escolher outra coisa não arranca os cartões de componente da lista num quadro — eles
/// fecham-se com mola (depois do 1.º quadro a lista está A MEIO caminho, não no fim) e saem; voltar
/// abre-os do mesmo modo. O quadro do teste dura até 0,1 s (o tecto do `dt` da lista), então «um
/// quadro» é meio caminho da mola, não um passo pequeno: o que distingue é não chegar ao fim.
/// *Mutação: `entra_sai::itens` sem `saindo`/`enter` ⇒ a lista chega ao fim no 1.º quadro.*
#[test]
fn a_section_that_leaves_or_enters_with_the_selection_never_jumps() {
    let saindo = || ph2d_editor_core::panel::cards::entra_sai::saindo(INSP_PANEL);
    com_componentes(5);
    let mut h = hero_com_aberto(FISICA);
    pinta(&mut h, 120);
    let cheia = altura_da_lista(&h);
    sem_componentes();
    pinta(&mut h, 1);
    let um = altura_da_lista(&h);
    assert!(
        saindo().contains(&FISICA),
        "o cartão da física não está a sair"
    );
    pinta(&mut h, 200);
    let vazia = altura_da_lista(&h);
    assert!(
        cheia - vazia > m::CARD_HEAD_H * 2.0,
        "não saiu: {cheia} → {vazia}"
    );
    assert!(saindo().is_empty(), "ficou a sair: {:?}", saindo());
    assert!(
        um - vazia > m::CARD_HEAD_H,
        "a saída chegou ao fim num quadro: {cheia} → {um} (fim {vazia})"
    );
    com_componentes(6);
    pinta(&mut h, 1);
    let entra = altura_da_lista(&h);
    pinta(&mut h, 200);
    let aberta = altura_da_lista(&h);
    assert!(
        (aberta - cheia).abs() < 1.0,
        "não abriu inteira: {aberta} × {cheia}"
    );
    assert!(
        aberta - entra > m::CARD_HEAD_H,
        "a entrada chegou ao fim num quadro: {vazia} → {entra} (fim {aberta})"
    );
}

/// T41/T42: o solto não preso que outra classe esconde esmaece (sem clique) e volta a aparecer
/// esmaecido para inteiro.
/// *Mutação: o passo de opacidade = 1 ⇒ some e volta num quadro.*
#[test]
fn a_loose_card_hidden_by_the_selection_fades_out_and_back_in() {
    let alfa = || ph2d_editor_core::panel::cards::solto_some::alfa(FISICA);
    com_componentes(7);
    let mut h = hero_com_aberto(FISICA);
    solto_de(&mut h, FISICA, 280.0);
    pinta(&mut h, 5);
    assert_eq!(alfa(), Some(1.0));
    sem_componentes();
    pinta(&mut h, 1);
    let a = alfa().expect("a foto do solto");
    assert!(a > 0.0 && a < 1.0, "sumiu sem esmaecer: {a}");
    assert_eq!(
        h.store.panel_rect(loose_rect_of(FISICA)),
        None,
        "clicável a esmaecer"
    );
    pinta(&mut h, 120);
    assert_eq!(alfa(), Some(0.0));
    com_componentes(8);
    pinta(&mut h, 1);
    let a = alfa().expect("a foto do solto");
    assert!(a > 0.0 && a < 1.0, "voltou sem esmaecer: {a}");
    pinta(&mut h, 120);
    assert_eq!(alfa(), Some(1.0));
}

pub(super) fn objeto(bits: u64, x: f32) -> InspectorTransformInfo {
    InspectorTransformInfo {
        entity_bits: bits,
        translation: [x, 0.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
        skew_rad: [0.0, 0.0],
    }
}

pub(super) fn escolhe(bits: u64, nome: &str, x: f32) {
    ph2d_panel_inspector::set_current_inspector_transform(Some(objeto(bits, x)));
    ph2d_panel_inspector::set_current_inspector_name(Some(InspectorNameInfo {
        entity_bits: bits,
        name: nome.to_owned(),
    }));
}

/// T36 no Inspector: o cartão Transform solto tem alfinete; preso ao Circle, com o Square escolhido
/// ele mostra o Transform do Circle e o gesto nele edita o Circle.
/// *Mutação: `transform_do_gesto` = a seleção ⇒ o pedido vai ao Square.*
#[test]
fn the_inspector_transform_card_pins_to_its_object_and_edits_it() {
    use ph2d_panel_inspector::CARD_TRANSFORM;
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_panel_inspector::set_cards_on(true);
    ph2d_panel_inspector::set_current_inspector_sprite(None);
    let (circulo, quadrado) = (11_u64, 12_u64);
    escolhe(circulo, "Circle", 1.0);
    let mut h = HeroScreen::new(ph2d_editor_core::NodeId(1));
    let ids: Vec<&'static str> = ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels().iter().map(|p| p.manifest.id).collect()
    });
    for n in ids {
        h.panel_visibility.insert(n, n == "inspector");
    }
    pinta(&mut h, 30);
    solta(&mut h, CARD_TRANSFORM);
    clica(&mut h, loose_pin_of(CARD_TRANSFORM));
    assert_eq!(
        h.store.loose_card(CARD_TRANSFORM).and_then(|c| c.pin),
        Some(circulo)
    );
    escolhe(quadrado, "Square", 9.0);
    let publica = |h: &HeroScreen| {
        let pins = h.store.loose_pins_of(ph2d_editor_core::ids::INSP_PANEL);
        ph2d_panel_inspector::com_cada_preso(&pins, |b| escolhe(b, "Circle", 1.0));
    };
    publica(&h);
    pinta(&mut h, 5);
    let r = h
        .store
        .panel_rect(loose_rect_of(CARD_TRANSFORM))
        .expect("o preso fica no canvas");
    assert_eq!(h.store.card_subject_name(circulo), Some("Circle"));
    let x = ph2d_panel_inspector::ids::INSP_TRANSFORM_POS_X;
    let do_preso = match h
        .store
        .pinned_states(CARD_TRANSFORM)
        .and_then(|e| e.estado(x))
    {
        Some(ph2d_editor_core::interaction::InteractiveState::NumberInput { value, .. }) => {
            Some(*value)
        }
        _ => None,
    };
    let do_quadrado = h.store.number_value(x);
    assert!(
        do_preso.is_some() && do_preso != do_quadrado,
        "{do_preso:?} × {do_quadrado:?}"
    );
    let dentro = (r.x + r.w * 0.5, r.y + r.h - 3.0);
    mao(&mut h, PointerKind::Down, dentro);
    mao(&mut h, PointerKind::Up, dentro);
    assert_eq!(h.store.card_press_subject(), Some(circulo));
    let _ = h.bus.drain().count();
    h.apply_event(WidgetEvent::ValueChanged(x));
    let alvo = h.bus.drain().find_map(|a| match a {
        EditorAction::InspectorTransformEdit(i) => Some(i.entity_bits),
        _ => None,
    });
    assert_eq!(alvo, Some(circulo), "o gesto no preso editou outro objecto");
    publica(&h);
}
