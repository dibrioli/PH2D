//! **O cartão «Animation» e o editor de animações: o que leem e o que escrevem**
//! ([plano 07](../../../docs/GameEngineFuncional/07_animacao_num_cartao_plano.md) §Desenho técnico).
//!
//! Estende a §11 de [`crate::inspector`]: a lista de quadros, o modo do ciclo, «quem escolhe» e os
//! estados do personagem. Tudo passa pela fila de comandos (`queue_set`/`queue_remove`), então o undo
//! regista por diff como o resto do Inspector.

use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{
    AnimDirection, Entity, SpriteAnimCursor, SpriteAnimFrames, SpriteAnimations, SpriteAnimator,
    SpriteGrid, World,
};
use ph2d_editor_core::{
    AnimCardInfo, AnimCharacterInfo, AnimChooser, AnimCycleMode, AnimFieldEdit, SheetImage,
};
use ph2d_inspector_ordering::{queue_remove, queue_set};
use ph2d_physics_ecs::{CharacterAnimations, PlatformPlayer};

pub(crate) const FRAMES: &str = "ph2d::ecs::SpriteAnimFrames";
pub(crate) const CURSOR: &str = "ph2d::ecs::SpriteAnimCursor";
const CHARACTER: &str = "ph2d::physics::CharacterAnimations";
const ANIMATIONS: &str = "ph2d::ecs::SpriteAnimations";
const ANIMATOR: &str = "ph2d::ecs::SpriteAnimator";
const GRID: &str = "ph2d::ecs::SpriteGrid";

/// Maior lado de uma grelha que o cartão aceita (o `SpriteGrid` guarda `u32`; isto é o que cabe a
/// clicar numa folha: 256 × 256 células).
pub const GRID_SIDE_MAX: u32 = 256;

/// Os nomes dos quatro estados, na ordem Parado · Correr · Saltar · Cair.
#[must_use]
pub fn state_names(a: &CharacterAnimations) -> [String; 4] {
    [
        a.idle.clone(),
        a.run.clone(),
        a.jump.clone(),
        a.fall.clone(),
    ]
}

fn state_mut(a: &mut CharacterAnimations, i: u8) -> Option<&mut String> {
    Some(match i {
        0 => &mut a.idle,
        1 => &mut a.run,
        2 => &mut a.jump,
        3 => &mut a.fall,
        _ => return None,
    })
}

/// O que o cartão lê. `sheet` vem de quem tem os pixels (a shell); aqui sai `None`.
#[must_use]
pub fn build_card_info(world: &World, entity: Entity) -> AnimCardInfo {
    let character = world.get::<CharacterAnimations>(entity);
    let player = world.get::<SpriteAnimator>(entity);
    let chooser = if character.is_some() {
        AnimChooser::Character
    } else if player.is_some_and(|p| !p.autoplay) {
        AnimChooser::Script
    } else {
        AnimChooser::Me
    };
    let grid = world
        .get::<SpriteGrid>(entity)
        .copied()
        .unwrap_or(SpriteGrid::SINGLE);
    let step = world
        .get::<SpriteAnimCursor>(entity)
        .map(|c| c.step)
        .filter(|_| {
            player.is_some_and(|p| {
                world
                    .get::<SpriteAnimFrames>(entity)
                    .and_then(|f| f.get(&p.current))
                    .is_some()
            })
        })
        .unwrap_or_else(|| {
            let lo = player
                .and_then(|p| {
                    world
                        .get::<SpriteAnimations>(entity)?
                        .get(&p.current)
                        .cloned()
                })
                .and_then(|t| t.resolve(grid.cells().max(1)))
                .map_or(0, |(lo, _)| lo);
            grid.frame.saturating_sub(lo)
        });
    AnimCardInfo {
        chooser,
        has_character_body: world.get::<PlatformPlayer>(entity).is_some(),
        character: character.map(|a| AnimCharacterInfo {
            names: state_names(a),
            run_from: a.run_from,
            flip: a.flip,
        }),
        hframes: grid.hframes,
        vframes: grid.vframes,
        step,
        sheet: None::<SheetImage>,
    }
}

/// A lista de quadros de cada linha da biblioteca (vazia = intervalo), na ordem da biblioteca.
#[must_use]
pub fn row_frames(world: &World, entity: Entity, name: &str) -> Vec<u32> {
    world
        .get::<SpriteAnimFrames>(entity)
        .and_then(|f| f.get(name))
        .map(<[u32]>::to_vec)
        .unwrap_or_default()
}

