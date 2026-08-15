//! Os gates do **READOUT** do player — o que a lei DIZ sobre si mesma
//! (`W-PlayerOut`), contra o `lib_tests.rs` ao lado, que mede o que ela
//! COMPUTA.
//!
//! ⚠️ **O corte é por ASSUNTO, e é o assunto que esta wave criou:** a
//! [`PlayerView`] é um canal novo — estado CONTÍNUO, publicável, sem
//! empréstimo — e os eventos (as TRANSIÇÕES) nascem ao lado dela, aqui, em vez
//! de voltarem a empurrar o pai contra o teto de LOC.
//!
//! ⚠️ Módulo FILHO (via `#[path]`), como os irmãos — é isso que mantém
//! `use super::*` a alcançar o que não é `pub`.

use super::*;

/// A capsula flutuante — o modo de partida destes gates.
const SPRING: Support = Support::Spring;

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
        brink: crate::Brink::NONE,
    }
}

/// **O READOUT diz o que a lei decidiu, nas TRÊS posturas** (`W-PlayerOut`).
///
/// ⚠️ **A postura publicada é a da porta única**, não uma segunda leitura do
/// sensor: a rampa de 60° está ao ALCANCE da perna (o raio a vê) e mesmo assim
/// não é chão — quem responde isso é a [`footing_verdict`], e um readout que
/// re-perguntasse ao `sample` diria *chão* ali.
#[test]
fn the_readout_names_the_posture_the_law_chose() {
    let cfg = PlayerConfig::STARTING_POINT;
    let flat = at(cfg.ride.float_height, [0.0, 1.0]);
    let steep = at(cfg.ride.float_height, [-0.866_025_4, 0.5]); // 60°
    let look = |sample: Option<&GroundSample>| {
        player_motor(
            &cfg,
            sample,
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
            SPRING,
        )
        .view
    };
    assert_eq!(look(Some(&flat)).footing, FootingKind::Ground);
    assert_eq!(look(Some(&steep)).footing, FootingKind::Steep);
    assert_eq!(look(None).footing, FootingKind::Airborne);

    // ⚠️ **E a normal publicada é a do chão ACEITE:** numa rampa recusada não há
    // plataforma que carregue ninguém, então ela é `None` — o mesmo raciocínio
    // da `ground_velocity`.
    assert!(look(Some(&flat)).ground_normal.is_some());
    assert!(
        look(Some(&steep)).ground_normal.is_none(),
        "uma rampa recusada nao e' chao, e o readout nao pode dizer que e'"
    );
}

/// **O `facing` sobrevive ao arranque DESLIGADO** (`W-PlayerOut`).
///
/// ⚠️ **É o gate que prova que o campo não é do arranque.** Ele morava dentro do
/// [`DashState`] e era escrito lá; com `dash_speed = 0` a capacidade não existe,
/// e um consumidor de animação continuaria a precisar de saber para que lado o
/// personagem olha — que é a razão de o campo ter mudado de casa.
///
/// ⚠️ E a 2ª metade é a lei que o par de asserções acima não alcança: **um eixo
/// neutro não vira ninguém para lugar nenhum**.
#[test]
fn the_facing_survives_the_dash_being_off() {
    let cfg = PlayerConfig::STARTING_POINT; // ⚠️ o arranque nasce DESLIGADO
    assert_eq!(cfg.dash.speed, 0.0, "o controle: a capacidade nao existe");
    let ground = at(cfg.ride.float_height, [0.0, 1.0]);
    let drive = |state: PlayerState, drive: f32| {
        player_motor(
            &cfg,
            Some(&ground),
            None,
            None,
            None,
            None,
            PlayerInput {
                drive,
                ..PlayerInput::default()
            },
            state,
            [0.0, 0.0],
            G,
            UP,
            DT,
            DRY,
            SPRING,
        )
    };
    let left = drive(PlayerState::default(), -1.0);
    assert_eq!(left.view.facing, -1.0, "ele virou-se para a esquerda");
    let stopped = drive(left.state, 0.0);
    assert_eq!(
        stopped.view.facing, -1.0,
        "parar de andar nao e' virar-se para lugar nenhum"
    );
}
