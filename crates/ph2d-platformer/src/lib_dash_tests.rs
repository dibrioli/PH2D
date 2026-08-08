//! **O ARRANQUE na porta única** (W14) — irmão de `lib_tests.rs` pelo teto de
//! 700 LOC.
//!
//! ⚠️ **O corte é por RESPONSABILIDADE:** o pai fica com *o que a porta É* e
//! este filho com *o que o arranque faz a ela* — as três coisas que ele cala e a
//! precedência do pulo sobre ele.
//!
//! ⚠️ Módulo FILHO por `#[path]`, como o pai, e **um glob basta** — um `use` do
//! pai é privado, mas privado quer dizer *visível aos descendentes*, então o
//! `super::*` traz as fixtures dele (`at`, `UP`, `G`, `DT`) **e** os tipos da lei
//! que ele já importou. (Eu tinha escrito aqui que eram precisos dois; o clippy
//! disse que não, e ele tem razão.)
use super::*;

/// A capsula flutuante — o modo que estes gates medem.
const SPRING: Support = Support::Spring;

/// Ar seco — todo gate deste arquivo mede o arco BALÍSTICO.
const DRY: Buoyed = Buoyed::DRY;

// ── O ARRANQUE, na PORTA ÚNICA (W14) ─────────────────────────────────────────

/// Uma config com o arranque LIGADO — a capacidade nasce desligada.
fn dashing_cfg() -> PlayerConfig {
    PlayerConfig {
        dash: DashConfig {
            speed: 18.0,
            time: 0.15,
            cooldown: 0.2,
        },
        ..PlayerConfig::STARTING_POINT
    }
}

/// A entrada de quem aperta o arranque a andar para a direita.
fn dash_input() -> PlayerInput {
    PlayerInput {
        drive: 1.0,
        dash: true,
        ..PlayerInput::default()
    }
}

/// **Enquanto o arranque dura, a perna, a caminhada e a gravidade calam-se — e
/// as três num gate só, porque são UMA frase.**
///
/// O oráculo é o motor INTEIRO: se a mola ou a caminhada tivessem sobrado, o
/// boost não seria exactamente o do arranque e a asserção diria qual.
///
/// ⚠️ **Mutação medida:** tirar o `&& !dashing` da linha que cala a perna
/// (`standing`) faz a mola voltar — e o `gravity_hold` continua correcto, então
/// **só o boost sangra**. É por isso que este gate compara o motor e não o
/// canal.
#[test]
fn while_dashing_the_leg_the_walk_and_gravity_are_all_silent() {
    let cfg = dashing_cfg();
    let ground = at(cfg.ride.float_height, UP);
    // Uma velocidade de partida que a mola e a caminhada teriam opinião sobre.
    let vel = [2.0, -3.0];
    let step = player_motor(
        &cfg,
        Some(&ground),
        None,
        None,
        None,
        dash_input(),
        PlayerState::default(),
        vel,
        G,
        UP,
        DT,
        DRY,
        SPRING,
    );
    let want = dash_burst(&cfg.dash, 1.0, [0.0, 0.0], vel, UP, G);
    assert!(
        (step.motor.boost[0] - want.boost[0]).abs() < 1e-4
            && (step.motor.boost[1] - want.boost[1]).abs() < 1e-4,
        "alguem alem do arranque escreveu velocidade: {:?} contra {:?}",
        step.motor.boost,
        want.boost
    );
    assert_eq!(
        step.gravity_hold,
        [-G[0], -G[1]],
        "o arranque tem de declarar o cancelamento da gravidade"
    );
    // E o que sobra para o caminho agrupado é ZERO: a gravidade cancelada é
    // paga por sub-passo, e mais ninguém pôs `accel` neste tique.
    let lumped = [
        step.motor.accel[0] - step.gravity_hold[0],
        step.motor.accel[1] - step.gravity_hold[1],
    ];
    assert!(
        lumped[0].abs() < 1e-4 && lumped[1].abs() < 1e-4,
        "sobrou aceleracao agrupada num tique de arranque: {lumped:?}"
    );
}

