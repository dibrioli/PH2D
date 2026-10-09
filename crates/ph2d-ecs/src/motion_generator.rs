//! ⭐⭐ **O objecto MOTION** — o marcador que faz de uma entidade um gerador do Motion Nodes (doc
//! ObjectsAndComponents 01 §13, F5; escolha do dono 9.3: *«Motion vira objecto na árvore, com o que
//! cria como filhos; pode haver vários»*).
//!
//! Sem dados: os nós vivem no `MotionDoc` (um documento, cada nó com o `StableId` deste objecto como
//! dono), o lugar é o `Transform` dele (a origem do que os sinks geram) e os moldes são os filhos.
//! Registado ⇒ grava-se com a cena; o tipo `ObjectKind::Motion` é derivado DELE.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// A entidade é um objecto Motion.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MotionGenerator;

impl SimComponent for MotionGenerator {}

/// ⭐⭐ **Esta entidade é MOLDE de um objecto Motion** — descendente dele (escolha do dono, 08/10: *«a
/// forma que o Motion copia vira filha dele e não se desenha sozinha — só as cópias»*).
///
/// DERIVADA, como o `MasterPiece`: quem a mantém é [`assign_motion_templates`], e o desenho pergunta-a
/// (`ph2d_entity_visibility::off_canvas::is_off_canvas`). ⛔ Não é registada — a árvore é a verdade;
/// guardá-la seria uma segunda resposta a envelhecer. O molde continua publicado ao cozido (que lê
/// `Sprite`/`Name` sem perguntar se desenha): é ele que as cópias carimbam.
#[derive(bevy_ecs::component::Component, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct MotionTemplate;

/// Re-carimba [`MotionTemplate`] na descendência (sem a raiz) de cada [`MotionGenerator`], e tira-a
/// de quem deixou de o ser (arrastado para fora). Devolve se mexeu em alguma entidade.
pub fn assign_motion_templates(world: &mut bevy_ecs::world::World) -> bool {
    use std::collections::BTreeSet;
    let roots: Vec<bevy_ecs::entity::Entity> = {
        let mut q = world.query_filtered::<bevy_ecs::entity::Entity, bevy_ecs::query::With<MotionGenerator>>();
        q.iter(world).collect()
    };
    let mut want: BTreeSet<bevy_ecs::entity::Entity> = BTreeSet::new();
    let mut stack: Vec<bevy_ecs::entity::Entity> = roots
        .iter()
        .filter_map(|r| world.get::<bevy_ecs::hierarchy::Children>(*r))
        .flat_map(|k| k.iter().copied().collect::<Vec<_>>())
        .collect();
    while let Some(e) = stack.pop() {
        if !want.insert(e) {
            continue;
        }
        if let Some(kids) = world.get::<bevy_ecs::hierarchy::Children>(e) {
            stack.extend(kids.iter().copied());
        }
    }
    let have: BTreeSet<bevy_ecs::entity::Entity> = {
        let mut q = world.query_filtered::<bevy_ecs::entity::Entity, bevy_ecs::query::With<MotionTemplate>>();
        q.iter(world).collect()
    };
    let mut touched = false;
    for &e in want.difference(&have) {
        if let Ok(mut em) = world.get_entity_mut(e) {
            em.insert(MotionTemplate);
            touched = true;
        }
    }
    for &e in have.difference(&want) {
        if let Ok(mut em) = world.get_entity_mut(e) {
            em.remove::<MotionTemplate>();
            touched = true;
        }
    }
    touched
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::hierarchy::ChildOf;
    use bevy_ecs::world::World;

    /// ⭐ O filho (e o neto) de um objecto Motion é molde; a raiz não; quem sai deixa de ser.
    /// (Mutação: marcar também a raiz ⇒ RED.)
    #[test]
    fn the_descendants_of_a_motion_object_are_its_templates() {
        let mut w = World::new();
        let motion = w.spawn(MotionGenerator).id();
        let bola = w.spawn(ChildOf(motion)).id();
        let neto = w.spawn(ChildOf(bola)).id();
        let solto = w.spawn_empty().id();
        assert!(assign_motion_templates(&mut w));
        assert!(w.get::<MotionTemplate>(motion).is_none(), "a raiz não é molde");
        assert!(w.get::<MotionTemplate>(bola).is_some());
        assert!(w.get::<MotionTemplate>(neto).is_some());
        assert!(w.get::<MotionTemplate>(solto).is_none());
        assert!(!assign_motion_templates(&mut w), "estável: nada a mexer");
        w.entity_mut(bola).remove::<ChildOf>();
        assert!(assign_motion_templates(&mut w));
        assert!(w.get::<MotionTemplate>(bola).is_none(), "arrastado para fora, desenha-se");
        assert!(w.get::<MotionTemplate>(neto).is_none());
    }
}
