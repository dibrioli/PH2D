//! **O cadeado de uma PASTA tranca o que está dentro** (dono 15/11, idioma Figma/Photoshop).
//!
//! A pasta é o tipo da Hierarquia sem arte própria ([`ph2d_editor_core::hier_types::ObjectType::is_folder`]: o vazio e a
//! montagem). A marca [`LockCoversChildren`] é DERIVADA aqui, a cada quadro, e nunca gravada — o
//! `Locked` sim, então depois de abrir um projeto o primeiro quadro repõe-na. Quem a lê é
//! [`ph2d_ecs::locked_by_a_folder`]; um objecto com arte continua a trancar só a si (26/05).

use bevy_ecs::query::Or;
use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::{Entity, LockCoversChildren, Locked, SimWorld, With};

/// Põe ou tira a marca em quem está trancado (ou ainda a tem); só escreve onde ela difere.
/// Devolve quantas entidades mudaram.
pub fn sync(sim: &mut SimWorld, registry: &ComponentRegistry) -> usize {
    let w = sim.world_mut();
    let alvo: Vec<Entity> = w
        .query_filtered::<Entity, Or<(With<Locked>, With<LockCoversChildren>)>>()
        .iter(w)
        .collect();
    if alvo.is_empty() {
        return 0;
    }
    let tipos = crate::row_components::RowComponents::of(w, registry);
    let mudar: Vec<(Entity, bool)> = alvo
        .into_iter()
        .filter_map(|e| {
            let quer = tipos.object_type(w, e.to_bits()).is_folder();
            (quer != w.get::<LockCoversChildren>(e).is_some()).then_some((e, quer))
        })
        .collect();
    for &(e, quer) in &mudar {
        let mut ent = w.entity_mut(e);
        if quer {
            ent.insert(LockCoversChildren);
        } else {
            ent.remove::<LockCoversChildren>();
        }
    }
    mudar.len()
}

#[cfg(test)]
#[path = "folder_lock_tests.rs"]
mod tests;
