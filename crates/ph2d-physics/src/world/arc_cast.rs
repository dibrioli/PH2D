//! **Uma forma varrida por uma polilinha contra formas paradas, sem mundo** (GEF 04, a curva do pulo
//! pousa no terreno). Não lê o BVH do `PhysicsWorld`: com o relógio parado ele é o do último `step`, e
//! o artista edita a cena por baixo dele. O chamador passa as formas da VERDADE DO ECS.
//!
//! Duas perguntas, sobre o mesmo [`Terrain`]: a forma do corpo varrida pelo traço (paredes, tetos,
//! quinas) e os raios da PERNA em cada pose (é ela que pousa: o jogo pousa quando a perna agarra o
//! chão, não quando o pé o cruza — `ph2d-physics-ecs/src/arc_terrain.rs`).

use rapier2d::geometry::SharedShape;
use rapier2d::parry::query::{Ray, ShapeCastOptions, cast_shapes};

use super::collider_build::shape_builder;
use super::outline::Outlines;
use super::shape::ShapeDesc;
use crate::rmath::{Pose, Vector};

/// Uma forma posta no mundo: o centro da forma e a rotação dela (radianos CCW).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PlacedShape {
    pub shape: ShapeDesc,
    pub centre: [f32; 2],
    pub rotation: f32,
    /// Plataforma de mão única: só pára quem DESCE sobre ela por cima (contra `up`).
    pub one_way: bool,
}

/// Onde a varredura parou: no segmento `segment` (de `path[segment]` a `path[segment + 1]`), à
/// fracção `fraction` dele.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PathHit {
    pub segment: usize,
    pub fraction: f32,
    /// O ponto tocado NA superfície do obstáculo, em mundo (o pé ou o corpo).
    pub contact: [f32; 2],
    /// A normal da superfície no toque, unitária, a sair do obstáculo (contra o sondador).
    pub normal: [f32; 2],
}

struct Prepared {
    shape: SharedShape,
    pose: Pose,
    min: [f32; 2],
    max: [f32; 2],
    one_way: bool,
}

/// A forma de colisão de um [`ShapeDesc`]; um `Outline` vira a caixa da imagem (as peças moram na
/// tabela do mundo, que esta porta não lê).
fn shared(shape: ShapeDesc) -> SharedShape {
    shape_builder(&Outlines::new(), shape)
        .build()
        .shared_shape()
        .clone()
}

fn prepare(p: &PlacedShape) -> Prepared {
    let shape = shared(p.shape);
    let pose = Pose::new(Vector::new(p.centre[0], p.centre[1]), p.rotation);
    let bb = shape.compute_aabb(&pose);
    Prepared {
        shape,
        pose,
        min: [bb.mins.x, bb.mins.y],
        max: [bb.maxs.x, bb.maxs.y],
        one_way: p.one_way,
    }
}

fn overlap(a_min: [f32; 2], a_max: [f32; 2], b: &Prepared) -> bool {
    a_min[0] <= b.max[0] && a_max[0] >= b.min[0] && a_min[1] <= b.max[1] && a_max[1] >= b.min[1]
}

/// **O terreno de uma curva, preparado UMA vez** para as duas perguntas do corte: o corpo varrido
/// pelo traço ([`Self::sweep_body`]) e a perna, raio a raio ([`Self::cast_ray`]).
pub struct Terrain {
    obs: Vec<Prepared>,
}

/// O que um raio achou: a distância da origem, o ponto e a normal da superfície (unitária).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RayHit {
    pub distance: f32,
    pub contact: [f32; 2],
    pub normal: [f32; 2],
}

impl PlacedShape {
    /// A caixa da forma posta no mundo: `(min, max)`.
    #[must_use]
    pub fn aabb(&self) -> ([f32; 2], [f32; 2]) {
        let p = prepare(self);
        (p.min, p.max)
    }
}

impl Terrain {
    #[must_use]
    pub fn new(obstacles: &[PlacedShape]) -> Self {
        Self {
            obs: obstacles.iter().map(prepare).collect(),
        }
    }

