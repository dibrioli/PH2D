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
use ph2d_component_desc::{ComponentDesc, FieldDesc, FieldKind, View, WHOLE};
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
    /// Uma escolha que o campo não tem; `options` são as que tem.
    NoSuchOption { name: String, options: String },
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
            // A palavra do tipo é de quem pinta a mensagem (a ponte dos scripts, pela tabela).
            Self::WrongType { name, kind } => write!(f, "'{name}' takes a {kind:?} value"),
            Self::NoEntity => f.write_str("the object no longer exists"),
            Self::NoSuchOption { name, options } => write!(f, "'{name}' takes one of: {options}"),
        }
    }
}

impl std::error::Error for PropertyError {}

/// Um nome resolvido: a entrada do registo e o campo.
#[derive(Clone, Copy)]
pub struct Resolved<'r> {
    pub entry: &'r ComponentTypeEntry,
    pub field: &'static FieldDesc,
}

/// `"health.max"` → a entrada e o campo, ou porque não. Um campo que se chama como o componente
/// tem o nome curto: `"gravity_scale"`, e nunca `"gravity_scale.gravity_scale"` ([`full_name`]).
pub fn resolve<'r>(reg: &'r ComponentRegistry, name: &str) -> Result<Resolved<'r>, PropertyError> {
    let entry_of = |code: &str| reg.iter().find(|e| e.desc.is_some_and(|d| d.code == code));
    let (code, field) = match name.split_once('.') {
        Some((code, field)) if field != code => (code, field),
        Some(_) => return Err(PropertyError::NotQualified(name.to_owned())),
        None if entry_of(name).is_some() => (name, name),
        None => return Err(PropertyError::NotQualified(name.to_owned())),
    };
    let entry = entry_of(code).ok_or_else(|| PropertyError::UnknownComponent(code.to_owned()))?;
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
                .map(|f| full_name(d.code, f.name)),
        );
    }
    out
}

/// O nome com que um script alcança o campo `field` do componente `code`.
#[must_use]
pub fn full_name(code: &str, field: &str) -> String {
    if code == field { code.to_owned() } else { format!("{code}.{field}") }
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
    get_stored(reg, world, entity, name).map(|(v, r)| seen(r.field.view, v))
}

/// O valor GUARDADO de `name` (sem a [`View`]) — o que o ledger da corrida grava e repõe.
pub fn get_stored<'r>(
    reg: &'r ComponentRegistry,
    world: &World,
    entity: Entity,
    name: &str,
) -> Result<(Value, Resolved<'r>), PropertyError> {
    let r = resolve(reg, name)?;
    let mut whole = component_value(reg, &r, world, entity)?;
    let v = slot(&r, &mut whole)?.clone();
    Ok((v, r))
}

/// O valor guardado, como o script o vê ([`View`]).
fn seen(view: View, v: Value) -> Value {
    let k = |x: f64, f: &dyn Fn(f64) -> f64| Value::from(f(x));
    match (view, v) {
        (View::Degrees, Value::Array(a)) => Value::Array(a.into_iter().map(|x| seen(view, x)).collect()),
        (View::Degrees, Value::Number(n)) => k(n.as_f64().unwrap_or(0.0), &f64::to_radians),
        (View::Scale(q), Value::Number(n)) => k(n.as_f64().unwrap_or(0.0), &|x| x / f64::from(q)),
        (View::Offset(o), Value::Number(n)) => match n.as_i64() {
            Some(i) => Value::from(i + i64::from(o)),
            None => Value::Number(n),
        },
        (View::Not, Value::Bool(b)) => Value::Bool(!b),
        (View::Toggle(t, _), v) => Value::Bool(json(t) == v),
        (View::Options(opts), v) => opts
            .iter()
            .find(|(_, j)| json(j) == v)
            .map_or(v, |(n, _)| Value::String((*n).to_owned())),
        (_, v) => v,
    }
}

/// O valor que o script deu, como o componente o guarda; `stored` é o que lá está (um inteiro
/// guardado continua inteiro).
fn stored_from(view: View, v: Value, stored: &Value, name: &str) -> Result<Value, PropertyError> {
    let num = |v: &Value| v.as_f64();
    let inteiro = |x: f64| if stored.is_i64() || stored.is_u64() { Value::from(x.round() as i64) } else { Value::from(x) };
    Ok(match (view, v) {
        (View::Degrees, Value::Array(a)) => Value::Array(
            a.into_iter()
                .map(|x| stored_from(view, x, &Value::Null, name))
                .collect::<Result<_, _>>()?,
        ),
        (View::Degrees, v) if num(&v).is_some() => Value::from(num(&v).unwrap_or(0.0).to_degrees()),
        (View::Scale(q), v) if num(&v).is_some() => inteiro(num(&v).unwrap_or(0.0) * f64::from(q)),
        (View::Offset(o), v) if num(&v).is_some() => inteiro(num(&v).unwrap_or(0.0) - f64::from(o)),
        (View::Not, Value::Bool(b)) => Value::Bool(!b),
        (View::Toggle(t, f), Value::Bool(b)) => json(if b { t } else { f }),
        (View::Options(opts), v) => match v.as_str().and_then(|s| opts.iter().find(|(n, _)| *n == s)) {
            Some((_, j)) => json(j),
            None => {
                return Err(PropertyError::NoSuchOption {
                    name: name.to_owned(),
                    options: opts.iter().map(|(n, _)| format!("\"{n}\"")).collect::<Vec<_>>().join(", "),
                });
            }
        },
        (_, v) => v,
    })
}

