//! Os gates do **PULO DO AR** (`W-MultiJump`) — ver a lei em `jump.rs`.
//!
//! ⚠️ **Irmão do `jump_tests.rs` por ASSUNTO**, e não por tamanho: aqueles medem
//! o arco de UM pulo (a altura autorada, as fases da gravidade, o corte), estes
//! medem *quantos pulos existem e de onde vem cada um*.

use super::*;

/// Ar seco — todo gate deste arquivo mede o arco BALÍSTICO.
const DRY: crate::Buoyed = crate::Buoyed::DRY;

const UP: Vec2 = [0.0, 1.0];
const G: Vec2 = [0.0, -9.81];
const DT: f32 = 1.0 / 60.0;

fn ground() -> GroundSample {
    GroundSample {
        distance: 0.9,
        normal: [0.0, 1.0],
        ground_velocity: [0.0, 0.0],
        one_way: false,
    }
}

/// A config de partida com o pulo do ar LIGADO.
fn with_air(n: u32, h: f32) -> JumpConfig {
    JumpConfig {
        air_jumps: n,
        air_jump_height: h,
        ..JumpConfig::STARTING_POINT
    }
}

/// Um tique da lei, com o chão e o botão que se pedir.
fn step(
    cfg: &JumpConfig,
    state: JumpState,
    footing: Option<&GroundSample>,
    rel_up: f32,
    held: bool,
) -> JumpStep {
    jump_step(
        cfg, state, footing, rel_up, held, false, None, G, UP, DT, DRY,
    )
}

/// Um TOQUE (presso + solto), devolvendo o passo do tique em que ele SAIU.
fn tap(
    cfg: &JumpConfig,
    state: JumpState,
    footing: Option<&GroundSample>,
    rel_up: f32,
) -> JumpStep {
    let down = step(cfg, state, footing, rel_up, true);
    let up = step(cfg, down.state, footing, rel_up, false);
    // O pulo sai no tique do PRESS; o segundo tique existe só para soltar o
    // botão, senão a borda do toque seguinte nunca acontece.
    JumpStep {
        state: up.state,
        ..down
    }
}

/// **O PULO DO AR SAI, e só enquanto houver carga.**
///
/// Três toques com UMA carga: chão (sai), ar (sai), ar de novo (não sai).
#[test]
fn an_air_jump_fires_once_per_charge_and_then_stops() {
    let cfg = with_air(1, 2.0);
    let first = tap(&cfg, JumpState::default(), Some(&ground()), 0.0);
    assert!(first.motor.boost[1] > 0.0, "o pulo do chao sai");
    assert!(first.takeoff, "e ele EMPURRA o chao");

    let second = tap(&cfg, first.state, None, -1.0);
    assert!(
        second.motor.boost[1] > 0.0,
        "o pulo do AR sai: {:?}",
        second.motor.boost
    );
    assert_eq!(second.state.air_jumps_left, 0, "e gasta a carga");

    let third = tap(&cfg, second.state, None, -1.0);
    assert_eq!(
        third.motor.boost,
        [0.0, 0.0],
        "sem carga nao ha 3o pulo: {:?}",
        third.motor.boost
    );
}

/// **A altura do pulo do ar é a DELE**, não a do primeiro.
///
/// ⚠️ E o boost leva a velocidade AO valor (`v0 − rel_up`), que é o que faz um
/// pulo dado em plena QUEDA alcançar a altura autorada em vez de ser comido
/// pela velocidade que já havia — a mesma lei do pulo do chão.
#[test]
fn the_air_jump_reaches_its_own_authored_height() {
    let cfg = with_air(1, 0.5);
    let falling = JumpState {
        airborne: true,
        air_jumps_left: 1,
        ..JumpState::default()
    };
    let rel_up = -3.0;
    let s = tap(&cfg, falling, None, rel_up);
    let v0 = (2.0 * 9.81 * 0.5_f32).sqrt();
    assert!(
        (s.motor.boost[1] - (v0 - rel_up)).abs() < 1.0e-3,
        "o boost tem de ser v0(air) - rel_up = {:.4}: {:?}",
        v0 - rel_up,
        s.motor.boost
    );
    // E o número é OUTRO: `jump_height` é 2,0 e `air_jump_height` 0,5.
    let ground_v0 = (2.0 * 9.81 * 2.0_f32).sqrt();
    assert!(
        (s.motor.boost[1] - (ground_v0 - rel_up)).abs() > 1.0,
        "a altura do ar NAO e' a do chao"
    );
}

