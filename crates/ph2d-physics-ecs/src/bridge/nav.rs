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
//! Ela sai dos corpos que não andam (os estáticos e os cinemáticos parados — [`malha`]) que não são
//! sensores e cuja camada está na máscara da região, recuados pelo raio do agente — **uma malha por
//! `(região, raio)`**, com o raio arredondado PARA CIMA a `1/256 m` (dois agentes de raios quase
//! iguais partilham a malha, e o arredondamento nunca dá menos folga do que o corpo pede). Cada uma
//! é por MOSAICOS: uma porta refaz só os que toca, e só os agentes da malha que mudou esquecem o
//! caminho.
//!
//! # ⭐ O DESVIO entre corpos (plano 30, W5) é a última palavra antes do mover
//!
//! A condução de TODOS os agentes corre primeiro; depois [`ph2d_orca`] corrige a direcção de cada um
//! contra os outros agentes, o herói e todo corpo sólido que anda — e contra as paredes da malha do
//! raio dele (como ela já está recuada pelo raio do corpo, ele é um PONTO contra elas). Os agentes
//! resolvem em sequência pela ordem das ENTIDADES: a mesma nos três sistemas e num replay. Um agente
//! com o desvio desligado vai a direito, e os outros desviam-se dele por inteiro; ninguém se desvia
//! do próprio ALVO (desviar dele seria nunca lhe tocar).
//!
//! # ⚠️ Os eventos saem por tique e só se PUBLICAM no laço da frente
//!
//! A condução acontece antes do passo e a publicação depois dele, na porta irmã — com
//! `publicar = false` no replay: um scrub não é uma tempestade de «chegou».

use std::collections::{BTreeMap, BTreeSet};

use ph2d_ecs::{Entity, SimWorld};
use ph2d_nav::{AgentConfig, AgentRuntime, Event, NavMesh, Polyanya, V2};
use ph2d_navmesh::TiledMesh;
use ph2d_physics::{BodyDesc, ShapeDesc};

use super::PhysicsBridge;
use crate::PlayerInput;
use crate::components::{NavAgent, NavRegion, NavTarget, PlatformPlayer, TopDownPlayer};

/// A resolução do raio na chave da malha: `1/256 m`.
pub(super) const RAIO_POR_METRO: f32 = 256.0;

/// A chave de uma malha: a região, o raio em `1/256 m` e (W7) a assinatura das zonas que o agente
/// evita (`0` = nenhuma) — ver `nav_custo.rs`.
pub(super) type ChaveMalha = (Entity, u32, u64);

/// **Um facto de navegação** — quem, e o quê.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NavEvent {
    pub agent: Entity,
    pub kind: Event,
}

/// O estado da navegação na ponte: as malhas derivadas, a memória de cada agente, e os factos.
pub(super) struct NavWorld {
    /// As malhas, por `(região, raio em 1/256 m, zonas evitadas)`.
    meshes: BTreeMap<ChaveMalha, TiledMesh>,
    /// As paredes de cada malha, como o desvio as lê — derivadas dela, com a mesma chave e a mesma
    /// vida (esquecidas quando ela é).
    walls: BTreeMap<ChaveMalha, ph2d_orca::Walls>,
    /// ⚠️ **A memória de cada agente** — entra no anel pelo [`super::tape::ControllerMemory`].
    pub(super) agents: BTreeMap<Entity, AgentRuntime>,
    /// ⭐ As ORDENS dos verbos `Start/Stop Navigation` (W6) — ver [`ordens`].
    pub(super) ordens: ordens::Ordens,
    /// ⭐ A RONDA de cada agente em patrulha (W6) — ⚠️ entra no anel. Ver [`alvo`].
    pub(super) rondas: BTreeMap<Entity, alvo::Ronda>,
    /// A árvore das tags do documento (para *a tag mais perto*), entregue pela shell.
    arvore: ph2d_tags::TagTree,
    /// (W7) Quem é cada atalho pelo `id` que a lei devolve (o índice da entidade) — do último tique.
    pub(super) atalhos: BTreeMap<u32, Entity>,
    /// Os buffers da procura (reaproveitados; nenhum estado entre consultas).
    search: Polyanya,
    /// O AVANÇO do último desvio quando OUTRO corpo cortou o pedido: a fracção da rapidez que ele
    /// deixou ir pelo caminho (ausente = `1`). Derivado por tique; só o `NavNow` o lê.
    pub(super) avanco: BTreeMap<Entity, f32>,
    /// Os factos DESTE tique, à espera da porta de depois do passo.
    tick_events: Vec<NavEvent>,
    /// Os factos DESTE dispatch, publicados.
    events: Vec<NavEvent>,
}

