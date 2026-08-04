//! **A CENA do personagem** — a fixture que os gates de comportamento
//! compartilham (W3 anda, W4 pula).
//!
//! ⚠️ **Incluída por `#[path]` nos DOIS arquivos de gate**, e não copiada: cada
//! arquivo em `tests/` é um crate próprio, então uma segunda cópia da cena seria
//! uma segunda resposta a *"como é um personagem em pé num chão"* — e as duas
//! divergiriam no dia em que a cápsula ou a altura de flutuação mudassem, com a
//! wave que não foi atualizada seguindo VERDE sobre outra premissa.
//!
//! ⚠️ **`LockRotation` faz parte de toda fixture porque faz parte do DESENHO**
//! (D4 do plano): em 2D trava-se a rotação de um personagem, como Unity e Godot
//! fazem. Sem ele a cápsula tomba e a perna vira um pêndulo — e o que estes
//! gates medem deixa de ser a caminhada.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, RigidBody,
};

/// ⚠️ **0,9 e não o `0,5` do ponto de partida, e o motivo é GEOMETRIA.**
///
/// O sensor mede na vertical, mas quem encosta na rampa é a cápsula ao longo da
/// normal dela: flutuar de verdade exige
/// `float_height > half_height + radius / cos(max_slope)`
/// ([`ph2d_platformer::RideConfig::min_float_height`], com a tabela). Para a
/// cápsula destes gates (`0,3` / `0,2`) o mínimo já é `0,5` **no plano** — ou
/// seja, o ponto de partida a deixa TANGENTE, e qualquer inclinação a faz
/// penetrar. Com `0,9` ela paira até 70,5°, que cobre as duas rampas daqui.
pub const FLOAT_HEIGHT: f32 = 0.9;

/// Um chão (estático, opcionalmente inclinado) e um player em cima dele.
///
/// `slope_rad` gira o chão; o player nasce um pouco acima do ponto de contato
/// para a mola o pegar no primeiro tick.
pub fn scene(slope_rad: f32, start_x: f32) -> (SimWorld, PhysicsBridge, ph2d_ecs::Entity) {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 40.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform {
            rotation: slope_rad,
            ..Transform::from_translation(Vec2::new(0.0, 0.0))
        },
    ));
    // O topo do chão sob `start_x`, quando ele está inclinado, sobe com o x.
    let top = 0.5 / slope_rad.cos() + start_x * slope_rad.tan();
    let player = sim
        .world_mut()
        .spawn((
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
                float_height: FLOAT_HEIGHT,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(start_x, top + FLOAT_HEIGHT)),
        ))
        .id();
    (sim, PhysicsBridge::new(), player)
}

pub fn pose(sim: &SimWorld) -> (f32, f32) {
    let mut found = None;
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == "Player" {
            found = Some((t.translation.x, t.translation.y));
        }
    }
    found.expect("o player tem de existir")
}
