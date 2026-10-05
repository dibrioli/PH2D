//! ⭐⭐ **A ÁREA ANDÁVEL na ponte da navegação** (plano 30 §2.3–2.4, W3 + W6) — *o que é obstáculo
//! neste tique* e *as malhas em dia com isso*. Ver o cabeçalho de [`super`]. ⚠️ Módulo FILHO de
//! `nav.rs`: lê os campos privados da `NavWorld`.
//!
//! # O obstáculo É o colisor
//!
//! Todo colisor sólido (não sensor) de um corpo que NÃO anda nesta altura: os **estáticos** (pela
//! pose autorada) e — W6 — os **cinemáticos PARADOS** (pela pose de agora). ⭐ É isto que faz uma
//! porta: um cinemático que desliza até fechar a passagem e pára passa a recortar a malha no tique
//! em que pára, e deixa de a recortar no tique em que volta a andar (enquanto anda, é o desvio que
//! o evita, como a todo corpo que anda). É o *«recorta só quando parado»* do `NavMeshObstacle` do
//! Unity, sem o relógio de espera dele (um número inventado): parado é a velocidade do solver ZERO,
//! que vai no anel e por isso é a mesma num replay.
//!
//! ⛔⛔ **Um cinemático com um MOVER não é geometria**: os personagens (o herói, os próprios agentes,
//! os projécteis) são corpos cinemáticos, e um herói parado a recortar a malha punha o alvo de todos
//! os perseguidores dentro de um furo — e um agente parado seria obstáculo de si mesmo (Q8).
//!
//! # A malha por mosaicos
//!
//! Cada `(região, raio)` é uma [`TiledMesh`]: uma porta que muda refaz só os mosaicos que toca
//! (medido, plano 30 §14.1). Uma malha que MUDOU devolve-se a quem chama, que faz os agentes dela
//! esquecerem o caminho; as outras não acordam ninguém.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_ecs::{Entity, SimWorld};
use ph2d_nav::V2;
use ph2d_navmesh::{Params, Shape, TILE_M, TiledMesh};
use ph2d_physics::{BodyDesc, ShapeDesc, capsule_vertices, ellipse_vertices};

use super::custo::Custos;
use super::{ChaveMalha, RAIO_POR_METRO, Regiao};
use crate::bridge::PhysicsBridge;
use crate::components::{BodyKind, PlatformPlayer, ProjectileMotion, TopDownPlayer};

/// Um seno e um cosseno — a rotação de uma pose, sem passar por um ângulo.
type Rot = (f32, f32);

impl PhysicsBridge {
    /// ⭐ **Põe em dia as malhas que os agentes pedem neste tique** — e só essas: uma chave que
    /// ninguém pede é esquecida (*a área andável é DE QUEM anda*). Devolve as que MUDARAM.
    ///
    /// ⭐⭐ (W15) **«Mudou» é o CONTEÚDO contra o tique anterior DESTA corrida** (a assinatura de cada
    /// malha, que entra no anel), e não o que a actualização diz: depois de um scrub as malhas são as do
    /// fim da corrida, e uma porta que mudou depois do âncora fazia o replay ver uma mudança que a
    /// corrida não viu (plano 30 §23.1). Uma malha cuja actualização partiu de OUTRO conteúdo fica em
    /// `sem_zona` (a zona do que mudou é relativa a outra malha).
    ///
    /// (W7) Cada malha recebe as áreas de custo, os furos proibidos e os das zonas que ESSA chave
    /// evita (`evita[chave]`, índices em `custos.ferem` — ver `nav_custo.rs`).
    pub(super) fn malhas_em_dia(
        &mut self,
        sim: &SimWorld,
        regioes: &[Regiao],
        chaves: &BTreeMap<ChaveMalha, Vec<usize>>,
        custos: &Custos,
    ) -> BTreeSet<ChaveMalha> {
        self.nav.meshes.retain(|k, _| chaves.contains_key(k));
        self.nav.walls.retain(|k, _| chaves.contains_key(k));
        self.nav.sinais.retain(|k, _| chaves.contains_key(k));
        self.nav.sem_zona.clear();
        let mut mudou = BTreeSet::new();
        if chaves.is_empty() {
            return mudou;
        }
        let obstaculos = self.obstaculos(&quem_anda(sim));
        for (&chave, evita) in chaves {
            let (regiao, raio, _) = chave;
            let Some(r) = regioes.iter().find(|r| r.entity == regiao) else {
                continue;
            };
            let [lo, hi] = r.rect;
            let poligono: Vec<V2> = [
                [lo[0], lo[1]],
                [hi[0], lo[1]],
                [hi[0], hi[1]],
                [lo[0], hi[1]],
            ]
            .iter()
            .map(|p| [f64::from(p[0]), f64::from(p[1])])
            .collect();
            let dela: Vec<&Shape> = obstaculos
                .iter()
                .filter(|(camada, _)| *camada < 8 && r.layers & (1u8 << *camada) != 0)
                .map(|(_, s)| s)
                .chain(custos.proibidas.iter())
                .chain(evita.iter().map(|&i| &custos.ferem[i].1))
                .collect();
            // ⚠️ Uma malha NOVA refaz todos os mosaicos dela, logo o `update` diz que mudou (o agente que
            // vem de outra esquece o caminho); a região vazia não tem polígonos e a condução diz
            // «sem caminho» sozinha.
            let malha = self.nav.meshes.entry(chave).or_insert_with(|| {
                TiledMesh::new(
                    Params {
                        agent_radius: f64::from(raio as f32 / RAIO_POR_METRO),
                        ..Params::default()
                    },
                    TILE_M,
                )
            });
            let base = malha.assinatura();
            malha.update_with_areas(&poligono, &dela, &custos.areas);
            let agora = malha.assinatura();
            let antes = self.nav.sinais.insert(chave, agora.unwrap_or(0));
            if antes != agora {
                mudou.insert(chave);
            }
            if base != antes {
                self.nav.sem_zona.insert(chave);
            }
        }
        mudou
    }