impl Default for NavWorld {
    fn default() -> Self {
        Self {
            meshes: BTreeMap::new(),
            walls: BTreeMap::new(),
            agents: BTreeMap::new(),
            ordens: ordens::Ordens::default(),
            rondas: BTreeMap::new(),
            arvore: ph2d_tags::TagTree::default(),
            atalhos: BTreeMap::new(),
            avanco: BTreeMap::new(),
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
        self.walls.clear();
        self.agents.clear();
        self.ordens = ordens::Ordens::default();
        self.rondas.clear();
        self.atalhos.clear();
        self.avanco.clear();
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
    avoidance: bool,
    avoid_harm: bool,
}

/// Um agente que a ponte conduz neste tique: o pedido, a velocidade do mover, onde está, o raio e a
/// malha que pede (`None` fora de toda região).
struct Vez {
    p: Pedido,
    /// O alvo que vale neste tique: o de uma ordem `Start` com nome, ou o autorado.
    target: NavTarget,
    speed: f64,
    pos: V2,
    raio: f32,
    chave: Option<ChaveMalha>,
}

/// Uma região deste tique: a entidade, o rectângulo de mundo e a máscara de camadas.
#[derive(Copy, Clone)]
pub(super) struct Regiao {
    pub(super) entity: Entity,
    pub(super) rect: [[f32; 2]; 2],
    pub(super) layers: u8,
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
                    avoidance: a.avoidance,
                    avoid_harm: a.avoid_harm,
                })
                .collect(),
            None => Vec::new(),
        };
        if pedidos.is_empty() {
            self.nav.agents.clear();
            self.nav.meshes.clear();
            self.nav.walls.clear();
            return;
        }
        let vivos: BTreeSet<Entity> = pedidos.iter().map(|p| p.entity).collect();
        self.nav.agents.retain(|e, _| vivos.contains(e));
        let dt = f64::from(self.world.dt());
        if !(dt.is_finite() && dt > 0.0) {
            return;
        }
        let regioes = regioes(sim);
        // (W7) O que custa, o que fere e os atalhos deste tique.
        let custos = self.custos_deste_tique(sim);
        let (links, quem_atalho) = self.atalhos_deste_tique(sim);
        self.nav.atalhos = quem_atalho;
        let mut evita_por: BTreeMap<ChaveMalha, Vec<usize>> = BTreeMap::new();

        // 1.ª passagem: quem a ponte conduz, onde está, e que malha pede.
        let mut vez: Vec<Vez> = Vec::with_capacity(pedidos.len());
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
            // ⭐ A ORDEM de um verbo manda mais que o autorado (W6), e nunca o reescreve.
            let ordem = self
                .nav
                .ordens
                .em_vigor
                .get(&p.entity)
                .copied()
                .unwrap_or_default();
            if !ordem.ligado.unwrap_or(p.active) {
                let mut rt = self.nav.agents.remove(&p.entity).unwrap_or_default();
                rt.forget_path();
                rt.status = ph2d_nav::Status::Idle;
                self.player_input.insert(p.entity, PlayerInput::default());
                self.nav.agents.insert(p.entity, rt);
                continue;
            }
            let Some(pose) = self.world.body_pose(body.handle) else {
                self.nav.agents.entry(p.entity).or_default();
                continue;
            };
            let pos: V2 = [f64::from(pose.translation.x), f64::from(pose.translation.y)];
            let raio = if p.radius > 0.0 {
                p.radius
            } else {
                raio_que_envolve(&body.rest)
            };
            let chave_raio = (raio * RAIO_POR_METRO).ceil().max(0.0) as u32;
            let evita = self.zonas_que_evita(sim, p.entity, p.avoid_harm, &custos);
            let quem: Vec<Entity> = evita.iter().map(|&i| custos.ferem[i].0).collect();
            let chave = regioes
                .iter()
                .find(|r| contem(r.rect, pos))
                .map(|r| (r.entity, chave_raio, custo::assinatura(&quem)));
            if let Some(k) = chave {
                evita_por.insert(k, evita);
            }
            vez.push(Vez {
                p,
                target: if ordem.alvo != 0 {
                    NavTarget::Named(ordem.alvo)
                } else {
                    p.target
                },
                speed: f64::from(mover.speed.max(0.0)),
                pos,
                raio,
                chave,
            });
        }
        let mudou = self.malhas_em_dia(sim, &regioes, &evita_por, &custos);
        let q = ph2d_nav::Query {
            costs: &custos.tabela,
            links: &links,
        };

        // 2.ª passagem: a condução, contra as malhas em dia.
        let mut pedidas: Vec<desvio::Pedida> = Vec::with_capacity(vez.len());
        let mut por_tag = alvo::PorTag::new();
        for v in vez {
            let p = v.p;
            let mut rt = self.nav.agents.remove(&p.entity).unwrap_or_default();
            if v.chave.is_some_and(|k| mudou.contains(&k)) {
                rt.forget_path();
            }
            let chegada = f64::from(p.arrive.max(0.0));
            let (quem, alvo) = self.alvo_de(sim, p.entity, v.target, v.pos, chegada, &mut por_tag);
            let cfg = AgentConfig {
                arrive_distance: f64::from(p.arrive.max(0.0)),
                repath_distance: f64::from(p.repath.max(0.0)),
                stuck_after_s: f64::from(p.stuck.max(0.0)),
                speed: v.speed,
            };
            let NavWorld { meshes, search, .. } = &mut self.nav;
            let malha = v.chave.and_then(|k| meshes.get(&k)).map(TiledMesh::mesh);
            let steer =
                ph2d_nav::agent::step_with(&mut rt, malha, search, &q, v.pos, alvo, &cfg, dt);
            // (W7) Um TELEPORTE: o corpo vai já para a saída (velocidade a zero), dentro do tique —
            // o replay corre a mesma lei e salta no mesmo tique.
            if let Some(p2) = steer.teleport
                && let Some(b) = self.bodies.get(&p.entity)
                && let Some(pose) = self.world.body_pose(b.handle)
            {
                let rot = libm::atan2f(pose.rotation.im, pose.rotation.re);
                self.world
                    .set_body_pose(b.handle, p2[0] as f32, p2[1] as f32, rot, true);
            }
            if let Some(id) = steer.crossed {
                self.nav.tick_events.push(NavEvent {
                    agent: p.entity,
                    kind: Event::Crossed(id),
                });
            }
            pedidas.push(desvio::Pedida {
                entity: p.entity,
                pos: v.pos,
                dir: steer.dir,
                speed: cfg.speed,
                raio: f64::from(v.raio),
                malha: v.chave,
                avoidance: p.avoidance,
                alvo: quem,
            });
            if let Some(kind) = steer.event {
                self.nav.tick_events.push(NavEvent {
                    agent: p.entity,
                    kind,
                });
            }
            self.nav.agents.insert(p.entity, rt);
        }
        self.desvia(pedidas, dt);
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

    /// (W7) Quem é o atalho com este `id` (o que a lei devolve no `Event::Crossed`).
    #[must_use]
    pub fn nav_atalho(&self, id: u32) -> Option<&Entity> {
        self.nav.atalhos.get(&id)
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

    /// ⭐ **A ÁREA ANDÁVEL, como linhas** (plano 30, W4) — as paredes de cada malha construída, uma
    /// por `(região, raio)`: a fronteira por onde o CENTRO de um agente daquele raio pode andar.
    ///
    /// ⚠️ **Uma malha por raio, logo um contorno por raio** — é isso que mostra porque o agente
    /// GRANDE não passa a porta por onde o pequeno passa: o contorno dele fecha-a. E só há malha para
    /// os raios que um agente pede, logo uma região sem agentes não desenha nada (*a área andável é
    /// DE QUEM anda*).
    #[must_use]
    pub fn nav_mesh_marks(&self) -> Vec<crate::ProbeMark> {
        let mut out = Vec::new();
        for mesh in self.nav.meshes.values().map(TiledMesh::mesh) {
            for &(de, para) in mesh.walls() {
                let a = mesh.vert(de);
                let b = mesh.vert(para);
                let a = [a[0] as f32, a[1] as f32];
                let d = [b[0] as f32 - a[0], b[1] as f32 - a[1]];
                let l = comprimento(d[0], d[1]);
                if l <= 1e-6 {
                    continue;
                }
                out.push(crate::ProbeMark {
                    kind: crate::ProbeKind::NavEdge,
                    state: crate::ProbeState::Idle,
                    shape: crate::ProbeShape::Ray {
                        origin: a,
                        dir: [d[0] / l, d[1] / l],
                        reach: l,
                        hit: None,
                        skin: 0.0,
                    },
                });
            }
        }
        out
    }

    /// **Publica o agente AGORA no mundo** ([`crate::NavNow`]) — no fim de todo `dispatch`, pelas
    /// quatro saídas dele, como a vida (o único ponto por onde todas passam).
    ///
    /// ⚠️ **Só escreve quando MUDA**, e quem a ponte não conduz perde o readout: um agente sem memória
    /// (antes do 1.º tique, ou saltado por não ter mover) não tem número de agora.
    pub(super) fn publica_navegacao(&self, sim: &mut SimWorld) {
        let w = sim.world_mut();
        let mut velhos: Vec<Entity> = Vec::new();
        if let Some(mut q) = w.try_query::<(Entity, &crate::NavNow)>() {
            velhos.extend(
                q.iter(w)
                    .filter(|(e, _)| !self.nav.agents.contains_key(e))
                    .map(|(e, _)| e),
            );
        }
        for e in velhos {
            w.entity_mut(e).remove::<crate::NavNow>();
        }
        for (&e, rt) in &self.nav.agents {
            let Some(b) = self.bodies.get(&e) else {
                continue;
            };
            let Some(p) = self.world.body_pose(b.handle) else {
                continue;
            };
            let pos = [f64::from(p.translation.x), f64::from(p.translation.y)];
            let autorado = w.get::<NavAgent>(e).map_or(0.0, |a| a.radius);
            let ordem = self
                .nav
                .ordens
                .em_vigor
                .get(&e)
                .copied()
                .unwrap_or_default();
            let agora = crate::NavNow {
                ordem: ordem.ligado,
                alvo_da_ordem: ordem.alvo,
                avanco: self.nav.avanco.get(&e).copied().unwrap_or(1.0),
                status: rt.status,
                remaining: rt.remaining(pos) as f32,
                radius: if autorado > 0.0 {
                    autorado
                } else {
                    raio_que_envolve(&b.rest)
                },
            };
            let Ok(mut em) = w.get_entity_mut(e) else {
                continue;
            };
            match em.get_mut::<crate::NavNow>() {
                Some(n) if *n == agora => {}
                Some(mut n) => *n = agora,
                None => {
                    em.insert(agora);
                }
            }
        }
    }

    /// **As malhas andáveis construídas**, por região e raio (em metros) — o que o overlay desenha.
    pub fn nav_meshes(&self) -> impl Iterator<Item = (Entity, f32, &NavMesh)> {
        self.nav
            .meshes
            .iter()
            .map(|(&(e, r, _), m)| (e, r as f32 / RAIO_POR_METRO, m.mesh()))
    }

    /// Quem tem este `stable_name_id`.
    fn entidade_do_alvo(&self, sim: &SimWorld, nome: u64) -> Option<Entity> {
        let world = sim.world();
        let mut q = world.try_query::<(Entity, &ph2d_ecs::Name)>()?;
        q.iter(world)
            .find(|(_, n)| ph2d_ecs::stable_name_id(n.as_str()) == nome)
            .map(|(e, _)| e)
    }

    /// A posição de uma entidade: a do CORPO se ela tiver um (o solver é a verdade num replay),
    /// senão a do `Transform` de mundo.
    fn posicao_de(&self, sim: &SimWorld, e: Entity) -> Option<V2> {
        let world = sim.world();
        if let Some(b) = self.bodies.get(&e)
            && let Some(p) = self.world.body_pose(b.handle)
        {
            return Some([f64::from(p.translation.x), f64::from(p.translation.y)]);
        }
        let t = ph2d_ecs::world_transform(world, e)?;
        Some([f64::from(t.translation.x), f64::from(t.translation.y)])
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

#[path = "nav_desvio.rs"]
mod desvio;

#[path = "nav_malha.rs"]
mod malha;

#[path = "nav_ordens.rs"]
mod ordens;

#[path = "nav_alvo.rs"]
mod alvo;

#[path = "nav_custo.rs"]
mod custo;
pub use alvo::Ronda;
pub use ordens::{OrdemDeNavegacao, PedidoDeNavegacao};

#[cfg(test)]
#[path = "nav_tests.rs"]
mod tests;
