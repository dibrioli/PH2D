//! **O conta-gotas do From do Motion Input** (GEF 08 onda 2) — mora aqui, junto da porta de nomes que o From já lê
//! ([`crate::gotas::resolve`]); o molde é o Target da bala: o próximo clique no palco ou na Hierarchy escolhe.

use bevy_ecs::prelude::Resource;
use ph2d_ecs::{Entity, Name, World};
use ph2d_editor_core::action_bus::{ActionBus, ComponentEdit, EditorAction};
use ph2d_editor_core::motion_input_edits::MotionInputFieldEdit;

/// O Motion Input cujo From o próximo clique escolhe. Estado de sessão.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FromArmado(pub Option<u64>);

/// Arma (ou desarma) o conta-gotas para `bits`.
pub fn arma(world: &mut World, bits: u64, on: bool) {
    world.insert_resource(FromArmado(on.then_some(bits)));
}

/// Está armado — para `quem`, ou para alguém com `None` (o molde de [`crate::gotas::armado`]).
#[must_use]
pub fn armado(world: &World, quem: Option<u64>) -> bool {
    let a = world.get_resource::<FromArmado>().and_then(|a| a.0);
    a.is_some() && (quem.is_none() || a == quem)
}

/// Desarma (o `Esc`) e devolve quem estava armado.
pub fn desarma(world: &mut World) -> Option<u64> {
    world.get_resource_mut::<FromArmado>()?.0.take()
}

/// ⭐ O clique do conta-gotas: o de cima em `debaixo` com nome vira o From (o próprio objecto = `""`, «This
/// object»); escolha ou vazio, desarma. `false` = não estava armado.
pub fn escolher(world: &mut World, bus: &mut ActionBus, debaixo: &[u64]) -> bool {
    let Some(dono) = desarma(world) else {
        return false;
    };
    let nome = debaixo.iter().find_map(|&b| {
        (b == dono).then(String::new).or_else(|| {
            world
                .get_entity(Entity::from_bits(b))
                .ok()
                .and_then(|e| e.get::<Name>().map(|n| n.as_str().trim().to_string()))
                .filter(|n| !n.is_empty())
        })
    });
    if let Some(nome) = nome {
        bus.push(EditorAction::InspectorComponentEdit {
            entity_bits: dono,
            edit: ComponentEdit::MotionInput(MotionInputFieldEdit::From(nome)),
        });
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::motion_input_edits::MotionInputFieldEdit as E;

    /// Armar é da sessão; o clique escreve o NOME pela porta do Inspector e desarma; o próprio = `""`.
    #[test]
    fn o_conta_gotas_do_from_escreve_o_nome_e_desarma() {
        let mut w = World::new();
        let dono = w.spawn_empty().id().to_bits();
        let alvo = w.spawn(Name("Boss".into())).id().to_bits();
        arma(&mut w, dono, true);
        assert!(armado(&w, None) && armado(&w, Some(dono)) && !armado(&w, Some(alvo)));
        let mut bus = ActionBus::default();
        for (debaixo, nome) in [(alvo, "Boss"), (dono, "")] {
            arma(&mut w, dono, true);
            assert!(escolher(&mut w, &mut bus, &[debaixo]));
            assert!(!armado(&w, None));
            let e: Vec<_> = bus.drain().collect();
            assert!(
                matches!(
                    e.as_slice(),
                    [EditorAction::InspectorComponentEdit { entity_bits, edit: ComponentEdit::MotionInput(E::From(n)) }]
                        if *entity_bits == dono && n == nome
                ),
                "{e:?}"
            );
        }
        assert!(
            !escolher(&mut w, &mut bus, &[alvo]),
            "desarmado não consome"
        );
    }
}
