//! **A cena dos GABARITOS de junta** (`PH2D_PHYSICS_SMOKE=123`, Joint Editor W1).
//!
//! Quatro máquinas montadas pela porta dos gabaritos (`joint_assembly::assemble`) sobre peças comuns,
//! com as formas-guia do catálogo: Pin · Door (o batente é o cenário), Rope · Bridge (com uma caixa
//! a cair no meio), Slider · Elevator e Wheel · Car. Cada montagem é um objeto na Hierarchy («Door 1»,
//! «Bridge 1», «Elevator 1», «Car 1»). PAUSADA: o dono dá Play.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, World};
use ph2d_joint_templates::{Guide, JointKind, Template, by_name};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, InitialVelocity, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

const GROUND: [f32; 4] = [0.40, 0.42, 0.48, 1.0];
const SCENERY: [f32; 4] = [0.62, 0.62, 0.68, 1.0];
const PART: [f32; 4] = [0.95, 0.62, 0.25, 1.0];
const TYRE: [f32; 4] = [0.22, 0.22, 0.26, 1.0];
const LOAD: [f32; 4] = [0.45, 0.80, 0.95, 1.0];

/// As quatro máquinas: o gabarito, onde fica, e se as peças de cenário ficam por conta do mundo.
pub const MACHINES: [(JointKind, &str, f32, bool); 4] = [
    (JointKind::Pin, "Door", 2.5, true),
    (JointKind::Rope, "Bridge", 7.0, false),
    (JointKind::Slider, "Elevator", 11.5, false),
    (JointKind::Wheel, "Car", 14.5, false),
];

/// Enquadra as quatro e a pista do carro até ao muro (fotografado a 1930×1040, cartões abertos).
const CAMERA_CENTRE: [f32; 2] = [12.0, 3.0];
const CAMERA_HEIGHT: f32 = 16.0;
/// O muro onde o carro para, ainda à vista com o painel da direita aberto (foto de 12/11).
const END_WALL_X: f32 = 21.0;

fn block(
    world: &mut World,
    name: &str,
    shape: ColliderShape,
    size: [f32; 2],
    rgba: [f32; 4],
    at: [f32; 2],
    kind: BodyKind,
) -> Entity {
    world
        .spawn((
            Transform::from_translation(Vec2::new(at[0], at[1])),
            Sprite::atlas(WHITE_TILE_KEY, size, rgba),
            Name::new(name.to_owned()),
            RigidBody { kind },
            Collider {
                shape,
                friction: 1.0,
                ..Collider::default()
            },
        ))
        .id()
}

fn wall(world: &mut World, name: &str, half: [f32; 2], at: [f32; 2]) {
    let shape = ColliderShape::Cuboid {
        half_x: half[0],
        half_y: half[1],
    };
    let size = [half[0] * 2.0, half[1] * 2.0];
    block(world, name, shape, size, GROUND, at, BodyKind::Static);
}

/// As peças de fábrica do gabarito em `x0`. Devolve `(papel, peça)`, `None` = o cenário.
fn parts(
    world: &mut World,
    t: &Template,
    x0: f32,
    scenery_is_world: bool,
) -> Vec<(u16, Option<Entity>)> {
    let mut out = Vec::new();
    for (i, role) in t.roles.iter().enumerate() {
        let i = u16::try_from(i).unwrap_or(u16::MAX);
        if role.may_be_world && scenery_is_world {
            out.push((i, None));
            continue;
        }
        let (count, step) = role.many.map_or((1, [0.0, 0.0]), |m| (m.count, m.step));
        for n in 0..count {
            let k = f32::from(n);
            let at = [x0 + role.at[0] + step[0] * k, role.at[1] + step[1] * k];
            let name = t.part_name(i, n);
            let (shape, size, rgba) = match role.guide {
                Guide::Box { half } => (
                    ColliderShape::Cuboid {
                        half_x: half[0],
                        half_y: half[1],
                    },
                    [half[0] * 2.0, half[1] * 2.0],
                    if role.fixed { SCENERY } else { PART },
                ),
                Guide::Disc { radius } => (
                    ColliderShape::Ball { radius },
                    [radius * 2.0, radius * 2.0],
                    TYRE,
                ),
            };
            let kind = if role.fixed {
                BodyKind::Static
            } else {
                BodyKind::Dynamic
            };
            out.push((i, Some(block(world, &name, shape, size, rgba, at, kind))));
        }
    }
    out
}

/// **Cena 123.** As quatro máquinas, o chão e um muro no fim da pista do carro. PAUSADA.
pub fn physics_smoke_joint_templates(ctx: &mut crate::SceneCtx<'_>) {
    build(ctx.world);
    ctx.want.camera_center = Some(CAMERA_CENTRE);
    ctx.want.camera_height_world = Some(CAMERA_HEIGHT);
    ctx.want.panels.push("physics");
}

/// A cena sobre um `World` cru — a mesma que o gate mede.
pub fn build(world: &mut World) {
    wall(world, "Ground", [11.0, 0.5], [10.5, -0.5]);
    wall(world, "End Wall", [0.25, 1.5], [END_WALL_X, 1.5]);
    // A montagem pede o `SimWorld` (nomes únicos): o mundo da cena entra e sai dele inteiro.
    let mut sim = SimWorld::new();
    std::mem::swap(sim.world_mut(), world);
    for (kind, name, x0, scenery_is_world) in MACHINES {
        let Some(t) = by_name(kind, name) else {
            continue;
        };
        let p = parts(sim.world_mut(), t, x0, scenery_is_world);
        // Um gabarito que recusa as próprias peças é um defeito que o gate acusa; a cena segue.
        let _ = crate::joint_assembly::assemble(&mut sim, t, &p);
        if kind == JointKind::Pin
            && let Some(&(_, Some(door))) = p.iter().find(|(r, _)| *r == 1)
        {
            // Um empurrão ao dar Play: a porta abre até o batente e volta.
            sim.world_mut().entity_mut(door).insert(InitialVelocity {
                linvel: [0.0, 0.0],
                angvel: 6.0,
            });
        }
    }
    let crate_shape = ColliderShape::Cuboid {
        half_x: 0.25,
        half_y: 0.25,
    };
    let w = sim.world_mut();
    let c = block(
        w,
        "Crate",
        crate_shape,
        [0.5, 0.5],
        LOAD,
        [7.0, 3.0],
        BodyKind::Dynamic,
    );
    // 0,25 m² × 40 = 10 kg: o peso com que o gate mede a ponte.
    if let Some(mut col) = w.get_mut::<Collider>(c) {
        col.density = 40.0;
    }
    std::mem::swap(sim.world_mut(), world);
}

#[cfg(test)]
#[path = "physics_smoke_joint_templates_tests.rs"]
mod tests;
