//! **O cartão «Animation» e o editor de animações: o que leem e o que escrevem**
//! ([plano 07](../../../docs/GameEngineFuncional/07_animacao_num_cartao_plano.md) §Desenho técnico).
//!
//! Estende a §11 de [`crate::inspector`]: a lista de quadros, o modo do ciclo, «quem escolhe» e as
//! regras de animação ([plano 08](../../../docs/GameEngineFuncional/08_regras_de_animacao_plano.md)). Tudo passa pela fila de comandos (`queue_set`/`queue_remove`), então o undo
//! regista por diff como o resto do Inspector.

use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{
    AnimDirection, Entity, SpriteAnimCursor, SpriteAnimFrames, SpriteAnimations, SpriteAnimator,
    SpriteGrid, World,
};
use ph2d_editor_core::{
    AnimCardInfo, AnimCharacterInfo, AnimChooser, AnimCycleMode, AnimFieldEdit, AnimRuleInfo,
    AnimWhenInfo,
};
use ph2d_inspector_ordering::{queue_remove, queue_set};
use ph2d_physics_ecs::{
    AnimRule, AnimRules, AnimWhen, CharacterAnimations, CharacterPose, LifeEvent, PlatformPlayer,
};

pub(crate) const FRAMES: &str = "ph2d::ecs::SpriteAnimFrames";
pub(crate) const CURSOR: &str = "ph2d::ecs::SpriteAnimCursor";
const CHARACTER: &str = "ph2d::physics::CharacterAnimations";
const RULES: &str = "ph2d::physics::AnimRules";
const ANIMATIONS: &str = "ph2d::ecs::SpriteAnimations";
const ANIMATOR: &str = "ph2d::ecs::SpriteAnimator";
const GRID: &str = "ph2d::ecs::SpriteGrid";

pub use ph2d_anim_editor::GRID_SIDE_MAX;

/// A condição da física → a do cartão.
#[must_use]
pub fn when_info(w: &AnimWhen) -> AnimWhenInfo {
    let idx = |i: Option<usize>| i.and_then(|i| u8::try_from(i).ok()).unwrap_or(0);
    match w {
        AnimWhen::Pose(p) => {
            AnimWhenInfo::Pose(idx(CharacterPose::ALL.iter().position(|x| x == p)))
        }
        AnimWhen::Action(n) => AnimWhenInfo::Action(n.clone()),
        AnimWhen::Life(l) => AnimWhenInfo::Life(idx(LifeEvent::ALL.iter().position(|x| x == l))),
        AnimWhen::Signal(n) => AnimWhenInfo::Signal(n.clone()),
    }
}

/// A condição do cartão → a da física (`None` = índice fora das tabelas).
#[must_use]
pub fn when_of(w: &AnimWhenInfo) -> Option<AnimWhen> {
    Some(match w {
        AnimWhenInfo::Pose(i) => AnimWhen::Pose(*CharacterPose::ALL.get(usize::from(*i))?),
        AnimWhenInfo::Action(n) => AnimWhen::Action(n.clone()),
        AnimWhenInfo::Life(i) => AnimWhen::Life(*LifeEvent::ALL.get(usize::from(*i))?),
        AnimWhenInfo::Signal(n) => AnimWhen::Signal(n.clone()),
    })
}

/// As regras que valem neste objecto (gravadas, ou as derivadas do personagem).
fn rules_of(world: &World, entity: Entity) -> Option<AnimRules> {
    AnimRules::of_entity(world, entity)
}