/// **Um pulo dado a partir de um arranque é um PULO** — e o arranque acaba ali.
///
/// ⚠️ **Mutação medida:** trocar o `jumped` pelo `jump.takeoff` do chão deixa
/// este gate VERDE (a fixture pula do chão) e mata o pulo de PAREDE feito
/// durante um arranque — que é o gesto que mais se encadeia com ele. É o gate
/// seguinte que a apanha, e por isso os dois existem.
#[test]
fn a_jump_out_of_a_dash_is_a_jump() {
    let cfg = dashing_cfg();
    let ground = at(cfg.ride.float_height, UP);
    let started = player_motor(
        &cfg,
        Some(&ground),
        None,
        None,
        None,
        dash_input(),
        PlayerState::default(),
        [0.0, 0.0],
        G,
        UP,
        DT,
        DRY,
        SPRING,
    );
    assert!(
        started.state.dash.left > 0.0,
        "o arranque tem de ter comecado"
    );
    let jumped = player_motor(
        &cfg,
        Some(&ground),
        None,
        None,
        None,
        PlayerInput {
            drive: 1.0,
            jump: true,
            dash: true,
            ..PlayerInput::default()
        },
        started.state,
        [0.0, 0.0],
        G,
        UP,
        DT,
        DRY,
        SPRING,
    );
    assert!(
        jumped.motor.boost[1] > 0.0,
        "o pulo tem de sair, e para CIMA: {:?}",
        jumped.motor.boost
    );
    assert!(
        jumped.state.dash.left <= 0.0,
        "e o arranque acaba no mesmo tique"
    );
    assert_eq!(
        jumped.gravity_hold,
        [0.0, 0.0],
        "quem pulou nao esta' a segurar a gravidade"
    );
}

/// **O arranque desligado é o mundo de antes desta wave, AO BIT.**
///
/// O mesmo tique com o botão apertado e a capacidade em zero tem de dar o motor
/// EXACTO de quem não apertou nada — é a prova executável do opt-in.
#[test]
fn a_dash_button_with_the_capability_off_changes_nothing() {
    let cfg = PlayerConfig::STARTING_POINT;
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
    let pressed = player_motor(
        &cfg,
        Some(&ground),
        None,
        None,
        None,
        PlayerInput {
            drive: 1.0,
            dash: true,
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
    assert_eq!(
        quiet.motor, pressed.motor,
        "com a capacidade desligada o botao nao pode mover um bit"
    );
    assert_eq!(quiet.gravity_hold, pressed.gravity_hold);
}

/// **E um pulo de PAREDE dado a partir de um arranque também o acaba.**
///
/// ⚠️ **É este gate que torna o `jumped` load-bearing**, e não o irmão acima: o
/// pulo de parede devolve `takeoff: false` de propósito (a 3ª lei não devolve a
/// uma parede o que o pé não empurrou no chão), então quem perguntasse pelo
/// `takeoff` deixaria o arranque VIVO por cima de um pulo de parede — os dois a
/// escrever a mesma velocidade, com o último a ganhar.
#[test]
fn a_wall_jump_out_of_a_dash_also_ends_it() {
    let cfg = PlayerConfig {
        wall: WallConfig {
            slide_speed: 3.0,
            jump_height: 2.0,
            ..PlayerConfig::STARTING_POINT.wall
        },
        ..dashing_cfg()
    };
    // ⚠️ **O arranque começa NO AR, e a premissa é declarada porque a primeira
    // versão desta fixture media o ramo errado:** arrancando do chão, o COYOTE
    // fica cheio, e com ele o pulo de CHÃO precede o de parede — por desenho
    // (o chão é o apoio mais forte, `jump.rs` §pulo de parede). O gate ficava
    // vermelho a dizer *"o empurrão não saiu"* sobre um produto correcto, e o
    // que faltava era a cena, não a lei.
    let started = player_motor(
        &cfg,
        None,
        None,
        None,
        None,
        dash_input(),
        PlayerState::default(),
        [0.0, -1.0],
        G,
        UP,
        DT,
        DRY,
        SPRING,
    );
    assert!(
        started.state.dash.left > 0.0,
        "o arranque tem de ter comecado"
    );
    assert_eq!(
        started.state.jump.coyote, 0.0,
        "a premissa: sem coyote, para que quem responda ao aperto seja a PAREDE"
    );
    // A meio do arranque encosta numa parede à direita e aperta o pulo. Sem
    // chão: é a PAREDE que oferece o salto.
    let wall = WallProbe {
        side: 1.0,
        hits: [
            Some(WallHit {
                distance: 0.05,
                normal: [-1.0, 0.0],
            }),
            None,
            None,
        ],
    };
    let jumped = player_motor(
        &cfg,
        None,
        None,
        Some(&wall),
        None,
        PlayerInput {
            drive: 1.0,
            jump: true,
            dash: true,
            ..PlayerInput::default()
        },
        started.state,
        // ⚠️ A velocidade do escorregamento EM REGIME: com ela, o termo do
        // `wall_slide` deste tique é exactamente zero (`−slide − (−slide)`), e o
        // que sobra no boost é só o pulo. Fora do regime ele subtrai, e o gate
        // mediria os dois somados.
        [0.0, -cfg.wall.slide_speed],
        G,
        UP,
        DT,
        DRY,
        SPRING,
    );
    assert!(
        jumped.motor.boost[1] > 0.0,
        "o pulo de parede tem de sair: {:?}",
        jumped.motor.boost
    );
    assert!(
        jumped.motor.boost[0] < 0.0,
        "e para LONGE da parede, nao na direcao do arranque: {:?}",
        jumped.motor.boost
    );
    assert!(
        jumped.state.dash.left <= 0.0,
        "o arranque tinha de acabar no mesmo tique"
    );
}
