//! **A PONTE da NAVEGAÇÃO** (plano 30, W3) — a lei pura [`ph2d_nav`] corrida contra o mundo, UMA vez
//! por tique, ANTES dos movers.
//!
//! # ⭐⭐⭐ O agente PEDE e o mover ANDA
//!
//! Por tique, cada agente activo: lê a posição do CORPO dele (a do solver, não a do `Transform` —
//! num replay o `Transform` é de outro tique), escolhe a região que o contém, pede à condução
//! ([`ph2d_nav::agent::step`]) uma direcção, e escreve-a no canal de entrada do mover
//! ([`PhysicsBridge::set_player_input`]) — o mesmo que o teclado usa. ⇒ o [`TopDownPlayer`] que corre
//! a seguir, no MESMO tique, acelera, trava e desliza na parede com a lei que já foi medida contra o
//! oráculo (TOP-20 #13). ⛔ Um mover próprio seria uma segunda lei de movimento.
//!
//! ⚠️ **É por isto que esta ponte corre DENTRO da porta dos controladores** e não ao lado dela: os
//! dois laços que andam o relógio (o da frente e o do replay) chamam essa porta, e a intenção tem de
//! ser reescrita antes do mover nos dois — senão um scrub replayava um agente parado.
//!
//! # ⚠️ A malha andável é DERIVADA e nunca gravada
//!
//! Ela sai dos corpos ESTÁTICOS (e das peças de corpos estáticos) que não são sensores e cuja camada
//! está na máscara da região, recuados pelo raio do agente — **uma malha por `(região, raio)`**, com
//! o raio arredondado PARA CIMA a `1/256 m` (dois agentes de raios quase iguais partilham a malha, e
//! o arredondamento nunca dá menos folga do que o corpo pede). Uma ASSINATURA dos obstáculos e das
//! regiões decide quando reconstruir; reconstruir esquece o caminho de todo agente.
//!
//! # ⚠️ Os eventos saem por tique e só se PUBLICAM no laço da frente
//!
//! A condução acontece antes do passo e a publicação depois dele, na porta irmã — com
//! `publicar = false` no replay: um scrub não é uma tempestade de «chegou».

use std::collections::{BTreeMap, BTreeSet};

use ph2d_ecs::{Entity, SimWorld};
use ph2d_nav::{AgentConfig, AgentRuntime, Event, NavMesh, Polyanya, V2};
use ph2d_navmesh::{Params, Shape};
use ph2d_physics::{BodyDesc, ShapeDesc, capsule_vertices, ellipse_vertices};

use super::PhysicsBridge;
use crate::PlayerInput;
use crate::components::{BodyKind, NavAgent, NavRegion, NavTarget, PlatformPlayer, TopDownPlayer};

/// A resolução do raio na chave da malha: `1/256 m`.
const RAIO_POR_METRO: f32 = 256.0;

/// **Um facto de navegação** — quem, e o quê.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NavEvent {
    pub agent: Entity,
    pub kind: Event,
}

/// O estado da navegação na ponte: as malhas derivadas, a memória de cada agente, e os factos.
pub(super) struct NavWorld {
    /// As malhas, por `(região, raio em 1/256 m)`.
    meshes: BTreeMap<(Entity, u32), NavMesh>,
    /// A assinatura dos obstáculos e regiões com que as malhas foram construídas.
    sig: Option<u64>,
    /// ⚠️ **A memória de cada agente** — entra no anel pelo [`super::tape::ControllerMemory`].
    pub(super) agents: BTreeMap<Entity, AgentRuntime>,
    /// Os buffers da procura (reaproveitados; nenhum estado entre consultas).
    search: Polyanya,
    /// Os factos DESTE tique, à espera da porta de depois do passo.
    tick_events: Vec<NavEvent>,
    /// Os factos DESTE dispatch, publicados.
    events: Vec<NavEvent>,
}

impl Default for NavWorld {
    fn default() -> Self {
        Self {
            meshes: BTreeMap::new(),
            sig: None,
            agents: BTreeMap::new(),
            search: Polyanya::new(),
            tick_events: Vec::new(),
            events: Vec::new(),
        }
    }
}