/// **A carga recarrega no CHÃO, pela mesma porta do coyote.**
///
/// ⚠️ O oráculo é *os dois enchem no MESMO tique* — é isso que a porta única
/// [`JumpState::on_ground`] garante, e é a divergência que daria *"às vezes o
/// duplo pulo não recarrega"* sem nada na tela a dizê-lo.
#[test]
fn the_charge_refills_with_the_coyote_on_the_same_tick() {
    let cfg = with_air(2, 2.0);
    let spent = JumpState {
        airborne: false,
        air_jumps_left: 0,
        coyote: 0.0,
        ..JumpState::default()
    };
    let s = step(&cfg, spent, Some(&ground()), 0.0, false);
    assert_eq!(s.state.air_jumps_left, 2, "as cargas voltaram");
    assert!(
        (s.state.coyote - cfg.coyote_time).abs() < 1.0e-6,
        "e o coyote encheu no MESMO tique: {}",
        s.state.coyote
    );
}

/// **Um pulo do ar não empurra o chão** — a 3ª lei devolve ao chão o que o pé
/// nele empurrou, e este pé não empurrou nada.
///
/// ⚠️ **Mas ele É um pulo**, e o `jumped` diz isso — é o campo que o ARRANQUE
/// pergunta, e sem ele o proxy antigo (a transição para o ar) responderia *não*
/// justamente aqui.
#[test]
fn an_air_jump_is_a_jump_but_pushes_nothing() {
    let cfg = with_air(1, 2.0);
    let falling = JumpState {
        airborne: true,
        air_jumps_left: 1,
        ..JumpState::default()
    };
    let s = tap(&cfg, falling, None, -2.0);
    assert!(s.jumped, "e' um pulo");
    assert!(!s.takeoff, "e NAO empurra o chao");
}

/// **Um pulo de PAREDE não gasta carga do ar** — a parede é apoio próprio.
#[test]
fn a_wall_jump_does_not_spend_an_air_charge() {
    let cfg = with_air(1, 2.0);
    let clinging = JumpState {
        airborne: true,
        air_jumps_left: 1,
        ..JumpState::default()
    };
    let wall = crate::WallLaunch {
        height: 2.0,
        away: [4.0, 0.0],
        lockout: 0.2,
    };
    let s = jump_step(
        &cfg,
        clinging,
        None,
        -1.0,
        true,
        false,
        Some(wall),
        G,
        UP,
        DT,
        DRY,
    );
    assert!(s.motor.boost[0] > 0.0, "o pulo de parede sai");
    assert_eq!(s.state.air_jumps_left, 1, "e a carga do ar continua la'");
    assert!(s.jumped, "e ele tambem e' um pulo");
}

/// **UM aperto é UM pulo, mesmo com muitas cargas.**
///
/// ⚠️ **Este é o gate que o `next.buffer = 0.0` protege**, e o modo de falha é
/// grande: o buffer sobrevive `jump_buffer` segundos, então sem o consumo o
/// mesmo aperto re-dispara em tiques CONSECUTIVOS e queima as três cargas em
/// ~6 tiques — três boosts empilhados, um foguete.
#[test]
fn one_press_burns_exactly_one_charge() {
    let cfg = with_air(3, 2.0);
    let falling = JumpState {
        airborne: true,
        air_jumps_left: 3,
        ..JumpState::default()
    };
    // Um press, e depois SEGURA — nenhuma borda nova.
    let mut st = step(&cfg, falling, None, -1.0, true).state;
    assert_eq!(st.air_jumps_left, 2, "o press gastou UMA");
    for i in 0..20 {
        let s = step(&cfg, st, None, -1.0, true);
        assert_eq!(
            s.motor.boost,
            [0.0, 0.0],
            "segurar nao re-dispara (tique {i}): {:?}",
            s.motor.boost
        );
        st = s.state;
    }
    assert_eq!(st.air_jumps_left, 2, "e as outras duas continuam la'");
}

/// **Com `air_jumps = 0` o mundo é o de antes desta wave.**
///
/// ⚠️ O controle é o perfil de partida INTEIRO (a capacidade nasce desligada),
/// então este gate também é o que prova que o default não move física nenhuma.
#[test]
fn zero_air_jumps_is_the_world_before_this_wave() {
    let cfg = JumpConfig::STARTING_POINT;
    assert_eq!(cfg.air_jumps, 0, "a capacidade nasce DESLIGADA");
    let first = tap(&cfg, JumpState::default(), Some(&ground()), 0.0);
    assert!(first.motor.boost[1] > 0.0, "o pulo do chao sai");
    let second = tap(&cfg, first.state, None, -1.0);
    assert_eq!(
        second.motor.boost,
        [0.0, 0.0],
        "e no ar nao ha segundo pulo: {:?}",
        second.motor.boost
    );
    assert!(!second.jumped, "nem `jumped`");
}