    /// **O raio `dir` (unitário) de `origin` até `max`**, contra o terreno sólido (a mão única só
    /// por cima, contra `up`). O mais próximo ganha.
    #[must_use]
    pub fn cast_ray(
        &self,
        origin: [f32; 2],
        dir: [f32; 2],
        max: f32,
        up: [f32; 2],
    ) -> Option<RayHit> {
        let end = [origin[0] + dir[0] * max, origin[1] + dir[1] * max];
        let lo = [origin[0].min(end[0]), origin[1].min(end[1])];
        let hi = [origin[0].max(end[0]), origin[1].max(end[1])];
        let ray = Ray::new(
            Vector::new(origin[0], origin[1]),
            Vector::new(dir[0], dir[1]),
        );
        let mut best: Option<RayHit> = None;
        for o in self.obs.iter().filter(|o| overlap(lo, hi, o)) {
            let Some(hit) = o.shape.cast_ray_and_get_normal(&o.pose, &ray, max, true) else {
                continue;
            };
            let n = hit.normal;
            if o.one_way && (hit.time_of_impact <= 0.0 || n.x * up[0] + n.y * up[1] <= 0.0) {
                continue;
            }
            let t = hit.time_of_impact;
            if best.is_none_or(|b| t < b.distance) {
                best = Some(RayHit {
                    distance: t,
                    contact: [origin[0] + dir[0] * t, origin[1] + dir[1] * t],
                    normal: [n.x, n.y],
                });
            }
        }
        best
    }

    /// **Varre o CORPO** (`body`, o centro relativo ao pé) pelo traço `path` (pontos em mundo do
    /// pé) e devolve o primeiro toque: paredes, tetos, quinas. A mão única só a perna a pisa. Um
    /// toque com o corpo a AFASTAR-SE da superfície não conta: o arranque está sempre no chão.
    #[must_use]
    pub fn sweep_body(&self, path: &[[f32; 2]], body: &PlacedShape) -> Option<PathHit> {
        let s = shared(body.shape);
        let (c, rot) = (body.centre, body.rotation);
        let pad = {
            let bb = s.compute_local_aabb();
            bb.mins.length().max(bb.maxs.length())
        };
        let options = ShapeCastOptions::with_max_time_of_impact(1.0);
        for (k, w) in path.windows(2).enumerate() {
            let (a, b) = (w[0], w[1]);
            let d = [b[0] - a[0], b[1] - a[1]];
            let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
            if !len.is_finite() || len <= f32::EPSILON {
                continue;
            }
            let lo = [a[0].min(b[0]) + c[0] - pad, a[1].min(b[1]) + c[1] - pad];
            let hi = [a[0].max(b[0]) + c[0] + pad, a[1].max(b[1]) + c[1] + pad];
            // (tempo, contacto, normal) do primeiro impacto deste segmento.
            let mut best: Option<(f32, [f32; 2], [f32; 2])> = None;
            for o in self.obs.iter().filter(|o| !o.one_way && overlap(lo, hi, o)) {
                let pose = Pose::new(Vector::new(a[0] + c[0], a[1] + c[1]), rot);
                let vel = Vector::new(d[0], d[1]);
                if let Ok(Some(hit)) = cast_shapes(
                    &pose,
                    vel,
                    s.as_ref(),
                    &o.pose,
                    Vector::ZERO,
                    o.shape.as_ref(),
                    options,
                ) {
                    // A normal do corpo, em mundo: aponta para o obstáculo quando se aproxima.
                    let n = pose.rotation * hit.normal1;
                    if n.x * d[0] + n.y * d[1] > 0.0
                        && best.is_none_or(|b| hit.time_of_impact < b.0)
                    {
                        let at = o.pose.transform_point(hit.witness2);
                        let n2 = (o.pose.rotation * hit.normal2).normalize_or_zero();
                        best = Some((hit.time_of_impact, [at.x, at.y], [n2.x, n2.y]));
                    }
                }
            }
            if let Some((t, contact, normal)) = best {
                return Some(PathHit {
                    segment: k,
                    fraction: t.clamp(0.0, 1.0),
                    contact,
                    normal,
                });
            }
        }
        None
    }
}