    /// Os obstáculos DESTE tique, com a camada de cada um, pela ordem das entidades — os corpos e as
    /// peças, com a descrição em MUNDO (a pose da peça já composta com a do dono). Ver o cabeçalho.
    fn obstaculos(&self, movers: &BTreeSet<Entity>) -> Vec<(u8, Shape)> {
        let mut out = Vec::new();
        for (&e, b) in &self.bodies {
            if b.rest.is_sensor || movers.contains(&e) {
                continue;
            }
            if let Some((d, rot)) = self.pose_de_obstaculo(b.kind, b.handle, &b.rest) {
                out.push((d.layer, forma(&d, rot)));
            }
        }
        for part in self.parts.values() {
            let Some(dono) = self.bodies.get(&part.owner) else {
                continue;
            };
            if part.rest.is_sensor || movers.contains(&part.owner) {
                continue;
            }
            let Some((d_dono, (s, c))) = self.pose_de_obstaculo(dono.kind, dono.handle, &dono.rest)
            else {
                continue;
            };
            let [lx, ly, lr] = part.local;
            let mut d = part.rest;
            d.x = d_dono.x + c * lx - s * ly;
            d.y = d_dono.y + s * lx + c * ly;
            d.rotation = d_dono.rotation + lr;
            // O estático pela rotação SOMADA (a resposta de sempre, ao bit); o cinemático compõe
            // a do dono, que não tem ângulo.
            let rot = if dono.kind == BodyKind::Static {
                libm::sincosf(d.rotation)
            } else {
                let (sl, cl) = libm::sincosf(lr);
                (s * cl + c * sl, c * cl - s * sl)
            };
            out.push((d.layer, forma(&d, rot)));
        }
        out
    }

    /// A pose de um corpo COMO OBSTÁCULO — `None` se ele não o é neste tique (anda, ou é dinâmico).
    pub(super) fn pose_de_obstaculo(
        &self,
        kind: BodyKind,
        handle: ph2d_physics::RigidBodyHandle,
        rest: &BodyDesc,
    ) -> Option<(BodyDesc, Rot)> {
        match kind {
            BodyKind::Static => Some((*rest, libm::sincosf(rest.rotation))),
            BodyKind::Kinematic => {
                let parado = self.world.body_velocity(handle)? == [0.0, 0.0]
                    && self.world.body_angvel(handle)? == 0.0;
                if !parado {
                    return None;
                }
                let pose = self.world.body_pose(handle)?;
                let mut d = *rest;
                d.x = pose.translation.x;
                d.y = pose.translation.y;
                Some((d, (pose.rotation.im, pose.rotation.re)))
            }
            _ => None,
        }
    }
}

/// Quem tem um MOVER (personagens e projécteis) — ver o cabeçalho.
pub(super) fn quem_anda(sim: &SimWorld) -> BTreeSet<Entity> {
    let world = sim.world();
    let mut out = BTreeSet::new();
    if let Some(mut q) = world.try_query::<(Entity, &TopDownPlayer)>() {
        out.extend(q.iter(world).map(|(e, _)| e));
    }
    if let Some(mut q) = world.try_query::<(Entity, &PlatformPlayer)>() {
        out.extend(q.iter(world).map(|(e, _)| e));
    }
    if let Some(mut q) = world.try_query::<(Entity, &ProjectileMotion)>() {
        out.extend(q.iter(world).map(|(e, _)| e));
    }
    out
}

/// A forma de um obstáculo, em mundo, a partir da descrição do corpo e da rotação dele.
///
/// ⚠️ **A elipse e o estádio usam os MESMOS vértices que o solver** ([`ellipse_vertices`],
/// [`capsule_vertices`]): o colisor desses dois É aquele polígono, logo a malha recua a parede que o
/// corpo de facto bate.
pub(super) fn forma(d: &BodyDesc, (s, c): Rot) -> Shape {
    let rot = |p: [f32; 2]| [c * p[0] - s * p[1], s * p[0] + c * p[1]];
    let o = rot(d.offset);
    let centro = [d.x + o[0], d.y + o[1]];
    let mundo = |p: [f32; 2]| {
        let q = rot(p);
        [f64::from(centro[0] + q[0]), f64::from(centro[1] + q[1])]
    };
    match d.shape {
        ShapeDesc::Ball { radius } => Shape::Circle {
            center: mundo([0.0, 0.0]),
            radius: f64::from(radius.abs()),
        },
        ShapeDesc::Cuboid { half_x, half_y } => Shape::Convex(
            [
                [-half_x, -half_y],
                [half_x, -half_y],
                [half_x, half_y],
                [-half_x, half_y],
            ]
            .into_iter()
            .map(mundo)
            .collect(),
        ),
        ShapeDesc::Ellipse { rx, ry } => {
            Shape::Convex(ellipse_vertices(rx, ry).into_iter().map(mundo).collect())
        }
        ShapeDesc::Capsule {
            half_height,
            radius,
        } => Shape::Capsule {
            a: mundo([0.0, -half_height]),
            b: mundo([0.0, half_height]),
            radius: f64::from(radius.abs()),
        },
        ShapeDesc::Stadium {
            half_height,
            rx,
            ry,
        } => Shape::Convex(
            capsule_vertices(half_height, rx, ry)
                .into_iter()
                .map(mundo)
                .collect(),
        ),
    }
}