/// Aplica uma edição do cartão. `None` = não é uma edição do cartão (a §11 de sempre trata-a).
pub fn apply_card_edit(
    world: &World,
    entity_bits: u64,
    edit: &AnimFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) -> Option<()> {
    let entity = Entity::from_bits(entity_bits);
    match edit {
        AnimFieldEdit::Frames(row, cells) => {
            let lib = world.get::<SpriteAnimations>(entity)?;
            let name = lib.iter().nth(usize::from(*row))?.name.clone();
            let mut frames = world
                .get::<SpriteAnimFrames>(entity)
                .cloned()
                .unwrap_or_default();
            frames.set(&name, cells.clone());
            if frames.is_empty() {
                queue_remove(queue, registry, entity_bits, FRAMES);
                queue_remove(queue, registry, entity_bits, CURSOR);
            } else {
                queue_set(queue, registry, entity_bits, FRAMES, &frames);
                if world.get::<SpriteAnimCursor>(entity).is_none() {
                    queue_set(
                        queue,
                        registry,
                        entity_bits,
                        CURSOR,
                        &SpriteAnimCursor::default(),
                    );
                }
            }
        }
        AnimFieldEdit::CycleMode(row, mode) => {
            let mut lib = world.get::<SpriteAnimations>(entity)?.clone();
            let t = lib.0.get_mut(usize::from(*row))?;
            match AnimCycleMode::from_tag(*mode) {
                AnimCycleMode::Repeat => {
                    t.repeat = None;
                    t.direction = AnimDirection::Forward;
                }
                AnimCycleMode::Once => {
                    t.repeat = Some(1);
                    t.direction = AnimDirection::Forward;
                }
                AnimCycleMode::PingPong => {
                    t.repeat = None;
                    t.direction = AnimDirection::PingPong;
                }
            }
            queue_set(queue, registry, entity_bits, ANIMATIONS, &lib);
        }
        AnimFieldEdit::Chooser(tag) => {
            let choice = AnimChooser::from_tag(*tag);
            let has_character = world.get::<CharacterAnimations>(entity).is_some();
            match choice {
                AnimChooser::Character => {
                    if world.get::<PlatformPlayer>(entity).is_none() {
                        return Some(());
                    }
                    if !has_character {
                        queue_set(
                            queue,
                            registry,
                            entity_bits,
                            CHARACTER,
                            &CharacterAnimations::default(),
                        );
                    }
                }
                AnimChooser::Me | AnimChooser::Script => {
                    if has_character {
                        queue_remove(queue, registry, entity_bits, CHARACTER);
                    }
                    let mut p = world
                        .get::<SpriteAnimator>(entity)
                        .cloned()
                        .unwrap_or_default();
                    if p.speed_q16 == 0 {
                        p.speed_q16 = ph2d_ecs::SPEED_ONE_Q16;
                    }
                    p.autoplay = choice == AnimChooser::Me;
                    queue_set(queue, registry, entity_bits, ANIMATOR, &p);
                }
            }
        }
        AnimFieldEdit::CharacterState(i, name) => {
            let mut a = world.get::<CharacterAnimations>(entity)?.clone();
            *state_mut(&mut a, *i)? = name.clone();
            queue_set(queue, registry, entity_bits, CHARACTER, &a);
        }
        AnimFieldEdit::CharacterRunFrom(v) => {
            let mut a = world.get::<CharacterAnimations>(entity)?.clone();
            a.run_from = if v.is_finite() {
                v.max(0.0)
            } else {
                a.run_from
            };
            queue_set(queue, registry, entity_bits, CHARACTER, &a);
        }
        AnimFieldEdit::CharacterFlip(on) => {
            let mut a = world.get::<CharacterAnimations>(entity)?.clone();
            a.flip = *on;
            queue_set(queue, registry, entity_bits, CHARACTER, &a);
        }
        AnimFieldEdit::SetStep(n) => {
            let mut p = world.get::<SpriteAnimator>(entity)?.clone();
            let tag = world
                .get::<SpriteAnimations>(entity)?
                .get(&p.current)?
                .clone();
            let mut grid = world.get::<SpriteGrid>(entity).copied()?;
            let list = world
                .get::<SpriteAnimFrames>(entity)
                .and_then(|f| f.get(&tag.name));
            let seq = ph2d_ecs::tag_cells(&tag, list, grid.cells().max(1))?;
            let step = (*n).min(seq.len() as u32 - 1);
            p.playing = false;
            p.elapsed_ticks = 0;
            queue_set(queue, registry, entity_bits, ANIMATOR, &p);
            if list.is_some() {
                queue_set(
                    queue,
                    registry,
                    entity_bits,
                    CURSOR,
                    &SpriteAnimCursor { step },
                );
            }
            let cell = seq[step as usize];
            if grid.frame != cell {
                grid.frame = cell;
                queue_set(queue, registry, entity_bits, GRID, &grid);
            }
        }
        AnimFieldEdit::Grid(h, v) => {
            let mut g = world
                .get::<SpriteGrid>(entity)
                .copied()
                .unwrap_or(SpriteGrid::SINGLE);
            g.hframes = (*h).clamp(1, GRID_SIDE_MAX);
            g.vframes = (*v).clamp(1, GRID_SIDE_MAX);
            g.frame = g.frame.min(g.cells().saturating_sub(1));
            queue_set(queue, registry, entity_bits, GRID, &g);
        }
        _ => return None,
    }
    Some(())
}

/// O renomear leva também os estados do personagem que apontavam para o nome velho (a biblioteca, a
/// lista e o tocador vão por `ph2d_ecs::rename_tag`; o personagem vive noutra crate).
pub(crate) fn follow_character_rename(
    world: &World,
    entity_bits: u64,
    old: &str,
    new: &str,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) {
    let Some(mut a) = world
        .get::<CharacterAnimations>(Entity::from_bits(entity_bits))
        .cloned()
    else {
        return;
    };
    let mut moved = false;
    for i in 0..4 {
        if let Some(s) = state_mut(&mut a, i)
            && s == old
        {
            *s = new.to_owned();
            moved = true;
        }
    }
    if moved {
        queue_set(queue, registry, entity_bits, CHARACTER, &a);
    }
}

/// Uma animação saiu da biblioteca: a lista dela sai também.
pub(crate) fn follow_remove(
    world: &World,
    entity_bits: u64,
    name: &str,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) {
    let entity = Entity::from_bits(entity_bits);
    let Some(mut f) = world.get::<SpriteAnimFrames>(entity).cloned() else {
        return;
    };
    if f.get(name).is_none() {
        return;
    }
    f.remove(name);
    if f.is_empty() {
        queue_remove(queue, registry, entity_bits, FRAMES);
        queue_remove(queue, registry, entity_bits, CURSOR);
    } else {
        queue_set(queue, registry, entity_bits, FRAMES, &f);
    }
}

#[cfg(test)]
#[path = "card_tests.rs"]
mod tests;
