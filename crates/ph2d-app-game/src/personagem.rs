//! ⭐ **A animação segue as REGRAS do objecto** (`docs/GameEngineFuncional/08_regras_de_animacao_plano.md`) —
//! «quando ___ → toca ___», a de cima ganha. Sem regras gravadas, um personagem usa as 4 do
//! `CharacterAnimations` (parado · a correr · a subir · a cair), como sempre.
//!
//! Corre depois do passo da física. O animador é o do próprio objecto ou, se ele não tem, o do 1.º
//! filho que tem (a imagem pode ser uma peça dentro do corpo, como na Godot).

use std::collections::{BTreeMap, BTreeSet};

use ph2d_ecs::{Entity, Has, Resource, SpriteAnimator, World};
use ph2d_input::{ActionState, Input, InputMap};
use ph2d_physics_ecs::{
    AnimRule, AnimRules, AnimWhen, CharacterAnimations, CharacterPose, LifeEvent, PhysicsBridge,
};
use ph2d_render::Sprite;
use ph2d_runtime::{SignalOutbox, SignalReader};

/// Quem tem o animador que este objecto conduz.
fn alvo(world: &World, e: Entity) -> Option<Entity> {
    ph2d_physics_ecs::anim_target(world, e)
}

/// A troca de estado: a animação nova começa do início e TOCA — uma tag de uma vez só (saltar) que acabou
/// deixou o animador parado, e a seguinte ficaria presa no 1.º desenho.
fn troca(a: &mut SpriteAnimator, tag: String) {
    a.current = tag;
    a.elapsed_ticks = 0;
    a.pingpong_reverse = false;
    a.repeat_count = 0;
    a.playing = true;
}

/// As ações do mapa premidas agora, pelo NOME (o que uma regra `AnimWhen::Action` guarda).
#[must_use]
pub fn acoes_premidas(map: Option<&InputMap>, actions: &ActionState) -> BTreeSet<String> {
    let Some(map) = map else {
        return BTreeSet::new();
    };
    let input = Input::new(map, actions);
    map.actions()
        .iter()
        .filter(|a| input.pressed(&a.name))
        .map(|a| a.name.clone())
        .collect()
}

/// O que vale neste quadro para UM objecto.
pub(crate) struct Factos<'a> {
    pub pose: Option<CharacterPose>,
    /// Agachado (`PlayerView::crouching`) — vale junto com a pose base.
    pub agachado: bool,
    pub premidas: &'a BTreeSet<String>,
    pub vida: &'a [LifeEvent],
    pub ouvidos: &'a BTreeSet<String>,
}

fn vale(when: &AnimWhen, f: &Factos) -> bool {
    match when {
        AnimWhen::Pose(CharacterPose::Crouch) => f.agachado,
        AnimWhen::Pose(CharacterPose::CrouchWalk) => {
            f.agachado && f.pose == Some(CharacterPose::Run)
        }
        AnimWhen::Pose(p) => f.pose == Some(*p),
        AnimWhen::Action(n) => f.premidas.contains(n),
        AnimWhen::Life(l) => f.vida.contains(l),
        AnimWhen::Signal(n) => f.ouvidos.contains(n),
    }
}

/// Uma regra de facto instantâneo que está a SEGURAR a sua animação até ela fechar uma volta.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Segura {
    pub regra: usize,
    pub tag: String,
}

/// Os seguros vivos e o cursor dos sinais. Estado de corrida (não se grava).
#[derive(Resource, Default)]
pub struct RegrasEmCurso {
    seguras: BTreeMap<Entity, Segura>,
    ouvido: Option<SignalReader>,
}

/// O que o animador deve fazer neste quadro.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Passo {
    /// Nada muda.
    Fica,
    /// Toca esta tag (se já é a que toca, continua).
    Toca(String),
    /// Recomeça esta tag do início (um facto instantâneo dispara sempre de novo).
    Recomeca(String),
}

