//! **Os conta-gotas «Aim at» e «Also ignore» do Ray Sensor** (RS-3, RS-12, dono 15/11) — irmãos do
//! [`crate::topo`]: o componente guarda `StableId`s, o painel mostra e recebe NOMES, e o conta-gotas
//! escolhe no palco (ou na Hierarchy). Um conta-gotas armado é estado de sessão: nenhum desfazer.

use bevy_ecs::prelude::Resource;
use ph2d_ecs::{Entity, Name, StableId, World};
use ph2d_editor_core::action_bus::{ActionBus, ComponentEdit, EditorAction};
use ph2d_editor_core::ray_edits::RayFieldEdit;

/// Qual dos dois campos o próximo clique escolhe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gotas {
    /// «Aim at»: o alvo.
    Mira,
    /// «Also ignore»: mais um na lista.
    Lista,
}

/// O raio (`Entity::to_bits`) e o campo armados.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GotasDoRaio(pub Option<(u64, Gotas)>);

/// O nome de ninguém que o artista escreveu no «Aim at», por raio — a caixa mostra-o com o erro.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct MiraEscrita(pub Option<(u64, String)>);

/// Arma (`Some`) ou desarma.
pub fn arma(world: &mut World, alvo: Option<(u64, Gotas)>) {
    world.insert_resource(GotasDoRaio(alvo));
}

/// Está armado (para `alvo`, ou qualquer um com `None`)?
#[must_use]
pub fn armado(world: &World, alvo: Option<(u64, Gotas)>) -> bool {
    world
        .get_resource::<GotasDoRaio>()
        .and_then(|g| g.0)
        .is_some_and(|a| alvo.is_none_or(|b| a == b))
}

/// Desarma (o `Esc`) e devolve o que estava armado.
pub fn desarma(world: &mut World) -> Option<(u64, Gotas)> {
    world.get_resource_mut::<GotasDoRaio>()?.0.take()
}

/// Guarda (`Some`) ou apaga o nome de ninguém do «Aim at» do raio `raio`.
pub fn escrita(world: &mut World, raio: u64, texto: Option<&str>) {
    world.insert_resource(MiraEscrita(texto.map(|t| (raio, t.trim().to_owned()))));
}

fn nome(world: &World, e: Entity) -> Option<String> {
    world
        .get::<Name>(e)
        .map(|n| n.as_str().trim().to_owned())
        .filter(|n| !n.is_empty())
}

/// ⭐ **O clique com um destes conta-gotas armado**: o primeiro objecto com nome sob o cursor (que não seja o
/// próprio raio) vai para o campo, por um passo de desfazer, e desarma. `false` = nenhum armado.
pub fn escolher(world: &mut World, bus: &mut ActionBus, debaixo: &[u64]) -> bool {
    let Some((raio, campo)) = world.get_resource::<GotasDoRaio>().and_then(|g| g.0) else {
        return false;
    };
    let n = debaixo
        .iter()
        .filter(|&&b| b != raio)
        .filter_map(|&b| Entity::try_from_bits(b))
        .find_map(|e| world.get_entity(e).ok().and_then(|_| nome(world, e)));
    if let Some(n) = n {
        desarma(world);
        bus.push(EditorAction::InspectorComponentEdit {
            entity_bits: raio,
            edit: ComponentEdit::Ray(match campo {
                Gotas::Mira => RayFieldEdit::AimAt(n),
                Gotas::Lista => RayFieldEdit::IgnoreAdd(n),
            }),
        });
    }
    true
}

/// O `StableId` do objecto com este nome; `""` = `0` (ninguém). `None` = ninguém se chama assim.
#[must_use]
pub fn resolve(world: &World, texto: &str) -> Option<u64> {
    let t = texto.trim();
    if t.is_empty() {
        return Some(0);
    }
    Some(ph2d_ecs::stable_id_named(world, t)).filter(|&id| id != 0)
}

/// Os nomes dos objectos da lista que ainda existem, pela ordem dela.
#[must_use]
pub fn nomes(world: &World, ids: &[u64]) -> Vec<String> {
    ids.iter()
        .filter_map(|&id| ph2d_ecs::entity_with_stable_id(world, StableId(id)))
        .filter_map(|e| nome(world, e))
        .collect()
}

/// O que a caixa «Aim at» mostra: (nome, o alvo sumiu, ninguém se chama assim). O nome de ninguém escrito
/// para ESTE raio ganha ao guardado.
#[must_use]
pub fn leitura_da_mira(world: &World, raio: u64, alvo: u64) -> (String, bool, bool) {
    if let Some(MiraEscrita(Some((bits, t)))) = world.get_resource::<MiraEscrita>()
        && *bits == raio
    {
        return (t.clone(), false, true);
    }
    if alvo == 0 {
        return (String::new(), false, false);
    }
    match ph2d_ecs::entity_with_stable_id(world, StableId(alvo)).and_then(|e| nome(world, e)) {
        Some(n) => (n, false, false),
        None => (String::new(), true, false),
    }
}

/// ⭐ **O «material» do acerto** (RS-4, dono 15/11): as tags DIRECTAS da forma acertada (a «Collision Shape»
/// filha leva as dela: «Ice», «Grass»), pelo rótulo, juntas por vírgula; `""` = nenhuma.
#[must_use]
pub fn tags_da_forma(world: &World, tree: &ph2d_tags::TagTree, forma: Entity) -> String {
    world
        .get::<ph2d_ecs::tags::Tags>(forma)
        .map(|t| {
            t.direct_ids()
                .filter_map(|id| tree.get(id).map(|t| t.label().to_owned()))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default()
}