impl NavWorld {
    /// Esquece tudo o que é chaveado por `Entity` (os bits são reciclados num `rebuild`).
    pub(super) fn clear_all(&mut self) {
        self.meshes.clear();
        self.sig = None;
        self.agents.clear();
        self.tick_events.clear();
        self.events.clear();
    }
}

/// O que a condução lê de um agente neste tique (cópia leve, sem as strings dos sinais).
#[derive(Copy, Clone)]
struct Pedido {
    entity: Entity,
    target: NavTarget,
    radius: f32,
    arrive: f32,
    repath: f32,
    stuck: f32,
    active: bool,
}

/// Uma região deste tique: a entidade, o rectângulo de mundo e a máscara de camadas.
#[derive(Copy, Clone)]
struct Regiao {
    entity: Entity,
    rect: [[f32; 2]; 2],
    layers: u8,
}

impl PhysicsBridge {
    /// **Um tique de TODOS os agentes.** Ver o cabeçalho do módulo.
    pub(super) fn drive_nav_agents(&mut self, sim: &SimWorld) {
        self.nav.tick_events.clear();
        let world = sim.world();
        let pedidos: Vec<Pedido> = match world.try_query::<(Entity, &NavAgent)>() {
            Some(mut q) => q
                .iter(world)
                .map(|(entity, a)| Pedido {
                    entity,
                    target: a.target,
                    radius: a.radius,
                    arrive: a.arrive_distance,
                    repath: a.repath_distance,
                    stuck: a.stuck_after_s,
                    active: a.active,
                })
                .collect(),
            None => Vec::new(),
        };
        if pedidos.is_empty() {
            self.nav.agents.clear();
            return;
        }
        let vivos: BTreeSet<Entity> = pedidos.iter().map(|p| p.entity).collect();
        self.nav.agents.retain(|e, _| vivos.contains(e));

        let regioes = regioes(sim);
        let sig = self.assinatura(&regioes);
        if self.nav.sig != Some(sig) {
            self.nav.meshes.clear();
            self.nav.sig = Some(sig);
            for rt in self.nav.agents.values_mut() {
                rt.forget_path();
            }
        }
        let dt = f64::from(self.world.dt());
        if !(dt.is_finite() && dt > 0.0) {
            return;
        }

        for p in pedidos {
            let Some(body) = self.bodies.get(&p.entity).copied() else {
                continue;
            };
            // ⚠️ **O agente só conduz um mover que o ouve**: um `TopDownPlayer` com os controlos de
            // fábrica desligados, e sem um `PlatformPlayer` na mesma entidade (esse ganharia a pose).
            // Quem AVISA o artista é o Inspector; aqui decide-se, e sempre igual.
            let Some(mover) = world.get::<TopDownPlayer>(p.entity).copied() else {
                continue;
            };
            if mover.default_controls || world.get::<PlatformPlayer>(p.entity).is_some() {
                continue;
            }
            let mut rt = self.nav.agents.remove(&p.entity).unwrap_or_default();
            if !p.active {
                rt.forget_path();
                rt.status = ph2d_nav::Status::Idle;
                self.player_input.insert(p.entity, PlayerInput::default());
                self.nav.agents.insert(p.entity, rt);
                continue;
            }
            let Some(pose) = self.world.body_pose(body.handle) else {
                self.nav.agents.insert(p.entity, rt);
                continue;
            };
            let pos: V2 = [f64::from(pose.translation.x), f64::from(pose.translation.y)];
            let raio = if p.radius > 0.0 {
                p.radius
            } else {
                raio_que_envolve(&body.rest)
            };
            let chave_raio = (raio * RAIO_POR_METRO).ceil().max(0.0) as u32;
            let regiao = regioes.iter().copied().find(|r| contem(r.rect, pos));
            if let Some(r) = regiao
                && !self.nav.meshes.contains_key(&(r.entity, chave_raio))
            {
                let malha = self.constroi_malha(r, chave_raio as f32 / RAIO_POR_METRO);
                self.nav.meshes.insert((r.entity, chave_raio), malha);
            }
            let alvo = match p.target {
                NavTarget::None => None,
                NavTarget::Point(q) => Some([f64::from(q[0]), f64::from(q[1])]),
                NavTarget::Named(id) => self.posicao_do_alvo(sim, id),
            };
            let cfg = AgentConfig {
                arrive_distance: f64::from(p.arrive.max(0.0)),
                repath_distance: f64::from(p.repath.max(0.0)),
                stuck_after_s: f64::from(p.stuck.max(0.0)),
                speed: f64::from(mover.speed.max(0.0)),
            };
            let NavWorld { meshes, search, .. } = &mut self.nav;
            let malha = regiao.and_then(|r| meshes.get(&(r.entity, chave_raio)));
            let steer = ph2d_nav::agent::step(&mut rt, malha, search, pos, alvo, &cfg, dt);
            self.player_input.insert(
                p.entity,
                PlayerInput {
                    drive: steer.dir[0] as f32,
                    drive_y: steer.dir[1] as f32,
                    ..PlayerInput::default()
                },
            );
            if let Some(kind) = steer.event {
                self.nav.tick_events.push(NavEvent {
                    agent: p.entity,
                    kind,
                });
            }
            self.nav.agents.insert(p.entity, rt);
        }
    }

