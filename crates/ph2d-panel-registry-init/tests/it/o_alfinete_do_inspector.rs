//! ⭐ **T44/T53 — o alfinete com CORDA no cartão Transform do Inspector**
//! (`docs/CartoesUI/01_decisoes_do_dono.md`), no hero montado: o mesmo ícone cicla preso → preso com
//! corda → solto, e a dica diz sempre o que o próximo clique faz; o cartão arrastado de volta à
//! coluna segue a mão até largar; a corda só acaba quando ele encaixa, e aí recolhe-se até ao objecto.
#![cfg(feature = "panel-inspector")]

use super::cartoes_mao::{clica, mao, pinta, solta};
use super::o_bloco5_abertos::escolhe;
use ph2d_editor_core::ids::{INSP_PANEL, grip_de};
use ph2d_editor_core::interaction::loose_pin_of;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_host::PointerKind;
use ph2d_panel_inspector::CARD_TRANSFORM;
use ph2d_tool_vector::appearance::{self, Appearance, BasePaint, PaintRow};

const OBJETO: u64 = 8;

fn camada(is_fill: bool) -> PaintRow {
    PaintRow {
        is_fill,
        color: [76, 141, 240, 255],
        width: 3.0,
        enabled: true,
        opacity: 1.0,
        blend: ph2d_vec_scene::BlendMode::Normal,
        offset: [0.0, 0.0],
        dilate: 0.0,
        dilate_join: 0,
        mixed: Default::default(),
        ramp: None,
        align: ph2d_vec_scene::StrokeAlign::Centre,
    }
}

/// O hero só com o Inspector, e uma forma com Fill e Stroke: a lista tem cartões acima e abaixo do
/// Transform, para o vão onde ele cai poder andar.
fn hero_com_o_inspector() -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_panel_inspector::set_cards_on(true);
    ph2d_panel_inspector::set_current_inspector_sprite(None);
    escolhe(OBJETO, "Circle", 1.0);
    appearance::set_base_paints(Some([BasePaint::Absent, BasePaint::Absent]));
    appearance::set_current_appearance(Some(Appearance {
        opacity: 1.0,
        blend: ph2d_vec_scene::BlendMode::Normal,
        layers: vec![camada(true), camada(false)],
        mixed_opacity: false,
        mixed_blend: false,
        layers_differ: false,
        order: Vec::new(),
    }));
    let mut h = HeroScreen::new(ph2d_editor_core::NodeId(1));
    let ids: Vec<&'static str> = ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels().iter().map(|p| p.manifest.id).collect()
    });
    for n in ids {
        h.panel_visibility.insert(n, n == "inspector");
    }
    pinta(&mut h, 30);
    h
}

/// O que a shell faz a cada quadro: publica o retrato de cada objecto preso.
fn publica(h: &HeroScreen) {
    let pins = h.store.loose_pins_of(INSP_PANEL);
    ph2d_panel_inspector::com_cada_preso(&pins, |b| escolhe(b, "Circle", 1.0));
}

/// Pinta `n` quadros andando a corda e publicando os presos entre eles, como a shell.
fn anda(h: &mut HeroScreen, n: usize) {
    for _ in 0..n {
        publica(h);
        ph2d_card_rope::anda_com(
            &mut h.store,
            INSP_PANEL,
            |_| Some((800.0, 600.0)),
            1.0 / 60.0,
        );
        pinta(h, 1);
    }
}

/// *Mutação: o alfinete de dois estados ⇒ o 2.º clique solta em vez de pôr a corda.*
#[test]
fn the_pin_cycles_pinned_then_rope_then_loose() {
    let mut h = hero_com_o_inspector();
    solta(&mut h, CARD_TRANSFORM);
    let estado = |h: &HeroScreen| {
        let c = h.store.loose_card(CARD_TRANSFORM).expect("solto");
        let dica = h
            .store
            .tooltip_for(loose_pin_of(CARD_TRANSFORM))
            .map(str::to_owned);
        (c.pin, c.rope, dica)
    };
    let dica = |k: &str| Some(ph2d_i18n::tr(k).to_owned());
    assert_eq!(estado(&h), (None, false, dica("card.loose.pin")));
    for (pin, rope, proxima) in [
        (Some(OBJETO), false, "card.loose.rope"),
        (Some(OBJETO), true, "card.loose.unpin"),
        (None, false, "card.loose.pin"),
    ] {
        clica(&mut h, loose_pin_of(CARD_TRANSFORM));
        anda(&mut h, 2);
        assert_eq!(estado(&h), (pin, rope, dica(proxima)));
        let com_corda = h.store.loose_ropes_of(INSP_PANEL);
        assert_eq!(com_corda.is_empty(), !rope, "{com_corda:?}");
    }
}

