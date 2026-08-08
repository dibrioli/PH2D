//! **O AGACHAR na porta única** (W15) — módulo FILHO de `lib_tests.rs` pelo teto
//! de 700 LOC, exactamente como o `lib_dash_tests.rs` ao lado.
//!
//! ⚠️ O corte é por RESPONSABILIDADE: o pai fica com *o que a porta É*, este
//! filho com *o que o agachar faz a ela* — a perna que encurta, a caminhada que
//! abranda, e o invariante que deixa a ponte não saber de nada disto.
//!
//! ⚠️ E ele é FILHO e não irmão da raiz: é isso que faz `super::*` alcançar as
//! fixtures do pai (`at`, `UP`, `G`, `DT`) além dos tipos da lei.
use super::*;

/// A capsula flutuante — o modo que estes gates medem.
const SPRING: Support = Support::Spring;

/// Ar seco — todo gate deste arquivo mede o arco BALÍSTICO.
const DRY: Buoyed = Buoyed::DRY;

/// Uma config com o agachar LIGADO — a capacidade nasce desligada.
fn crouching_cfg() -> PlayerConfig {
    PlayerConfig {
        crouch: CrouchConfig {
            height: 0.25,
            speed: 2.0,
        },
        ..PlayerConfig::STARTING_POINT
    }
}

/// A entrada de quem segura BAIXO a andar para a direita.
fn down_input() -> PlayerInput {
    PlayerInput {
        drive: 1.0,
        down: true,
        ..PlayerInput::default()
    }
}

/// **O VEREDITO DO CHÃO NÃO SE MOVE** — e é isto que deixa a ponte castar e
/// julgar com a config AUTORADA sem saber que esta wave existe.
///
/// A `footing_verdict` é calculada no topo do `player_motor`, **antes** de o
/// agachar ser conhecido; se ela respondesse diferente depois, a porta estaria
/// a decidir sobre um chão que ela já tinha classificado.
///
/// ⚠️ **Mutação medida:** não crescer a faixa de agarre no `ride_for` faz o
/// veredito passar de `Ground` a `Airborne` para uma amostra a `1,2 m` — o
/// personagem que agacha fica **no AR** aos olhos da lei e a mola cala-se.
#[test]
fn crouching_does_not_change_what_counts_as_ground() {
    let cfg = crouching_cfg();
    // Uma amostra que só está ao alcance por causa da faixa de agarre.
    let far = at(cfg.ride.float_height + cfg.ride.cling_distance * 0.5, UP);
    let low = effective_crouched(&cfg, true);
    assert!(
        footing(&cfg, Some(&far), UP).is_some(),
        "de pe' isto tem de ser chao"
    );
    assert!(
        footing(&low, Some(&far), UP).is_some(),
        "e agachado tambem, senao a ponte e a lei discordam"
    );
}

/// **Agachado, a perna segura o personagem MAIS BAIXO** — é a wave inteira em
/// um número.
///
/// O oráculo é o SINAL do empurrão da mola: parado à altura de pé, com a perna
/// encurtada, ela tem de puxar para BAIXO (o personagem está acima do novo
/// repouso).
///
/// ⚠️ **Mutação medida:** ignorar o agachar em `ride_for` deixa o empurrão em
/// **+9,81** (só o peso cancelado, o personagem já está no repouso) em vez de
/// puxar para baixo.
#[test]
fn a_crouched_leg_pulls_him_down_to_the_lower_rest() {
    let cfg = crouching_cfg();
    let ground = at(cfg.ride.float_height, UP);
    let up = player_motor(
        &cfg,
        Some(&ground),
        None,
        None,
        None,
        PlayerInput::default(),
        PlayerState::default(),
        [0.0, 0.0],
        G,
        UP,
        DT,
        DRY,
        SPRING,
    );
    let down = player_motor(
        &cfg,
        Some(&ground),
        None,
        None,
        None,
        down_input(),
        PlayerState::default(),
        [0.0, 0.0],
        G,
        UP,
        DT,
        DRY,
        SPRING,
    );
    assert!(down.state.crouch.crouched, "o botao tem de agachar");
    assert!(
        down.motor.accel[1] < up.motor.accel[1],
        "a perna encurtada tem de empurrar MENOS para cima: {} contra {}",
        down.motor.accel[1],
        up.motor.accel[1]
    );
}

