//! ⭐⭐⭐ **O EMISSOR DE PARTÍCULAS de um objecto** — a lei da fachada (TOP-20 #18,
//! `docs/Components/14_plano_particle_emitter.md`).
//!
//! O componente [`ph2d_ecs::ParticleEmitter`] é CONFIG; esta crate faz dele uma simulação:
//!
//! - [`plan`] — a emissão pedida (ligar, desligar, rajada, explosividade) nos números do
//!   `motion.emitter`, incluindo a **agenda** que a W0 lhe deu;
//! - [`compile`] — o grafo de **nós REAIS** do Motion que o objecto corre;
//! - [`run`] — o cozinheiro de cada emissor, o relógio local e o fim (`finished`).
//!
//! ⛔ **Nenhum motor de partículas próprio** — o mesmo grafo é o que *«abrir como grafo»* vai
//! mostrar, e a mesma lei que o Motion conferiu nó a nó.
//!
//! O relógio é o do oráculo (Godot 4.7.2, MIT, corrido sem interface —
//! `docs/Components/ferramentas/godot_particles_probe.gd`).

#![forbid(unsafe_code)]

pub mod compile;
pub mod plan;
pub mod registry;
pub mod run;

pub use compile::{Compiled, compile};
pub use ph2d_node_motion_emitter::schedule::Schedule;
pub use plan::{Emission, EmitParams, emission_end, emit_params};
pub use registry::node_registry;
pub use run::{EmitterRun, Pose, SUB_STEP};
