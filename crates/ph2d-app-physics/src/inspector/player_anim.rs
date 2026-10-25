//! **§14 — o bloco *Animations*** (GameEngineFuncional N1, `docs/GameEngineFuncional/05_N1_plano.md` §3):
//! que animação toca em cada estado do personagem. O snapshot e a escrita, juntos.
//!
//! As animações a escolher são as do objecto ou, se ele não tem animador, as do 1.º filho que tem — o
//! mesmo alvo que a ponte da família do jogo conduz (`ph2d_app_game::personagem`).

use ph2d_ecs::{Children, Entity, SpriteAnimations, SpriteAnimator, World};
use ph2d_editor_core::PlayerFieldEdit;
use ph2d_editor_core::screens::hero::InspectorCharacterAnimInfo;
use ph2d_physics_ecs::CharacterAnimations;

/// O teto do `run_from`, m/s: o `speed` de partida do player é 6; 10 dá folga sem fazer do slider um
/// campo de 0 a mil onde a faixa útil é o primeiro centésimo (conveniência de UI, não recurso — a
/// caixa aceita o que se digitar e o dreno só prende o piso).
pub const RUN_FROM_MAX: f32 = 10.0;

/// A lista de animações que este personagem conduz.
fn tags_of(world: &World, e: Entity) -> Option<&SpriteAnimations> {
    if world.get::<SpriteAnimator>(e).is_some() {
        return world.get::<SpriteAnimations>(e);
    }
    let alvo = world
        .get::<Children>(e)?
        .iter()
        .copied()
        .find(|c| world.get::<SpriteAnimator>(*c).is_some())?;
    world.get::<SpriteAnimations>(alvo)
}

fn names(a: &CharacterAnimations) -> [&str; 4] {
    [&a.idle, &a.run, &a.jump, &a.fall]
}

/// O snapshot do bloco — `None` sem o componente.
#[must_use]
pub fn anim_info(world: &World, e: Entity) -> Option<InspectorCharacterAnimInfo> {
    let a = world.get::<CharacterAnimations>(e)?;
    let tags = tags_of(world, e);
    let n = names(a);
    Some(InspectorCharacterAnimInfo {
        names: n.map(str::to_owned),
        known: n.map(|name| tags.is_some_and(|t| t.get(name).is_some())),
        has_tags: tags.is_some_and(|t| !t.0.is_empty()),
        run_from: a.run_from,
        flip: a.flip,
    })
}

/// O nome que vem depois de `current` na lista (o 1.º se `current` não está nela), em volta.
fn next_name(tags: &SpriteAnimations, current: &str) -> Option<String> {
    let list = &tags.0;
    let i = list.iter().position(|t| t.name == current);
    let j = i.map_or(0, |i| (i + 1) % list.len());
    list.get(j).map(|t| t.name.clone())
}

/// ⭐ Honra uma edição do bloco. `false` = a edição não é deste bloco.
///
/// ⚠️ Sem componente, só o `AnimateByState(true)` age; os outros não têm onde escrever e o painel não
/// os oferece — a recusa aqui é a mesma resposta, não um no-op escondido.
pub fn apply_anim_edit(world: &mut World, e: Entity, edit: PlayerFieldEdit) -> bool {
    match edit {
        PlayerFieldEdit::AnimateByState(on) => {
            let mut ent = world.entity_mut(e);
            if on {
                if !ent.contains::<CharacterAnimations>() {
                    ent.insert(CharacterAnimations::default());
                }
            } else {
                ent.remove::<CharacterAnimations>();
            }
        }
        PlayerFieldEdit::AnimNext(i) => {
            let current = world
                .get::<CharacterAnimations>(e)
                .and_then(|a| names(a).get(usize::from(i)).map(|s| (*s).to_owned()));
            let next = current.and_then(|c| tags_of(world, e).and_then(|t| next_name(t, &c)));
            if let (Some(next), Some(mut a)) = (next, world.get_mut::<CharacterAnimations>(e)) {
                match i {
                    0 => a.idle = next,
                    1 => a.run = next,
                    2 => a.jump = next,
                    _ => a.fall = next,
                }
            }
        }
        PlayerFieldEdit::AnimRunFrom(v) => {
            if let Some(mut a) = world.get_mut::<CharacterAnimations>(e) {
                a.run_from = v.max(0.0);
            }
        }
        PlayerFieldEdit::AnimFlip(on) => {
            if let Some(mut a) = world.get_mut::<CharacterAnimations>(e) {
                a.flip = on;
            }
        }
        _ => return false,
    }
    true
}

#[cfg(test)]
#[path = "player_anim_tests.rs"]
mod tests;
