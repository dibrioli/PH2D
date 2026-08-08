//! **A CENA DE ÁGUA** — a fixture que as sondas e os gates do player na água
//! compartilham.
//!
//! ⚠️ **Incluída por `#[path]`, nunca copiada** — o motivo é o mesmo do
//! `platform_scene.rs`: cada arquivo em `tests/` é um crate próprio, então uma
//! segunda cópia da poça seria uma segunda resposta a *"como é uma poça"*, e as
//! duas divergiriam no dia em que a densidade ou o arrasto mudassem, com o
//! arquivo não-atualizado seguindo VERDE sobre outra premissa.
//!
//! ⚠️ **O `allow(dead_code)` é do formato, não do descuido:** cada consumidor usa
//! o subconjunto de que precisa (o gate não constrói jangada; a sonda do pouso
//! não precisa de correnteza).
#![allow(dead_code)]

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    AreaBuoyancy, AreaDrag, AreaEffector, BodyKind, Collider, ColliderShape, LockRotation,
    PlatformPlayer, RigidBody,
};

/// A cápsula das fixtures do player (a mesma do `platform_scene`).
pub const HALF_H: f32 = 0.3;
pub const RADIUS: f32 = 0.2;
/// A altura de flutuação das fixtures do player.
pub const FLOAT: f32 = 0.9;
/// A densidade do fluido, 4× a do corpo — o mesmo par do `compound_zone`.
///
/// ⚠️ **A densidade DIFERENTE é o que torna um oráculo legível:** com densidades
/// iguais o empuxo é neutro e a linha d'água de equilíbrio desaparece.
pub const FLUID: f32 = 4.0;
/// O passo do relógio do módulo.
pub const DT: f32 = 1.0 / 60.0;

#[must_use]
pub fn capsule() -> Collider {
    Collider {
        shape: ColliderShape::Capsule {
            half_height: HALF_H,
            radius: RADIUS,
        },
        density: 1.0,
        ..Collider::default()
    }
}

/// A poça: um SENSOR com empuxo e arrasto, do `y = -6` ao `y = 0`.
///
/// ⚠️ **O arrasto não é enfeite** — empuxo sem resistência é uma mola sem
/// amortecimento, e o repouso que estes oráculos leem nunca chegaria (a lição
/// já escrita no `compound_zone`).
///
/// `current != 0` acrescenta uma correnteza horizontal, em newtons.
pub fn pool(sim: &mut SimWorld, current: f32) {
    let mut e = sim.world_mut().spawn((
        Name::new("Pool"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            is_sensor: true,
            shape: ColliderShape::Cuboid {
                half_x: 20.0,
                half_y: 3.0,
            },
            ..Collider::default()
        },
        AreaBuoyancy(FLUID),
        AreaDrag(0.6),
        Transform::from_translation(Vec2::new(0.0, -3.0)),
    ));
    if current != 0.0 {
        e.insert(AreaEffector {
            force: [current, 0.0],
        });
    }
}

/// Um chão SÓLIDO cujo topo fica em `top`.
pub fn floor(sim: &mut SimWorld, top: f32) {
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
        Transform::from_translation(Vec2::new(0.0, top - 0.5)),
    ));
}

/// Uma jangada — caixa larga e fina, livre para boiar.
pub fn raft(sim: &mut SimWorld, y: f32) {
    sim.world_mut().spawn((
        Name::new("Raft"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 1.5,
                half_y: 0.2,
            },
            density: 1.0,
            ..Collider::default()
        },
        LockRotation,
        Transform::from_translation(Vec2::new(0.0, y)),
    ));
}

/// O sujeito: a MESMA cápsula, com ou sem o `PlatformPlayer`.
///
/// ⚠️ **`player = false` é o CONTROLE, e é ele que dá o oráculo:** mesma forma,
/// mesma densidade, mesma poça. Tudo o que os dois fizerem diferente é do
/// player, e nenhum número desta família precisa de um literal.
pub fn subject(sim: &mut SimWorld, player: bool, y: f32) -> Entity {
    subject_tuned(sim, player, y, None)
}

/// O mesmo, com a config do player ajustada (`None` = os defaults do produto).
pub fn subject_tuned(
    sim: &mut SimWorld,
    player: bool,
    y: f32,
    tune: Option<PlatformPlayer>,
) -> Entity {
    let mut e = sim.world_mut().spawn((
        Name::new("Subject"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        capsule(),
        LockRotation,
        Transform::from_translation(Vec2::new(0.0, y)),
    ));
    if player {
        e.insert(tune.unwrap_or(PlatformPlayer {
            float_height: FLOAT,
            ..PlatformPlayer::default()
        }));
    }
    e.id()
}

/// O `y` de um corpo pelo nome.
#[must_use]
pub fn y_of(sim: &SimWorld, who: &str) -> f32 {
    xy_of(sim, who).1
}

/// A pose de um corpo pelo nome.
#[must_use]
pub fn xy_of(sim: &SimWorld, who: &str) -> (f32, f32) {
    let mut found = None;
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == who {
            found = Some((t.translation.x, t.translation.y));
        }
    }
    found.expect("o corpo tem de existir")
}
