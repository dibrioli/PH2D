//! Os gates do canal de EVENTOS (`W-PlayerOut`, A2).
//!
//! ⚠️ **O que estes gates NÃO podem provar, e onde a prova mora:** a lei do
//! `A2` que carrega mais peso é *o evento nasce por TIQUE, e não por quadro*, e
//! ela vive no LAÇO DE TIQUES da ponte — aqui só se vê o par
//! `(vista_antes, passo_depois)` já separado. O gate daquela metade é o irmão
//! da ponte, e sem ele estes ficariam verdes sobre um dispatch que perde o
//! pouso do meio (o defeito exato que o `W-TickContacts` mediu no canal de
//! contatos).

use super::*;
use crate::{FootingKind, Motor, PlayerState, PlayerView};

const UP: Vec2 = [0.0, 1.0];

/// Um passo com o mínimo preenchido — a vista é o que os gates variam.
fn step(view: PlayerView, jump: Option<JumpKind>) -> PlayerStep {
    PlayerStep {
        motor: Motor::default(),
        state: PlayerState::default(),
        view,
        reaction: None,
        nudge: [0.0, 0.0],
        gravity_hold: [0.0, 0.0],
        drop_through: false,
        jump,
    }
}

fn airborne(vy: f32) -> PlayerView {
    PlayerView {
        footing: FootingKind::Airborne,
        velocity: [0.0, vy],
        ..PlayerView::default()
    }
}

fn grounded(vy: f32) -> PlayerView {
    PlayerView {
        footing: FootingKind::Ground,
        velocity: [0.0, vy],
        ..PlayerView::default()
    }
}

fn events(before: &PlayerView, s: &PlayerStep) -> Vec<PlayerEvent> {
    let mut out = Vec::new();
    events_between(before, s, UP, &mut out);
    out
}

/// **O CONTROLE**: duas vistas iguais não publicam nada.
///
/// Sem ele, um canal que disparasse em TODO tique passaria em todos os outros
/// gates deste arquivo — cada um deles procura um evento *presente*.
#[test]
fn two_identical_views_publish_nothing() {
    let v = grounded(0.0);
    assert!(events(&v, &step(v, None)).is_empty());

    let a = airborne(-4.0);
    assert!(events(&a, &step(a, None)).is_empty());
}

/// **Qual pulo saiu é a LEI quem diz** — o canal apenas repassa.
///
/// ⚠️ E o gate mostra por que adivinhar de fora não serve: o pulo do AR e o de
/// PAREDE saem os dois com o personagem NÃO apoiado, então a pergunta que um
/// palpite faria (*"ele estava no chão?"*) responde **não** para os dois e não
/// os distingue. O `Ground` está aqui como o terceiro caso, para o gate cobrir
/// a família inteira em vez do par ambíguo.
#[test]
fn the_law_names_which_jump_left_and_the_channel_only_relays_it() {
    for (kind, before) in [
        (JumpKind::Ground, grounded(0.0)),
        (JumpKind::Air, airborne(-2.0)),
        (JumpKind::Wall, airborne(-2.0)),
    ] {
        let after = airborne(6.0);
        let got = events(&before, &step(after, Some(kind)));
        assert!(
            got.contains(&PlayerEvent::Jumped { kind }),
            "{kind:?} não foi repassado: {got:?}"
        );
    }
}

/// **Um pouso mede a velocidade contra o CHÃO em que pousou.**
///
/// O elevador é a fixture que separa as duas leis: caindo a 5 m/s sobre uma
/// plataforma que sobe a 5 m/s, o encontro é SUAVE — e uma medida absoluta
/// diria 5, que é a dureza de uma queda que não aconteceu.
#[test]
fn a_landing_measures_its_speed_against_the_ground_it_landed_on() {
    let before = airborne(-5.0);

    let hard = grounded(-5.0);
    let got = events(&before, &step(hard, None));
    assert_eq!(got, vec![PlayerEvent::Landed { speed: 5.0 }]);

    // O MESMO corpo, o MESMO instante, sobre um chão que sobe junto.
    let lift = PlayerView {
        footing: FootingKind::Ground,
        velocity: [0.0, -5.0],
        ground_velocity: [0.0, -5.0],
        ..PlayerView::default()
    };
    let got = events(&before, &step(lift, None));
    assert_eq!(got, vec![PlayerEvent::Landed { speed: 0.0 }]);
}

/// **A velocidade de um pouso nunca é negativa.**
///
/// Chegar ao chão SUBINDO (a plataforma o alcançou por baixo) é um encontro de
/// dureza zero, não de dureza negativa — e um número negativo aqui viraria uma
/// poeira ao contrário no consumidor, sem ninguém saber por quê.
#[test]
fn a_landing_speed_is_never_negative() {
    let before = airborne(2.0);
    let after = grounded(2.0);
    assert_eq!(
        events(&before, &step(after, None)),
        vec![PlayerEvent::Landed { speed: 0.0 }]
    );
}

/// **Uma rampa RECUSADA não é chão, então alcançar o patamar é um POUSO.**
///
/// A postura `Steep` existe precisamente para não ser confundida com apoio
/// (W9): quem escorrega por uma encosta de 60° e chega a um plano *aterra*, e
/// um gate que só olhasse `Airborne → Ground` perderia o caso.
#[test]
fn a_refused_slope_is_not_ground_so_reaching_the_ledge_lands() {
    let before = PlayerView {
        footing: FootingKind::Steep,
        velocity: [3.0, -3.0],
        ..PlayerView::default()
    };
    let after = PlayerView {
        footing: FootingKind::Ground,
        velocity: [3.0, -3.0],
        ..PlayerView::default()
    };
    assert_eq!(
        events(&before, &step(after, None)),
        vec![PlayerEvent::Landed { speed: 3.0 }]
    );
}