/// Arrasta o Transform solto (com `cliques` no alfinete) de volta à lista e continua a andar com a
/// mão DENTRO dela: devolve, por passo, a distância entre a mão e o voo da lista, e no fim (depois
/// de largar) se o cartão assentou no vão onde a mão o largou.
fn volta_com_a_mao(cliques: usize) -> (Vec<Option<f32>>, bool) {
    let mut h = hero_com_o_inspector();
    let grip = grip_de(CARD_TRANSFORM);
    let lista = h.store.panel_rect(INSP_PANEL).expect("a lista");
    solta(&mut h, CARD_TRANSFORM);
    for _ in 0..cliques {
        clica(&mut h, loose_pin_of(CARD_TRANSFORM));
        anda(&mut h, 2);
    }
    let pega = h.hit_index.rect_for(grip).expect("a pega do solto");
    let de = (pega.x + 12.0, pega.y + pega.h * 0.5);
    let x = lista.x + lista.w * 0.5;
    let y0 = lista.y + 60.0;
    mao(&mut h, PointerKind::Down, de);
    anda(&mut h, 2);
    for i in 1..=6 {
        let t = i as f32 / 6.0;
        mao(
            &mut h,
            PointerKind::Move,
            (de.0 + (x - de.0) * t, de.1 + (y0 - de.1) * t),
        );
        anda(&mut h, 2);
    }
    assert_eq!(h.store.loose_card(CARD_TRANSFORM), None, "não reentrou");
    let mut desvios = Vec::new();
    let mut fim = (x, y0);
    for i in 1..=5 {
        fim = (x, y0 + 30.0 * i as f32);
        mao(&mut h, PointerKind::Move, fim);
        anda(&mut h, 2);
        let f = h.store.card_list_mut(INSP_PANEL).stack.flight().copied();
        desvios.push(f.map(|f| (f.at.0 - fim.0).abs() + (f.at.1 - fim.1).abs()));
    }
    let vao = h
        .store
        .card_list_mut(INSP_PANEL)
        .stack
        .flight()
        .and_then(|f| f.to);
    mao(&mut h, PointerKind::Up, fim);
    anda(&mut h, 40);
    let lugar = h
        .store
        .card_list_mut(INSP_PANEL)
        .items
        .iter()
        .position(|it| it.key == CARD_TRANSFORM);
    let assentou = h.store.loose_card(CARD_TRANSFORM).is_none() && vao.is_some() && vao == lugar;
    (desvios, assentou)
}

/// ⭐ T44d: de volta à lista o cartão segue a mão ATÉ largar — com corda, preso ou solto.
/// *Mutação: o arrasto continua a marcar-se «solto» depois de reentrar ⇒ o voo congela onde reentrou.*
#[test]
fn a_card_dragged_back_keeps_following_the_hand_in_every_pin_state() {
    let falhas: Vec<_> = [(2, "com corda"), (1, "preso"), (0, "solto")]
        .into_iter()
        .map(|(cliques, nome)| (nome, volta_com_a_mao(cliques)))
        .filter(|(_, (desvios, assentou))| {
            !assentou || !desvios.iter().all(|d| d.is_some_and(|d| d < 0.5))
        })
        .collect();
    assert!(
        falhas.is_empty(),
        "o voo não segue a mão ou não assenta: {falhas:?}"
    );
}

