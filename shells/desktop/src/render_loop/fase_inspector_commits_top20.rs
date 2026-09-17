//! **Os commits das secções do TOP-20 que cabem numa porta só** (TOP-DOWN · PROJECTILE ·
//! STATE MACHINE · SCRIPT · PARTICLES · HUD · SEQUENCE · COUNTER WATCH) — fase-filha do [`super`],
//! num ficheiro irmão.
//!
//! ⚠️ **O corte foi imposto pelo tecto de FUNÇÃO da fase-mãe, DUAS vezes** (`204` ao ganhar o HUD,
//! `201` ao ganhar a vigia) **e é o certo por responsabilidade**: as seis aplicam-se pelo MESMO
//! gesto — o corpo mora na família e aqui é uma chamada —, e é isso que as junta.
//! ⛔ *A cura de um tecto é o CORTE, nunca uma entrada nova no `FN_OVERAGE_OK`.*

use ph2d_ecs::SimWorld;

/// Aplica as oito. `true` = alguma coisa mudou.
///
/// ⚠️ **QUATRO secções mudaram-se para cá** (#13 · #14 · #15 · #16) quando o tecto de 200 LOC da
/// fase-mãe voltou a estourar, **e é o certo por responsabilidade**: as oito aplicam-se pelo MESMO
/// gesto — o corpo mora na família e aqui é uma chamada. ⛔ *A cura de um tecto é o CORTE, nunca
/// uma entrada nova no `FN_OVERAGE_OK`.*
///
/// ⚠️ **Os oito tipos de edição são DISTINTOS**, e é isso que torna esta lista segura: uma troca de
/// posição entre dois argumentos **não compila**. *Uma lista longa de argumentos do MESMO tipo é
/// onde dois instantâneos trocam de sítio em silêncio.*
///
/// ⚠️ **`|` e não `||`:** as seguintes TÊM de correr, e o curto-circuito engoliria uma edição do
/// HUD sempre que uma de partículas já tivesse mudado alguma coisa.
#[allow(clippy::too_many_arguments)]
pub(super) fn aplicar(
    sim: &mut SimWorld,
    particles: &[(u64, ph2d_editor_core::particles_edits::ParticlesFieldEdit)],
    hud: &[(u64, ph2d_editor_core::hud_edits::HudFieldEdit)],
    sequence: &[(u64, ph2d_editor_core::sequence_edits::SequenceFieldEdit)],
    watch: &[(
        u64,
        ph2d_editor_core::counter_watch_edits::CounterWatchFieldEdit,
    )],
    topdown: &[(u64, ph2d_editor_core::topdown_edits::TopDownFieldEdit)],
    projectile: &[(u64, ph2d_editor_core::projectile_edits::ProjectileFieldEdit)],
    statemachine: &[(
        u64,
        ph2d_editor_core::statemachine_edits::StateMachineFieldEdit,
    )],
    script: &[(u64, ph2d_editor_core::script_edits::ScriptFieldEdit)],
) -> bool {
    super::topdown_commits::aplicar(sim, topdown)
        | super::projectile_commits::aplicar(sim, projectile)
        | super::statemachine_commits::aplicar(sim, statemachine)
        | super::script_commits::aplicar(sim, script)
        | ph2d_app_components::particles_inspector::apply_all(sim, particles)
        | ph2d_app_components::hud_inspector::apply_all(sim, hud)
        | ph2d_app_components::sequence_inspector::apply_all(sim, sequence)
        | ph2d_app_components::counter_watch_inspector::apply_all(sim, watch)
}