/// **O ápice pede o AR, e a guarda é o que impede um elevador de o publicar.**
///
/// A subida relativa cruzando de positiva para não-positiva descreve o topo de
/// um salto — mas com os pés no chão ela descreve uma plataforma a travar, e
/// quem chegou ao topo foi a plataforma.
#[test]
fn the_apex_needs_the_air() {
    let rising = airborne(3.0);
    let falling = airborne(-0.5);
    assert!(events(&rising, &step(falling, None)).contains(&PlayerEvent::Apex));

    // O topo EXATO conta: a lei é `> 0` antes e `<= 0` depois.
    let peak = airborne(0.0);
    assert!(events(&rising, &step(peak, None)).contains(&PlayerEvent::Apex));

    // O mesmo cruzamento, com os pés apoiados: silêncio.
    let up = grounded(3.0);
    let down = grounded(-0.5);
    assert!(!events(&up, &step(down, None)).contains(&PlayerEvent::Apex));
}

/// **As travas publicam a BORDA, não o estado.**
///
/// Cada uma é um booleano que a vista já carrega; o evento é a transição dele,
/// então segurar o arranque por trinta tiques publica **um** `Dashed`.
#[test]
fn the_locks_publish_the_edge_and_not_the_state() {
    let off = PlayerView::default();

    let on = PlayerView {
        dashing: true,
        ..PlayerView::default()
    };
    assert_eq!(events(&off, &step(on, None)), vec![PlayerEvent::Dashed]);
    assert!(events(&on, &step(on, None)).is_empty());

    let hanging = PlayerView {
        ledging: true,
        ..PlayerView::default()
    };
    assert_eq!(
        events(&off, &step(hanging, None)),
        vec![PlayerEvent::LedgeGrabbed]
    );
    assert!(events(&hanging, &step(hanging, None)).is_empty());
}

/// **A água publica as DUAS bordas** — entrar e sair são eventos distintos.
///
/// É o único par simétrico do canal: os outros booleanos só interessam ao
/// subir, e a água interessa nos dois sentidos (o som de mergulho tem irmão).
#[test]
fn the_water_publishes_both_edges() {
    let dry = PlayerView::default();
    let wet = PlayerView {
        swimming: true,
        ..PlayerView::default()
    };

    assert_eq!(
        events(&dry, &step(wet, None)),
        vec![PlayerEvent::EnteredWater]
    );
    assert_eq!(events(&wet, &step(dry, None)), vec![PlayerEvent::LeftWater]);
    assert!(events(&wet, &step(wet, None)).is_empty());
}

/// **Um tique pode publicar VÁRIOS eventos**, e a lista não os ordena.
///
/// Saltar de dentro da água é um tique só, com duas coisas verdadeiras — e o
/// canal é uma lista precisamente para não ter de escolher qual delas contar.
#[test]
fn one_tick_can_publish_more_than_one_event() {
    let before = PlayerView {
        footing: FootingKind::Ground,
        swimming: true,
        ..PlayerView::default()
    };
    let after = PlayerView {
        footing: FootingKind::Airborne,
        velocity: [0.0, 6.0],
        ..PlayerView::default()
    };

    let got = events(&before, &step(after, Some(JumpKind::Ground)));
    assert!(got.contains(&PlayerEvent::Jumped {
        kind: JumpKind::Ground
    }));
    assert!(got.contains(&PlayerEvent::LeftWater));
    assert_eq!(got.len(), 2, "nem mais, nem menos: {got:?}");
}

/// Uma vista de cabeça bloqueada, subindo a `vy`.
fn bonking(vy: f32) -> PlayerView {
    PlayerView {
        footing: FootingKind::Airborne,
        velocity: [0.0, vy],
        ceiling: true,
        ..PlayerView::default()
    }
}

/// **A batida carrega a subida que ele trazia, e a subida nunca é negativa.**
///
/// ⚠️ O segundo caso é o espelho exato do `a_landing_speed_is_never_negative`, e
/// pelo mesmo motivo: um número negativo aqui viraria, no consumidor, uma poeira
/// ao contrário — um efeito para BAIXO num evento cuja palavra é *subir*. Ele é
/// inalcançável no produto (o sensor do teto só é consultado com a subida
/// positiva) e o piso fica porque a vista é dado simples: quem a construir de
/// outro sítio não herda essa garantia.
#[test]
fn a_bonk_carries_the_ascent_and_never_a_negative() {
    // ⚠️ As duas vistas trazem subidas DIFERENTES de propósito: é o que separa
    // *a velocidade no tique da batida* (6,0) de *a do tique anterior* (9,0), e
    // sem essa diferença a escolha entre as duas seria indistinguível — a mesma
    // régua e o mesmo instante do `Landed`, que também lê o `after`.
    let before = airborne(9.0);
    assert_eq!(
        events(&before, &step(bonking(6.0), None)),
        vec![PlayerEvent::Bonked { speed: 6.0 }]
    );

    let mut sinking = bonking(-2.0);
    sinking.ceiling = true;
    assert_eq!(
        events(&airborne(-2.0), &step(sinking, None)),
        vec![PlayerEvent::Bonked { speed: 0.0 }]
    );
}

/// **A cabeça que CONTINUA encostada não publica um segundo evento.**
///
/// ⚠️ É o controle que separa este canal do da VISTA: o bit `ceiling` é estado
/// contínuo e pode ficar de pé vários tiques (medido na ponte: dois), enquanto
/// um evento vale por um tique — e a única coisa que impõe isso é a borda.
#[test]
fn a_head_that_stays_blocked_publishes_no_second_bonk() {
    let held = bonking(4.0);
    assert!(events(&held, &step(bonking(3.8), None)).is_empty());
}
