//! Os gates da lei do PULO — ver `jump.rs`.
use super::*;

const UP: Vec2 = [0.0, 1.0];
const G: Vec2 = [0.0, -9.81];
/// O tique de 60 Hz, o mesmo do produto.
const DT: f32 = 1.0 / 60.0;

fn ground() -> GroundSample {
    GroundSample {
        distance: 0.9,
        normal: [0.0, 1.0],
        ground_velocity: [0.0, 0.0],
        one_way: false,
    }
}

/// **A altura vira velocidade UMA vez, pela fórmula.**
///
/// `v₀ = √(2·g·h)`: com `h = 2` e `g = 9,81` são 6,264 m/s.
#[test]
fn the_takeoff_speed_comes_from_the_authored_height() {
    let cfg = JumpConfig::STARTING_POINT;
    let s = jump_step(
        &cfg,
        JumpState::default(),
        Some(&ground()),
        0.0,
        true,
        false,
        None,
        G,
        UP,
        DT,
    );
    let expected = (2.0 * 9.81 * 2.0_f32).sqrt();
    assert!(
        (s.motor.boost[1] - expected).abs() < 1.0e-3,
        "o boost tem de ser v0 = sqrt(2gh) = {expected:.4}: {:?}",
        s.motor.boost
    );
    assert!(s.state.airborne, "decolou");
    assert!(!s.spring_armed, "e a perna CALA");
}

/// ⚠️ **Pular já subindo dá a MESMA altura** — o boost leva AO valor, não
/// soma a ele.
///
/// É a frase inteira do *"parametrizado por altura"*: sem isto, pular de uma
/// plataforma que sobe daria um pulo mais alto do que o autorado, e o número
/// na row deixaria de descrever o que acontece.
#[test]
fn jumping_while_already_rising_reaches_the_same_height() {
    let cfg = JumpConfig::STARTING_POINT;
    let v0 = (2.0 * 9.81 * 2.0_f32).sqrt();
    for start in [0.0_f32, 2.0, -1.0] {
        let s = jump_step(
            &cfg,
            JumpState::default(),
            Some(&ground()),
            start,
            true,
            false,
            None,
            G,
            UP,
            DT,
        );
        let after = start + s.motor.boost[1];
        assert!(
            (after - v0).abs() < 1.0e-3,
            "partindo de {start}: a velocidade final tem de ser {v0:.4}, deu {after:.4}"
        );
    }
}

/// ⚠️ **Segurar a tecla NÃO re-pula** — a decolagem é na BORDA.
///
/// Sem isto, um dedo apoiado no botão daria um impulso por tick e o
/// personagem subiria para sempre.
#[test]
fn holding_the_button_does_not_re_jump() {
    let cfg = JumpConfig::STARTING_POINT;
    let first = jump_step(
        &cfg,
        JumpState::default(),
        Some(&ground()),
        0.0,
        true,
        false,
        None,
        G,
        UP,
        DT,
    );
    assert!(first.motor.boost[1] > 0.0);
    // O tick seguinte, ainda com a tecla presa e ainda vendo o chão.
    let second = jump_step(
        &cfg,
        first.state,
        Some(&ground()),
        3.0,
        true,
        false,
        None,
        G,
        UP,
        DT,
    );
    assert_eq!(second.motor.boost, [0.0, 0.0], "nada de segundo impulso");
}

/// **Nem pular no AR** — o pulo duplo é uma feature, não um acidente.
#[test]
fn a_second_press_in_mid_air_does_nothing() {
    let cfg = JumpConfig::STARTING_POINT;
    let flying = JumpState {
        airborne: true,
        cut: false,
        was_held: false,
        ..JumpState::default()
    };
    let s = jump_step(&cfg, flying, None, 3.0, true, false, None, G, UP, DT);
    assert_eq!(s.motor.boost, [0.0, 0.0]);
}

/// **Soltar durante a subida ARMA o corte, e ele não desarma.**
///
/// Segurar de novo no ar não devolve a altura — senão ela seria função de
/// quantas vezes o jogador tamborilou o dedo.
#[test]
fn releasing_arms_the_cut_and_re_holding_does_not_undo_it() {
    let cfg = JumpConfig::STARTING_POINT;
    let flying = JumpState {
        airborne: true,
        cut: false,
        was_held: true,
        ..JumpState::default()
    };
    let released = jump_step(&cfg, flying, None, 4.0, false, false, None, G, UP, DT);
    assert!(released.state.cut, "soltar arma o corte");
    assert!(
        (released.motor.accel[1] - G[1] * (cfg.cut_gravity - 1.0)).abs() < 1.0e-4,
        "e o corte pesa: {:?}",
        released.motor.accel
    );

    let re_held = jump_step(
        &cfg,
        released.state,
        None,
        3.0,
        true,
        false,
        None,
        G,
        UP,
        DT,
    );
    assert!(re_held.state.cut, "segurar de novo NAO desfaz o corte");
}

/// **As quatro fases dão quatro gravidades**, e a do ápice é a única ABAIXO
/// de 1 — a decisão do módulo.
#[test]
fn each_phase_gets_its_own_gravity() {
    let cfg = JumpConfig::STARTING_POINT;
    let flying = JumpState {
        airborne: true,
        cut: false,
        was_held: true,
        ..JumpState::default()
    };
    let scale = |rel: f32, st: JumpState, held: bool| {
        let s = jump_step(&cfg, st, None, rel, held, false, None, G, UP, DT);
        s.motor.accel[1] / G[1] + 1.0
    };
    let rising = scale(5.0, flying, true);
    let peak = scale(0.0, flying, true);
    let falling = scale(-5.0, flying, true);
    let cut = scale(
        5.0,
        JumpState {
            cut: true,
            ..flying
        },
        true,
    );
    assert!(
        (rising - 1.0).abs() < 1.0e-4,
        "subindo (takeoff inerte): {rising}"
    );
    assert!((peak - cfg.peak_gravity).abs() < 1.0e-4, "apice: {peak}");
    assert!(
        (falling - cfg.fall_gravity).abs() < 1.0e-4,
        "queda: {falling}"
    );
    assert!((cut - cfg.cut_gravity).abs() < 1.0e-4, "corte: {cut}");
    assert!(
        peak < 1.0,
        "⚠️ o apice ALONGA (Celeste), nao encurta (tnua): {peak}"
    );
}

