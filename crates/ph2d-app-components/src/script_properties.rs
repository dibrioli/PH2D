//! ⭐ **As propriedades nomeadas que um script lê e escreve** (`ph2d.get(e, "health.max")`, doc
//! CodeEditor 08) — o lado da cena da porta [`ph2d_script::SceneDoors`]: a lei do nome
//! ([`ph2d_ecs::scene::property`]), o MESMO saneamento que o Inspector aplica, e o ledger da corrida
//! ([`ph2d_preview_drive::FieldDrive`]) para a escrita do script não virar documento.

use bevy_ecs::prelude::Resource;
use ph2d_component_desc::FieldKind;
use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::scene::property::{self, PropertyError};
use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics_ecs::{Health, HealthNow, PedidoDeVida, PedidosDeVida};
use ph2d_preview_drive::{FieldRef, PreviewDrive};
use ph2d_script::HealthRequest;
use serde_json::Value;

/// As escritas deste quadro, à espera do ledger (a porta não o vê; o `frame` sim).
#[derive(Resource, Default)]
struct ScriptFieldWrites(Vec<(Entity, FieldRef, Value, Value, bool)>);

/// O erro como o console do script o mostra — a palavra do tipo pela tabela (HR-15).
fn says(e: PropertyError) -> String {
    let PropertyError::WrongType { name, kind } = &e else {
        return e.to_string();
    };
    let key = match kind {
        FieldKind::Scalar | FieldKind::Angle => "app.components.script_property.takes_number",
        FieldKind::Int | FieldKind::Seed => "app.components.script_property.takes_whole_number",
        FieldKind::Vec2 => "app.components.script_property.takes_vec2",
        FieldKind::Vec4 => "app.components.script_property.takes_four_numbers",
        FieldKind::Color => "app.components.script_property.takes_color",
        FieldKind::Toggle => "app.components.script_property.takes_bool",
        FieldKind::Enum => "app.components.script_property.takes_option",
        FieldKind::Text | FieldKind::Ref => "app.components.script_property.takes_text",
        FieldKind::Curve | FieldKind::Gradient | FieldKind::Palette => {
            "app.components.script_property.takes_table"
        }
        FieldKind::Marker => "app.components.script_property.takes_nothing",
    };
    ph2d_i18n::tr(key).replace("{name}", name)
}

/// ⭐ Os valores da CORRIDA (`ph2d_script::api_table::RUNTIME_PROPERTIES`): a vida e o escudo de
/// agora. Sem corrida ainda, são os do nascimento (`start`, `shield`).
fn runtime_get(sim: &SimWorld, entity: Entity, name: &str) -> Option<Result<f64, String>> {
    let pick: fn(&HealthNow) -> f64 = match name {
        "health.now" => |n| n.pontos,
        "health.shield_now" => |n| n.escudo,
        _ => return None,
    };
    let w = sim.world();
    let Some(h) = w.get::<Health>(entity) else {
        return Some(Err(says(PropertyError::Missing {
            component: "health".into(),
        })));
    };
    let born = if name == "health.now" { h.start } else { h.shield_start };
    Some(Ok(w.get::<HealthNow>(entity).map_or(f64::from(born), pick)))
}

/// Escrever a vida/o escudo de agora é um pedido à ponte da física (`PedidoDeVida::Define`/`Escudo`):
/// sem piscar nem números, e em zero morre (decisão do dono nº 9).
fn runtime_set(sim: &mut SimWorld, entity: Entity, name: &str, value: &Value, dry: bool) -> Option<Result<(), String>> {
    runtime_get(sim, entity, name)?.err().map(Err).or_else(|| {
        let Some(x) = value.as_f64().filter(|x| x.is_finite()) else {
            return Some(Err(says(PropertyError::WrongType {
                name: name.to_owned(),
                kind: FieldKind::Scalar,
            })));
        };
        if !dry {
            let pedido = if name == "health.now" {
                PedidoDeVida::Define(x.max(0.0))
            } else {
                PedidoDeVida::Escudo(x.max(0.0))
            };
            pede(sim, entity, pedido);
        }
        Some(Ok(()))
    })
}