fn json(s: &str) -> Value {
    serde_json::from_str(s).expect("o json de uma View é literal do catálogo")
}

/// O lugar de uma chave dentro do valor do componente: o valor inteiro num newtype ([`WHOLE`]), um
/// ELEMENTO de um array com `[i]` (`"axis[0]"`; `"[2]"` num newtype de array) — a linha «Axis X» do
/// cartão é um elemento, não o par. A lei única de quem lê ou repõe um campo pela chave.
pub fn slot_in<'v>(whole: &'v mut Value, key: &str) -> Option<&'v mut Value> {
    if key == WHOLE {
        return Some(whole);
    }
    let Some((base, rest)) = key.split_once('[') else {
        return whole.get_mut(key);
    };
    let i: usize = rest.strip_suffix(']')?.parse().ok()?;
    let arr = if base.is_empty() {
        whole
    } else {
        whole.get_mut(base)?
    };
    arr.get_mut(i)
}

/// O lugar do campo dentro do valor do componente ([`slot_in`]).
fn slot<'v>(r: &Resolved<'_>, whole: &'v mut Value) -> Result<&'v mut Value, PropertyError> {
    slot_in(whole, r.field.serde_key()).ok_or_else(|| PropertyError::UnknownField {
        component: code_of(r).to_owned(),
        field: r.field.name.to_owned(),
    })
}

/// Escreve `value` em `name` no objecto; os outros campos do componente ficam. `Ok(true)` = a
/// escrita CRIOU o componente (uma linha que estava no neutro, [`ComponentDesc::row_of`]).
pub fn set(
    reg: &ComponentRegistry,
    world: &mut World,
    entity: Entity,
    name: &str,
    value: Value,
) -> Result<bool, PropertyError> {
    let r = resolve(reg, name)?;
    let mut whole = component_value(reg, &r, world, entity)?;
    let born = matches!((r.entry.value_of)(world, entity), Ok(None));
    let s = slot(&r, &mut whole)?;
    *s = normalize(r.field.kind, stored_from(r.field.view, value, s, name)?);
    let wrong = || PropertyError::WrongType {
        name: name.to_owned(),
        kind: r.field.kind,
    };
    if born {
        let insert = r.entry.insert_default.ok_or_else(wrong)?;
        insert(world, entity).map_err(|_| wrong())?;
    }
    let res = (r.entry.set_value)(world, entity, whole).map_err(|_| wrong());
    if res.is_err() && born {
        (r.entry.remove)(world, entity);
    }
    res.map(|()| born)
}

/// Um número inteiro chega do Luau como real (`3.0`); o serde de um `i32`/`u64` não o aceita.
fn normalize(kind: FieldKind, value: Value) -> Value {
    match (kind, &value) {
        (FieldKind::Int | FieldKind::Seed, Value::Number(n)) => match n.as_f64() {
            Some(x) if x.fract() == 0.0 && x >= 0.0 && x <= u64::MAX as f64 => {
                Value::from(x as u64)
            }
            Some(x) if x.fract() == 0.0 && x >= i64::MIN as f64 => Value::from(x as i64),
            _ => value,
        },
        _ => value,
    }
}

fn code_of(r: &Resolved<'_>) -> &'static str {
    r.entry.desc.map_or("", |d| d.code)
}

fn component_value(
    reg: &ComponentRegistry,
    r: &Resolved<'_>,
    world: &World,
    entity: Entity,
) -> Result<Value, PropertyError> {
    match (r.entry.value_of)(world, entity) {
        Ok(Some(v)) => Ok(v),
        Ok(None) => neutral_row(reg, r, world, entity).ok_or_else(|| PropertyError::Missing {
            component: code_of(r).to_owned(),
        }),
        Err(_) => Err(PropertyError::NoEntity),
    }
}

/// O valor neutro de uma LINHA que falta onde o dono dela está: o que o cartão mostra.
fn neutral_row(reg: &ComponentRegistry, r: &Resolved<'_>, world: &World, entity: Entity) -> Option<Value> {
    let host = r.entry.desc.map(|d| d.row_of).filter(|h| !h.is_empty())?;
    let host = reg.get_by_name(host)?;
    (host.value_of)(world, entity).ok()??;
    let insert = r.entry.insert_default?;
    let mut scratch = World::new();
    let e = scratch.spawn_empty().id();
    insert(&mut scratch, e).ok()?;
    (r.entry.value_of)(&scratch, e).ok()?
}

/// Como [`set`], sem escrever: `Ok` = a escrita pousaria.
pub fn check(
    reg: &ComponentRegistry,
    world: &World,
    entity: Entity,
    name: &str,
    value: Value,
) -> Result<(), PropertyError> {
    let r = resolve(reg, name)?;
    let mut whole = component_value(reg, &r, world, entity)?;
    if let Ok(s) = slot(&r, &mut whole) {
        *s = normalize(r.field.kind, stored_from(r.field.view, value, s, name)?);
    }
    (r.entry.check_value)(&whole).map_err(|_| PropertyError::WrongType {
        name: name.to_owned(),
        kind: r.field.kind,
    })
}
