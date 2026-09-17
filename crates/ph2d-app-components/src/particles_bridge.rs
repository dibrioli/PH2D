//! ⭐⭐⭐ **A PONTE dos emissores de partículas** (TOP-20 #18) — a lei vive na `ph2d-particles`;
//! aqui ela encontra o QUADRO: quando corre, o que desenha, o que os sinais lhe fazem e o que
//! acontece ao rebobinar. Plano: `docs/Components/14_plano_particle_emitter.md`.
//!
//! # As leis desta ponte
//!
//! - ⭐ **A corrida é o relógio A ANDAR** (`playhead.is_playing()`) — a lei da fábrica, do
//!   projéctil e do script. Parado, as partículas ficam onde estão e **continuam a desenhar-se**.
//! - ⭐⭐ **Um passo fixo, UM passo do emissor** — `ticks × dt` numa chamada faria o replay
//!   depender da taxa de quadros.
//! - ⭐⭐⭐ **Rebobinar é RENASCER** — as corridas são deitadas fora, e o emissor volta ao instante
//!   zero dele. É a mesma lei do `ph2d_ecs::rewind_runtime`, e ela vive aqui porque o que uma
//!   corrida de partículas guarda **não é um componente**: registá-lo poria cada tique na pilha de
//!   `Ctrl+Z`.
//! - ⛔ **Nada do que as partículas fazem toca no mundo** — elas não são entidades, não escrevem
//!   `Transform` nenhum e por isso **não passam pelo ledger** do `preview_drive`. A única coisa
//!   que sai daqui são instâncias para desenhar e um sinal quando a emissão acaba.
//!
//! ⚠️ **A ordem é a da IDENTIDADE** (`StableId`, depois os bits): dois emissores desenham sempre
//! na mesma ordem, em qualquer máquina.

use std::collections::BTreeMap;

use ph2d_ecs::{Entity, ParticleEmitter, SimWorld, StableId};
use ph2d_particles::{EmitterRun, Pose};
use ph2d_render::RenderInstance;

/// **O que a ponte fez neste quadro.**
#[derive(Debug, Default)]
pub struct ParticlesFrame {
    /// Emissores que ACABARAM neste quadro, com o nome do sinal — pela ordem da identidade.
    pub finished: Vec<(u64, String)>,
    /// Quantas partículas foram desenhadas.
    pub drawn: usize,
}

/// **Os emissores a correr** — um por objecto com o componente, fora do mundo.
pub struct ParticlesState {
    registry: ph2d_node_registry::NodeRegistry,
    runs: BTreeMap<u64, EmitterRun>,
    /// As instâncias deste quadro, no mundo, prontas para o passo de sprites.
    pub instances: Vec<RenderInstance>,
    /// O recorte do átlas que uma partícula desenha — o ladrilho branco, o mesmo que o Motion usa.
    pub uv: [f32; 4],
}

impl Default for ParticlesState {
    fn default() -> Self {
        Self::new()
    }
}

/// A pose de MUNDO de uma entidade, na convenção da `RenderInstance`.
fn pose_of(sim: &SimWorld, entity: Entity) -> Pose {
    let Some(t) = ph2d_ecs::transform_inverse::world_transform(sim.world(), entity) else {
        return Pose::IDENTITY;
    };
    let m = ph2d_ecs::GlobalTransform::from_transform(t).matrix;
    Pose {
        translation: [m.z_axis.x, m.z_axis.y],
        basis: [m.x_axis.x, m.x_axis.y, m.y_axis.x, m.y_axis.y],
    }
}

/// Os emissores da cena, pela ordem da identidade.
fn actors(sim: &mut SimWorld) -> Vec<(u64, u64, ParticleEmitter)> {
    let world = sim.world_mut();
    let mut out: Vec<(u64, u64, ParticleEmitter)> = world
        .query::<(Entity, &ParticleEmitter, Option<&StableId>)>()
        .iter(world)
        .map(|(e, cfg, sid)| (sid.map_or(0, |s| s.0), e.to_bits(), cfg.clone()))
        .collect();
    out.sort_by_key(|(sid, bits, _)| (*sid, *bits));
    out
}

