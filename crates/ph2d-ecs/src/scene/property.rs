//! ⭐ **Uma propriedade por NOME** — `"health.max"` → o componente (pelo [`ComponentDesc::code`]) e
//! o campo (pelo [`FieldDesc::name`]), lido e escrito pela vtable do registo (doc CodeEditor 08 §3).
//!
//! É a lei única de quem nomeia um campo: os scripts (`ph2d.get/set`), o lint do editor e o arrasto
//! do Inspector para o código (8b). ⚠️ A pose (`x`, `rotation`…) não passa por aqui: é do script.
//!
//! Divergência declarada do Godot (oráculo `ferramentas/oraculo_propriedades/`): lá o `set` de um
//! nome ou tipo errado é calado; aqui é um [`PropertyError`] com o que se esperava.

use super::registry::{ComponentRegistry, ComponentTypeEntry};
use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_component_desc::{FieldDesc, FieldKind};
use serde_json::Value;

/// Porque um nome não se lê nem se escreve.
#[derive(Clone, Debug, PartialEq)]
pub enum PropertyError {
    /// Sem `.`: não é `componente.campo`.
    NotQualified(String),
    /// Nenhum componente se chama assim.
    UnknownComponent(String),
    /// O componente existe e não tem este campo.
    UnknownField { component: String, field: String },
    /// O objecto não tem o componente.
    Missing { component: String },
    /// O valor não serve ao campo.
    WrongType { name: String, kind: FieldKind },
    /// A entidade já não existe.
    NoEntity,
}

impl std::fmt::Display for PropertyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotQualified(n) => write!(f, "'{n}' is not a property: write component.field"),
            Self::UnknownComponent(c) => write!(f, "no component named '{c}'"),
            Self::UnknownField { component, field } => {
                write!(f, "'{component}' has no property '{field}'")
            }
            Self::Missing { component } => write!(f, "the object has no '{component}'"),
            Self::WrongType { name, kind } => {
                write!(f, "'{name}' takes {}", kind_word(*kind))
            }
            Self::NoEntity => f.write_str("the object no longer exists"),
        }
    }
}

impl std::error::Error for PropertyError {}

/// A palavra que diz ao script que valor um campo aceita.
#[must_use]
pub fn kind_word(kind: FieldKind) -> &'static str {
    match kind {
        FieldKind::Scalar | FieldKind::Angle => "a number",
        FieldKind::Int | FieldKind::Seed => "a whole number",
        FieldKind::Vec2 => "a vec2",
        FieldKind::Vec4 => "four numbers",
        FieldKind::Color => "a color",
        FieldKind::Toggle => "true or false",
        FieldKind::Enum => "one of its option names",
        FieldKind::Text | FieldKind::Ref => "a text",
        FieldKind::Curve | FieldKind::Gradient | FieldKind::Palette => "a table",
        FieldKind::Marker => "nothing (it is a marker)",
    }
}

/// Um nome resolvido: a entrada do registo e o campo.
#[derive(Clone, Copy)]
pub struct Resolved<'r> {
    pub entry: &'r ComponentTypeEntry,
    pub field: &'static FieldDesc,
}

/// `"health.max"` → a entrada e o campo, ou porque não.
pub fn resolve<'r>(reg: &'r ComponentRegistry, name: &str) -> Result<Resolved<'r>, PropertyError> {
    let (code, field) = name
        .split_once('.')
        .ok_or_else(|| PropertyError::NotQualified(name.to_owned()))?;
    let entry = reg
        .iter()
        .find(|e| e.desc.is_some_and(|d| d.code == code))
        .ok_or_else(|| PropertyError::UnknownComponent(code.to_owned()))?;
    let desc = entry.desc.expect("filtrado acima");
    let field = desc
        .fields
        .iter()
        .find(|f| !f.name.is_empty() && f.name == field)
        .ok_or_else(|| PropertyError::UnknownField {
            component: code.to_owned(),
            field: field.to_owned(),
        })?;
    Ok(Resolved { entry, field })
}

/// Todos os nomes que um script alcança, na ordem do catálogo.
pub fn all_names(reg: &ComponentRegistry) -> Vec<String> {
    let mut out = Vec::new();
    for d in ph2d_component_desc::all() {
        if d.code.is_empty() || reg.get_by_name(d.canonical_name).is_none() {
            continue;
        }
        out.extend(
            d.fields
                .iter()
                .filter(|f| !f.name.is_empty())
                .map(|f| format!("{}.{}", d.code, f.name)),
        );
    }
    out
}

/// Os nomes que ESTE objecto tem (o `get_property_list` do Godot), na ordem do catálogo.
pub fn names_of(reg: &ComponentRegistry, world: &World, entity: Entity) -> Vec<String> {
    all_names(reg)
        .into_iter()
        .filter(|n| get(reg, world, entity, n).is_ok())
        .collect()
}

/// Lê `name` do objecto.
pub fn get(
    reg: &ComponentRegistry,
    world: &World,
    entity: Entity,
    name: &str,
) -> Result<Value, PropertyError> {
    let r = resolve(reg, name)?;
    let whole = component_value(&r, world, entity)?;
    whole
        .get(r.field.serde_key())
        .cloned()
        .ok_or_else(|| PropertyError::UnknownField {
            component: code_of(&r).to_owned(),
            field: r.field.name.to_owned(),
        })
}

/// Escreve `value` em `name` no objecto; os outros campos do componente ficam.
pub fn set(
    reg: &ComponentRegistry,
    world: &mut World,
    entity: Entity,
    name: &str,
    value: Value,
) -> Result<(), PropertyError> {
    let r = resolve(reg, name)?;
    let mut whole = component_value(&r, world, entity)?;
    let slot = whole
        .get_mut(r.field.serde_key())
        .ok_or_else(|| PropertyError::UnknownField {
            component: code_of(&r).to_owned(),
            field: r.field.name.to_owned(),
        })?;
    *slot = normalize(r.field.kind, value);
    let wrong = || PropertyError::WrongType {
        name: name.to_owned(),
        kind: r.field.kind,
    };
    (r.entry.set_value)(world, entity, whole).map_err(|_| wrong())
}

/// Um número inteiro chega do Luau como real (`3.0`); o serde de um `i32`/`u64` não o aceita.
fn normalize(kind: FieldKind, value: Value) -> Value {
    match (kind, &value) {
        (FieldKind::Int | FieldKind::Seed, Value::Number(n)) => match n.as_f64() {
            Some(x) if x.fract() == 0.0 && x >= 0.0 && x <= u64::MAX as f64 => Value::from(x as u64),
            Some(x) if x.fract() == 0.0 && x >= i64::MIN as f64 => Value::from(x as i64),
            _ => value,
        },
        _ => value,
    }
}

fn code_of(r: &Resolved<'_>) -> &'static str {
    r.entry.desc.map_or("", |d| d.code)
}

fn component_value(r: &Resolved<'_>, world: &World, entity: Entity) -> Result<Value, PropertyError> {
    match (r.entry.value_of)(world, entity) {
        Ok(Some(v)) => Ok(v),
        Ok(None) => Err(PropertyError::Missing {
            component: code_of(r).to_owned(),
        }),
        Err(_) => Err(PropertyError::NoEntity),
    }
}
