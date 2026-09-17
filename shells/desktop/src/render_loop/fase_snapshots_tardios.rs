//! **Fase do quadro: OS INSTANTÂNEOS QUE O `publish` NÃO PODE CALCULAR** (SCRIPT · PARTICLES) —
//! fase-filha do [`super`], num ficheiro irmão.
//!
//! ⚠️ **O corte foi imposto pelo tecto de FUNÇÃO** (a `fase_snapshots_publish` chegou a `208` contra
//! `200` ao ganhar o emissor) **e é o certo por responsabilidade**: as duas secções leem coisas que
//! **não estão no mundo** — a VM dos scripts, e o relógio mais as partículas vivas da corrida —, e é
//! exactamente por isso que nenhuma delas cabe na lista de argumentos do `publish`.
//!
//! ⚠️ **Depois do passo dos motores** (`fase_signal_outbox`): publicar antes mostraria a contagem de
//! partículas do quadro ANTERIOR.
//!
//! ⛔ **O corpo mora na família** (`ph2d_app_components::{script_inspector, particles_inspector}`);
//! daqui sai só a escolha de quem é o sujeito — o objecto activo.

use ph2d_app_components::particles_bridge::ParticlesState;
use ph2d_ecs::SimWorld;
use ph2d_script::ScriptHost;

/// Publica os dois instantâneos do objecto ACTIVO. `None` = ninguém escolhido ⇒ nenhuma das duas
/// secções existe, que é a lei do ADR-0166.
pub(super) fn publica(
    sim: &SimWorld,
    script: Option<&ScriptHost>,
    particles: &ParticlesState,
    escolhido: Option<u64>,
    quantos: usize,
    a_correr: bool,
) {
    ph2d_panel_inspector::set_current_inspector_script(escolhido.and_then(|b| {
        ph2d_app_components::script_inspector::build_info(sim, script, b, quantos, a_correr)
    }));
    ph2d_panel_inspector::set_current_inspector_particles(escolhido.and_then(|b| {
        ph2d_app_components::particles_inspector::build_info(
            sim,
            b,
            quantos,
            a_correr,
            particles.alive_of(b),
        )
    }));
}
