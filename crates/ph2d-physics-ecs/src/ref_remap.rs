//! ⭐ **O REMAP das referências da física** — o que faz a junta de uma cópia prender os corpos
//! **DELA** (ADR-0164 / F4.2).
//!
//! # A diferença para o [`crate::name_refs`], que é o módulo irmão
//!
//! Os dois reescrevem os mesmos campos, e respondem a perguntas diferentes:
//!
//! - `name_refs::resolve_body_names` traduz **nome → identidade**, uma vez, quando a cena é
//!   montada por autoria (o roteador dos smokes, a migração v95→v96).
//! - isto traduz **identidade → identidade**, e corre sempre que uma subárvore é copiada ou
//!   propagada.
//!
//! # ⚠️ As duas leis
//!
//! 1. **Só as entidades dadas.** Reescrever o mundo inteiro reescreveria as juntas do próprio
//!    mestre, e a cópia passaria a comandar o original.
//! 2. **Uma referência para FORA do que se copiou fica como está.** Ela não está no mapa, a
//!    busca falha, e o elo continua a apontar para o objeto da cena — que é exatamente o certo:
//!    um ragdoll pendurado num gancho do cenário continua pendurado nesse gancho.
//!
//! ⚠️ **Uma função por COMPONENTE**, e não uma que varre os dois: a tabela de remapeadores da
//! shell é conferida campo a campo contra o catálogo (`instance_refs.rs`), e um remapeador que
//! servisse dois tipos deixaria metade do censo sem par.

use bevy_ecs::prelude::{Entity, World};
use std::collections::BTreeMap;

/// **`PhysicsJoint.body_a` / `body_b`** — os dois corpos que a junta prende. Devolve quantas
/// juntas mudaram.
pub fn remap_joint_refs(
    world: &mut World,
    entities: &[Entity],
    by_id: &BTreeMap<u64, u64>,
) -> usize {
    let mut hits = 0;
    for &e in entities {
        let Some(mut j) = world.get_mut::<crate::PhysicsJoint>(e) else {
            continue;
        };
        let mut moved = false;
        if let Some(&id) = by_id.get(&j.body_a) {
            j.body_a = id;
            moved = true;
        }
        if let Some(&id) = by_id.get(&j.body_b) {
            j.body_b = id;
            moved = true;
        }
        if moved {
            hits += 1;
        }
    }
    hits
}

/// **`PulleyWheel.rope` / `body`** — a corda a que a roldana pertence e o corpo em que ela é
/// montada (`0` = pregada no cenário).
///
/// ⚠️ **A roldana é a SEXTA consulta da ponte** — a que a refutação 1 não nomeava (ela cita uma
/// faixa de linhas de um ficheiro, e a roldana nasceu noutro depois disso). Uma corda alcançada
/// pelo nome faz de uma referência por remapear uma roldana da instância a disputar a corda do
/// mestre.
pub fn remap_wheel_refs(
    world: &mut World,
    entities: &[Entity],
    by_id: &BTreeMap<u64, u64>,
) -> usize {
    let mut hits = 0;
    for &e in entities {
        let Some(mut w) = world.get_mut::<crate::PulleyWheel>(e) else {
            continue;
        };
        let mut moved = false;
        if let Some(&id) = by_id.get(&w.rope) {
            w.rope = id;
            moved = true;
        }
        // ⚠️ O `0` é *"pregada no cenário"*, e nunca uma identidade — o mapa nunca o contém
        // (o `StableId::FIRST` é 1), mas perguntar aqui torna a intenção legível.
        if w.body != 0
            && let Some(&id) = by_id.get(&w.body)
        {
            w.body = id;
            moved = true;
        }
        if moved {
            hits += 1;
        }
    }
    hits
}

#[cfg(test)]
#[path = "ref_remap_tests.rs"]
mod tests;
