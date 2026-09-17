//! **Os commits das secções do TOP-20 que cabem numa porta só** (PARTICLES · HUD) — fase-filha do
//! [`super`], num ficheiro irmão.
//!
//! ⚠️ **O corte foi imposto pelo tecto de FUNÇÃO** (a `fase_inspector_commits` chegou a `204`
//! contra `200` ao ganhar o HUD) **e é o certo por responsabilidade**: as duas secções aplicam-se
//! pelo MESMO gesto — o corpo mora na família e aqui é uma chamada —, e é isso que as junta.
//! ⛔ *A cura de um tecto é o CORTE, nunca uma entrada nova no `FN_OVERAGE_OK`.*

use ph2d_ecs::SimWorld;

/// Aplica as duas. `true` = alguma coisa mudou.
///
/// ⚠️ **`|` e não `||`:** a segunda TEM de correr, e o curto-circuito engoliria uma edição do HUD
/// sempre que uma de partículas já tivesse mudado alguma coisa.
pub(super) fn aplicar(
    sim: &mut SimWorld,
    particles: &[(u64, ph2d_editor_core::particles_edits::ParticlesFieldEdit)],
    hud: &[(u64, ph2d_editor_core::hud_edits::HudFieldEdit)],
) -> bool {
    ph2d_app_components::particles_inspector::apply_all(sim, particles)
        | ph2d_app_components::hud_inspector::apply_all(sim, hud)
}
