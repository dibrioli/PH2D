//! **«Ignore up to» do Ray Sensor** (GEF 07, auditoria C8, dono 15/11): até onde o raio ignora a
//! própria família. O componente guarda o `StableId` do topo (`0` = automático, ver
//! [`ph2d_physics_ecs::topo_da_familia`]); o painel mostra e recebe o NOME, e o conta-gotas escolhe-o no
//! palco. Só um antepassado do raio é topo: um nome fora da cadeia queixa-se no cartão.

use bevy_ecs::prelude::Resource;
use ph2d_ecs::{ChildOf, Entity, Name, StableId, World};
use ph2d_editor_core::action_bus::{ActionBus, ComponentEdit, EditorAction};
use ph2d_editor_core::ray_edits::RayFieldEdit;
use ph2d_physics_ecs::RaySensor;

/// O raio cujo «Ignore up to» o próximo clique no palco escolhe (o conta-gotas). Estado de sessão.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TopoArmado(pub Option<u64>);

/// O conta-gotas do «Ignore up to» está armado (para `bits`, ou para qualquer raio com `None`)?
#[must_use]
pub fn armado(world: &World, bits: Option<u64>) -> bool {
    world
        .get_resource::<TopoArmado>()
        .and_then(|a| a.0)
        .is_some_and(|a| bits.is_none_or(|b| b == a))
}

/// Arma (`Some(raio)`) ou desarma (`None`) — sessão: nenhum passo de desfazer.
pub fn arma(world: &mut World, raio: Option<u64>) {
    world.insert_resource(TopoArmado(raio));
}

/// Desarma (o `Esc`) e devolve o raio que estava armado.
pub fn desarma(world: &mut World) -> Option<u64> {
    world.get_resource_mut::<TopoArmado>()?.0.take()
}

/// A cadeia de `e` para cima, sem `e`: os candidatos a topo, do mais perto ao mais alto.
fn acima(world: &World, e: Entity) -> impl Iterator<Item = Entity> + '_ {
    std::iter::successors(world.get::<ChildOf>(e).map(ChildOf::parent), |&c| {
        world.get::<ChildOf>(c).map(ChildOf::parent)
    })
}

fn nome(world: &World, e: Entity) -> Option<String> {
    world
        .get::<Name>(e)
        .map(|n| n.as_str().trim().to_owned())
        .filter(|n| !n.is_empty())
}

/// ⭐ **O clique do conta-gotas**: o primeiro objecto sob o cursor (ou um pai dele) que está ACIMA do raio
/// vira o topo, pela porta do Inspector (um passo de desfazer), e desarma. Um clique fora da cadeia não
/// escreve e CONTINUA armado (o padrão dos pickers: errar o alvo não é cancelar). `false` = não armado.
pub fn escolher_topo(world: &mut World, bus: &mut ActionBus, debaixo: &[u64]) -> bool {
    let Some(raio) = world.get_resource::<TopoArmado>().and_then(|a| a.0) else {
        return false;
    };
    let r = Entity::from_bits(raio);
    if world.get_entity(r).is_err() {
        desarma(world);
        return true;
    }
    let cadeia: Vec<Entity> = acima(world, r).collect();
    let topo = debaixo.iter().find_map(|&b| {
        let clicado = Entity::from_bits(b);
        world.get_entity(clicado).ok()?;
        std::iter::successors(Some(clicado), |&c| {
            world.get::<ChildOf>(c).map(ChildOf::parent)
        })
        .find(|c| cadeia.contains(c))
    });
    if let Some(n) = topo.and_then(|t| nome(world, t)) {
        desarma(world);
        bus.push(EditorAction::InspectorComponentEdit {
            entity_bits: raio,
            edit: ComponentEdit::Ray(RayFieldEdit::IgnoreUpTo(n)),
        });
    }
    true
}

/// O `StableId` a guardar para o nome que o artista escreveu: o antepassado MAIS PERTO com esse nome;
/// senão o objecto da cena com esse nome (fica fora da cadeia e o cartão queixa-se); `""` = `0`
/// (automático). `None` = ninguém se chama assim: a caixa volta ao que era.
#[must_use]
pub fn resolve(world: &World, raio: Entity, texto: &str) -> Option<u64> {
    let t = texto.trim();
    if t.is_empty() {
        return Some(0);
    }
    let stable = |e: Entity| world.get::<StableId>(e).map(|s| s.0);
    acima(world, raio)
        .find(|&c| nome(world, c).as_deref() == Some(t))
        .and_then(stable)
        .or_else(|| Some(ph2d_ecs::stable_id_named(world, t)).filter(|&id| id != 0))
}

/// O que o cartão mostra: (nome guardado, está fora da cadeia ou sumiu, nome do topo automático).
#[must_use]
pub fn leitura(world: &World, raio: Entity, r: &RaySensor) -> (String, bool, String) {
    let auto = ph2d_physics_ecs::topo_da_familia(world, raio, 0)
        .and_then(|t| nome(world, t))
        .unwrap_or_default();
    if r.ignore_up_to == 0 {
        return (String::new(), false, auto);
    }
    let alvo = ph2d_ecs::entity_with_stable_id(world, StableId(r.ignore_up_to));
    let n = alvo.and_then(|e| nome(world, e)).unwrap_or_default();
    let fora = alvo.is_none_or(|a| !acima(world, raio).any(|c| c == a));
    (n, fora, auto)
}