/// ⚠️ **Multiplicadores em `1.0` são gravidade extra ZERO** — o mundo sem
/// pulo, byte a byte. É o que torna cada knob uma escolha e não um imposto.
#[test]
fn neutral_multipliers_add_exactly_nothing() {
    let cfg = JumpConfig {
        takeoff_gravity: 1.0,
        peak_gravity: 1.0,
        fall_gravity: 1.0,
        cut_gravity: 1.0,
        ..JumpConfig::STARTING_POINT
    };
    let flying = JumpState {
        airborne: true,
        cut: true,
        was_held: false,
        ..JumpState::default()
    };
    for rel in [-9.0_f32, -1.0, 0.0, 1.0, 9.0] {
        let s = jump_step(&cfg, flying, None, rel, false, false, None, G, UP, DT);
        assert_eq!(s.motor.accel, [0.0, 0.0], "a {rel} m/s");
    }
}

/// **O pouso pede as DUAS metades** — descendo E com chão ao alcance.
///
/// Só "achou chão" pousaria no tick da decolagem (o raio ainda vê o chão) e
/// a mola puxaria de volta o pulo inteiro; só "descendo" nunca pousaria.
#[test]
fn landing_needs_both_halves() {
    let cfg = JumpConfig::STARTING_POINT;
    let flying = JumpState {
        airborne: true,
        cut: false,
        was_held: false,
        ..JumpState::default()
    };
    // Subindo COM chão ao alcance (o tick logo após a decolagem): não pousa.
    let rising = jump_step(
        &cfg,
        flying,
        Some(&ground()),
        5.0,
        false,
        false,
        None,
        G,
        UP,
        DT,
    );
    assert!(rising.state.airborne, "ainda subindo, nao pousou");
    assert!(!rising.spring_armed, "e a perna segue CALADA");

    // Descendo SEM chão: também não.
    let falling = jump_step(&cfg, flying, None, -5.0, false, false, None, G, UP, DT);
    assert!(falling.state.airborne);

    // Descendo COM chão: pousou, e a perna volta.
    let landed = jump_step(
        &cfg,
        flying,
        Some(&ground()),
        -5.0,
        false,
        false,
        None,
        G,
        UP,
        DT,
    );
    assert!(!landed.state.airborne, "pousou");
    assert!(landed.spring_armed, "e a perna volta a agir");
}

/// ⚠️ **O pouso limpa o CORTE também, e ele é alcançável.**
///
/// O corte é lido num sítio só — subindo acima do `peak_speed` — e no chão a
/// perna arma primeiro, então um corte esquecido parece inofensivo. Não é: se
/// o personagem **sai do chão SUBINDO sem pular** (andar para fora da borda de
/// uma plataforma kinematic que ASCENDE, que este produto já tem desde o W4),
/// não há decolagem para zerá-lo — e a subida ganha `cut_gravity` (4×) onde
/// devia ganhar `takeoff_gravity` (1×), puxando o personagem para baixo por um
/// corte de um pulo que acabou.
///
/// ⚠️ Este gate nasceu de uma mutação que **sobreviveu à suíte inteira** (a de
/// comportamento inclusive): tirar o `next.cut = false` do pouso não move um
/// milímetro em nenhuma cena que só pula e cai.
#[test]
fn landing_clears_the_cut_so_the_next_rise_is_not_punished() {
    let cfg = JumpConfig::STARTING_POINT;
    let cut_in_flight = JumpState {
        airborne: true,
        cut: true,
        was_held: false,
        ..JumpState::default()
    };

    // Pousa com o corte armado.
    let landed = jump_step(
        &cfg,
        cut_in_flight,
        Some(&ground()),
        -5.0,
        false,
        false,
        None,
        G,
        UP,
        DT,
    );
    assert!(!landed.state.airborne, "pousou");
    assert!(!landed.state.cut, "e o corte MORREU com o pouso");

    // Agora sai do chão SUBINDO sem pular (a plataforma o levou).
    let rising = jump_step(&cfg, landed.state, None, 5.0, false, false, None, G, UP, DT);
    let scale = rising.motor.accel[1] / G[1] + 1.0;
    assert!(
        (scale - cfg.takeoff_gravity).abs() < 1.0e-4,
        "subir sem pular usa a gravidade de SAIDA, nao a do corte: {scale}"
    );
}

/// Sem pulo nenhum, andar para fora de uma borda ainda ganha a gravidade de
/// QUEDA — ela é sobre cair, não sobre pular.
#[test]
fn walking_off_a_ledge_still_falls_faster() {
    let cfg = JumpConfig::STARTING_POINT;
    let s = jump_step(
        &cfg,
        JumpState::default(),
        None,
        -5.0,
        false,
        false,
        None,
        G,
        UP,
        DT,
    );
    assert!(!s.state.airborne, "ele nao pulou");
    assert!(
        (s.motor.accel[1] - G[1] * (cfg.fall_gravity - 1.0)).abs() < 1.0e-4,
        "mas cai com a gravidade de queda: {:?}",
        s.motor.accel
    );
}

#[path = "jump_forgive_tests.rs"]
mod forgive;
