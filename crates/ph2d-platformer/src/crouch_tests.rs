//! Os gates da lei do AGACHAR (W15).

use super::*;
use crate::{PlayerConfig, RideConfig, WalkConfig};

fn ride() -> RideConfig {
    RideConfig {
        float_height: 1.2,
        cling_distance: 0.25,
        ..RideConfig::STARTING_POINT
    }
}

fn armed() -> CrouchConfig {
    CrouchConfig {
        height: 0.6,
        speed: 2.0,
    }
}

fn blocked() -> Headroom {
    Headroom { blocked: true }
}

/// **O ALCANCE DO SENSOR NÃO SE MOVE** — o invariante de que esta wave inteira
/// vive.
///
/// A soma `float_height + cling_distance` é exactamente o que
/// [`crate::within_reach`] compara, e é ela que decide *"isto conta como chão?"*.
/// Mantendo-a fixa, a ponte pode castar e julgar com a config **autorada** sem
/// saber que o agachar existe — e nenhuma das duas metades precisa de ser
/// ordenada em relação à outra.
///
/// ⚠️ **Mutação medida:** não crescer a faixa de agarre
/// (`cling_distance: ride.cling_distance`) encolhe o alcance de **1,45 para
/// 0,85 m**. O personagem que agacha sai do alcance da própria mola no instante
/// do gesto e cai os 0,6 m em queda livre até ela o reencontrar — e, pior, a
/// ponte passa a achar chão onde a lei já não acha.
#[test]
fn the_crouch_does_not_move_what_the_sensor_reaches() {
    let (r, c) = (ride(), armed());
    let stand = r.float_height + r.cling_distance;
    let crouched = ride_for(&c, &r, true);
    let low = crouched.float_height + crouched.cling_distance;
    assert!(
        (low - stand).abs() < 1e-6,
        "o alcance mudou: de pe' {stand} contra agachado {low}"
    );
}

/// **A perna encurta exactamente o que foi autorado** — e é isso, e só isso, que
/// baixa a silhueta.
#[test]
fn the_leg_shortens_by_exactly_the_authored_amount() {
    let (r, c) = (ride(), armed());
    let low = ride_for(&c, &r, true);
    assert!((low.float_height - c.height).abs() < 1e-6);
    assert!(
        (c.rise(&r) - (r.float_height - c.height)).abs() < 1e-6,
        "a subida ao levantar tem de ser o delta"
    );
    // De pé é a config autorada, ao bit.
    assert_eq!(ride_for(&c, &r, false), r);
}

/// **Levantar-se é RECUSADO sob um teto** — e é por isso que existe um estado.
///
/// ⚠️ **Mutação medida:** ignorar o sensor (`stuck = false`) faz o personagem
/// levantar-se para dentro da pedra, e quem fica a resolver a penetração é o
/// solver — um empurrão que o artista nunca autorou.
#[test]
fn standing_up_is_refused_under_a_ceiling() {
    let (r, c) = (ride(), armed());
    let down = crouch_step(&c, &r, CrouchState::default(), true, true, None);
    assert!(down.crouched, "o botao agacha");

    let up = crouch_step(&c, &r, down, true, false, Some(&blocked()));
    assert!(up.crouched, "sob o teto ele continua agachado");
}

/// **E ele levanta-se assim que houver espaço** — a recusa é do teto, não uma
/// prisão.
#[test]
fn he_stands_up_once_there_is_room() {
    let (r, c) = (ride(), armed());
    let down = crouch_step(&c, &r, CrouchState::default(), true, true, None);
    let stuck = crouch_step(&c, &r, down, true, false, Some(&blocked()));
    let free = crouch_step(&c, &r, stuck, true, false, Some(&Headroom::CLEAR));
    assert!(!free.crouched, "com o ceu limpo ele levanta-se");
}

/// **O agachar exige o CHÃO** — no ar a perna está calada e o botão já tem dono
/// (a descida da W12).
#[test]
fn the_crouch_needs_the_ground() {
    let (r, c) = (ride(), armed());
    let air = crouch_step(&c, &r, CrouchState::default(), false, true, None);
    assert!(!air.crouched, "no ar o botao de baixo nao agacha");
}