/// ⭐ **A lei**: o seguro solta-se quando a animação dele acabou uma volta ou foi trocada por fora; a
/// 1.ª regra que vale ganha — com um seguro vivo, só uma ACIMA dele o interrompe; nenhuma = fica.
pub(crate) fn decide(
    rules: &[AnimRule],
    f: &Factos,
    seguro: &mut Option<Segura>,
    animador: &SpriteAnimator,
) -> Passo {
    if seguro.as_ref().is_some_and(|s| {
        animador.current != s.tag || animador.repeat_count >= 1 || !animador.playing
    }) {
        *seguro = None;
    }
    let limite = seguro
        .as_ref()
        .map_or(rules.len(), |s| s.regra.min(rules.len()));
    let Some((i, r)) = rules[..limite]
        .iter()
        .enumerate()
        .find(|(_, r)| vale(&r.when, f))
    else {
        return Passo::Fica;
    };
    if r.when.is_instant() {
        *seguro = Some(Segura {
            regra: i,
            tag: r.play.clone(),
        });
        Passo::Recomeca(r.play.clone())
    } else {
        *seguro = None;
        Passo::Toca(r.play.clone())
    }
}

/// ⭐ Em cada objecto com regras (gravadas, ou as 4 do personagem): escolhe a animação e escreve o
/// espelho pelo lado para que o personagem olha.
pub fn anima_personagens(
    world: &mut World,
    physics: &PhysicsBridge,
    premidas: &BTreeSet<String>,
    outbox: &SignalOutbox,
) {
    let mut em_curso = world.remove_resource::<RegrasEmCurso>().unwrap_or_default();
    let reader = em_curso
        .ouvido
        .get_or_insert_with(|| SignalReader::at(outbox));
    let ouvidos: BTreeSet<String> = outbox.read(reader).map(|s| s.name.to_string()).collect();
    let mut vida: BTreeMap<Entity, Vec<LifeEvent>> = BTreeMap::new();
    for h in physics.health_events() {
        if let Some(l) = LifeEvent::of(h.kind) {
            vida.entry(h.target).or_default().push(l);
        }
    }
    let mut q = world.query::<(Entity, Has<AnimRules>, Has<CharacterAnimations>)>();
    let ids: Vec<Entity> = q
        .iter(world)
        .filter_map(|(e, r, c)| (r || c).then_some(e))
        .collect();
    let objectos: Vec<(Entity, AnimRules, Option<CharacterAnimations>)> = ids
        .into_iter()
        .filter_map(|e| {
            let rules = AnimRules::of_entity(world, e)?;
            Some((e, rules, world.get::<CharacterAnimations>(e).cloned()))
        })
        .collect();
    let vivos: BTreeSet<Entity> = objectos.iter().map(|(e, ..)| *e).collect();
    em_curso.seguras.retain(|e, _| vivos.contains(e));
    for (e, rules, character) in objectos {
        let view = physics.player_view(e);
        let Some(alvo) = alvo(world, e) else {
            continue;
        };
        let f = Factos {
            pose: view
                .zip(character.as_ref())
                .map(|(v, c)| CharacterPose::of(v, c.run_from)),
            agachado: view.is_some_and(|v| v.crouching),
            premidas,
            vida: vida.get(&e).map_or(&[], Vec::as_slice),
            ouvidos: &ouvidos,
        };
        let mut seguro = em_curso.seguras.remove(&e);
        if let Some(mut a) = world.get_mut::<SpriteAnimator>(alvo) {
            match decide(&rules.0, &f, &mut seguro, &a) {
                Passo::Fica => {}
                Passo::Toca(tag) if a.current == tag => {}
                Passo::Toca(tag) | Passo::Recomeca(tag) => troca(&mut a, tag),
            }
        }
        if let Some(s) = seguro {
            em_curso.seguras.insert(e, s);
        }
        let flip = view
            .zip(character.as_ref())
            .and_then(|(v, c)| c.flip.then_some(v.facing < 0.0));
        if let (Some(flip), Some(mut s)) = (flip, world.get_mut::<Sprite>(alvo))
            && s.flip_x != flip
        {
            s.flip_x = flip;
        }
    }
    world.insert_resource(em_curso);
}

#[cfg(test)]
#[path = "personagem_tests.rs"]
mod tests;