    /// **A metade de depois do passo**: os factos deste tique publicam-se só no laço da frente.
    pub(super) fn publish_nav_events(&mut self, publicar: bool) {
        if publicar {
            let NavWorld {
                tick_events,
                events,
                ..
            } = &mut self.nav;
            events.append(tick_events);
        } else {
            self.nav.tick_events.clear();
        }
    }

    /// Esquece os factos do dispatch anterior (são do dispatch, como os da vida).
    pub(super) fn discard_nav_events(&mut self) {
        self.nav.events.clear();
    }

    /// **Os factos de navegação deste dispatch** (chegou · sem caminho · preso), pela ordem em que
    /// a condução os produziu. Vazio num replay.
    #[must_use]
    pub fn nav_events(&self) -> &[NavEvent] {
        &self.nav.events
    }

    /// **A memória de um agente, agora** — o caminho, o ponto em curso e o estado. `None` se ele
    /// ainda não correu um tique.
    #[must_use]
    pub fn nav_agent(&self, entity: Entity) -> Option<&AgentRuntime> {
        self.nav.agents.get(&entity)
    }

    /// ⭐ **O caminho de cada agente, como linhas** — o que o overlay desenha, pelo MESMO pintor dos
    /// sensores ([`crate::ProbeMark`], [`crate::ProbeKind::Path`]).
    ///
    /// Do sítio onde o agente ESTÁ ao próximo ponto, e de ponto em ponto até ao fim: o troço já
    /// andado não se desenha (é a pergunta *«para onde ele vai»*, não *«por onde veio»*). O estado
    /// diz se é um caminho até ao alvo (`Hit`) ou só até ao ponto mais perto dele (`Clear`).
    #[must_use]
    pub fn nav_marks(&self) -> Vec<crate::ProbeMark> {
        let mut out = Vec::new();
        for (&e, rt) in &self.nav.agents {
            if rt.next == 0 || rt.next >= rt.path.len() {
                continue;
            }
            let Some(b) = self.bodies.get(&e) else {
                continue;
            };
            let Some(p) = self.world.body_pose(b.handle) else {
                continue;
            };
            let mut a = [p.translation.x, p.translation.y];
            for q in &rt.path[rt.next..] {
                let q = [q[0] as f32, q[1] as f32];
                let d = [q[0] - a[0], q[1] - a[1]];
                let l = comprimento(d[0], d[1]);
                if l > 1e-6 {
                    let dir = [d[0] / l, d[1] / l];
                    // ⚠️ O «acerto» é o FIM do troço quando o caminho chega ao alvo — o tique do
                    // pintor marca então cada canto; um caminho parcial fica sem eles.
                    let hit = (!rt.partial).then_some(l);
                    out.push(crate::ProbeMark::ray(
                        crate::ProbeKind::Path,
                        a,
                        dir,
                        l,
                        hit,
                        0.0,
                    ));
                }
                a = q;
            }
        }
        out
    }

    /// **As malhas andáveis construídas**, por região e raio (em metros) — o que o overlay desenha.
    pub fn nav_meshes(&self) -> impl Iterator<Item = (Entity, f32, &NavMesh)> {
        self.nav
            .meshes
            .iter()
            .map(|(&(e, r), m)| (e, r as f32 / RAIO_POR_METRO, m))
    }

