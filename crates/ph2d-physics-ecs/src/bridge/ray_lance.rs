//! ⭐⭐⭐ **O LANCE de um raio** — a porta ÚNICA do que um raio VÊ, com dois chamadores: o objecto
//! «Ray Sensor» a cada tique ([`PhysicsBridge::cast_ray_sensors`](super::PhysicsBridge)) e o tiro do
//! script (`ph2d.raycast`, RS-8). Os mesmos filtros nos dois: é a lei do `RaySensor`, e uma segunda
//! cópia dela no script divergiria no primeiro campo novo.
//!
//! | forma | motor | acertos |
//! |---|---|---|
//! | linha (`thickness 0`, `cone 0`) | [`cast_ray_query`](ph2d_physics::PhysicsWorld::cast_ray_query) | o primeiro, ou todos com «See all hits» |
//! | círculo (`thickness > 0`, RS-1) | [`cast_circle_query`](ph2d_physics::PhysicsWorld::cast_circle_query) | idem |
//! | leque (`cone > 0`, RS-2) | [`shapes_in_disc`](ph2d_physics::PhysicsWorld::shapes_in_disc) + uma linha de visão por candidato | um por objecto dentro do leque e não tapado |
//!
//! ⭐ **O leque é a linha de visão do Construct** (Line-of-sight: alcance + ângulo + obstáculos), não
//! um leque de N raios: N raios deixam passar o que cabe entre dois (o vão de 0,20 m que o
//! `measure_the_gap_between_rays` mediu no agachar). Cada objecto no disco do alcance é testado por
//! duas linhas de visão — para o ponto dele mais perto da origem e para o centro da forma, cada uma
//! presa dentro do leque —; é visto se uma delas (linha ou círculo) o acerta primeiro. Uma só (o ponto
//! mais perto) lia como tapado um objecto cujo canto de perto estava atrás da parede e o resto não
//! (o gate do leque apanhou-o).

use std::collections::BTreeMap;

use ph2d_ecs::{ChildOf, Entity, StableId, World};
use ph2d_physics::{CastHit, ColliderHandle, PhysicsWorld, RayQuery, RigidBodyHandle};

use super::ray_sensors::RaioEmMundo;
use super::{BodyRef, PhysicsBridge};
use crate::{RayHit, RaySensor};

/// O que o lance lê do mundo — emprestado, nenhum estado.
pub(super) struct Mundo<'a> {
    pub fisica: &'a PhysicsWorld,
    pub world: &'a World,
    /// Corpo (handle) → entidade.
    pub by_handle: &'a BTreeMap<(u32, u32), Entity>,
    /// Forma das PEÇAS (handle do collider) → a entidade dela.
    pub pecas: &'a BTreeMap<(u32, u32), Entity>,
    pub bodies: &'a BTreeMap<Entity, BodyRef>,
    pub tree: &'a ph2d_tags::TagTree,
}

/// Os buffers do lance — do chamador, a capacidade fica (HR-3).
#[derive(Default)]
pub(super) struct Buffers {
    pub acertos: Vec<CastHit>,
    pub disco: Vec<(ColliderHandle, Option<RigidBodyHandle>, [f32; 2], [f32; 2])>,
}

