//! **Empuxo — a área sabe QUANTO do corpo está dentro dela** (ADR-0131 W-Buoyancy).
//!
//! Fecha a lacuna que o W-AreaDrag deixou nomeada: até aqui, "flutuar" era uma
//! `Force Y` constante para cima vencendo o peso, o que tem três defeitos que um
//! artista sente imediatamente —
//!
//! 1. **não se auto-nivela**: a força não sabe onde está a superfície, então o corpo
//!    leve é arremessado para fora da piscina em vez de parar na linha d'água;
//! 2. **é por MASSA, não por densidade**: o número certo tem de ser re-descoberto para
//!    cada objeto (uma caixa 4× mais pesada precisa de 4× a força), quando a intuição
//!    real é *madeira boia, pedra afunda*, que é uma propriedade do MATERIAL;
//! 3. **não endireita nada**: um barco tombado fica tombado.
//!
//! Arquimedes resolve os três com um número só: a força é `ρ_fluido · |g| · A_submersa`,
//! **para cima**, aplicada no **centroide da parte submersa**. O corpo sobe até a área
//! submersa gerar exatamente o próprio peso — a linha d'água cai de graça. Um corpo
//! mais denso que o fluido nunca chega lá e afunda. E como o centroide se desloca
//! quando o corpo inclina, o braço de alavanca **endireita** o barco sozinho: nada disso
//! é código extra, é a mesma fórmula.
//!
//! ## A superfície é perpendicular à GRAVIDADE, não ao eixo Y
//!
//! Água tem superfície horizontal mesmo numa piscina torta — então a superfície não é a
//! borda de cima do collider, é o plano perpendicular à gravidade que passa pelo ponto
//! mais alto dele. Cai de graça e apaga dois casos especiais: uma zona **rotacionada**
//! se comporta certo, e gravidade lateral (ou zero) também. ⚠️ Com gravidade **zero** não
//! existe empuxo — o que é fisicamente certo, não um caso degenerado a tratar.
//!
//! ## O recorte
//!
//! Sutherland–Hodgman contra um semi-plano, e a área + centroide pela fórmula do
//! shoelace.
//!
//! ⚠️ O polígono vem do **collider VIVO do rapier** ([`local_polygon`]), nunca do
//! `BodyDesc`: é o collider que a escala do `Transform` já alcançou (W6), então uma
//! piscina ou um barco escalados boiam com o tamanho que estão DESENHADOS. Re-derivar a
//! silhueta do descritor seria uma segunda resposta para *"que forma é esta?"*, e as duas
//! divergiriam exatamente onde ninguém olha — num corpo escalado.
//!
//! Para bola e cápsula o rapier não tem vértices (ele as representa exatamente), então
//! elas são tesseladas pelas MESMAS portas que o collider de elipse e o overlay usam
//! (`ellipse_vertices`/`capsule_vertices`). Um polígono regular de N lados inscrito num
//! círculo tem `(N/2π)·sin(2π/N)` da área dele — **0,64% a menos** em `ELLIPSE_SEGS = 32`.
//! Nomeado aqui e pinado num gate, porque é um viés que o empuxo herda.

use rapier2d::dynamics::{RigidBodyHandle, RigidBodySet};
use rapier2d::geometry::ColliderSet;
use rapier2d::na::{Point2, Vector2};

use rapier2d::geometry::Shape;

use super::shape::{capsule_vertices, ellipse_vertices};

/// O polígono do collider **que o solver de fato tem**, em espaço local.
///
/// A porta única de *"que forma é esta?"* para o empuxo. `Cuboid` são seus quatro
/// cantos; `ConvexPolygon` (elipse e stadium, W6) devolve os próprios pontos do casco;
/// `Ball` e `Capsule` — que o rapier representa exatamente — são tesseladas pelas mesmas
/// funções que constroem o collider de elipse e que o overlay desenha, com o viés de
/// 0,64% que o cabeçalho nomeia. `None` para uma forma que este módulo não conhece:
/// nenhum empuxo é melhor que um empuxo sobre uma silhueta inventada.
#[must_use]
fn local_polygon(shape: &dyn Shape) -> Option<Vec<Point2<f32>>> {
    let pt = |[x, y]: [f32; 2]| Point2::new(x, y);
    if let Some(c) = shape.as_cuboid() {
        let (hx, hy) = (c.half_extents.x, c.half_extents.y);
        return Some(vec![
            Point2::new(hx, hy),
            Point2::new(-hx, hy),
            Point2::new(-hx, -hy),
            Point2::new(hx, -hy),
        ]);
    }
    if let Some(b) = shape.as_ball() {
        return Some(
            ellipse_vertices(b.radius, b.radius)
                .into_iter()
                .map(pt)
                .collect(),
        );
    }
    if let Some(c) = shape.as_capsule() {
        // O segmento do rapier é centrado na origem e alinhado em Y (a convenção que
        // `ShapeDesc::Capsule` impõe), então a meia-altura é a do ponto `b`.
        let half_height = c.segment.b.y;
        return Some(
            capsule_vertices(half_height, c.radius, c.radius)
                .into_iter()
                .map(pt)
                .collect(),
        );
    }
    shape.as_convex_polygon().map(|p| p.points().to_vec())
}

