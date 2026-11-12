//! ⭐ **A curva do pulo pousa no TERRENO** (ficha `docs/GameEngineFuncional/componentes/04_curva_do_pulo.md`).
//!
//! A lei (`ph2d_platformer::arc`) é pura e desce abaixo do arranque; aqui cada curva é cortada no
//! primeiro toque com a geometria sólida da cena. Mora nesta crate porque é a que conhece as duas
//! metades — os componentes (`Collider`, `RigidBody`, as peças do ADR-0199, a escala) e a varredura
//! do `ph2d-physics` — sem subir de camada; o painel só a chama.
//!
//! A consulta lê a VERDADE DO ECS, nunca o mundo da ponte: com o relógio parado o BVH dela é o do
//! último `step`, e o artista edita por baixo dele.

use ph2d_ecs::{Entity, World};
use ph2d_physics::{LayerMatrix, PlacedShape, sweep_path};
use ph2d_platformer::arc::{ArcTouch, JumpArc, JumpArcs, TouchKind};

use crate::seam::ColliderPose;
use crate::{
    Collider, FormaDesligada, OneWayPlatform, PhysicsSettings, PlatformPlayer, RigidBody, parts,
};

/// A forma `e` posta no mundo (a pose autorada, a escala e o offset), ou `None` sem pose.
fn placed(world: &World, e: Entity, col: &Collider) -> Option<PlacedShape> {
    let t = ph2d_ecs::world_transform(world, e)?;
    let col = crate::shape_look::forma_resolvida(world, e, col);
    let p = ColliderPose::resolve(&col, &t);
    Some(PlacedShape {
        shape: p.shape,
        centre: p.centre,
        rotation: p.rotation,
        one_way: world.get::<OneWayPlatform>(e).is_some(),
    })
}

/// **O terreno que o pulo de `player` enxerga**: toda forma sólida de outro corpo — sem sensores,
/// sem formas desligadas, sem o próprio player e as peças dele — numa camada com que a dele colide.
#[must_use]
pub fn solid_terrain(
    world: &World,
    settings: &PhysicsSettings,
    player: Entity,
) -> Vec<PlacedShape> {
    let matrix = LayerMatrix::from_rows(settings.layer_matrix);
    let own_layer = parts::main_shape(world, player)
        .and_then(|m| world.get::<Collider>(m))
        .map_or(0, |c| c.layer);
    let mut out = Vec::new();
    let Some(mut q) = world.try_query::<(Entity, &Collider)>() else {
        return out;
    };
    for (e, col) in q.iter(world) {
        let owner = if world.get::<RigidBody>(e).is_some() {
            Some(e)
        } else {
            parts::owner_body(world, e)
        };
        let Some(owner) = owner else { continue };
        if owner == player
            || col.is_sensor
            || world.get::<FormaDesligada>(e).is_some()
            || parts::forma_desligada(world, e)
            || !matrix.collides(usize::from(own_layer), usize::from(col.layer))
        {
            continue;
        }
        out.extend(placed(world, e, col));
    }
    out
}

/// A forma principal do player relativa ao `chao` (o pé da curva), ou `None` sem forma.
fn body_from_foot(world: &World, player: Entity, chao: [f32; 2]) -> Option<PlacedShape> {
    let m = parts::main_shape(world, player)?;
    let mut p = placed(world, m, world.get::<Collider>(m)?)?;
    p.centre = [p.centre[0] - chao[0], p.centre[1] - chao[1]];
    p.one_way = false;
    Some(p)
}

/// **Pouso × choque**: o toque é um pouso quando a superfície olha para cima dentro do limite de
/// rampa do player (`normal·up > cos(max_slope)`, a mesma pergunta da perna); senão a cabeça ou o
/// lado bateram.
#[must_use]
pub fn touch_kind(normal: [f32; 2], up: [f32; 2], max_slope_cos: f32) -> TouchKind {
    if normal[0] * up[0] + normal[1] * up[1] > max_slope_cos {
        TouchKind::Land
    } else {
        TouchKind::Bump
    }
}

/// Corta `arc` (pontos relativos a `chao`) no primeiro toque; marca o ponto e o tipo em `touch`.
fn cut(
    arc: &mut JumpArc,
    chao: [f32; 2],
    body: Option<&PlacedShape>,
    terrain: &[PlacedShape],
    up: [f32; 2],
    slope_cos: f32,
) {
    let world_pts: Vec<[f32; 2]> = arc
        .points
        .iter()
        .map(|p| [chao[0] + p[0], chao[1] + p[1]])
        .collect();
    let Some(hit) = sweep_path(&world_pts, body, terrain, up) else {
        return;
    };
    let (a, b) = (arc.points[hit.segment], arc.points[hit.segment + 1]);
    let f = hit.fraction;
    let at = [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f];
    arc.points.truncate(hit.segment + 1);
    arc.points.push(at);
    arc.touch = Some(ArcTouch {
        at,
        contact: [hit.contact[0] - chao[0], hit.contact[1] - chao[1]],
        kind: touch_kind(hit.normal, up, slope_cos),
    });
}

/// **Corta todas as curvas de `player` no terreno da cena.** `chao` é o pé do arranque, em mundo.
pub fn cut_on_terrain(
    world: &World,
    settings: &PhysicsSettings,
    player: Entity,
    chao: [f32; 2],
    up: [f32; 2],
    arcs: &mut JumpArcs,
) {
    let terrain = solid_terrain(world, settings, player);
    let body = body_from_foot(world, player, chao);
    // Sem o componente, o limite de rampa de fábrica.
    let slope_cos = world
        .get::<PlatformPlayer>(player)
        .map_or_else(
            || PlatformPlayer::default().config(),
            PlatformPlayer::config,
        )
        .walk
        .max_slope_cos();
    for arc in std::iter::once(&mut arcs.full)
        .chain(std::iter::once(&mut arcs.short))
        .chain(std::iter::once(&mut arcs.standing))
        .chain(arcs.air.iter_mut())
    {
        cut(arc, chao, body.as_ref(), &terrain, up, slope_cos);
    }
}