/// O que o cartão lê. `sheet` vem da [`crate::sheet::SheetCache`] (resolvida por quadro antes disto).
#[must_use]
pub fn build_card_info(world: &World, entity: Entity) -> AnimCardInfo {
    let character = world.get::<CharacterAnimations>(entity);
    let player = world.get::<SpriteAnimator>(entity);
    let rules = rules_of(world, entity);
    let chooser = if rules.is_some() {
        AnimChooser::Rules
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
            player
                .and_then(|p| {
                    world
                        .get::<SpriteAnimations>(entity)?
                        .get(&p.current)
                        .cloned()
                })
                .and_then(|t| {
                    let sheets = ph2d_ecs::sheet_cells_of(world, entity);
                    let lo = t.resolve(ph2d_ecs::sheet_cells(&sheets, t.sheet))?.0;
                    grid.shown().cell_on(t.sheet).map(|c| c.saturating_sub(lo))
                })
                .unwrap_or(grid.frame)
        });
    let starts_with = player
        .zip(world.get::<SpriteAnimations>(entity))
        .map_or_else(String::new, |(p, lib)| p.start_or_current(lib).to_owned());
    AnimCardInfo {
        chooser,
        starts_with,
        has_character_body: world.get::<PlatformPlayer>(entity).is_some(),
        can_crouch: world
            .get::<PlatformPlayer>(entity)
            .is_some_and(|p| p.config().crouch.armed(&p.config().ride)),
        character: character.map(|a| AnimCharacterInfo {
            run_from: a.run_from,
            flip: a.flip,
        }),
        rule_signals: ph2d_anim_signals::authored_signals(world, rules.as_ref()),
        rules: rules
            .iter()
            .flat_map(|r| r.0.iter())
            .map(|r| AnimRuleInfo {
                when: when_info(&r.when),
                play: r.play.clone(),
            })
            .collect(),
        live_rule: None,
        // As ações moram no mapa do projecto (fora do mundo): quem monta o retrato acrescenta-as.
        rule_actions: Vec::new(),
        hframes: grid.hframes,
        vframes: grid.vframes,
        step,
        // A folha que `sheet::refresh` descodificou neste quadro (a sprite escolhida e as abas do editor).
        sheet: world
            .get_resource::<crate::sheet::SheetCache>()
            .and_then(|c| c.of_entity(entity.to_bits()))
            .map(|e| e.sheet.clone()),
        extra_sheets: extra_sheets(world, entity),
        sync: world
            .get_resource::<crate::sheet::SheetCache>()
            .and_then(|c| c.sync(entity.to_bits())),
    }
}

/// As folhas extra como o cartão as pinta: a imagem descodificada (se `sheet::refresh` a resolveu) e o corte.
fn extra_sheets(
    world: &World,
    entity: Entity,
) -> Vec<(Option<ph2d_editor_core::SheetImage>, u32, u32)> {
    let cache = world.get_resource::<crate::sheet::SheetCache>();
    world
        .get::<ph2d_ecs::SpriteSheets>(entity)
        .map(|s| {
            (1u8..)
                .zip(&s.0)
                .map(|(k, x)| {
                    let img = cache.and_then(|c| c.of_sheet(entity.to_bits(), k));
                    (img.map(|e| e.sheet.clone()), x.hframes, x.vframes)
                })
                .collect()
        })
        .unwrap_or_default()
}

