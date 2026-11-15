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
    ///
    /// ⭐ **O custo MEDIDO antes de qualquer tecto** (RS-9, §0.0; `mede_o_custo_de_see_all_hits`, 200 peças
    /// numa fila e um raio que as atravessa todas, `--profile smoke`, o mínimo de 7 blocos de 50 tiques,
    /// 15/11). O tique inteiro do mundo com os 200 corpos e um raio:
    ///
    /// | raio | acertos | µs por tique |
    /// |---|---|---|
    /// | linha, o primeiro | 1 | `101,8` |
    /// | linha, «See all hits» | 200 | `133,4` |
    /// | círculo 0,3 m, o primeiro | 1 | `99,9` |
    /// | círculo 0,3 m, «See all hits» (uma cápsula, um toque por candidato) | 200 | `183,0` |
    /// | círculo, «See all hits», a varrer o mundo de novo por acerto (a 1.ª versão, recusada) | 200 | `1 196,3` |
    /// | leque 60° (duas linhas de visão por candidato) | 1 visível | `209,7` |
    ///
    /// ⇒ duzentos acertos custam `+32 µs` (linha) e `+83 µs` (círculo): **nenhum tecto de acertos** — o
    /// `max_results = 32` do Godot seria um limite sem recurso. O desenho é que tem o seu
    /// (`MAX_TIQUES_DO_RAIO`, a marca `Copy`).
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
        ordena(out);
    }

    /// **O primeiro acerto de um CÍRCULO** de raio `radius` varrido ao longo do raio («Thickness»,
    /// RS-1: o ShapeCast2D do Godot) — as mesmas camadas, o mesmo `accept` e o mesmo `RayQuery`. O
    /// ponto e a normal são os da forma ACERTADA, em mundo; `distance` é o quanto o centro andou.
    #[must_use]
    pub fn cast_circle_query(
        &self,
        origin: [f32; 2],
        dir: [f32; 2],
        max_dist: f32,
        radius: f32,
        q: RayQuery,
        accept: &dyn Fn(Option<RigidBodyHandle>, bool) -> bool,
    ) -> Option<CastHit> {
        let (ray, max) = raio(origin, dir, max_dist)?;
        let mut achou = None;
        self.with_query_pipeline(origin, q, accept, |p| {
            achou = circulo(p, &ray, max, radius).map(|(c, h)| self.shape_hit_of(c, &h));
        });
        achou
    }

    /// **TODOS os acertos do círculo varrido**, por distância (e pelo handle no empate), em `out`.
    ///
    /// ⭐ Uma consulta só: as formas que tocam a CÁPSULA varrida (o círculo do início ao fim do alcance)
    /// são os candidatos, e cada um dá o seu toque pelo `cast_shapes` do parry contra ele só. Medido em
    /// `mede_o_custo_de_see_all_hits` (200 peças, `--profile smoke`, 15/11): varrer o mundo de novo por
    /// acerto custava `1 196 µs` por tique; assim, ver a tabela do `cast_ray_query_all`.
    pub fn cast_circle_query_all(
        &self,
        origin: [f32; 2],
        dir: [f32; 2],
        max_dist: f32,
        radius: f32,
        q: RayQuery,
        accept: &dyn Fn(Option<RigidBodyHandle>, bool) -> bool,
        out: &mut Vec<CastHit>,
    ) {
        out.clear();
        let Some((ray, max)) = raio(origin, dir, max_dist) else {
            return;
        };
        let r = radius.max(0.0);
        let capsula = rapier2d::parry::shape::Capsule::new(ray.origin, ray.point_at(max), r);
        let disco = rapier2d::parry::shape::Ball::new(r);
        let inicio = rapier2d::math::Pose::from_translation(ray.origin);
        let opcoes = rapier2d::parry::query::ShapeCastOptions::with_max_time_of_impact(max);
        self.with_query_pipeline(origin, q, accept, |p| {
            for (h, c) in p.intersect_shape(rapier2d::math::Pose::IDENTITY, &capsula) {
                let Ok(Some(t)) = rapier2d::parry::query::cast_shapes(
                    c.position(),
                    Vector::ZERO,
                    c.shape(),
                    &inicio,
                    ray.dir,
                    &disco,
                    opcoes,
                ) else {
                    continue;
                };
                let pose = c.position();
                let ponto = pose.transform_point(t.witness1);
                let normal = pose.transform_vector(t.normal1);
                out.push(CastHit {
                    collider: h,
                    body: c.parent(),
                    distance: t.time_of_impact,
                    point: [ponto.x, ponto.y],
                    normal: [normal.x, normal.y],
                });
            }
        });
        ordena(out);
    }

    /// **As formas que tocam o DISCO** de raio `radius` à volta de `origin` — os candidatos do
    /// «Cone» (RS-2), com o ponto de cada uma mais perto da origem e o centro dela, em `out` (ordenado
    /// pelo handle).
    /// As mesmas camadas, `accept` e `RayQuery` do raio.
    pub fn shapes_in_disc(
        &self,
        origin: [f32; 2],
        radius: f32,
        q: RayQuery,
        accept: &dyn Fn(Option<RigidBodyHandle>, bool) -> bool,
        out: &mut Vec<(ColliderHandle, Option<RigidBodyHandle>, [f32; 2], [f32; 2])>,
    ) {
        out.clear();
        if !radius.is_finite() || radius <= 0.0 {
            return;
        }
        let o = Vector::new(origin[0], origin[1]);
        let disco = rapier2d::parry::shape::Ball::new(radius);
        let pose = rapier2d::math::Pose::from_translation(o);
        self.with_query_pipeline(origin, q, accept, |p| {
            for (h, c) in p.intersect_shape(pose, &disco) {
                let perto = c.shape().project_point(c.position(), o, true).point;
                let centro = c.position().translation;
                out.push((h, c.parent(), [perto.x, perto.y], [centro.x, centro.y]));
            }
        });
        out.sort_by_key(|(h, ..)| h.into_raw_parts());
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

    /// O acerto de uma forma varrida: o ponto e a normal da forma acertada (mundo, ver o rapier).
    fn shape_hit_of(
        &self,
        collider: ColliderHandle,
        h: &rapier2d::parry::query::ShapeCastHit,
    ) -> CastHit {
        CastHit {
            collider,
            body: self.colliders.get(collider).and_then(|c| c.parent()),
            distance: h.time_of_impact,
            point: [h.witness1.x, h.witness1.y],
            normal: [h.normal1.x, h.normal1.y],
        }
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

/// Por distância e, no empate, pelo handle da forma (determinista).
fn ordena(out: &mut [CastHit]) {
    out.sort_by(|a, b| {
        a.distance.total_cmp(&b.distance).then_with(|| {
            a.collider
                .into_raw_parts()
                .cmp(&b.collider.into_raw_parts())
        })
    });
}

/// Um disco de raio `radius` varrido ao longo de `ray` até `max` — o primeiro toque.
fn circulo(
    p: &QueryPipeline<'_>,
    ray: &Ray,
    max: f32,
    radius: f32,
) -> Option<(ColliderHandle, rapier2d::parry::query::ShapeCastHit)> {
    let disco = rapier2d::parry::shape::Ball::new(radius.max(0.0));
    let pose = rapier2d::math::Pose::from_translation(ray.origin);
    p.cast_shape(
        &pose,
        ray.dir,
        &disco,
        rapier2d::parry::query::ShapeCastOptions::with_max_time_of_impact(max),
    )
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
