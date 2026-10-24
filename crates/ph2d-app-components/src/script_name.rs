//! ⭐ **O nome de um objecto trocado por um script a correr** — `ph2d.set(e, "name", "Porta Aberta")`
//! (doc CodeEditor 11, decisão do dono 24/10). O `node.name = …` do Godot e o `gameObject.name` do Unity,
//! como OVERRIDE da corrida ([`RunName`]): o `ph2d.object`/`ph2d.find_by_name` seguintes acham-no pelo
//! nome novo; o [`Name`] do documento (a identidade) não muda; o rebobinar ([`rewind`]) devolve o nome
//! do artista. A Hierarquia mostra o documento ([`ph2d_ecs::scene::build_hierarchy_snapshot`]).

use ph2d_component_desc::FieldKind;
use ph2d_ecs::scene::property::PropertyError;
use ph2d_ecs::{Entity, Name, RunName, SimWorld, name_in_run};
use serde_json::Value;

/// O nome da propriedade (`ph2d_script::api_table::RUNTIME_PROPERTIES`).
pub const NAME: &str = "name";

/// `ph2d.get(e, "name")`: o nome por que o jogo o acha agora.
pub(crate) fn get(sim: &SimWorld, entity: Entity) -> (Value, FieldKind) {
    let v = name_in_run(sim.world(), entity).map_or(Value::Null, Value::from);
    (v, FieldKind::Text)
}

/// `ph2d.set(e, "name", texto)`; `dry` só verifica. O nome do artista não é override: escrevê-lo é
/// voltar a ele.
pub(crate) fn set(
    sim: &mut SimWorld,
    entity: Entity,
    value: &Value,
    dry: bool,
) -> Result<(), String> {
    let Value::String(s) = value else {
        return Err(crate::script_properties::says(PropertyError::WrongType {
            name: NAME.to_owned(),
            kind: FieldKind::Text,
        }));
    };
    if s.is_empty() {
        return Err(ph2d_i18n::tr("app.components.script_property.empty_name").to_owned());
    }
    if dry {
        return Ok(());
    }
    let mut em = sim.world_mut().entity_mut(entity);
    if em.get::<Name>().is_some_and(|n| n.as_str() == s) {
        em.remove::<RunName>();
    } else {
        em.insert(RunName(s.clone()));
    }
    Ok(())
}

/// O rebobinar: o nome do artista volta.
pub(crate) fn rewind(sim: &mut SimWorld) {
    let w = sim.world_mut();
    let alvos: Vec<Entity> = w
        .query_filtered::<Entity, bevy_ecs::prelude::With<RunName>>()
        .iter(w)
        .collect();
    for e in alvos {
        w.entity_mut(e).remove::<RunName>();
    }
}

#[cfg(test)]
#[path = "script_name_tests.rs"]
mod tests;