/// A lista de quadros da animação `name` (`None` = o intervalo; vazia = sem quadros).
#[must_use]
pub fn row_frames(
    world: &World,
    entity: Entity,
    name: &str,
) -> Option<Vec<ph2d_editor_core::AnimFrameRef>> {
    world
        .get::<SpriteAnimFrames>(entity)
        .and_then(|f| f.get(name))
        .map(|l| l.iter().map(|f| crate::panel_frame::to_panel(*f)).collect())
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
        AnimFieldEdit::MarkerAdd(_)
        | AnimFieldEdit::MarkerStep(..)
        | AnimFieldEdit::MarkerSignal(..)
        | AnimFieldEdit::MarkerRemove(..) => {
            ph2d_anim_signals::apply_marker_edit(world, entity_bits, edit, queue, registry)?;
        }
        AnimFieldEdit::Frames(row, cells) => {
            let lib = world.get::<SpriteAnimations>(entity)?;
            let name = lib.iter().nth(usize::from(*row))?.name.clone();
            let cells = cells.iter().map(|f| crate::panel_frame::from_panel(*f));
            set_frames(world, entity_bits, &name, cells.collect(), queue, registry);
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
            let has_rules = world.get::<AnimRules>(entity).is_some();
            match choice {
                // Num personagem, «Rules» pré-preenche as 4 (derivadas do `CharacterAnimations`);
                // noutro objecto começa vazia («+ Rule»).
                AnimChooser::Rules if has_character || has_rules => {}
                AnimChooser::Rules if world.get::<PlatformPlayer>(entity).is_some() => queue_set(
                    queue,
                    registry,
                    entity_bits,
                    CHARACTER,
                    &CharacterAnimations::default(),
                ),
                AnimChooser::Rules => {
                    queue_set(queue, registry, entity_bits, RULES, &AnimRules::default());
                }
                AnimChooser::Me | AnimChooser::Script => {
                    if has_character {
                        queue_remove(queue, registry, entity_bits, CHARACTER);
                    }
                    if has_rules {
                        queue_remove(queue, registry, entity_bits, RULES);
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
        AnimFieldEdit::StartsWith(name) => {
            world.get::<SpriteAnimations>(entity)?.get(name)?;
            let mut p = world
                .get::<SpriteAnimator>(entity)
                .cloned()
                .unwrap_or_default();
            p.start = Some(name.clone());
            queue_set(queue, registry, entity_bits, ANIMATOR, &p);
        }
        AnimFieldEdit::RuleWhen(..)
        | AnimFieldEdit::RulePlay(..)
        | AnimFieldEdit::RuleAdd(..)
        | AnimFieldEdit::RuleRemove(_)
        | AnimFieldEdit::RuleMove(..) => {
            let mut r = rules_of(world, entity)?;
            edit_rules(&mut r.0, edit)?;
            queue_set(queue, registry, entity_bits, RULES, &r);
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
            let seq = ph2d_ecs::tag_cells(&tag, list, &ph2d_ecs::sheet_cells_of(world, entity))?;
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
            if grid.shown() != cell {
                grid.show(cell);
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
            // A grade mudou = o aviso sobre a de antes morre, e voltar a ela não o ressuscita.
            if let Some(mut lib) = world.get::<SpriteAnimations>(entity).cloned()
                && lib.1.is_some_and(|n| n != [g.hframes, g.vframes])
            {
                lib.1 = None;
                queue_set(queue, registry, entity_bits, ANIMATIONS, &lib);
            }
        }
        _ => return None,
    }
    Some(())
}

/// Uma edição de regra sobre a lista. `None` = índice fora (nada se grava).
fn edit_rules(rules: &mut Vec<AnimRule>, edit: &AnimFieldEdit) -> Option<()> {
    match edit {
        AnimFieldEdit::RuleWhen(i, w) => rules.get_mut(usize::from(*i))?.when = when_of(w)?,
        AnimFieldEdit::RulePlay(i, name) => {
            name.clone_into(&mut rules.get_mut(usize::from(*i))?.play)
        }
        AnimFieldEdit::RuleAdd(w, name) => rules.push(AnimRule {
            when: when_of(w)?,
            play: name.clone(),
        }),
        AnimFieldEdit::RuleRemove(i) => {
            (usize::from(*i) < rules.len()).then_some(())?;
            rules.remove(usize::from(*i));
        }
        AnimFieldEdit::RuleMove(from, to) => {
            let (from, to) = (usize::from(*from), usize::from(*to));
            (from < rules.len() && from != to).then_some(())?;
            let r = rules.remove(from);
            rules.insert(to.min(rules.len()), r);
        }
        _ => return None,
    }
    Some(())
}

/// O renomear leva também as regras (e os estados do personagem) que apontavam para o nome velho (a
/// biblioteca, a lista e o tocador vão por `ph2d_ecs::rename_tag`; o personagem vive noutra crate).
pub(crate) fn follow_character_rename(
    world: &World,
    entity_bits: u64,
    old: &str,
    new: &str,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) {
    let entity = Entity::from_bits(entity_bits);
    if let Some(mut r) = world.get::<AnimRules>(entity).cloned()
        && r.rename_play(old, new)
    {
        queue_set(queue, registry, entity_bits, RULES, &r);
    }
    let Some(mut a) = world.get::<CharacterAnimations>(entity).cloned() else {
        return;
    };
    let mut moved = false;
    for s in [&mut a.idle, &mut a.run, &mut a.jump, &mut a.fall] {
        if s == old {
            new.clone_into(s);
            moved = true;
        }
    }
    if moved {
        queue_set(queue, registry, entity_bits, CHARACTER, &a);
    }
}

/// Escreve a lista da animação `name` — vazia = sem quadros (tirar o último quadro não a devolve ao intervalo).
pub(crate) fn set_frames(
    world: &World,
    entity_bits: u64,
    name: &str,
    cells: Vec<ph2d_ecs::FrameRef>,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) {
    let entity = Entity::from_bits(entity_bits);
    let mut frames = world
        .get::<SpriteAnimFrames>(entity)
        .cloned()
        .unwrap_or_default();
    frames.set(name, cells);
    queue_set(queue, registry, entity_bits, FRAMES, &frames);
    if world.get::<SpriteAnimCursor>(entity).is_none() {
        let c = SpriteAnimCursor::default();
        queue_set(queue, registry, entity_bits, CURSOR, &c);
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
    ph2d_anim_signals::forget_tag(world, entity_bits, name, queue, registry);
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
