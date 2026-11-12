//! **Uma forma varrida por uma polilinha contra formas paradas, sem mundo** (GEF 04, a curva do pulo
//! pousa no terreno). Não lê o BVH do `PhysicsWorld`: com o relógio parado ele é o do último `step`, e
//! o artista edita a cena por baixo dele. O chamador passa as formas da VERDADE DO ECS.
//!
//! Dois sondadores por segmento: a forma do corpo (paredes, tetos, quinas) e um raio no pé (o traço
//! desenhado — é ele que pousa, porque a perna segura o corpo acima do chão). O primeiro impacto ganha.

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

/// **Varre `path` (pontos em mundo do PÉ) contra `obstacles`** e devolve o primeiro toque.
///
/// `body` é a forma do corpo e o seu centro relativo ao pé (`centre` = deslocamento pé → centro);
/// `None` varre só o raio do pé. `up` é o «cima» do mundo (o da mão única). Um toque com o sondador
/// a AFASTAR-SE da superfície (começa encostado e sai) não conta: o arranque está sempre no chão.
#[must_use]
pub fn sweep_path(
    path: &[[f32; 2]],
    body: Option<&PlacedShape>,
    obstacles: &[PlacedShape],
    up: [f32; 2],
) -> Option<PathHit> {
    let obs: Vec<Prepared> = obstacles.iter().map(prepare).collect();
    let mover = body.map(|b| (shared(b.shape), b.centre, b.rotation));
    let mover_reach = mover.as_ref().map_or(0.0, |(s, _, _)| {
        let bb = s.compute_local_aabb();
        bb.mins.length().max(bb.maxs.length())
    });
    let options = ShapeCastOptions::with_max_time_of_impact(1.0);
    for (k, w) in path.windows(2).enumerate() {
        let (a, b) = (w[0], w[1]);
        let d = [b[0] - a[0], b[1] - a[1]];
        let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
        if !len.is_finite() || len <= f32::EPSILON {
            continue;
        }
        let descending = d[0] * up[0] + d[1] * up[1] < 0.0;
        let off = mover.as_ref().map_or([0.0, 0.0], |(_, c, _)| *c);
        let pad = mover_reach;
        let lo = [
            a[0].min(b[0]).min(a[0] + off[0]) - pad,
            a[1].min(b[1]).min(a[1] + off[1]) - pad,
        ];
        let hi = [
            a[0].max(b[0]).max(b[0] + off[0]) + pad,
            a[1].max(b[1]).max(b[1] + off[1]) + pad,
        ];
        // (tempo, contacto, normal) do primeiro impacto deste segmento.
        let mut best: Option<(f32, [f32; 2], [f32; 2])> = None;
        let mut keep = |t: f32, contact: [f32; 2], n: Vector| {
            if best.is_none_or(|b| t < b.0) {
                best = Some((t, contact, [n.x, n.y]));
            }
        };
        for o in obs.iter().filter(|o| overlap(lo, hi, o)) {
            if o.one_way && !descending {
                continue;
            }
            // O pé: o traço desenhado.
            let ray = Ray::new(Vector::new(a[0], a[1]), Vector::new(d[0], d[1]));
            if let Some(hit) = o.shape.cast_ray_and_get_normal(&o.pose, &ray, 1.0, true) {
                let n = hit.normal;
                let approaching = n.x * d[0] + n.y * d[1] < 0.0;
                let from_above = !o.one_way || n.x * up[0] + n.y * up[1] > 0.0;
                if approaching && from_above && hit.time_of_impact > 0.0 {
                    let t = hit.time_of_impact;
                    keep(t, [a[0] + d[0] * t, a[1] + d[1] * t], n);
                }
            }
            // O corpo: paredes e tetos. A mão única só a perna a pisa.
            if let (Some((s, c, rot)), false) = (&mover, o.one_way) {
                let pose = Pose::new(Vector::new(a[0] + c[0], a[1] + c[1]), *rot);
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
                    if n.x * d[0] + n.y * d[1] > 0.0 {
                        let at = o.pose.transform_point(hit.witness2);
                        let n2 = (o.pose.rotation * hit.normal2).normalize_or_zero();
                        keep(hit.time_of_impact, [at.x, at.y], n2);
                    }
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