impl ParticlesState {
    /// Um estado vazio, com o registo dos nós que o compilador usa.
    #[must_use]
    pub fn new() -> Self {
        Self {
            registry: ph2d_particles::node_registry(),
            runs: BTreeMap::new(),
            instances: Vec::new(),
            uv: [0.0, 0.0, 1.0, 1.0],
        }
    }

    /// Quantos emissores estão a correr.
    #[must_use]
    pub fn live_count(&self) -> usize {
        self.runs.len()
    }

    /// **Quantas partículas de UM objecto vivem agora** — o número que o Inspector mostra.
    ///
    /// ⚠️ **Por OBJECTO e não o total**: a pergunta que o painel responde é *«este emissor está a
    /// fazer alguma coisa?»*, e um total tornaria a resposta dependente do vizinho.
    /// `0` também é a resposta de quem ainda não nasceu — que é exactamente o que a linha de aviso
    /// da secção quer dizer.
    #[must_use]
    pub fn alive_of(&self, bits: u64) -> usize {
        self.runs.get(&bits).map_or(0, EmitterRun::alive)
    }

    /// **O quadro** — ouve os sinais, anda o relógio, e deixa em [`Self::instances`] o que desenhar.
    ///
    /// `heard` são os nomes que soaram neste quadro; `rank_of` dá a profundidade de desenho de um
    /// objecto (`None` = à frente de tudo).
    pub fn frame(
        &mut self,
        sim: &mut SimWorld,
        playing: bool,
        ticks: u32,
        fixed_dt: f64,
        heard: &[String],
        rank_of: impl Fn(Entity) -> Option<u32>,
    ) -> ParticlesFrame {
        let mut report = ParticlesFrame::default();
        self.instances.clear();
        let actors = actors(sim);
        self.runs
            .retain(|bits, _| actors.iter().any(|(_, b, _)| b == bits));
        for (_, bits, cfg) in &actors {
            let entity = Entity::from_bits(*bits);
            let pose = pose_of(sim, entity);
            let born = |s: &Self| EmitterRun::born(cfg, &s.registry, pose, s.uv);
            // ⭐⭐ Por ORDEM de um sinal, nasce a EMITIR — ver `Emission::born_with`.
            let disparado = |s: &Self| EmitterRun::born_started(cfg, &s.registry, pose, s.uv);
            // Um componente editado RECOMEÇA o emissor: o grafo é outro, e o estado do integrador
            // é do grafo velho.
            let stale = self.runs.get(bits).is_none_or(|r| r.config() != cfg);
            if stale {
                let run = born(self);
                self.runs.insert(*bits, run);
            }
            let heard_it = |name: &String| !name.is_empty() && heard.iter().any(|h| h == name);
            if heard_it(&cfg.restart_on) || (heard_it(&cfg.start_on) && self.wants_restart(bits)) {
                let run = disparado(self);
                self.runs.insert(*bits, run);
            } else if heard_it(&cfg.start_on)
                && let Some(run) = self.runs.get_mut(bits)
            {
                run.start();
            }
            if heard_it(&cfg.stop_on)
                && let Some(run) = self.runs.get_mut(bits)
            {
                run.stop();
            }
            let Some(run) = self.runs.get_mut(bits) else {
                continue;
            };
            if playing {
                for _ in 0..ticks {
                    if run.step(&self.registry, pose, fixed_dt) && !cfg.finished_signal.is_empty() {
                        report.finished.push((*bits, cfg.finished_signal.clone()));
                    }
                }
            }
            run.append_instances(
                &mut self.instances,
                pose,
                rank_of(entity).unwrap_or(u32::MAX),
            );
        }
        report.drawn = self.instances.len();
        report
    }

    /// Numa rajada única, *ligar* é *recomeçar* — ver [`EmitterRun::wants_restart_to_start`].
    fn wants_restart(&self, bits: &u64) -> bool {
        self.runs
            .get(bits)
            .is_some_and(EmitterRun::wants_restart_to_start)
    }

    /// ⭐⭐⭐ **Rebobinar é RENASCER** — devolve quantas corridas foram deitadas fora.
    pub fn rewind(&mut self) -> usize {
        let n = self.runs.len();
        self.runs.clear();
        self.instances.clear();
        n
    }
}

#[cfg(test)]
#[path = "particles_bridge_tests.rs"]
mod tests;
