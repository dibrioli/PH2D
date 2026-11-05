//! ⭐ **T44 — o alfinete com CORDA** (`docs/CartoesUI/01_decisoes_do_dono.md`), no hero montado: o
//! mesmo ícone cicla preso → preso com corda → solto; a dica diz sempre o que o próximo clique faz.
#![cfg(feature = "panel-skeleton")]

use super::o_cartao_solto_no_canvas::{bones, clica, hero_com_o_bones, osso_em_foco, pinta, solta};
use ph2d_editor_core::interaction::loose_pin_of;
use ph2d_panel_skeleton::cards::{
    CARD_BONE, CARD_IK, CARD_LIMIT, CARD_SKIN, CARD_SMART, CARD_TOOL, CARD_WEIGHT,
};

/// *Mutação: o alfinete de dois estados ⇒ o 2.º clique solta em vez de pôr a corda.*
#[test]
fn the_pin_cycles_pinned_then_rope_then_loose() {
    let mut h = hero_com_o_bones();
    osso_em_foco(8, "Bone 8", 0.5);
    pinta(&mut h, 20);
    solta(&mut h, CARD_IK);
    let estado = |h: &ph2d_editor_core::screens::hero::HeroScreen| {
        let c = h.store.loose_card(CARD_IK).expect("solto");
        let dica = h
            .store
            .tooltip_for(loose_pin_of(CARD_IK))
            .map(str::to_owned);
        (c.pin, c.rope, dica)
    };
    let dica = |k: &str| Some(ph2d_i18n::tr(k).to_owned());
    assert_eq!(estado(&h), (None, false, dica("card.loose.pin")));
    for (pin, rope, proxima) in [
        (Some(8), false, "card.loose.rope"),
        (Some(8), true, "card.loose.unpin"),
        (None, false, "card.loose.pin"),
    ] {
        clica(&mut h, loose_pin_of(CARD_IK));
        pinta(&mut h, 2);
        assert_eq!(estado(&h), (pin, rope, dica(proxima)));
        let com_corda = h.store.loose_ropes_of(bones());
        assert_eq!(com_corda.is_empty(), !rope, "{com_corda:?}");
    }
}

/// Arrasta o IK solto (com `cliques` no alfinete) de volta à lista e continua a andar com a mão
/// DENTRO dela: devolve, por passo, a distância entre a mão e o voo da lista, e no fim (depois de
/// largar) se a pega assentou à altura onde a mão largou.
fn volta_com_a_mao(cliques: usize) -> (Vec<Option<f32>>, bool) {
    use super::o_cartao_solto_no_canvas::mao;
    use ph2d_host::PointerKind;
    let mut h = hero_com_o_bones();
    osso_em_foco(8, "Bone 8", 0.5);
    pinta(&mut h, 20);
    let grip = ph2d_editor_core::ids::grip_de(CARD_IK);
    let lista = h.store.panel_rect(bones()).expect("a lista");
    solta(&mut h, CARD_IK);
    for _ in 0..cliques {
        clica(&mut h, loose_pin_of(CARD_IK));
        pinta(&mut h, 2);
    }
    let pega = h.hit_index.rect_for(grip).expect("a pega do solto");
    let de = (pega.x + 12.0, pega.y + pega.h * 0.5);
    let x = lista.x + lista.w * 0.5;
    let y0 = lista.y + 60.0;
    mao(&mut h, PointerKind::Down, de);
    pinta(&mut h, 2);
    for i in 1..=6 {
        let t = i as f32 / 6.0;
        mao(
            &mut h,
            PointerKind::Move,
            (de.0 + (x - de.0) * t, de.1 + (y0 - de.1) * t),
        );
        pinta(&mut h, 2);
    }
    assert_eq!(h.store.loose_card(CARD_IK), None, "não reentrou");
    let mut desvios = Vec::new();
    let mut fim = (x, y0);
    for i in 1..=5 {
        fim = (x, y0 + 30.0 * i as f32);
        mao(&mut h, PointerKind::Move, fim);
        pinta(&mut h, 2);
        let f = h.store.card_list_mut(bones()).stack.flight().copied();
        desvios.push(f.map(|f| (f.at.0 - fim.0).abs() + (f.at.1 - fim.1).abs()));
    }
    let vao = h
        .store
        .card_list_mut(bones())
        .stack
        .flight()
        .and_then(|f| f.to);
    mao(&mut h, PointerKind::Up, fim);
    pinta(&mut h, 40);
    // O lugar onde assentou: quantos cartões da lista ficaram acima dele.
    let y = |h: &ph2d_editor_core::screens::hero::HeroScreen, k| {
        h.hit_index
            .rect_for(ph2d_editor_core::ids::grip_de(k))
            .map(|r| r.y)
    };
    let ik = y(&h, CARD_IK);
    let acima = [
        CARD_TOOL,
        CARD_WEIGHT,
        CARD_SKIN,
        CARD_BONE,
        CARD_LIMIT,
        CARD_SMART,
    ]
    .into_iter()
    .filter(|k| y(&h, *k).zip(ik).is_some_and(|(a, b)| a < b))
    .count();
    let assentou = h.store.loose_card(CARD_IK).is_none() && vao == Some(acima);
    (desvios, assentou)
}

