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
use ph2d_physics::world::layer_override::{Side, pair_collides};
use ph2d_physics::{LayerMatrix, PlacedShape, RayHit, Terrain};
use ph2d_platformer::arc::{ArcTouch, JumpArc, JumpArcs, TouchKind};
use ph2d_platformer::{MAX_WALL_SAMPLES, PlayerConfig, StepLimit, odd_samples, wall_offsets};

use crate::seam::ColliderPose;
use crate::{
    Collider, FormaDesligada, OneWayPlatform, PhysicsSettings, PlatformPlayer, RigidBody, excecao,
    parts,
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

/// O lado de camadas da forma `e` (a camada e a exceção dela), o mesmo que a ponte dá ao colisor.
fn side(world: &World, e: Entity, col: &Collider) -> Side {
    let [include, exclude] = excecao::mascaras(world, e);
    Side {
        layer: usize::from(col.layer),
        include,
        exclude,
    }
}

/// **O terreno que o pulo de `player` enxerga**: toda forma sólida de outro corpo — sem sensores,
/// sem formas desligadas, sem o próprio player e as peças dele — com que a forma principal dele
/// colide pela regra dos contactos (`pair_collides`: a matriz e a exceção de CADA lado, a dele e
/// a da forma da cena; a perna do jogo pergunta em nome da mesma forma).
#[must_use]
pub fn solid_terrain(
    world: &World,
    settings: &PhysicsSettings,
    player: Entity,
) -> Vec<PlacedShape> {
    let matrix = LayerMatrix::from_rows(settings.layer_matrix);
    let own = parts::main_shape(world, player)
        .and_then(|m| Some(side(world, m, world.get::<Collider>(m)?)))
        .unwrap_or(Side {
            layer: 0,
            include: 0,
            exclude: 0,
        });
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
            || !pair_collides(own, side(world, e, col), matrix)
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

/// **A perna do jogo**, a régua do pouso (`bridge/player_leg.rs::cast_leg`): o leque de raios do
/// centro do corpo, para baixo, com o alcance `float_height + cling_distance` — o corpo pousa no
/// tique em que ela agarra chão caminhável a descer, não quando o pé cruza a linha (abaixo de
/// ~1 m de Jump Height eram 2 tiques: a mola puxa-o para baixo dentro do `cling_distance`).
struct Leg {
    /// Centro do corpo relativo ao pé.
    centre: [f32; 2],
    offsets: [f32; MAX_WALL_SAMPLES],
    n: usize,
    reach: f32,
    /// O degrau caminhável — a MESMA régua da perna do jogo.
    step: StepLimit,
    slope_cos: f32,
}

impl Leg {
    fn new(cfg: &PlayerConfig, body: Option<&PlacedShape>) -> Self {
        let c = cfg.walk.max_slope_cos();
        let step = cfg.walk.step_limit();
        let (centre, half_width) = body.map_or(([0.0, cfg.ride.float_height], 0.0), |b| {
            let (lo, hi) = b.aabb();
            (b.centre, (hi[0] - lo[0]) * 0.5)
        });
        let (offsets, n) = if half_width > 0.0 && half_width.is_finite() {
            (
                wall_offsets(half_width, cfg.ride.samples, cfg.ride.spread),
                odd_samples(cfg.ride.samples, MAX_WALL_SAMPLES),
            )
        } else {
            ([0.0; MAX_WALL_SAMPLES], 1)
        };
        Self {
            centre,
            offsets,
            n,
            reach: cfg.ride.float_height + cfg.ride.cling_distance,
            step,
            slope_cos: c,
        }
    }

    /// O chão que a perna agarra com o pé em `foot` (mundo): o mais próximo, caminhável.
    fn grab(&self, terrain: &Terrain, foot: [f32; 2], up: [f32; 2]) -> Option<RayHit> {
        let o = [foot[0] + self.centre[0], foot[1] + self.centre[1]];
        let down = [-up[0], -up[1]];
        let mut mid: Option<f32> = None;
        let mut best: Option<RayHit> = None;
        for (i, off) in self.offsets.iter().enumerate().take(self.n) {
            let at = [o[0] + up[1] * off, o[1] - up[0] * off];
            let Some(h) = terrain.cast_ray(at, down, self.reach, up) else {
                continue;
            };
            if i == 0 {
                mid = Some(h.distance);
            } else if let Some(c) = mid
                && self.step.refuses(c - h.distance, off.abs())
            {
                continue;
            }
            if best.is_none_or(|b| h.distance < b.distance) {
                best = Some(h);
            }
        }
        best.filter(|h| h.normal[0] * up[0] + h.normal[1] * up[1] >= self.slope_cos)
    }
}

fn sub(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

fn dot(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[0] + a[1] * b[1]
}

/// Corta `arc` (pontos relativos a `chao`) no primeiro toque; marca o ponto e o tipo em `touch`.
///
/// O pouso é o tique `k + 1` em que a perna, lida na pose `k` a descer, agarra o chão (o jogo lê a
/// perna da pose do fim do tique anterior); o traço acaba na pose `k + 1` com o pé na superfície.
/// O corpo varrido pára antes nas paredes e tetos.
fn cut(
    arc: &mut JumpArc,
    chao: [f32; 2],
    body: Option<&PlacedShape>,
    terrain: &Terrain,
    leg: &Leg,
    up: [f32; 2],
    slope_cos: f32,
) {
    let world_pts: Vec<[f32; 2]> = arc
        .points
        .iter()
        .map(|p| [chao[0] + p[0], chao[1] + p[1]])
        .collect();
    let n = world_pts.len();
    let grab = (1..n).find_map(|k| {
        // A descer no fim do tique `k`: o passo seguinte desce (o jogo lê a velocidade do fim do
        // tique, não a média dele — no ápice as duas discordam de sinal).
        let (a, b) = if k + 1 < n { (k, k + 1) } else { (k - 1, k) };
        let falling = dot(sub(world_pts[b], world_pts[a]), up) <= 0.0;
        falling
            .then(|| leg.grab(terrain, world_pts[k], up))
            .flatten()
            .map(|h| (k, h))
    });
    // O corpo varre até à pose do pouso; um choque no MESMO segmento do pouso é a perna que ganha.
    let last = grab.map_or(world_pts.len(), |(k, _)| (k + 2).min(world_pts.len()));
    let bump = body
        .and_then(|b| terrain.sweep_body(&world_pts[..last], b))
        .filter(|h| grab.is_none_or(|(k, _)| h.segment < k));
    if let Some(hit) = bump {
        let (a, b) = (arc.points[hit.segment], arc.points[hit.segment + 1]);
        let f = hit.fraction;
        let at = [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f];
        arc.points.truncate(hit.segment + 1);
        arc.points.push(at);
        arc.touch = Some(ArcTouch {
            at,
            contact: sub(hit.contact, chao),
            kind: touch_kind(hit.normal, up, slope_cos),
        });
    } else if let Some((k, h)) = grab {
        // O pé sobre a superfície que a perna achou, na pose seguinte (o lado anda, a mola desce).
        let next = arc.points[(k + 1).min(arc.points.len() - 1)];
        let height = dot(sub(next, arc.points[k]), up) + h.distance - dot(leg.centre, up);
        let at = [next[0] - up[0] * height, next[1] - up[1] * height];
        arc.points.truncate(k + 1);
        arc.points.push(at);
        arc.touch = Some(ArcTouch {
            at,
            contact: sub(h.contact, chao),
            kind: TouchKind::Land,
        });
    }
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
    let terrain = Terrain::new(&solid_terrain(world, settings, player));
    let body = body_from_foot(world, player, chao);
    // Sem o componente, a configuração de fábrica.
    let cfg = world.get::<PlatformPlayer>(player).map_or_else(
        || PlatformPlayer::default().config(),
        PlatformPlayer::config,
    );
    let leg = Leg::new(&cfg, body.as_ref());
    let slope_cos = cfg.walk.max_slope_cos();
    for arc in std::iter::once(&mut arcs.full)
        .chain(std::iter::once(&mut arcs.short))
        .chain(std::iter::once(&mut arcs.standing))
        .chain(arcs.air.iter_mut())
    {
        cut(arc, chao, body.as_ref(), &terrain, &leg, up, slope_cos);
    }
}
