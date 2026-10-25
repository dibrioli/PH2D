//! ⭐⭐ **Largar linhas da Hierarquia sobre outra — a lei do reparentar, para UMA ou VÁRIAS.**
//!
//! Report do dono (25/10, F30 §F): *«não consigo colocar múltiplos objetos selecionados como filhos
//! de outro de uma única vez»*. O arrasto levava um `NodeId` só e o dreno da shell aplicava esse um.
//! A lei (Unity/Godot/Blender), escrita UMA vez aqui e chamada pelo dreno
//! (`shells/desktop/src/hero_intents/hierarchy.rs`):
//!
//! * a linha agarrada **seleccionada** leva a selecção inteira; não seleccionada, vai sozinha;
//! * um seleccionado com um antepassado também seleccionado vai JUNTO com ele (não se arranca);
//! * o bloco mantém a ordem da Hierarquia ([`ph2d_ecs::list_order`]), não a do clique;
//! * cada um fica onde está no MUNDO; o gesto inteiro é atómico — ou vão todos, ou nenhum.
//!
//! Plano e mecanismo: `docs/UI_New_and_Simple/F30_plano.md` §F.

use ph2d_ecs::{ChildOf, Entity, RootOrder, SimWorld, Transform, World};

/// Onde a queda aterrou, já resolvida em entidades (a ponte `NodeId → Entity` é da shell).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Queda {
    /// O pai novo pedido (`None` = a raiz, ou o pai do alvo `before`/`after`).
    pub new_parent: Option<Entity>,
    /// O bloco entra ANTES deste irmão.
    pub before: Option<Entity>,
    /// O bloco entra DEPOIS deste irmão (só lido sem `before`).
    pub after: Option<Entity>,
}

/// O que a queda fez.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Desfecho {
    /// Mudaram de lugar, na ordem do bloco. `into_a_recipe` = o pai novo vive numa receita.
    Movidos {
        quem: Vec<Entity>,
        into_a_recipe: bool,
    },
    /// Uma das peças é dada por uma receita, e o lugar dela é da receita.
    RecusaPecaDeReceita,
    /// O pai novo é um dos que viajam, ou descende de um deles.
    RecusaCiclo,
}

/// ⭐ **Quem viaja** quando se agarra `agarrado` com `seleccionados` acesos.
#[must_use]
pub fn quem_viaja(world: &mut World, agarrado: Entity, seleccionados: &[Entity]) -> Vec<Entity> {
    if !seleccionados.contains(&agarrado) {
        return vec![agarrado];
    }
    let tem_antepassado_seleccionado = |world: &World, e: Entity| {
        let mut cur = world.get::<ChildOf>(e).map(ChildOf::parent);
        while let Some(p) = cur {
            if seleccionados.contains(&p) {
                return true;
            }
            cur = world.get::<ChildOf>(p).map(ChildOf::parent);
        }
        false
    };
    let ordem = ph2d_ecs::list_order(world);
    let quem: Vec<Entity> = ordem
        .into_iter()
        .filter(|e| seleccionados.contains(e) && !tem_antepassado_seleccionado(world, *e))
        .collect();
    if quem.is_empty() {
        vec![agarrado]
    } else {
        quem
    }
}

/// O pai que a queda pede: o explícito, ou o do irmão-alvo.
fn pai_da_queda(world: &World, queda: Queda) -> Option<Entity> {
    if queda.new_parent.is_some() {
        return queda.new_parent;
    }
    queda
        .before
        .or(queda.after)
        .and_then(|t| world.get::<ChildOf>(t).map(ChildOf::parent))
}

/// Os irmãos do pai (ou as raízes com `Transform`), na ordem da Hierarquia.
fn irmaos(world: &mut World, pai: Option<Entity>) -> Vec<Entity> {
    match pai {
        Some(p) => ph2d_ecs::ordered_children(world, p),
        None => {
            let mut raizes: Vec<Entity> = world
                .query_filtered::<Entity, (ph2d_ecs::With<Transform>, ph2d_ecs::Without<ChildOf>)>()
                .iter(world)
                .collect();
            raizes.sort_unstable_by_key(|&e| ph2d_ecs::root_key(world, e));
            raizes
        }
    }
}

