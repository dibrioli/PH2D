//! **A TRAVA DE BEIRADA, na LEI** (`W-Brink`) — o que o `clamp_target` promete.
//!
//! Estes gates não têm mundo: eles pinam a aritmética que a
//! [`crate::Brink`] e a [`crate::walk`] combinam. O comportamento — *ele para
//! antes de cair* — é medido pela porta do produto, na `ph2d-physics-ecs`.

use super::*;
use crate::{Brink, CrouchConfig, RideConfig, crouch};

const UP: Vec2 = [0.0, 1.0];
const DT: f32 = 1.0 / 60.0;

fn ground(brink: Brink) -> GroundSample {
    GroundSample {
        grip: 1.0,
        distance: 0.5,
        normal: [0.0, 1.0],
        ground_velocity: [0.0, 0.0],
        one_way: false,
        brink,
    }
}

/// **O mundo que já shipava é devolvido VERBATIM** — com a trava desarmada a
/// quina não tem voto, e é isso que torna a wave inteira inerte para toda cena
/// que nunca a autora.
#[test]
fn a_brink_changes_nothing_while_walking_off_is_allowed() {
    let cfg = WalkConfig::STARTING_POINT;
    assert!(
        cfg.walk_off_ledges,
        "o ponto de partida deixa andar para fora"
    );
    let free = walk(
        &cfg,
        Some(&ground(Brink::NONE)),
        [0.0, 0.0],
        UP,
        1.0,
        [0.0, 0.0],
        DT,
    );
    let at_edge = walk(
        &cfg,
        Some(&ground(Brink::RIGHT)),
        [0.0, 0.0],
        UP,
        1.0,
        [0.0, 0.0],
        DT,
    );
    assert_eq!(
        free.accel, at_edge.accel,
        "sem a trava, a quina nao pode mover a lei"
    );
    assert_eq!(free.boost, at_edge.boost);
}

/// **Com a trava armada, o alvo para o lado da quina é ZERO** — e o personagem
/// que já corre para lá **TRAVA**, com o orçamento inteiro.
///
/// ⚠️ As duas metades vivem no mesmo teste de propósito: separadas, a primeira
/// passaria com uma lei que apenas *deixa de empurrar*, que é o bug de deixá-lo
/// deslizar para fora por inércia.
#[test]
fn the_trava_zeroes_the_target_and_brakes_what_already_moves() {
    let cfg = WalkConfig {
        walk_off_ledges: false,
        ..WalkConfig::STARTING_POINT
    };
    // Parado, empurrando para a quina: nao ha' o que empurrar.
    let standing = walk(
        &cfg,
        Some(&ground(Brink::RIGHT)),
        [0.0, 0.0],
        UP,
        1.0,
        [0.0, 0.0],
        DT,
    );
    assert_eq!(
        (standing.accel, standing.boost),
        ([0.0, 0.0], [0.0, 0.0]),
        "com o alvo em zero e velocidade zero nao ha' delta: {standing:?}"
    );
    // Ja' correndo para a quina: a lei FREIA.
    let running = walk(
        &cfg,
        Some(&ground(Brink::RIGHT)),
        [cfg.speed, 0.0],
        UP,
        1.0,
        [0.0, 0.0],
        DT,
    );
    assert!(
        running.accel[0] < 0.0,
        "correndo para a quina a lei tem de FREAR: {running:?}"
    );
}

/// **A quina corta um sentido, nunca os dois** — é o que permite ANDAR PARA
/// LONGE da beirada em que se parou, em vez de ficar preso nela.
///
/// ⚠️ **Os DOIS lados, e a segunda metade foi escrita por uma MUTAÇÃO que
/// sobreviveu:** trocar o braço esquerdo do `clamp_target` por um `min` — o que
/// inverte a trava do lado de lá — passava por toda a suíte, porque cada gate
/// que eu tinha usava `Brink::RIGHT`. A esquerda é caminho VIVO do produto
/// (andar para a esquerda para fora de um patamar), e nada olhava para ela.
#[test]
fn the_trava_cuts_one_way_only_on_both_sides() {
    let cfg = WalkConfig {
        walk_off_ledges: false,
        ..WalkConfig::STARTING_POINT
    };
    let away_from_right = walk(
        &cfg,
        Some(&ground(Brink::RIGHT)),
        [0.0, 0.0],
        UP,
        -1.0,
        [0.0, 0.0],
        DT,
    );
    assert!(
        away_from_right.accel[0] < 0.0,
        "para longe de uma quina a' DIREITA a lei empurra como sempre: {away_from_right:?}"
    );
    let away_from_left = walk(
        &cfg,
        Some(&ground(Brink::LEFT)),
        [0.0, 0.0],
        UP,
        1.0,
        [0.0, 0.0],
        DT,
    );
    assert!(
        away_from_left.accel[0] > 0.0,
        "para longe de uma quina a' ESQUERDA, idem: {away_from_left:?}"
    );
    let into_left = walk(
        &cfg,
        Some(&ground(Brink::LEFT)),
        [0.0, 0.0],
        UP,
        -1.0,
        [0.0, 0.0],
        DT,
    );
    assert_eq!(
        (into_left.accel, into_left.boost),
        ([0.0, 0.0], [0.0, 0.0]),
        "e PARA a quina da esquerda o alvo e' zero: {into_left:?}"
    );
}

