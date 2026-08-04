//! **A CÁPSULA FLUTUANTE** (W2) — o personagem paira, e é aqui que o desenho
//! inteiro é julgado.
//!
//! A lei pura tem gates próprios na `ph2d-platformer` (dado um offset, qual
//! aceleração). Estes são de **COMPORTAMENTO**: o rapier de verdade, o corpo de
//! verdade, N ticks, e a pergunta que o kill-criterion do plano faz —
//!
//! > *se o personagem em repouso sobre chão ESTÁTICO oscilar com amplitude > 2%
//! > da `float_height` em regime, a cápsula flutuante é o desenho errado para o
//! > nosso solver e o plano volta à mesa.*
//!
//! Ele está medido em [`the_player_floats_and_settles`], com o número impresso.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, RigidBody,
};

/// Chão estático largo + um player caindo de 3 m.
///
/// ⚠️ `LockRotation` faz parte da fixture porque faz parte do DESENHO (D4 do
/// plano): em 2D trava-se a rotação de um personagem, como Unity e Godot fazem.
/// Sem ele a cápsula tomba e a perna vira um pêndulo — e o que este gate mede
/// deixa de ser a mola.
fn scene(float_height: f32) -> (SimWorld, PhysicsBridge) {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 20.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Player"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: 0.3,
                radius: 0.2,
            },
            ..Collider::default()
        },
        LockRotation,
        PlatformPlayer {
            float_height,
            ..PlatformPlayer::default()
        },
        Transform::from_translation(Vec2::new(0.0, 3.0)),
    ));
    (sim, PhysicsBridge::new())
}

fn player_y(sim: &SimWorld) -> f32 {
    let mut found = None;
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == "Player" {
            found = Some(t.translation.y);
        }
    }
    found.expect("o player tem de existir")
}

/// **O gate da wave.** Ele paira, assenta, e NÃO oscila.
///
/// ## Os números MEDIDOS (2026-08-03, `float_height = 0.9`)
///
/// | grandeza | valor |
/// |---|---|
/// | alvo (`0.5` do topo do chão + `float_height`) | 1,4000 m |
/// | repouso medido | 1,4057 m |
/// | **viés** | **+0,0057 m** |
/// | **ondulação em regime** | **0,00000 m** (0,00% da `float_height`) |
///
/// ⚠️ **O kill-criterion passa com folga total** — a ondulação é zero ao nível do
/// `f32`, não "pequena". A cápsula flutuante é o desenho certo para este solver.
///
/// ⚠️ **E o viés de 5,7 mm é NOMEADO, não escondido.** Ele é **constante**
/// (independe da `float_height`: o gate das duas alturas mede a diferença em
/// 0,6000 exato), o que o separa de um erro de lei — um erro na compensação de
/// gravidade seria *proporcional*. A causa provável é de discretização: o motor
/// entra como **um** impulso no topo do tick enquanto a gravidade é integrada
/// pelos sub-passos, e essa assimetria desloca o ponto de equilíbrio por algo da
/// ordem de `a·dt²`. Fica registrado como o número de hoje; se um dia alguém
/// quiser os 5,7 mm de volta, o caminho é aplicar o motor por SUB-PASSO (como o
/// `world::effector` faz), e o preço é medir de novo toda esta tabela.
#[test]
fn the_player_floats_and_settles() {
    let float_height = 0.9_f32;
    let (mut sim, mut bridge) = scene(float_height);

    // 4 s a 60 Hz: tempo de sobra para cair de 3 m e assentar.
    let mut ys = Vec::new();
    for tick in 1..=240 {
        bridge.dispatch(&mut sim, true, tick);
        ys.push(player_y(&sim));
    }

    // O topo do chão está em y = 0.5; o raio nasce no CENTRO do corpo, então o
    // repouso previsto é `0.5 + float_height`.
    let expected = 0.5 + float_height;
    let settled = &ys[180..]; // o último segundo
    let min = settled.iter().copied().fold(f32::INFINITY, f32::min);
    let max = settled.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mean = settled.iter().sum::<f32>() / settled.len() as f32;
    let ripple = max - min;

    eprintln!(
        "W2 KILL-CRITERION | alvo {expected:.4} | media {mean:.4} | vies {:+.4} m | \
         ondulacao {ripple:.5} m ({:.2}% da float_height)",
        mean - expected,
        100.0 * ripple / float_height
    );

    assert!(
        (mean - expected).abs() < 0.05,
        "o player tem de pairar na altura pedida: media {mean:.4}, alvo {expected:.4}"
    );
    assert!(
        ripple < 0.02 * float_height,
        "KILL-CRITERION: ondulacao {ripple:.5} m passa de 2% da float_height ({:.5})",
        0.02 * float_height
    );
}

/// ⚠️ **Sem o componente, nada muda** — a wave é byte-neutra para todo corpo que
/// não é player.
///
/// Sem este gate a mola poderia estar agindo em todo mundo e os outros testes
/// não notariam (eles não olham para corpos comuns).
#[test]
fn a_body_without_the_component_still_just_falls() {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 20.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Player"), // o nome é o mesmo; o que falta é o COMPONENTE
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.2 },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 3.0)),
    ));
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=240 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let y = player_y(&sim);
    assert!(
        y < 0.75,
        "um corpo comum assenta EM CIMA do chao (0.5 + raio 0.2), nao paira: {y:.4}"
    );
}

/// A altura de repouso **segue o que foi autorado** — dois players, duas
/// alturas, e a diferença é exatamente a diferença pedida.
///
/// É o gate que impede a mola de convergir para um número próprio (o modo de
/// falha em que a compensação de gravidade some: o personagem pararia onde o
/// erro da mola igualasse o peso, mais baixo do que o pedido, e um único
/// `float_height` não denunciaria).
#[test]
fn the_rest_height_is_the_one_that_was_authored() {
    let mut settled = Vec::new();
    for h in [0.7_f32, 1.3] {
        let (mut sim, mut bridge) = scene(h);
        for tick in 1..=240 {
            bridge.dispatch(&mut sim, true, tick);
        }
        settled.push(player_y(&sim));
    }
    let delta = settled[1] - settled[0];
    eprintln!(
        "alturas autoradas 0.7 e 1.3 -> repouso {:.4} e {:.4} (delta {delta:.4})",
        settled[0], settled[1]
    );
    assert!(
        (delta - 0.6).abs() < 0.05,
        "a diferenca de repouso tem de ser a diferenca autorada (0.6): {delta:.4}"
    );
}