/// **Agachado ele anda mais devagar** — a caminhada mira a velocidade do
/// agachar, e o oráculo é a velocidade que o motor PERSEGUE.
///
/// A fixture põe o corpo já à velocidade de cruzeiro de pé: em pé o motor não
/// tem nada a fazer (já está no alvo), agachado ele **freia**.
#[test]
fn a_crouched_walk_targets_the_crouch_speed() {
    let cfg = crouching_cfg();
    let ground = at(cfg.ride.float_height, UP);
    let vel = [cfg.walk.speed, 0.0];
    let up = player_motor(
        &cfg,
        Some(&ground),
        None,
        None,
        None,
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
        PlayerState::default(),
        vel,
        G,
        UP,
        DT,
        DRY,
        SPRING,
    );
    let low = player_motor(
        &cfg,
        Some(&ground),
        None,
        None,
        None,
        down_input(),
        PlayerState::default(),
        vel,
        G,
        UP,
        DT,
        DRY,
        SPRING,
    );
    let up_push = up.motor.accel[0] + up.motor.boost[0] / DT;
    let low_push = low.motor.accel[0] + low.motor.boost[0] / DT;
    assert!(
        up_push.abs() < 1.0,
        "a' velocidade de cruzeiro o motor nao empurra: {up_push}"
    );
    assert!(
        low_push < -1.0,
        "agachado ele tem de FREAR para o alvo mais lento: {low_push}"
    );
}

/// **A capacidade desligada é o mundo de antes desta wave** — o motor INTEIRO,
/// ao bit, com o botão de baixo segurado o tempo todo.
///
/// ⚠️ É a prova de opt-in mais forte que a lei consegue dar: não é um campo
/// comparado, é a saída inteira.
#[test]
fn a_down_button_with_the_capability_off_changes_nothing() {
    let cfg = PlayerConfig::STARTING_POINT;
    assert!(
        !cfg.crouch.armed(&cfg.ride),
        "o ponto de partida tem de nascer DESLIGADO"
    );
    let ground = at(cfg.ride.float_height, UP);
    let quiet = player_motor(
        &cfg,
        Some(&ground),
        None,
        None,
        None,
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
        PlayerState::default(),
        [1.0, 0.0],
        G,
        UP,
        DT,
        DRY,
        SPRING,
    );
    let held = player_motor(
        &cfg,
        Some(&ground),
        None,
        None,
        None,
        down_input(),
        PlayerState::default(),
        [1.0, 0.0],
        G,
        UP,
        DT,
        DRY,
        SPRING,
    );
    assert_eq!(quiet.motor, held.motor, "o botao nao pode mover um bit");
    assert!(!held.state.crouch.crouched);
}

/// **O teto RECUSA o levantar pela porta única** — a lei do `crouch_step` vista
/// de dentro da composição, com o estado a viajar no [`PlayerState`].
#[test]
fn a_ceiling_keeps_him_crouched_through_the_one_door() {
    let cfg = crouching_cfg();
    let ground = at(cfg.ride.float_height, UP);
    let down = player_motor(
        &cfg,
        Some(&ground),
        None,
        None,
        None,
        down_input(),
        PlayerState::default(),
        [0.0, 0.0],
        G,
        UP,
        DT,
        DRY,
        SPRING,
    );
    assert!(down.state.crouch.crouched);

    let blocked = Headroom {
        blocked: [false, true, false],
    };
    let stuck = player_motor(
        &cfg,
        Some(&ground),
        None,
        None,
        Some(&blocked),
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
        down.state,
        [0.0, 0.0],
        G,
        UP,
        DT,
        DRY,
        SPRING,
    );
    assert!(
        stuck.state.crouch.crouched,
        "sob o teto ele nao pode levantar-se"
    );

    let free = player_motor(
        &cfg,
        Some(&ground),
        None,
        None,
        Some(&Headroom::CLEAR),
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
        stuck.state,
        [0.0, 0.0],
        G,
        UP,
        DT,
        DRY,
        SPRING,
    );
    assert!(!free.state.crouch.crouched, "com espaco ele levanta-se");
}