fn pede(sim: &mut SimWorld, entity: Entity, pedido: PedidoDeVida) {
    sim.world_mut()
        .get_resource_or_init::<PedidosDeVida>()
        .0
        .push((entity, pedido));
}

/// `ph2d.damage`/`ph2d.heal`: o mesmo pedido que o verbo da tabela de acções faz.
pub(crate) fn health_request(
    sim: &mut SimWorld,
    entity: Entity,
    request: HealthRequest,
    dry: bool,
) -> Result<(), String> {
    if sim.world().get::<Health>(entity).is_none() {
        return Err(says(PropertyError::Missing {
            component: "health".into(),
        }));
    }
    let pedido = match request {
        HealthRequest::Damage(x) => PedidoDeVida::Dano(x),
        HealthRequest::Heal(x) => PedidoDeVida::Cura(x),
    };
    if !dry {
        pede(sim, entity, pedido);
    }
    Ok(())
}

/// `ph2d.get` de uma propriedade nomeada: o valor e o tipo do campo.
pub(crate) fn get(
    reg: &ComponentRegistry,
    sim: &SimWorld,
    entity: Entity,
    name: &str,
) -> Result<(Value, FieldKind), String> {
    if let Some(v) = runtime_get(sim, entity, name) {
        return v.map(|x| (Value::from(x), FieldKind::Scalar));
    }
    let r = property::resolve(reg, name).map_err(says)?;
    let v = property::get(reg, sim.world(), entity, name).map_err(says)?;
    Ok((v, r.field.kind))
}

/// `ph2d.set` de uma propriedade nomeada; `dry` só verifica.
pub(crate) fn set(
    reg: &ComponentRegistry,
    sim: &mut SimWorld,
    entity: Entity,
    name: &str,
    value: &Value,
    dry: bool,
) -> Result<(), String> {
    if let Some(r) = runtime_set(sim, entity, name, value, dry) {
        return r;
    }
    let world = sim.world_mut();
    if dry {
        return property::check(reg, world, entity, name, value.clone()).map_err(says);
    }
    // O ledger guarda o valor GUARDADO, nunca o que o script vê (a `View`).
    let (before, r) = property::get_stored(reg, world, entity, name).map_err(says)?;
    let born = property::set(reg, world, entity, name, value.clone()).map_err(says)?;
    crate::property_laws::sanitize(r.entry.canonical_name, world, entity);
    let (after, _) = property::get_stored(reg, world, entity, name).map_err(says)?;
    // Uma cópia da corrida não é documento: não há autorado a guardar por baixo dela.
    if (born || before != after) && !ph2d_ecs::is_transient(world, entity) {
        let field = FieldRef::of(r.entry, r.field.serde_key());
        world
            .get_resource_or_init::<ScriptFieldWrites>()
            .0
            .push((entity, field, before, after, born));
    }
    Ok(())
}

/// `ph2d.properties`: os nomes de componente que o objecto tem.
pub(crate) fn names(reg: &ComponentRegistry, sim: &SimWorld, entity: Entity) -> Vec<String> {
    let mut out = property::names_of(reg, sim.world(), entity);
    if sim.world().get::<Health>(entity).is_some() {
        out.extend(ph2d_script::api_table::RUNTIME_PROPERTIES.iter().map(|n| (*n).to_owned()));
    }
    out
}

/// Um quadro: antes dos ganchos, o que mudou sem o script é outra mão (o Inspector na pausa).
pub(crate) fn observe(sim: &SimWorld, drive: &mut PreviewDrive) {
    drive.fields.observe(sim);
}

/// Depois dos ganchos: as escritas do quadro entram no ledger.
pub(crate) fn declare(sim: &mut SimWorld, drive: &mut PreviewDrive) {
    let Some(mut w) = sim.world_mut().get_resource_mut::<ScriptFieldWrites>() else {
        return;
    };
    for (entity, field, before, after, born) in std::mem::take(&mut w.0) {
        drive.fields.driven(entity, field, before, after, born);
    }
}

#[cfg(test)]
#[path = "script_properties_tests.rs"]
mod tests;