/// ⭐ T44d (smoke do dono 02/11): de volta à lista o cartão segue a mão ATÉ largar — com corda,
/// preso ou solto. *Mutação: o arrasto continua a marcar-se «solto» depois de reentrar ⇒ o voo
/// congela onde reentrou.*
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

/// Pinta `n` quadros andando a corda entre eles, como a shell.
fn anda(h: &mut ph2d_editor_core::screens::hero::HeroScreen, n: usize) {
    for _ in 0..n {
        ph2d_card_rope::anda_com(&mut h.store, bones(), |_| Some((800.0, 600.0)), 1.0 / 60.0);
        pinta(h, 1);
    }
}

/// O IK preso COM corda levado pela mão até à coluna da lista, SEM largar. Devolve o hero, onde a
/// mão está e o alfinete de antes.
fn com_corda_ate_a_lista() -> (
    ph2d_editor_core::screens::hero::HeroScreen,
    (f32, f32),
    Option<u64>,
) {
    use super::o_cartao_solto_no_canvas::mao;
    use ph2d_host::PointerKind;
    let mut h = hero_com_o_bones();
    osso_em_foco(8, "Bone 8", 0.5);
    pinta(&mut h, 20);
    let lista = h.store.panel_rect(bones()).expect("a lista");
    solta(&mut h, CARD_IK);
    for _ in 0..2 {
        clica(&mut h, loose_pin_of(CARD_IK));
        anda(&mut h, 2);
    }
    let pin = h.store.loose_card(CARD_IK).and_then(|c| c.pin);
    anda(&mut h, 10);
    let pega = h
        .hit_index
        .rect_for(ph2d_editor_core::ids::grip_de(CARD_IK))
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
    assert_eq!(h.store.loose_card(CARD_IK), None, "não reentrou");
    (h, dentro, pin)
}