    /// A posição do alvo com este `stable_name_id`: a do CORPO se ele tiver um (o solver é a verdade
    /// num replay), senão a do `Transform` de mundo.
    fn posicao_do_alvo(&self, sim: &SimWorld, nome: u64) -> Option<V2> {
        let world = sim.world();
        let mut q = world.try_query::<(Entity, &ph2d_ecs::Name)>()?;
        let e = q
            .iter(world)
            .find(|(_, n)| ph2d_ecs::stable_name_id(n.as_str()) == nome)
            .map(|(e, _)| e)?;
        if let Some(b) = self.bodies.get(&e)
            && let Some(p) = self.world.body_pose(b.handle)
        {
            return Some([f64::from(p.translation.x), f64::from(p.translation.y)]);
        }
        let t = ph2d_ecs::world_transform(world, e)?;
        Some([f64::from(t.translation.x), f64::from(t.translation.y)])
    }

    /// A assinatura do que a malha lê: as regiões e os obstáculos estáticos (FNV-1a sobre os bits).
    fn assinatura(&self, regioes: &[Regiao]) -> u64 {
        let mut h = Fnv::new();
        for r in regioes {
            h.u64(r.entity.to_bits());
            for c in r.rect.iter().flatten() {
                h.u32(c.to_bits());
            }
            h.u32(u32::from(r.layers));
        }
        self.para_cada_obstaculo(|e, d| {
            h.u64(e.to_bits());
            h.desc(d);
        });
        h.0
    }

    /// Visita todo colisor ESTÁTICO e sólido — os corpos e as peças de corpos estáticos — com a
    /// descrição em MUNDO (a pose da peça já composta com a do dono).
    fn para_cada_obstaculo(&self, mut visita: impl FnMut(Entity, &BodyDesc)) {
        for (&e, b) in &self.bodies {
            if b.kind == BodyKind::Static && !b.rest.is_sensor {
                visita(e, &b.rest);
            }
        }
        for (&e, part) in &self.parts {
            let Some(dono) = self.bodies.get(&part.owner) else {
                continue;
            };
            if dono.kind != BodyKind::Static || part.rest.is_sensor {
                continue;
            }
            let (s, c) = libm::sincosf(dono.rest.rotation);
            let [lx, ly, lr] = part.local;
            let mut d = part.rest;
            d.x = dono.rest.x + c * lx - s * ly;
            d.y = dono.rest.y + s * lx + c * ly;
            d.rotation = dono.rest.rotation + lr;
            visita(e, &d);
        }
    }

    /// Constrói a malha de uma região para um raio.
    fn constroi_malha(&self, r: Regiao, raio: f32) -> NavMesh {
        let [lo, hi] = r.rect;
        let regiao: Vec<V2> = [
            [lo[0], lo[1]],
            [hi[0], lo[1]],
            [hi[0], hi[1]],
            [lo[0], hi[1]],
        ]
        .iter()
        .map(|p| [f64::from(p[0]), f64::from(p[1])])
        .collect();
        let mut obstaculos = Vec::new();
        self.para_cada_obstaculo(|_, d| {
            if d.layer < 8 && r.layers & (1u8 << d.layer) != 0 {
                obstaculos.push(forma(d));
            }
        });
        let params = Params {
            agent_radius: f64::from(raio),
            ..Params::default()
        };
        match ph2d_navmesh::build(&regiao, &obstaculos, &params) {
            Ok(b) => b.mesh,
            // ⚠️ Uma construção recusada dá uma malha VAZIA, e o agente diz «sem caminho» — o
            // silêncio de um agente parado sem razão é a queixa Q12 da pesquisa.
            Err(_) => NavMesh::from_polygons(Vec::new(), Vec::new())
                .unwrap_or_else(|_| unreachable!("uma malha sem polígonos é sempre válida")),
        }
    }
}

