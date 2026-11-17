//! **A cena dos GABARITOS de junta** (`PH2D_PHYSICS_SMOKE=123`, Joint Editor W1).
//!
//! Quatro máquinas montadas pela porta dos gabaritos (`joint_assembly::assemble`) sobre peças comuns,
//! com as formas-guia do catálogo: Pin · Door (o batente é o cenário), Rope · Bridge (com uma caixa
//! a cair no meio), Slider · Elevator e Wheel · Car. Cada montagem é um objeto na Hierarchy («Door 1»,
//! «Bridge 1», «Elevator 1», «Car 1»). PAUSADA: o dono dá Play.

use ph2d_ecs::{Name, SimWorld, World};
use ph2d_joint_factory::joint_factory::{block, make_parts};
use ph2d_joint_templates::{Guide, JointKind, by_name};
use ph2d_physics_ecs::{BodyKind, Collider, InitialVelocity};

const GROUND: [f32; 4] = [0.40, 0.42, 0.48, 1.0];
const LOAD: [f32; 4] = [0.45, 0.80, 0.95, 1.0];

/// As quatro máquinas: o gabarito, onde fica, e se as peças de cenário ficam por conta do mundo.
pub const MACHINES: [(JointKind, &str, f32, bool); 4] = [
    (JointKind::Pin, "Door", 2.5, true),
    (JointKind::Pin, "Bridge", 7.0, false),
    (JointKind::Slider, "Elevator", 11.5, false),
    (JointKind::Wheel, "Car", 14.5, false),
];

/// Enquadra as quatro e a pista do carro até ao muro (fotografado a 1930×1040, cartões abertos).
const CAMERA_CENTRE: [f32; 2] = [12.0, 3.0];
const CAMERA_HEIGHT: f32 = 16.0;
/// O muro onde o carro para, ainda à vista com o painel da direita aberto (foto de 12/11).
pub const END_WALL_X: f32 = 21.0;

fn wall(sim: &mut SimWorld, name: &str, half: [f32; 2], at: [f32; 2]) {
    block(
        sim,
        name,
        Guide::Box { half },
        Some(GROUND),
        at,
        BodyKind::Static,
    );
}

/// **Cena 123.** As quatro máquinas, o chão e um muro no fim da pista do carro. PAUSADA.
pub fn physics_smoke_joint_templates(ctx: &mut crate::SceneCtx<'_>) {
    build(ctx.world);
    ctx.want.camera_center = Some(CAMERA_CENTRE);
    ctx.want.camera_height_world = Some(CAMERA_HEIGHT);
    // As fotos do cartão (W2): com `PH2D_PHYSICS_SMOKE_SELECT` abre com esse objeto selecionado e o
    // Inspector à frente (o painel Physics cobri-lo-ia na mesma doca).
    let Ok(nome) = std::env::var("PH2D_PHYSICS_SMOKE_SELECT") else {
        ctx.want.panels.push("physics");
        return;
    };
    {
        let mut q = ctx.world.query::<(ph2d_ecs::Entity, &Name)>();
        ctx.want.select = q
            .iter(ctx.world)
            .find(|(_, n)| n.0 == nome.trim())
            .map(|(e, _)| e.to_bits());
    }
}

/// A cena sobre um `World` cru — a mesma que o gate mede.
pub fn build(world: &mut World) {
    // As peças e a montagem pedem o `SimWorld` (nomes únicos): o mundo da cena entra e sai dele inteiro.
    let mut sim = SimWorld::new();
    std::mem::swap(sim.world_mut(), world);
    wall(&mut sim, "Ground", [11.0, 0.5], [10.5, -0.5]);
    wall(&mut sim, "End Wall", [0.25, 1.5], [END_WALL_X, 1.5]);
    for (kind, name, x0, scenery_is_world) in MACHINES {
        let Some(t) = by_name(kind, name) else {
            continue;
        };
        let p = make_parts(&mut sim, t, [x0, 0.0], &[], scenery_is_world);
        // Um gabarito que recusa as próprias peças é um defeito que o gate acusa; a cena segue.
        let _ = ph2d_joint_assembly::joint_assembly::assemble(&mut sim, t, &p);
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
    let half = [0.25, 0.25];
    let c = block(
        &mut sim,
        "Crate",
        Guide::Box { half },
        Some(LOAD),
        [7.0, 3.0],
        BodyKind::Dynamic,
    );
    let w = sim.world_mut();
    // 0,25 m² × 40 = 10 kg: o peso com que o gate mede a ponte.
    if let Some(f) = ph2d_physics_ecs::forma_para_editar(w, c)
        && let Some(mut col) = w.get_mut::<Collider>(f)
    {
        col.density = 40.0;
    }
    std::mem::swap(sim.world_mut(), world);
}

#[cfg(test)]
#[path = "physics_smoke_joint_templates_tests.rs"]
mod tests;
