//! **A metade RODA da §12** (W-Pulley W1) — o que a seção da corda faz com as
//! roldanas dela.
//!
//! Irmão de [`super::inspector_joint`], e o corte é o assunto: lá mora *o que um
//! JOINT é* (os campos, o `clamped`, o gesto de criar); aqui, *o que a CORDA faz
//! com a lista de rodas dela*. Nasceu do cap de 600 LOC quando a roldana virou
//! entidade, e a próxima wave da polia (motor por roda, ruptura no centro) chega
//! aqui.

use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::PhysicsBridge;

/// Quantas roldanas apontam para esta corda, no mundo AUTORADO.
pub(crate) fn rope_wheel_count(sim: &mut SimWorld, joint: Entity) -> u32 {
    let Some(rope) = sim
        .world()
        .get::<Name>(joint)
        .map(|n| stable_name_id(n.as_str()))
    else {
        return 0;
    };
    let mut q = sim.world_mut().query::<&ph2d_physics_ecs::PulleyWheel>();
    u32::try_from(q.iter(sim.world()).filter(|w| w.rope == rope).count()).unwrap_or(u32::MAX)
}

/// **Acrescentar uma roldana a uma corda** — o pedido (4) do artista, *"escolher
/// o número de roldanas, em tempo real"*.
///
/// A roda nova entra ao FIM da rota, e **sobre a corda**: no meio do último
/// trecho que a rota desenha hoje. Duas razões, e as duas são sobre não
/// surpreender — ali o comprimento quase não muda (a corda já passava por aquele
/// ponto, e o que ela ganha é a diferença entre o arco e a corda do enlace, que é
/// pequena), e o artista vê a roda aparecer EM CIMA da corda que ele está
/// olhando, em vez de num canto que ele teria de ir procurar.
///
/// ⚠️ **Sem roldana nenhuma o "último trecho" é a corda inteira**, então a
/// primeira nasce no meio dela — que é onde uma roldana faria sentido.
///
/// O raio herda o da última roldana (ou o default), porque uma corda com rodas de
/// tamanhos aleatórios não é o que ninguém pede; e o `order` é o seguinte, então
/// a rota simplesmente cresce por onde ela já ia.
pub(crate) fn add_pulley_wheel(sim: &mut SimWorld, physics: &PhysicsBridge, joint_bits: u64) {
    let joint = Entity::from_bits(joint_bits);
    let Some(name) = sim
        .world()
        .get::<Name>(joint)
        .map(|n| n.as_str().to_string())
    else {
        return;
    };
    let rope = stable_name_id(&name);
    let Some(v) = physics.joint_views().find(|v| v.entity == joint) else {
        return;
    };
    // A geometria VIVA, pela mesma porta que o desenho usa — uma segunda
    // derivação poria a roda onde a corda não está.
    let wheels: Vec<_> = physics.rope_wheels(joint).map(|(_, w)| w).collect();
    let last = wheels.last().map_or(v.anchor_a, |w| w.centre);
    let centre = [
        0.5 * (last[0] + v.anchor_b[0]),
        0.5 * (last[1] + v.anchor_b[1]),
    ];
    let radius = wheels
        .last()
        .map_or(ph2d_physics_ecs::PulleyWheel::DEFAULT_RADIUS, |w| w.radius);
    let order = u16::try_from(wheels.len()).unwrap_or(u16::MAX);
    let label = crate::name_unique::unique_name(sim, &format!("{name} Wheel {}", order + 1));
    sim.world_mut().spawn((
        Name::new(label),
        ph2d_physics_ecs::PulleyWheel {
            rope,
            order,
            radius,
            wrap: ph2d_physics_ecs::WrapSide::Auto,
        },
        Transform::from_translation(ph2d_core::Vec2::new(centre[0], centre[1])),
    ));
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
}