/// As regiões deste tique, pela ordem das entidades.
fn regioes(sim: &SimWorld) -> Vec<Regiao> {
    let world = sim.world();
    let Some(mut q) = world.try_query::<(Entity, &NavRegion)>() else {
        return Vec::new();
    };
    let mut out: Vec<Regiao> = q
        .iter(world)
        .map(|(entity, r)| {
            let c = ph2d_ecs::world_transform(world, entity)
                .map_or([0.0, 0.0], |t| [t.translation.x, t.translation.y]);
            Regiao {
                entity,
                rect: r.rect(c),
                layers: r.obstacle_layers,
            }
        })
        .collect();
    out.sort_by_key(|r| r.entity);
    out
}

fn contem(rect: [[f32; 2]; 2], p: V2) -> bool {
    let [lo, hi] = rect;
    p[0] >= f64::from(lo[0])
        && p[0] <= f64::from(hi[0])
        && p[1] >= f64::from(lo[1])
        && p[1] <= f64::from(hi[1])
}

/// `√(a² + b²)` pela `sqrt`, que é IEEE exacta. ⛔ **Nunca o `f32::hypot`**: ele vai à libm do SO, que
/// erra o último ulp de maneira diferente em cada plataforma, e este caminho entra no hash que o CI
/// compara nos três (o gate `no_std_transcendental_reaches_the_deterministic_hash` apanhou-o).
fn comprimento(a: f32, b: f32) -> f32 {
    (a * a + b * b).sqrt()
}

/// O raio do círculo que envolve o colisor do corpo, contado do CENTRO DO CORPO (o offset soma).
fn raio_que_envolve(d: &BodyDesc) -> f32 {
    let r = match d.shape {
        ShapeDesc::Ball { radius } => radius,
        ShapeDesc::Cuboid { half_x, half_y } => comprimento(half_x, half_y),
        ShapeDesc::Ellipse { rx, ry } => rx.max(ry),
        ShapeDesc::Capsule {
            half_height,
            radius,
        } => half_height + radius,
        ShapeDesc::Stadium {
            half_height,
            rx,
            ry,
        } => half_height + rx.max(ry),
    };
    r.abs() + comprimento(d.offset[0], d.offset[1])
}

/// A forma de um obstáculo, em mundo, a partir da descrição do corpo.
///
/// ⚠️ **A elipse e o estádio usam os MESMOS vértices que o solver** ([`ellipse_vertices`],
/// [`capsule_vertices`]): o colisor desses dois É aquele polígono, logo a malha recua a parede que o
/// corpo de facto bate.
fn forma(d: &BodyDesc) -> Shape {
    let (s, c) = libm::sincosf(d.rotation);
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

/// FNV-1a de 64 bits — a assinatura não precisa de mais do que distinguir.
struct Fnv(u64);

impl Fnv {
    fn new() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }
    fn byte(&mut self, b: u8) {
        self.0 ^= u64::from(b);
        self.0 = self.0.wrapping_mul(0x0100_0000_01b3);
    }
    fn u32(&mut self, v: u32) {
        v.to_le_bytes().into_iter().for_each(|b| self.byte(b));
    }
    fn u64(&mut self, v: u64) {
        v.to_le_bytes().into_iter().for_each(|b| self.byte(b));
    }
    fn desc(&mut self, d: &BodyDesc) {
        for v in [d.x, d.y, d.rotation, d.offset[0], d.offset[1]] {
            self.u32(v.to_bits());
        }
        self.byte(d.layer);
        match d.shape {
            ShapeDesc::Ball { radius } => {
                self.byte(0);
                self.u32(radius.to_bits());
            }
            ShapeDesc::Cuboid { half_x, half_y } => {
                self.byte(1);
                self.u32(half_x.to_bits());
                self.u32(half_y.to_bits());
            }
            ShapeDesc::Ellipse { rx, ry } => {
                self.byte(2);
                self.u32(rx.to_bits());
                self.u32(ry.to_bits());
            }
            ShapeDesc::Capsule {
                half_height,
                radius,
            } => {
                self.byte(3);
                self.u32(half_height.to_bits());
                self.u32(radius.to_bits());
            }
            ShapeDesc::Stadium {
                half_height,
                rx,
                ry,
            } => {
                self.byte(4);
                self.u32(half_height.to_bits());
                self.u32(rx.to_bits());
                self.u32(ry.to_bits());
            }
        }
    }
}

#[cfg(test)]
#[path = "nav_tests.rs"]
mod tests;
