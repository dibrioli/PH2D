//! **O raio AUTORADO** (Ray Sensor, GEF 07 versão C, dono 15/11) — a porta do motor com as opções
//! que o artista escolhe, ao lado da [`super::cast`] dos sensores do personagem.
//!
//! ⚠️ **Uma porta à parte, e não um parâmetro novo no [`PhysicsWorld::cast_ray`]:** os cinco
//! consumidores dele (chão, dois tetos, headroom, parede) continuam a perguntar por MATÉRIA, com o
//! `EXCLUDE_SENSORS` e o `solid = true` fixos — nada do que está aqui os toca. O sexto consumidor (o
//! Ray Sensor) é o único autorável, e as suas escolhas vivem em [`RayQuery`].
//!
//! As mesmas duas metades do filtro do `cast` valem aqui: as CAMADAS (a matriz e as exceções) e o
//! corpo excluído. O resto é do artista:
//!
//! | campo | `false` | `true` |
//! |---|---|---|
//! | `hit_sensors` | um sensor/gatilho é transparente (a lei do `cast`) | ele responde como matéria |
//! | `hit_from_inside` | a forma que CONTÉM a origem chega ao `accept` com `dentro = true` | ela responde a `0` como qualquer outra (`dentro = false`) |
//!
//! O `accept(corpo, dentro)` é o predicado do chamador sobre o CORPO da forma candidata (a família do
//! dono, o filtro de tags) e corre DENTRO da consulta do rapier — nenhuma alocação, e nenhum tipo do
//! rapier além do handle na superfície. ⚠️ `dentro` é dele e não daqui (auditoria C1, 15/11): uma forma
//! que contém a origem é ignorada (Godot `hit_from_inside = false`) SALVO quando ela é uma parede que
//! TAPA («Walls block» e fora do filtro) — o olho enterrado numa parede não vê através dela. Aceite
//! assim, ela responde a `0` (o `solid` do raio).

use crate::rmath::Vector;
use rapier2d::dynamics::RigidBodyHandle;
use rapier2d::geometry::{Collider, ColliderHandle, Ray};
use rapier2d::parry::query::DefaultQueryDispatcher;
use rapier2d::pipeline::{QueryFilter, QueryFilterFlags, QueryPipeline};

use super::PhysicsWorld;
use super::cast::CastHit;

/// **As escolhas do raio autorado.** Ver o cabeçalho do módulo.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RayQuery {
    /// O corpo que sai da consulta (o dono, quando o artista o pede).
    pub exclude_body: Option<RigidBodyHandle>,
    /// A camada de quem pergunta.
    pub layer: u8,
    /// Os sensores/gatilhos respondem.
    pub hit_sensors: bool,
    /// A forma que contém a origem responde a `0`.
    pub hit_from_inside: bool,
}

impl PhysicsWorld {
    /// **O primeiro acerto** do raio autorado — `None` sem nada no alcance ou com entrada degenerada.
    #[must_use]
    pub fn cast_ray_query(
        &self,
        origin: [f32; 2],
        dir: [f32; 2],
        max_dist: f32,
        q: RayQuery,
        accept: &dyn Fn(Option<RigidBodyHandle>, bool) -> bool,
    ) -> Option<CastHit> {
        let (ray, max) = raio(origin, dir, max_dist)?;
        let mut achou = None;
        self.with_query_pipeline(origin, q, accept, |p| {
            achou = p
                .cast_ray_and_get_normal(&ray, max, true)
                .map(|(c, h)| self.hit_of(&ray, c, h.time_of_impact, h.normal));
        });
        achou
    }

    /// **TODOS os acertos** ao longo do alcance, em `out` (limpo primeiro), ordenados por distância
    /// e, no empate, pelo handle da forma (determinista). `out` é do chamador: a capacidade fica.
    pub fn cast_ray_query_all(
        &self,
        origin: [f32; 2],
        dir: [f32; 2],
        max_dist: f32,
        q: RayQuery,
        accept: &dyn Fn(Option<RigidBodyHandle>, bool) -> bool,
        out: &mut Vec<CastHit>,
    ) {
        out.clear();
        let Some((ray, max)) = raio(origin, dir, max_dist) else {
            return;
        };
        self.with_query_pipeline(origin, q, accept, |p| {
            for (c, _, h) in p.intersect_ray(ray, max, true) {
                out.push(self.hit_of(&ray, c, h.time_of_impact, h.normal));
            }
        });
        out.sort_by(|a, b| {
            a.distance.total_cmp(&b.distance).then_with(|| {
                a.collider
                    .into_raw_parts()
                    .cmp(&b.collider.into_raw_parts())
            })
        });
    }

    fn with_query_pipeline(
        &self,
        origin: [f32; 2],
        q: RayQuery,
        accept: &dyn Fn(Option<RigidBodyHandle>, bool) -> bool,
        f: impl FnOnce(&QueryPipeline<'_>),
    ) {
        let asker = super::layer_override::asker_of(q.layer, None);
        let sees = super::layer_override::query_sees(asker, self.layer_matrix);
        let o = Vector::new(origin[0], origin[1]);
        let pred = |h: ColliderHandle, c: &Collider| {
            sees(h, c)
                && accept(
                    c.parent(),
                    !q.hit_from_inside && c.shape().contains_point(c.position(), o),
                )
        };
        let filter = QueryFilter {
            groups: Some(super::layer_override::query_groups(
                asker,
                self.layer_matrix,
            )),
            predicate: Some(&pred),
            exclude_rigid_body: q.exclude_body,
            exclude_collider: None,
            flags: if q.hit_sensors {
                QueryFilterFlags::empty()
            } else {
                QueryFilterFlags::EXCLUDE_SENSORS
            },
        };
        let dispatcher = DefaultQueryDispatcher;
        let pipeline: QueryPipeline<'_> =
            self.broad_phase
                .as_query_pipeline(&dispatcher, &self.bodies, &self.colliders, filter);
        f(&pipeline);
    }

    fn hit_of(&self, ray: &Ray, collider: ColliderHandle, toi: f32, normal: Vector) -> CastHit {
        let point = ray.point_at(toi);
        CastHit {
            collider,
            body: self.colliders.get(collider).and_then(|c| c.parent()),
            distance: toi,
            point: [point.x, point.y],
            normal: [normal.x, normal.y],
        }
    }
}

/// O raio unitário e o alcance — `None` para entrada degenerada (a recusa do `cast`).
fn raio(origin: [f32; 2], dir: [f32; 2], max_dist: f32) -> Option<(Ray, f32)> {
    let d = Vector::new(dir[0], dir[1]);
    let n = d.length();
    if !n.is_finite() || n <= f32::EPSILON || !max_dist.is_finite() || max_dist < 0.0 {
        return None;
    }
    Some((Ray::new(Vector::new(origin[0], origin[1]), d / n), max_dist))
}