/// O Transform preso COM corda levado pela mão até à coluna da lista, SEM largar. Devolve o hero,
/// onde a mão está e o alfinete de antes.
fn com_corda_ate_a_lista() -> (HeroScreen, (f32, f32), Option<u64>) {
    let mut h = hero_com_o_inspector();
    let lista = h.store.panel_rect(INSP_PANEL).expect("a lista");
    solta(&mut h, CARD_TRANSFORM);
    for _ in 0..2 {
        clica(&mut h, loose_pin_of(CARD_TRANSFORM));
        anda(&mut h, 2);
    }
    let pin = h.store.loose_card(CARD_TRANSFORM).and_then(|c| c.pin);
    anda(&mut h, 10);
    let pega = h
        .hit_index
        .rect_for(grip_de(CARD_TRANSFORM))
        .expect("a pega do solto");
    let de = (pega.x + 12.0, pega.y + pega.h * 0.5);
    let dentro = (lista.x + lista.w * 0.5, lista.y + 60.0);
    mao(&mut h, PointerKind::Down, de);
    anda(&mut h, 2);
    for i in 1..=6 {
        let t = i as f32 / 6.0;
        let p = (de.0 + (dentro.0 - de.0) * t, de.1 + (dentro.1 - de.1) * t);
        mao(&mut h, PointerKind::Move, p);
        anda(&mut h, 2);
    }
    assert_eq!(h.store.loose_card(CARD_TRANSFORM), None, "não reentrou");
    (h, dentro, pin)
}

/// ⭐ T53: «a corda só deve romper se o card encaixar no inspector no mouse up» — dentro da coluna,
/// sem largar, a corda continua; de volta ao canvas e largado, o cartão é solto de novo com o MESMO
/// alfinete e a corda. *Mutação: devolver esquece alfinete e corda.*
#[test]
fn a_rope_survives_the_column_until_the_card_docks() {
    let (mut h, dentro, pin) = com_corda_ate_a_lista();
    assert_eq!(pin, Some(OBJETO), "não prendeu");
    let viva = h
        .store
        .loose_ropes_of(INSP_PANEL)
        .iter()
        .any(|(k, _)| *k == CARD_TRANSFORM);
    assert!(
        viva && !h.store.loose_rope(CARD_TRANSFORM).is_empty(),
        "a corda rompeu na coluna"
    );
    let canvas = h.last_layout.expect("houve um quadro").canvas;
    let fora = (canvas.x + canvas.w * 0.3, canvas.y + canvas.h * 0.4);
    for i in 1..=6 {
        let t = i as f32 / 6.0;
        let p = (
            dentro.0 + (fora.0 - dentro.0) * t,
            dentro.1 + (fora.1 - dentro.1) * t,
        );
        mao(&mut h, PointerKind::Move, p);
        anda(&mut h, 2);
    }
    mao(&mut h, PointerKind::Up, fora);
    anda(&mut h, 10);
    let c = h.store.loose_card(CARD_TRANSFORM).expect("solto de novo");
    assert_eq!(
        (c.pin, c.rope),
        (pin, true),
        "o alfinete ou a corda não voltaram"
    );
    assert!(
        !h.store.loose_rope(CARD_TRANSFORM).is_empty(),
        "sem corda depois de largar"
    );
    assert!(
        h.store.reeling_ropes_of(INSP_PANEL).is_empty(),
        "a corda recolheu-se"
    );
}

/// T53: largado DENTRO da coluna, encaixa — o suspenso acaba e a corda recolhe-se até ao objecto.
/// *Mutação: encaixar não entrega a corda à recolha ⇒ ela some num quadro.*
#[test]
fn a_rope_reels_in_when_the_card_docks_on_mouse_up() {
    let (mut h, dentro, _) = com_corda_ate_a_lista();
    mao(&mut h, PointerKind::Up, dentro);
    anda(&mut h, 3);
    assert_eq!(h.store.loose_card(CARD_TRANSFORM), None, "não encaixou");
    assert_eq!(h.store.suspended_card(), None, "o suspenso ficou");
    assert_eq!(h.store.loose_pin_key(CARD_TRANSFORM), None);
    assert!(
        !h.store.reeling_ropes_of(INSP_PANEL).is_empty(),
        "a corda sumiu sem recolher"
    );
    anda(&mut h, 40);
    assert!(
        h.store.reeling_ropes_of(INSP_PANEL).is_empty(),
        "não acabou de recolher"
    );
}