/// **Altura zero é o mundo de antes desta wave** — a prova de que a capacidade é
/// opt-in.
#[test]
fn a_height_of_zero_is_the_world_before_this_wave() {
    let r = ride();
    let off = CrouchConfig {
        height: 0.0,
        ..armed()
    };
    assert!(!off.armed(&r));
    assert!(!crouch_step(&off, &r, CrouchState::default(), true, true, None).crouched);
    assert_eq!(ride_for(&off, &r, true), r, "a perna nao pode encurtar");
    let cfg = PlayerConfig {
        ride: r,
        crouch: off,
        ..PlayerConfig::STARTING_POINT
    };
    assert_eq!(
        effective_crouched(&cfg, true),
        cfg,
        "com a capacidade desligada nada e' efectivo"
    );
}

/// **Um "agachar" que SOBE é recusado** — e o guard é load-bearing, não higiene:
/// sem ele o `drop` fica negativo e a faixa de agarre ENCOLHE, que é o defeito
/// que o invariante desta wave existe para não ter.
#[test]
fn a_crouch_taller_than_standing_is_refused() {
    let r = ride();
    let up = CrouchConfig {
        height: r.float_height + 0.3,
        ..armed()
    };
    assert!(!up.armed(&r));
    assert_eq!(ride_for(&up, &r, true), r);
    // E o caso de fronteira: igual à altura de pé não é um agachar.
    let same = CrouchConfig {
        height: r.float_height,
        ..armed()
    };
    assert!(!same.armed(&r));
}

/// **A caminhada fica mais LENTA, não menos responsiva** — a aceleração é outra
/// grandeza e não se move.
#[test]
fn the_walk_slows_but_the_acceleration_does_not() {
    let (r, c) = (ride(), armed());
    let w = WalkConfig::STARTING_POINT;
    let low = walk_for(&c, &r, &w, true);
    assert!((low.speed - c.speed).abs() < 1e-6);
    assert!((low.acceleration - w.acceleration).abs() < 1e-6);
    assert!((low.air_acceleration - w.air_acceleration).abs() < 1e-6);
    assert!((low.max_slope_deg - w.max_slope_deg).abs() < 1e-6);
    assert_eq!(walk_for(&c, &r, &w, false), w);
}

/// **Velocidade zero é um agachar em que não se anda** — e é uma escolha
/// legítima, ao contrário da altura zero.
#[test]
fn a_crouch_speed_of_zero_is_a_duck_in_place() {
    let r = ride();
    let c = CrouchConfig {
        speed: 0.0,
        ..armed()
    };
    assert!(c.armed(&r), "velocidade zero NAO desliga a capacidade");
    let w = walk_for(&c, &r, &WalkConfig::STARTING_POINT, true);
    assert!((w.speed - 0.0).abs() < 1e-6);
}

/// **O sensor só é pedido por quem está agachado e SOLTOU** — em todo outro
/// tique não há raio nenhum a lançar.
///
/// ⚠️ **Mutação medida:** devolver `true` sempre põe três raios por tique em
/// cada personagem do mundo, para responder a uma pergunta que ninguém fez.
#[test]
fn the_probe_is_only_wanted_when_he_is_crouched_and_released() {
    let (r, c) = (ride(), armed());
    let up = CrouchState::default();
    let down = CrouchState { crouched: true };
    assert!(!headroom_probe_wanted(&c, &r, up, false), "de pe', solto");
    assert!(
        !headroom_probe_wanted(&c, &r, up, true),
        "de pe', a apertar"
    );
    assert!(
        !headroom_probe_wanted(&c, &r, down, true),
        "agachado e a segurar: nao quer levantar-se"
    );
    assert!(
        headroom_probe_wanted(&c, &r, down, false),
        "agachado e soltou: E' a pergunta"
    );
    let off = CrouchConfig { height: 0.0, ..c };
    assert!(
        !headroom_probe_wanted(&off, &r, down, false),
        "capacidade desligada nao casta nada"
    );
}

/// **O neutro é céu limpo, e o bloqueio bloqueia.**
///
/// ⚠️ Isto era `the_headroom_grid_spans_the_body`, e a grade que ele media
/// **deixou de existir** com a `W-ShapeCast`: o sensor passou de três raios para
/// uma varredura do corpo, e uma varredura não tem amostras cujo alinhamento
/// alguém possa errar. O que sobra da pergunta antiga é o neutro — que é
/// load-bearing na mesma medida, porque é ele que uma cena sem teto produz.
#[test]
fn clear_sky_is_the_neutral_and_a_ceiling_is_not() {
    assert!(!Headroom::CLEAR.is_blocked());
    assert!(blocked().is_blocked());
}