/// Área (com sinal, CCW positivo) e centroide de um polígono — fórmula do shoelace.
/// `None` para um polígono degenerado (área ~zero), onde o centroide não existe.
#[must_use]
fn area_centroid(poly: &[Point2<f32>]) -> Option<(f32, Point2<f32>)> {
    if poly.len() < 3 {
        return None;
    }
    let (mut a2, mut cx, mut cy) = (0.0f32, 0.0f32, 0.0f32);
    for i in 0..poly.len() {
        let p = poly[i];
        let q = poly[(i + 1) % poly.len()];
        let cross = p.x * q.y - q.x * p.y;
        a2 += cross;
        cx += (p.x + q.x) * cross;
        cy += (p.y + q.y) * cross;
    }
    if a2.abs() < 1e-12 {
        return None;
    }
    let area = a2 * 0.5;
    Some((area.abs(), Point2::new(cx / (3.0 * a2), cy / (3.0 * a2))))
}

/// Recorta `poly` ao semi-plano `dot(p, up) <= level` — a parte SUBMERSA
/// (Sutherland–Hodgman). O polígono de entrada é convexo, então a saída também é.
#[must_use]
fn clip_below(poly: &[Point2<f32>], up: Vector2<f32>, level: f32) -> Vec<Point2<f32>> {
    let mut out: Vec<Point2<f32>> = Vec::with_capacity(poly.len() + 1);
    for i in 0..poly.len() {
        let p = poly[i];
        let q = poly[(i + 1) % poly.len()];
        let (dp, dq) = (p.coords.dot(&up) - level, q.coords.dot(&up) - level);
        if dp <= 0.0 {
            out.push(p);
        }
        // Cruzou a superfície: o ponto exato entra, para que a área varie
        // CONTINUAMENTE com a profundidade. Sem isto o empuxo saltaria de vértice
        // em vértice e o corpo tremeria na linha d'água.
        if (dp < 0.0 && dq > 0.0) || (dp > 0.0 && dq < 0.0) {
            let t = dp / (dp - dq);
            out.push(Point2::from(p.coords + (q.coords - p.coords) * t));
        }
    }
    out
}

/// A profundidade da superfície da zona ao longo de `up`: o extremo do collider dela
/// nessa direção. Um fluido tem superfície onde a poça acaba, e a poça é o collider.
#[must_use]
fn surface_level(
    poly: &[Point2<f32>],
    pose: &rapier2d::na::Isometry2<f32>,
    up: Vector2<f32>,
) -> f32 {
    poly.iter()
        .map(|p| (pose * p).coords.dot(&up))
        .fold(f32::NEG_INFINITY, f32::max)
}

/// A força de empuxo sobre um corpo, e onde ela é aplicada. `None` quando não há parte
/// submersa nenhuma (o corpo está todo acima da superfície) — o caso comum.
#[must_use]
pub(crate) fn buoyant_force(
    body: &dyn Shape,
    body_pose: &rapier2d::na::Isometry2<f32>,
    zone: &dyn Shape,
    zone_pose: &rapier2d::na::Isometry2<f32>,
    gravity: Vector2<f32>,
    fluid_density: f32,
) -> Option<(Vector2<f32>, Point2<f32>)> {
    let g = gravity.norm();
    // Sem gravidade não há empuxo — Arquimedes é consequência do peso do fluido, não uma
    // força independente. O caso degenerado se resolve pela própria física.
    if g <= 0.0 || fluid_density <= 0.0 {
        return None;
    }
    let up = -gravity / g;
    let level = surface_level(&local_polygon(zone)?, zone_pose, up);
    let poly: Vec<Point2<f32>> = local_polygon(body)?
        .into_iter()
        .map(|p| body_pose * p)
        .collect();
    let (area, centroid) = area_centroid(&clip_below(&poly, up, level))?;
    Some((up * (fluid_density * g * area), centroid))
}

/// Aplica o empuxo de uma zona a um corpo, por sub-passo.
///
/// ⚠️ **`apply_impulse_at_point`, não `apply_impulse`**: aplicado no centroide da parte
/// submersa, o empuxo gera torque quando esse centroide não está sobre o centro de massa
/// — que é exatamente o momento restaurador que endireita um barco tombado. Uma força no
/// centro de massa faria o corpo boiar igual e nunca endireitaria nada.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply(
    bodies: &mut RigidBodySet,
    colliders: &ColliderSet,
    body: RigidBodyHandle,
    zone_collider: rapier2d::geometry::ColliderHandle,
    gravity: Vector2<f32>,
    fluid_density: f32,
    dt: f32,
) {
    let Some(zone) = colliders.get(zone_collider) else {
        return;
    };
    let Some(rb) = bodies.get(body) else {
        return;
    };
    let Some(shape) = rb.colliders().first().and_then(|h| colliders.get(*h)) else {
        return;
    };
    let pose = *rb.position();
    let Some((force, at)) = buoyant_force(
        shape.shape(),
        &pose,
        zone.shape(),
        zone.position(),
        gravity,
        fluid_density,
    ) else {
        return;
    };
    if let Some(b) = bodies.get_mut(body) {
        b.apply_impulse_at_point(force * dt, at, true);
    }
}
