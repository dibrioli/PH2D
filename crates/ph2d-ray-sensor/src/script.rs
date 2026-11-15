//! ⭐ **O raio para o script** (RS-7, RS-8): o que o `ph2d.ray` e o `ph2d.raycast` respondem, lido da
//! ponte da física. Uma porta só para o lance — [`PhysicsBridge::raycast`], o mesmo `lanca` do objecto
//! «Ray Sensor» — e o filtro do tiro escreve-se num `RaySensor` pela MESMA conversão do `ph2d.set`
//! ([`ph2d_ecs::scene::property::set_in`]): os nomes do cartão, as referências pelo nome, os ângulos em
//! radianos.

use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::scene::property_ref::RefNames;
use ph2d_ecs::{Entity, World};
use ph2d_physics_ecs::{Collider, PhysicsBridge, RayHit, RaySensor};
use ph2d_script::{RayHitView, RayView};

/// As tags (caminhos) da forma `forma`.
fn tags(world: &World, tree: &ph2d_tags::TagTree, forma: Entity) -> Vec<String> {
    world
        .get::<ph2d_ecs::tags::Tags>(forma)
        .map(|t| {
            t.direct_ids()
                .filter_map(|id| tree.get(id).map(|t| t.path.clone()))
                .collect()
        })
        .unwrap_or_default()
}

/// Um acerto da física, como o script o lê.
#[must_use]
pub fn vista(world: &World, tree: &ph2d_tags::TagTree, h: &RayHit) -> RayHitView {
    let c = world.get::<Collider>(h.shape);
    RayHitView {
        object: h.body,
        shape: h.shape,
        point: [f64::from(h.point[0]), f64::from(h.point[1])],
        normal: [f64::from(h.normal[0]), f64::from(h.normal[1])],
        distance: f64::from(h.distance),
        tags: tags(world, tree, h.shape),
        friction: c.map(|c| f64::from(c.friction)),
        bounce: c.map(|c| f64::from(c.restitution)),
    }
}

/// ⭐ `ph2d.ray(e)`: `None` para quem não é um Ray Sensor. A lista é a do tique (`all_hits`/`cone`),
/// senão o mais perto.
#[must_use]
pub fn ray_view(
    world: &World,
    bridge: &PhysicsBridge,
    tree: &ph2d_tags::TagTree,
    e: Entity,
) -> Option<RayView> {
    world.get::<RaySensor>(e)?;
    let todos = bridge.ray_sensor_hits_all(e);
    let hits: Vec<RayHitView> = if todos.is_empty() {
        bridge
            .ray_sensor_hits()
            .get(&e)
            .into_iter()
            .map(|h| vista(world, tree, h))
            .collect()
    } else {
        todos.iter().map(|h| vista(world, tree, h)).collect()
    };
    Some(RayView {
        seeing: bridge.ray_seeing(e),
        hits,
    })
}

/// ⭐ `ph2d.raycast(de, até, filtro)`: o filtro por cima de um `RaySensor` de fábrica, nome a nome.
///
/// # Errors
/// Um nome que o Ray Sensor não tem, ou um valor que o campo não guarda (a frase do `ph2d.set`).
pub fn raycast(
    reg: &ComponentRegistry,
    world: &World,
    bridge: &PhysicsBridge,
    tree: &ph2d_tags::TagTree,
    (by, de, ate): (Option<Entity>, [f64; 2], [f64; 2]),
    filtro: &serde_json::Map<String, serde_json::Value>,
) -> Result<Vec<RayHitView>, String> {
    let mut r = serde_json::to_value(RaySensor::default()).map_err(|e| e.to_string())?;
    let names = RefNames { tags: Some(tree) };
    for (k, v) in filtro {
        let nome = format!("ray_sensor.{k}");
        ph2d_ecs::scene::property::set_in(reg, world, &mut r, &nome, v.clone(), names)
            .map_err(|e| e.to_string())?;
    }
    let mut r: RaySensor = serde_json::from_value(r).map_err(|e| e.to_string())?;
    crate::inspector::sanitize_ray_sensor(&mut r);
    #[allow(clippy::cast_possible_truncation)]
    let p = |v: [f64; 2]| [v[0] as f32, v[1] as f32];
    Ok(bridge
        .raycast(world, by, p(de), p(ate), &r)
        .iter()
        .map(|h| vista(world, tree, h))
        .collect())
}
