//! Os gates do **MODO DE SUPORTE** (`W-KinMove`) — o que muda, e o que NÃO
//! pode mudar, quando a perna deixa de ser uma mola.
//!
//! ⚠️ **Este arquivo não é um corte novo: ele é o capítulo que o
//! `lib_tests.rs` já declarava com um banner próprio**, mudado para um irmão
//! quando o pai cruzou o teto de LOC. O que o mantém coerente é a pergunta —
//! *a lei de intenção é a mesma nos dois modos?* — e o oráculo dela é sempre
//! uma SUBTRAÇÃO entre os dois, nunca uma tolerância.
//!
//! ⚠️ Módulo FILHO (via `#[path]`), como os irmãos.

use super::*;

/// Ar seco — todo gate deste arquivo mede o arco BALÍSTICO.
const DRY: Buoyed = Buoyed::DRY;

const UP: Vec2 = [0.0, 1.0];
const G: Vec2 = [0.0, -9.81];
const DT: f32 = 1.0 / 60.0;

/// Uma amostra de chão a `dist` do pé, com a normal dada.
fn at(dist: f32, normal: Vec2) -> GroundSample {
    GroundSample {
        grip: 1.0,
        distance: dist,
        normal,
        ground_velocity: [0.0, 0.0],
        one_way: false,
    }
}

// ═══ W-KinMove — O MODO DE SUPORTE ═══════════════════════════════════════

/// **A LEI DE INTENÇÃO é a MESMA nos dois modos** (K1) — só o que SEGURA muda.
///
/// ⚠️ É o gate que impede o modo novo de virar um segundo player: andar, virar
/// e frear têm de dar o mesmo número, e a única diferença permitida é o termo
/// da perna. O oráculo é a SUBTRAÇÃO — `dinâmico − cinemático == a mola` —, e
/// não uma tolerância: se a diferença fosse qualquer outra coisa, alguma outra
/// lei teria olhado para o modo.
#[test]
fn the_walk_is_identical_in_both_modes_and_only_the_leg_differs() {
    let cfg = PlayerConfig::STARTING_POINT;
    let ground = at(cfg.ride.float_height, UP);
    let input = PlayerInput {
        drive: 1.0,
        ..PlayerInput::default()
    };
    let call = |support| {
        player_motor(
            &cfg,
            Some(&ground),
            None,
            None,
            None,
            None,
            input,
            PlayerState::default(),
            [0.0, 0.0],
            G,
            UP,
            DT,
            DRY,
            support,
        )
    };
    let dynamic = call(Support::Spring);
    let kinematic = call(Support::Snap);
    let leg = ride_spring(&cfg.ride, Some(&ground), [0.0, 0.0], G, UP);
    assert_eq!(
        [
            dynamic.motor.accel[0] - kinematic.motor.accel[0],
            dynamic.motor.accel[1] - kinematic.motor.accel[1],
        ],
        leg.accel,
        "a UNICA diferenca entre os modos tem de ser a mola"
    );
    assert_eq!(
        dynamic.motor.boost[0] - kinematic.motor.boost[0],
        leg.boost[0],
        "e o boost dela"
    );
    assert_eq!(dynamic.nudge, kinematic.nudge, "a quina nao olha o modo");
    assert_eq!(
        dynamic.state, kinematic.state,
        "e o estado que a fita guarda tambem nao"
    );
}

/// **Sob Snap o chão sente o PESO, e a `gravity_hold` não reclama nada** (K6).
///
/// ⚠️ As duas metades são a wave: a 3ª lei **sobrevive** ao modo (é o que o Enio
/// pediu — *"o cinemático transmite peso ao chão"*) e o canal de cancelamento
/// **cala**, porque sob Snap não há termo de `− gravity` no `accel` para a ponte
/// subtrair.
///
/// # ⚠️ Duas coisas erradas na 1ª versão, e a MUTAÇÃO achou as duas
///
/// 1. **O oráculo chamava a função sob teste** (`ride_support_on_ground(Snap,…)`
///    para computar o que esperava) — a forma que este módulo já documentou três
///    vezes como *sempre verde*.
/// 2. **A fixture não continha o fenômeno:** ela punha o personagem exactamente
///    na `float_height`, onde o empurrão da mola **já é zero**. Os dois modos
///    coincidem ali por construção, então a mutação *"sob Snap o chão sente a
///    mola"* passava — não havia mola a sentir.
///
/// Agora ele está **COMPRIMIDO**, e o oráculo é o OUTRO MODO na altura de
/// repouso: ali a mola não contribui, logo o que sobra é o peso, e é a mesma
/// coisa que o Snap tem de transmitir a qualquer profundidade.
#[test]
fn under_snap_the_ground_feels_the_weight_and_nothing_claims_a_cancel() {
    let cfg = PlayerConfig::STARTING_POINT;
    // COMPRIMIDO: a mola empurra de verdade, então os dois modos divergem.
    let pressed = at(cfg.ride.float_height - 0.2, UP);
    let call = |sample, support| {
        player_motor(
            &cfg,
            Some(sample),
            None,
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
            support,
        )
    };
    let snap = call(&pressed, Support::Snap);
    assert_eq!(
        snap.gravity_hold,
        [0.0, 0.0],
        "sob Snap nao ha' cancelamento a declarar"
    );

    // O ORÁCULO: o mesmo personagem em modo mola, na altura de REPOUSO — ali o
    // empurrão é zero e o que o chão sente é só o peso.
    let at_rest = at(cfg.ride.float_height, UP);
    let weight_only = call(&at_rest, Support::Spring).reaction.expect("ha' chao");
    let felt = snap.reaction.expect("ha' chao, logo ha' em quem empurrar");
    assert_eq!(
        felt.accel, weight_only.accel,
        "o chao tem de sentir o PESO, e nada alem dele"
    );
    assert!(
        felt.accel[1] < 0.0,
        "e ele aponta para BAIXO: {:?}",
        felt.accel
    );

    // E a metade que prova que a fixture CONTÉM o fenômeno: comprimido, o modo
    // mola transmite MAIS que o peso. Sem esta linha o gate acima seria verde
    // sobre uma cena em que os dois modos não podiam diferir.
    let sprung = call(&pressed, Support::Spring).reaction.expect("ha' chao");
    assert!(
        sprung.accel[1] < felt.accel[1] - 1.0,
        "comprimido, a mola tem de transmitir MAIS que o peso: {:?} contra {:?}",
        sprung.accel,
        felt.accel
    );
}

/// **Sem chão os dois modos são INDISTINGUÍVEIS** — não há perna a calar.
///
/// ⚠️ **A 1ª versão deste gate afirmava `Motor::default()` e reprovou código
/// CORRETO**, repetindo palavra por palavra a armadilha que o
/// `the_spring_lets_go_of_a_wall` acima já documenta: eu QUIS dizer *"a perna
/// não empurra nada"* e ESCREVI *"o motor é zero"*, e no ar com `|v| ≈ 0` a
/// modelagem do ápice vale `+4,905` — metade da gravidade, para cima, e é a
/// feature. O oráculo de *"o modo não mudou nada aqui"* é o OUTRO MODO.
#[test]
fn in_the_air_the_two_modes_are_indistinguishable() {
    let cfg = PlayerConfig::STARTING_POINT;
    let call = |support| {
        player_motor(
            &cfg,
            None,
            None,
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
            support,
        )
    };
    let snap = call(Support::Snap);
    assert!(snap.reaction.is_none(), "sem chao nao ha' em quem empurrar");
    assert_eq!(
        snap,
        call(Support::Spring),
        "no ar nao ha' perna, logo nao ha' o que o modo mude"
    );
}
