//! ⭐ **A folha ao vivo do Painter só se lê À VISTA** (B10.3 da missão Painter 4K,
//! `docs/Painter/missao_4k/medicoes/DIAGNOSTICO_100MS.md`): a leitura é a tela inteira composta e acesa pelo relevo do
//! papel (~98 ms a 4K) e corria 30 quadros depois de cada traço, Up, Ctrl+Z e pausa, sem ninguém a mostrar.

use ph2d_ecs::{Entity, SimWorld, World};
use ph2d_editor_core::HeroScreen;

use super::AnimDocs;

/// **Alguém mostra a folha ao vivo de `bits`?** O cartão Animation do Inspector (`cartoes`: a seleção e os cartões soltos
/// presos — só se o objeto TEM o cartão) ou a aba do editor de animações à frente (`aba`).
#[must_use]
pub fn shown(
    world: &World,
    mut cartoes: impl Iterator<Item = u64>,
    aba: Option<u64>,
    bits: u64,
) -> bool {
    aba == Some(bits)
        || (crate::inspector::has_anim_card(world, Entity::from_bits(bits))
            && cartoes.any(|b| b == bits))
}

/// [`crate::sheet::follow_live`] com a vista perguntada ao herói: o Inspector (seleção + cartões soltos) e a aba à frente
/// (as abas do editor têm o id = os bits da sprite).
pub fn follow_live(
    sim: &mut SimWorld,
    hero: &HeroScreen,
    vivo: Option<(u64, (u64, u64))>,
    read: impl FnOnce() -> Option<(Vec<u8>, u32, u32)>,
) {
    let vista = vivo.is_some_and(|(b, _)| {
        let pins = hero.store.loose_pins_of(ph2d_editor_core::ids::INSP_PANEL);
        let cartoes = hero.gizmo.selection.into_iter().chain(pins);
        let editor = hero.documents.surfaces().find::<AnimDocs>();
        let aba = hero
            .documents
            .surface_active()
            .map(|t| t.id)
            .filter(|id| editor.is_some_and(|d| d.docs.iter().any(|x| x.entity == *id)));
        shown(sim.world(), cartoes, aba, b)
    });
    let agora = hero.store.clock_ns();
    crate::sheet::follow_live(sim.world_mut(), vivo, vista, agora, read);
}

#[cfg(test)]
#[path = "folha_viva_tests.rs"]
mod tests;