/// **O alcance é DERIVADO, e é a distância de PARAGEM** — o knob que ele
/// substituiu tinha o valor certo em função de outros dois.
#[test]
fn the_look_ahead_is_the_braking_distance_of_the_authored_speed() {
    let cfg = WalkConfig::STARTING_POINT;
    let expected = cfg.speed * cfg.speed / (2.0 * cfg.acceleration);
    assert!(
        (cfg.ledge_look() - expected).abs() < 1e-6,
        "{} != {expected}",
        cfg.ledge_look()
    );
    // Dobrar a velocidade quadruplica o alcance — a assinatura de `v^2`.
    let fast = WalkConfig {
        speed: cfg.speed * 2.0,
        ..cfg
    };
    assert!(
        (fast.ledge_look() / cfg.ledge_look() - 4.0).abs() < 1e-4,
        "o alcance tem de ser quadratico na velocidade: {} vs {}",
        fast.ledge_look(),
        cfg.ledge_look()
    );
    // Sem passo nao ha' para onde andar para fora.
    let still = WalkConfig { speed: 0.0, ..cfg };
    assert_eq!(still.ledge_look(), 0.0);
}

/// **A porta do custo só abre onde a trava pode agir** — armada, com alcance,
/// no chão, e com o dedo a empurrar. Uma cena que não a autora não paga um raio.
#[test]
fn the_probe_is_only_wanted_where_the_trava_can_act() {
    let free = WalkConfig::STARTING_POINT;
    let armed = WalkConfig {
        walk_off_ledges: false,
        ..free
    };
    assert!(!brink_probe_wanted(&free, true, 1.0), "desarmada: nunca");
    assert!(
        brink_probe_wanted(&armed, true, 1.0),
        "armada, no chao, a empurrar"
    );
    assert!(
        !brink_probe_wanted(&armed, false, 1.0),
        "no ar nao ha' patamar"
    );
    assert!(
        !brink_probe_wanted(&armed, true, 0.0),
        "sem dedo o alvo ja' e' zero"
    );
    let still = WalkConfig {
        speed: 0.0,
        ..armed
    };
    assert!(
        !brink_probe_wanted(&still, true, 1.0),
        "sem alcance nao ha' o que perguntar"
    );
}

/// **AGACHAR só APERTA** — um `=` aqui faria o agachar LIGAR a caminhada para
/// fora do patamar num personagem cujo autor a proibiu de pé, que é a trava a
/// fazer o contrário do próprio nome.
#[test]
fn crouching_can_tighten_the_trava_but_never_loosen_it() {
    let ride = RideConfig::STARTING_POINT;
    let standing_free = WalkConfig::STARTING_POINT;
    let standing_stops = WalkConfig {
        walk_off_ledges: false,
        ..standing_free
    };
    let crouch_free = CrouchConfig {
        height: 0.25,
        speed: 2.0,
        walk_off_ledges: true,
    };
    let crouch_stops = CrouchConfig {
        walk_off_ledges: false,
        ..crouch_free
    };
    assert!(
        !crouch::walk_for(&crouch_stops, &ride, &standing_free, true).walk_off_ledges,
        "agachado APERTA quem de pe' andava para fora"
    );
    assert!(
        !crouch::walk_for(&crouch_free, &ride, &standing_stops, true).walk_off_ledges,
        "agachado NAO devolve a liberdade a quem a perdeu de pe'"
    );
    // De pe', ou com o agachar desarmado, a config passa intacta.
    assert!(crouch::walk_for(&crouch_stops, &ride, &standing_free, false).walk_off_ledges);
}