/// **Lança `r` a partir de `raio`** e põe em `out` (limpo primeiro) o que ele VÊ, do mais perto ao mais
/// longe. `quem` = o objecto que olha (o raio, ou quem chama o script): é dele a família ignorada.
/// Devolve onde a LINHA acaba: no alcance, ou na parede que tapa (que não é um acerto, auditoria C3).
pub(super) fn lanca(
    m: &Mundo<'_>,
    quem: Option<Entity>,
    raio: &RaioEmMundo,
    r: &RaySensor,
    buf: &mut Buffers,
    out: &mut Vec<RayHit>,
) -> f32 {
    out.clear();
    let world = m.world;
    let dono = quem.and_then(|q| {
        if m.bodies.contains_key(&q) {
            Some(q)
        } else {
            crate::owner_body(world, q)
        }
    });
    let topo = quem
        .filter(|_| r.ignore_parts)
        .and_then(|q| crate::topo_da_familia(world, q, r.ignore_up_to));
    let ignorados: Vec<Entity> = r
        .also_ignore
        .iter()
        .filter_map(|&id| ph2d_ecs::entity_with_stable_id(world, StableId(id)))
        .collect();
    let entidade =
        |c: Option<RigidBodyHandle>| c.and_then(|b| m.by_handle.get(&b.into_raw_parts()).copied());
    let passa = |b: Option<Entity>| {
        r.only_see.passes(
            r.see_tag,
            m.tree,
            b.and_then(|b| world.get::<ph2d_ecs::tags::Tags>(b)),
        )
    };
    // «Also ignore» (RS-12): o objecto da lista, a família debaixo dele, e o corpo dono de uma forma
    // escolhida (o conta-gotas apanha a «Collision Shape» filha).
    let na_lista = |b: Entity| {
        !ignorados.is_empty()
            && (std::iter::successors(Some(b), |&c| world.get::<ChildOf>(c).map(ChildOf::parent))
                .any(|c| ignorados.contains(&c))
                || ignorados
                    .iter()
                    .any(|&i| crate::owner_body(world, i) == Some(b)))
    };
    let aceita = |c: Option<RigidBodyHandle>, dentro: bool| {
        let b = entidade(c);
        if let Some(b) = b {
            if Some(b) != dono && topo.is_some_and(|t| crate::na_familia(world, t, b)) {
                return false;
            }
            if na_lista(b) {
                return false;
            }
        }
        if dentro {
            return r.walls_block && b != dono && !passa(b);
        }
        r.walls_block || passa(b)
    };
    let consulta = RayQuery {
        exclude_body: if r.ignore_body { raio.excluir } else { None },
        layer: r.layer,
        hit_sensors: r.hit_triggers,
        hit_from_inside: r.starts_inside,
    };
    let forma = |h: &CastHit, b: Entity| {
        m.pecas
            .get(&h.collider.into_raw_parts())
            .copied()
            .unwrap_or_else(|| m.bodies.get(&b).map_or(b, |x| x.shape))
    };
    let acerto = |b: Entity, h: &CastHit| RayHit {
        body: b,
        shape: forma(h, b),
        distance: h.distance,
        point: h.point,
        normal: h.normal,
    };
    let raio_do_circulo = (r.thickness.is_finite() && r.thickness > 0.0).then(|| r.thickness * 0.5);
    let um = |o: [f32; 2], d: [f32; 2]| match raio_do_circulo {
        Some(rr) => m
            .fisica
            .cast_circle_query(o, d, raio.reach, rr, consulta, &aceita),
        None => m.fisica.cast_ray_query(o, d, raio.reach, consulta, &aceita),
    };

    if r.cone.is_finite() && r.cone > 0.0 {
        let meio = (r.cone.min(360.0) * 0.5).to_radians();
        m.fisica
            .shapes_in_disc(raio.origem, raio.reach, consulta, &aceita, &mut buf.disco);
        for &(_, corpo, perto, centro) in &buf.disco {
            let Some(b) = entidade(corpo) else { continue };
            if !passa(Some(b)) || out.iter().any(|x| x.body == b) {
                continue;
            }
            // Duas linhas de visão: para o ponto mais perto e para o centro (presos no leque). Um objecto
            // meio tapado é visto pela metade que se vê.
            let visto = [perto, centro]
                .into_iter()
                .filter_map(|p| rumo_no_leque(raio.origem, raio.dir, meio, p))
                .filter_map(|rumo| um(raio.origem, rumo))
                .find(|h| entidade(h.body) == Some(b));
            if let Some(h) = visto {
                out.push(acerto(b, &h));
            }
        }
        let chave = |e: Entity| world.get::<StableId>(e).map_or(e.to_bits(), |s| s.0);
        out.sort_by(|a, b| {
            a.distance
                .total_cmp(&b.distance)
                .then_with(|| chave(a.body).cmp(&chave(b.body)))
        });
        return raio.reach;
    }

    let mut fim = raio.reach;
    if r.all_hits {
        match raio_do_circulo {
            Some(rr) => m.fisica.cast_circle_query_all(
                raio.origem,
                raio.dir,
                raio.reach,
                rr,
                consulta,
                &aceita,
                &mut buf.acertos,
            ),
            None => m.fisica.cast_ray_query_all(
                raio.origem,
                raio.dir,
                raio.reach,
                consulta,
                &aceita,
                &mut buf.acertos,
            ),
        }
        for h in &buf.acertos {
            let b = entidade(h.body);
            if !passa(b) {
                fim = h.distance;
                break;
            }
            if let Some(b) = b
                && !out.iter().any(|x| x.body == b)
            {
                out.push(acerto(b, h));
            }
        }
    } else if let Some(h) = um(raio.origem, raio.dir) {
        match entidade(h.body) {
            Some(b) if passa(Some(b)) => out.push(acerto(b, &h)),
            _ => fim = h.distance,
        }
    }
    fim
}