/// ⭐⭐ **Aplica a queda a `quem`** (a saída de [`quem_viaja`]) — atómico, num quadro, logo um passo
/// de undo.
pub fn reparentar(sim: &mut SimWorld, quem: &[Entity], queda: Queda) -> Desfecho {
    let pai = pai_da_queda(sim.world(), queda);
    if let Some(np) = pai {
        let mut cur = Some(np);
        while let Some(c) = cur {
            if quem.contains(&c) {
                return Desfecho::RecusaCiclo;
            }
            cur = sim.world().get::<ChildOf>(c).map(ChildOf::parent);
        }
    }
    if quem
        .iter()
        .any(|&m| crate::instance_verbs_walk::refuses_reparent(sim, m, pai))
    {
        return Desfecho::RecusaPecaDeReceita;
    }
    let into_a_recipe = pai.is_some_and(|p| ph2d_ecs::master_root_of(sim.world(), p).is_some());
    let world = sim.world_mut();
    // A pose de MUNDO de cada um, sob a cadeia ANTIGA — para não saltar ao mudar de pai.
    let mundo_antes: Vec<Option<Transform>> = quem
        .iter()
        .map(|&m| {
            world
                .get::<Transform>(m)
                .copied()
                .map(|l| Transform::compose(ph2d_ecs::parent_world_transform(world, m), l))
        })
        .collect();
    // O lugar do bloco: quantos NÃO-viajantes ficam antes do alvo na fila de ANTES da mudança.
    let fila = irmaos(world, pai);
    let fica = |e: &Entity| !quem.contains(e);
    let alvo = queda
        .before
        .map(|t| (t, true))
        .or(queda.after.map(|t| (t, false)));
    let em = match alvo.and_then(|(t, antes)| fila.iter().position(|e| *e == t).map(|i| (i, antes)))
    {
        Some((i, true)) => fila[..i].iter().filter(|e| fica(e)).count(),
        Some((i, false)) => fila[..=i].iter().filter(|e| fica(e)).count(),
        None => fila.iter().filter(|e| fica(e)).count(),
    };
    let mut desejada: Vec<Entity> = fila.into_iter().filter(|e| fica(e)).collect();
    for (k, &m) in quem.iter().enumerate() {
        desejada.insert(em + k, m);
    }
    match pai {
        Some(p) => {
            // `Children` do bevy refeito na ordem (runtime) + `SiblingOrder` (o dado do ficheiro).
            for &c in &desejada {
                if let Ok(mut entry) = world.get_entity_mut(c) {
                    entry.remove::<ChildOf>();
                    entry.insert(ChildOf(p));
                }
            }
            ph2d_ecs::set_sibling_order(world, p, &desejada);
        }
        None => {
            for &m in quem {
                if let Ok(mut entry) = world.get_entity_mut(m) {
                    entry.remove::<ChildOf>();
                }
            }
            for (i, &e) in desejada.iter().enumerate() {
                if let Ok(mut entry) = world.get_entity_mut(e) {
                    entry.insert(RootOrder(i as u32));
                }
            }
        }
    }
    // `local = mundo_do_pai⁻¹ ∘ mundo_antes`, já com a cadeia NOVA no lugar.
    for (&m, antes) in quem.iter().zip(mundo_antes) {
        let Some(antes) = antes else { continue };
        let pw = ph2d_ecs::parent_world_transform(world, m);
        let np = ph2d_editor_core::TransformSnapshot {
            translation: [pw.translation.x, pw.translation.y],
            rotation: pw.rotation,
            scale: [pw.scale.x, pw.scale.y],
        };
        let lt = ph2d_editor_core::world_translation_to_local(
            np,
            [antes.translation.x, antes.translation.y],
        );
        let psx = if pw.scale.x.abs() > 1e-6 {
            pw.scale.x
        } else {
            1.0
        };
        let psy = if pw.scale.y.abs() > 1e-6 {
            pw.scale.y
        } else {
            1.0
        };
        if let Some(mut t) = world.get_mut::<Transform>(m) {
            t.translation = ph2d_core::Vec2::new(lt[0], lt[1]);
            t.rotation = antes.rotation - pw.rotation;
            t.scale = ph2d_core::Vec2::new(antes.scale.x / psx, antes.scale.y / psy);
            t.skew_x = antes.skew_x - pw.skew_x;
            t.skew_y = antes.skew_y - pw.skew_y;
        }
    }
    Desfecho::Movidos {
        quem: quem.to_vec(),
        into_a_recipe,
    }
}

#[cfg(test)]
#[path = "hierarchy_reparent_tests.rs"]
mod tests;
