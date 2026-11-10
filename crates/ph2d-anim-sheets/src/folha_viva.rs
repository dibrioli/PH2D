//! ⭐ **A folha ao vivo do Painter só se lê À VISTA** (B10.3 da missão Painter 4K,
//! `docs/Painter/missao_4k/medicoes/DIAGNOSTICO_100MS.md`): a leitura é a tela inteira composta e acesa pelo relevo do
//! papel (~98 ms a 4K) e corria 30 quadros depois de cada traço, Up, Ctrl+Z e pausa, sem ninguém a mostrar.

use ph2d_ecs::{Entity, SimWorld, SpriteAnimations, SpriteAnimator, World};
use ph2d_editor_core::HeroScreen;

/// **O cartão Animation (§11) existe para `entity`** — a porta do `build_anim_info` e da folha ao vivo (B10.3). Só uma
/// SPRITE tem animação de frames (o pool é a grelha dela); o Pixel Sprite também (os clips e a grelha DERIVAM das tags do
/// documento, P14 §4 A). Aparece com QUALQUER dos dois componentes (ADR-0166 F3): a biblioteca ou o tocador.
#[must_use]
pub fn has_anim_card(world: &World, entity: Entity) -> bool {
    (world.get::<ph2d_ecs::PixelSprite>(entity).is_some()
        || world.get::<ph2d_render::Sprite>(entity).is_some())
        && (world.get::<SpriteAnimations>(entity).is_some()
            || world.get::<SpriteAnimator>(entity).is_some())
}

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
        || (has_anim_card(world, Entity::from_bits(bits)) && cartoes.any(|b| b == bits))
}

/// [`crate::sheet_cache::follow_live`] com a vista perguntada ao herói: o Inspector (seleção + cartões soltos) e `aba`, a
/// sprite da aba do editor à frente (as abas do editor têm o id = os bits da sprite).
pub fn follow_live(
    sim: &mut SimWorld,
    hero: &HeroScreen,
    aba: Option<u64>,
    vivo: Option<(u64, (u64, u64))>,
    read: impl FnOnce() -> Option<(Vec<u8>, u32, u32)>,
) {
    let vista = vivo.is_some_and(|(b, _)| {
        let pins = hero.store.loose_pins_of(ph2d_editor_core::ids::INSP_PANEL);
        shown(
            sim.world(),
            hero.gizmo.selection.into_iter().chain(pins),
            aba,
            b,
        )
    });
    let agora = hero.store.clock_ns();
    crate::sheet_cache::follow_live(sim.world_mut(), vivo, vista, agora, read);
}

#[cfg(test)]
#[path = "folha_viva_tests.rs"]
mod tests;