impl PhysicsBridge {
    /// ⭐ **`ph2d.raycast(de, até, filtro)` (RS-8)** — um raio PONTUAL, com a resposta na hora
    /// (hitscan): o mesmo [`lanca`] do objecto «Ray Sensor», com `filtro` no papel dele (a forma, as
    /// opções e as listas). `quem` = o objecto do script: é a família dele que «Ignore my body/parts»
    /// tira. Lê o mundo da física do último passo — o mesmo que os sensores leram neste tique.
    #[must_use]
    pub fn raycast(
        &self,
        world: &World,
        quem: Option<Entity>,
        de: [f32; 2],
        ate: [f32; 2],
        filtro: &RaySensor,
    ) -> Vec<RayHit> {
        let d = [ate[0] - de[0], ate[1] - de[1]];
        let reach = libm::sqrtf(d[0] * d[0] + d[1] * d[1]);
        if !reach.is_finite() || reach <= f32::EPSILON {
            return Vec::new();
        }
        let dono = quem.and_then(|q| {
            if self.bodies.contains_key(&q) {
                Some(q)
            } else {
                crate::owner_body(world, q)
            }
        });
        let raio = RaioEmMundo {
            origem: de,
            dir: [d[0] / reach, d[1] / reach],
            reach,
            excluir: dono.and_then(|b| self.bodies.get(&b)).map(|b| b.handle),
        };
        let by_handle = self.handle_map();
        let pecas: BTreeMap<(u32, u32), Entity> = self
            .parts
            .iter()
            .map(|(e, p)| (p.handle.into_raw_parts(), *e))
            .collect();
        let m = Mundo {
            fisica: &self.world,
            world,
            by_handle: &by_handle,
            pecas: &pecas,
            bodies: &self.bodies,
            tree: &self.nav.arvore,
        };
        let mut out = Vec::new();
        let mut r = filtro.clone();
        // Um tiro pontual não tem «Aim at»: o rumo é o `até`.
        r.aim_at = 0;
        lanca(&m, quem, &raio, &r, &mut Buffers::default(), &mut out);
        if !(r.all_hits || r.cone > 0.0) {
            out.truncate(1);
        }
        out
    }

    /// ⭐ **«Seeing» (RS-5)**: o raio `e` vê alguém AGORA — a leitura do script e do cartão.
    #[must_use]
    pub fn ray_seeing(&self, e: Entity) -> bool {
        self.ray_hits.contains_key(&e)
    }
}

/// O rumo da linha de visão para um candidato cujo ponto mais perto é `perto`: o rumo para ele, preso
/// dentro do leque de meia abertura `meio` (rad) à volta de `dir`. `None` = fora do alcance angular
/// por mais do que o leque cobre (atrás, num leque menor que meia volta) — nunca acontece com 360°.
fn rumo_no_leque(origem: [f32; 2], dir: [f32; 2], meio: f32, perto: [f32; 2]) -> Option<[f32; 2]> {
    let v = [perto[0] - origem[0], perto[1] - origem[1]];
    let n = libm::sqrtf(v[0] * v[0] + v[1] * v[1]);
    // A origem dentro (ou em cima) da forma: o rumo é o do raio.
    if n <= 1e-6 {
        return Some(dir);
    }
    let a = libm::atan2f(dir[0] * v[1] - dir[1] * v[0], dir[0] * v[0] + dir[1] * v[1]);
    // Um candidato cujo ponto mais perto está fora do leque só é visto pela borda; mais de 90° fora da
    // borda ele está ATRÁS dela e a borda não o atravessa (uma forma convexa não dá a volta).
    if a.abs() > meio + std::f32::consts::FRAC_PI_2 {
        return None;
    }
    let p = a.clamp(-meio, meio);
    let (s, c) = (libm::sinf(p), libm::cosf(p));
    Some([dir[0] * c - dir[1] * s, dir[0] * s + dir[1] * c])
}

#[cfg(test)]
mod tests {
    use super::rumo_no_leque;

    /// O rumo para quem está dentro do leque é o rumo para ele; fora, a borda; atrás, nenhum.
    ///
    /// **Mutação que deve sangrar:** tirar o `clamp` (o rumo vai para fora do leque).
    #[test]
    fn o_rumo_fica_dentro_do_leque() {
        let meio = 30f32.to_radians();
        let r = rumo_no_leque([0.0, 0.0], [1.0, 0.0], meio, [1.0, 0.2]).expect("dentro");
        assert!(
            (r[1] / r[0] - 0.2).abs() < 1e-5,
            "dentro: o rumo é para ele, {r:?}"
        );
        let b = rumo_no_leque([0.0, 0.0], [1.0, 0.0], meio, [0.0, 1.0]).expect("borda");
        assert!(
            (libm::atan2f(b[1], b[0]) - meio).abs() < 1e-5,
            "fora: a borda de cima, {b:?}"
        );
        assert!(rumo_no_leque([0.0, 0.0], [1.0, 0.0], meio, [-1.0, -0.1]).is_none());
    }
}