/// ⭐ T53 (smoke do dono 02/11): «a corda só deve romper se o card encaixar no inspector no mouse
/// up» — dentro da coluna, sem largar, a corda continua; de volta ao canvas e largado, o cartão é
/// solto de novo com o MESMO alfinete e a corda. *Mutação: devolver esquece alfinete e corda.*
#[test]
fn a_rope_survives_the_column_until_the_card_docks() {
    use super::o_cartao_solto_no_canvas::mao;
    use ph2d_host::PointerKind;
    let (mut h, dentro, pin) = com_corda_ate_a_lista();
    assert!(pin.is_some(), "não prendeu");
    let viva = h
        .store
        .loose_ropes_of(bones())
        .iter()
        .any(|(k, _)| *k == CARD_IK);
    assert!(
        viva && !h.store.loose_rope(CARD_IK).is_empty(),
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
    let c = h.store.loose_card(CARD_IK).expect("solto de novo");
    assert_eq!(
        (c.pin, c.rope),
        (pin, true),
        "o alfinete ou a corda não voltaram"
    );
    assert!(
        !h.store.loose_rope(CARD_IK).is_empty(),
        "sem corda depois de largar"
    );
    assert!(
        h.store.reeling_ropes_of(bones()).is_empty(),
        "a corda recolheu-se"
    );
}

/// T53: largado DENTRO da coluna, encaixa — o suspenso acaba e a corda recolhe-se até ao objecto.
#[test]
fn a_rope_reels_in_when_the_card_docks_on_mouse_up() {
    use super::o_cartao_solto_no_canvas::mao;
    use ph2d_host::PointerKind;
    let (mut h, dentro, _) = com_corda_ate_a_lista();
    mao(&mut h, PointerKind::Up, dentro);
    anda(&mut h, 3);
    assert_eq!(h.store.loose_card(CARD_IK), None, "não encaixou");
    assert_eq!(h.store.suspended_card(), None, "o suspenso ficou");
    assert_eq!(h.store.loose_pin_key(CARD_IK), None);
    assert!(
        !h.store.reeling_ropes_of(bones()).is_empty(),
        "a corda sumiu sem recolher"
    );
    anda(&mut h, 40);
    assert!(
        h.store.reeling_ropes_of(bones()).is_empty(),
        "não acabou de recolher"
    );
}

/// ⭐ C3: o cartão do Bones preso a um osso, com outro em foco: o losango é o do osso dele e o clique
/// grava nele. *Mutação: `com_marcas_de` fora do corpo do preso ⇒ mostra o mapa do foco.*
#[test]
fn a_diamond_of_a_pinned_bone_card_is_of_that_bone() {
    use ph2d_editor_core::ids as painel;
    use ph2d_editor_core::interaction::{dispatch_pointer, loose_rect_of};
    use ph2d_editor_core::widget::key_mark::{self, KeyMark};
    use ph2d_host::{PointerButton, PointerEvent, PointerKind};
    use std::collections::BTreeMap;
    let mut sim = ph2d_ecs::SimWorld::new();
    let spawn = |sim: &mut ph2d_ecs::SimWorld| {
        let e = (
            ph2d_ecs::Transform::default(),
            ph2d_skeleton_ecs::Bone::default(),
        );
        sim.world_mut().spawn(e).id().to_bits()
    };
    let (preso, foco) = (spawn(&mut sim), spawn(&mut sim));
    let mut h = hero_com_o_bones();
    osso_em_foco(preso, "Bone A", 0.5);
    pinta(&mut h, 20);
    solta(&mut h, CARD_BONE);
    clica(&mut h, loose_pin_of(CARD_BONE));
    assert_eq!(
        h.store.loose_card(CARD_BONE).and_then(|c| c.pin),
        Some(preso)
    );
    let campo = painel::VECTOR_BONE_LENGTH;
    let publica_com =
        |h: &mut ph2d_editor_core::screens::hero::HeroScreen, m: BTreeMap<_, _>, f| {
            ph2d_panel_skeleton::publica_presos(&[preso], |b| osso_em_foco(b, "Bone A", 0.5));
            osso_em_foco(foco, "Bone B", 0.25);
            key_mark::set_key_marks(BTreeMap::from([(campo, f)]));
            key_mark::set_key_marks_dos_presos(m);
            pinta(h, 1);
        };
    // Só o foco tem mapa: o corpo do preso não o lê.
    publica_com(&mut h, BTreeMap::new(), KeyMark::Chave);
    let alvo = key_mark::key_mark_id(campo);
    let solto = h
        .store
        .panel_rect(loose_rect_of(CARD_BONE))
        .expect("o solto");
    let no_solto = |h: &ph2d_editor_core::screens::hero::HeroScreen| {
        let mut y = solto.y;
        while y < solto.y + solto.h {
            let mut x = solto.x;
            while x < solto.x + solto.w {
                if h.hit_index.hit(x, y) == Some(alvo) {
                    return Some((x, y));
                }
                x += 2.0;
            }
            y += 2.0;
        }
        None
    };
    assert!(no_solto(&h).is_none(), "o preso mostrou o losango do foco");
    // O mapa do preso: o losango aparece, e o clique é do osso preso.
    let dele = BTreeMap::from([(preso, BTreeMap::from([(campo, KeyMark::Livre)]))]);
    publica_com(&mut h, dele, KeyMark::Livre);
    let (x, y) = no_solto(&h).expect("o losango do preso");
    let arena = bumpalo::Bump::new();
    let ev = PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind: PointerKind::Down,
        source: ph2d_host::PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    };
    let _ = dispatch_pointer(&mut h.store, &h.hit_index, ev, &arena);
    let c = key_mark::take_key_mark_click().expect("o clique no losango");
    assert_eq!(c.sujeito, Some(preso));
    let tl = ph2d_timeline::TimelineDoc::new();
    let inst = ph2d_timeline::Instante {
        t_cut: 0.0,
        solo: true,
    };
    let ctx = ph2d_app_vec::marcas::Contexto::default();
    let (ctx_c, alvo_c) = ph2d_host_presos::losango::alvo_do_clique(
        &c,
        &sim,
        &ph2d_vec_scene::VecScene::new(),
        ctx,
        &[foco],
    );
    assert_eq!(alvo_c, [preso]);
    let pedido = ph2d_host_presos::losango::clique(
        &tl,
        &sim,
        ctx_c,
        &c.fields,
        &alvo_c,
        inst,
        |_| match ph2d_app_skeleton::canais_osso::canal_do_campo(campo) {
            Some(ph2d_timeline::PropKind::Field(k)) => vec![k],
            _ => Vec::new(),
        },
        |_, _| Some(1.0),
        |e, p| {
            Some(ph2d_timeline::TimelineIntent::AddKey {
                entity: e,
                prop: p,
                t: ph2d_anim::RationalTime::from_seconds(0.0),
                value: ph2d_anim::AnimValue::Float(1.0),
                interp: ph2d_anim::Interp::Linear,
            })
        },
    );
    let donos: Vec<u64> = pedido
        .iter()
        .filter_map(|i| match i {
            ph2d_timeline::TimelineIntent::AddKey { entity, .. } => Some(*entity),
            _ => None,
        })
        .collect();
    assert_eq!(donos, [preso], "a chave não foi ao osso preso");
}
