//! ⭐ **As referências por NOME** (doc CodeEditor 08 §5, d4) — o irmão de [`super::property`]: o campo
//! guarda a identidade do alvo (o `stable_name_id`, o `StableId`, o `TagId`) e o script lê e escreve
//! o NOME dele; e o campo de VARIANTE de um enum com dados (`"target:{\"Named\":0}"`,
//! [`ph2d_component_desc::FieldDesc::serde_key`]).
//!
//! Escrever o nome de um alvo que não existe é um erro (a gralha diz-se na hora); ler um alvo que já
//! não existe é `nil` — o `stable_name_id` é um hash e não devolve o texto.

use super::property::PropertyError;
use crate::{Name, StableId, stable_name_id};
use bevy_ecs::world::World;
use ph2d_component_desc::{FieldKind, Target};
use ph2d_tags::{TagId, TagTree};
use serde_json::Value;

/// O que nomeia uma identidade que o mundo não guarda.
#[derive(Clone, Copy, Default)]
pub struct RefNames<'a> {
    /// A árvore das tags do projecto; `None` = nenhuma tag tem nome.
    pub tags: Option<&'a TagTree>,
}

/// O nome do alvo guardado em `v`; `Null` = ninguém, ou um alvo que já não existe.
pub fn name_of(world: &World, names: RefNames<'_>, target: Target, v: &Value) -> Value {
    let id = |v: &Value| v.as_u64().filter(|id| *id != 0);
    let text = |s: Option<String>| s.map_or(Value::Null, Value::String);
    let tag = |id: u64| names.tags?.get(TagId(id)).map(|t| t.path.clone());
    match target {
        Target::Name => match v.as_str() {
            Some("") | None => Value::Null,
            Some(_) => v.clone(),
        },
        Target::NameHash => text(id(v).and_then(|id| object_hashed(world, id))),
        Target::Stable => text(
            id(v)
                .and_then(|id| crate::entity_with_stable_id(world, StableId(id)))
                .and_then(|e| world.get::<Name>(e))
                .map(|n| n.as_str().to_owned()),
        ),
        Target::Tag => text(id(v).and_then(tag)),
        Target::Layer => text(
            v.as_u64()
                .and_then(|i| u8::try_from(i).ok())
                .and_then(|i| layers(world).name(crate::LayerId(i)).map(str::to_owned)),
        ),
        Target::TagSet => Value::Array(
            v.as_array()
                .into_iter()
                .flatten()
                .filter_map(|x| id(x).and_then(tag))
                .map(Value::String)
                .collect(),
        ),
    }
}

/// A identidade a guardar no campo `name` para o nome que o script deu (`nil`/`""` = ninguém).
pub fn id_of(
    world: &World,
    names: RefNames<'_>,
    target: Target,
    name: &str,
    v: Value,
) -> Result<Value, PropertyError> {
    let wrong = || PropertyError::WrongType {
        name: name.to_owned(),
        kind: FieldKind::Ref,
    };
    let absent = |wanted: &str| PropertyError::NoTarget {
        name: name.to_owned(),
        wanted: wanted.to_owned(),
        target,
    };
    let tag = |wanted: &str| {
        names
            .tags
            .and_then(|t| t.find(wanted))
            .map(|t| Value::from(t.0))
            .ok_or_else(|| absent(wanted))
    };
    if target == Target::TagSet {
        let items = match v {
            Value::Array(a) => a,
            Value::Null => Vec::new(),
            Value::Object(o) if o.is_empty() => Vec::new(),
            _ => return Err(wrong()),
        };
        if items.len() > crate::tags::TAGS_MAX {
            return Err(PropertyError::TooMany {
                name: name.to_owned(),
                max: crate::tags::TAGS_MAX,
            });
        }
        return items
            .iter()
            .map(|x| x.as_str().ok_or_else(wrong).and_then(tag))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array);
    }
    let wanted = match &v {
        Value::Null => "",
        Value::String(s) => s.as_str(),
        _ => return Err(wrong()),
    };
    let object_exists = || {
        world
            .try_query::<&Name>()
            .is_some_and(|mut q| q.iter(world).any(|n| n.as_str() == wanted))
    };
    Ok(match target {
        Target::Name if wanted.is_empty() => Value::String(String::new()),
        Target::Layer if wanted.is_empty() => return Err(wrong()),
        _ if wanted.is_empty() => Value::from(0_u64),
        Target::Name if object_exists() => Value::String(wanted.to_owned()),
        Target::NameHash if object_exists() => Value::from(stable_name_id(wanted)),
        Target::Stable => match crate::stable_id_named(world, wanted) {
            0 => return Err(absent(wanted)),
            id => Value::from(id),
        },
        Target::Tag => tag(wanted)?,
        Target::Layer => match layers(world).iter().find(|(_, n)| *n == wanted) {
            Some((id, _)) => Value::from(id.0),
            None => return Err(absent(wanted)),
        },
        Target::Name | Target::NameHash | Target::TagSet => return Err(absent(wanted)),
    })
}

/// As camadas do projecto; sem o recurso, as de fábrica (a mesma regra da ordenação).
fn layers(world: &World) -> std::borrow::Cow<'_, crate::SortingLayers> {
    world.get_resource::<crate::SortingLayers>().map_or_else(
        || std::borrow::Cow::Owned(crate::SortingLayers::canonical()),
        std::borrow::Cow::Borrowed,
    )
}

/// O nome do objecto cujo `Name` dá este `stable_name_id`.
fn object_hashed(world: &World, id: u64) -> Option<String> {
    let mut q = world.try_query::<&Name>()?;
    q.iter(world)
        .find(|n| stable_name_id(n.as_str()) == id)
        .map(|n| n.as_str().to_owned())
}

/// `"target:{\"Named\":0}/x"` → (`"target"`, o molde, `"x"`); `None` = uma chave sem variante.
pub fn variant(key: &str) -> Option<(&str, Value, &str)> {
    let (top, rest) = key.split_once(':')?;
    let mut it = serde_json::Deserializer::from_str(rest).into_iter::<Value>();
    let molde = it.next()?.ok()?;
    let tail = &rest[it.byte_offset()..];
    Some((top, molde, tail.strip_prefix('/').unwrap_or(tail)))
}

/// O nome da variante de um valor de enum: `"Here"`, `{"Area": …}` → `Area`.
#[must_use]
pub fn variant_name(v: &Value) -> Option<&str> {
    match v {
        Value::String(s) => Some(s),
        Value::Object(o) if o.len() == 1 => o.keys().next().map(String::as_str),
        _ => None,
    }
}

/// O conteúdo da variante `name` em `v`, se for essa a variante.
pub fn payload_mut<'v>(v: &'v mut Value, name: &str) -> Option<&'v mut Value> {
    match v {
        Value::Object(o) if o.len() == 1 => o.get_mut(name),
        _ => None,
    }
}
