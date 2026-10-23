//! ⭐ **`ph2d.get/set` de uma propriedade nomeada** (`"health.max"`, doc CodeEditor 08 §3) — o valor
//! do Luau ↔ o `serde_json::Value` da lei `ph2d_ecs::scene::property`, que a porta da cena aplica.
//!
//! Posição e cor chegam e saem nas formas que o script já conhece (`ph2d.vec2`/`ph2d.color`, o
//! [`crate::valores`]); o resto é o valor tal como o componente o guarda.

use mlua::{Lua, Value};
use ph2d_component_desc::FieldKind;
use serde_json::Value as Json;

use crate::props::ScriptValue;
use crate::valores::{tabela_de, tabela_tipada};

/// O valor de um campo, visto do Luau.
pub(crate) fn to_lua(lua: &Lua, v: &Json, kind: FieldKind) -> mlua::Result<Value> {
    let par = |a: &[Json]| -> Option<Vec<f64>> { a.iter().map(Json::as_f64).collect() };
    match (kind, v) {
        (FieldKind::Vec2, Json::Array(a)) if a.len() == 2 => {
            if let Some(n) = par(a) {
                return Ok(tabela_de(lua, &ScriptValue::Vec2([n[0], n[1]]))?.unwrap_or(Value::Nil));
            }
        }
        // Uma cor sem opacidade (a da Luz, `[f32; 3]`) lê-se opaca.
        (FieldKind::Color, Json::Array(a)) if a.len() == 3 || a.len() == 4 => {
            if let Some(n) = par(a) {
                let c = ScriptValue::Color([n[0], n[1], n[2], n.get(3).copied().unwrap_or(1.0)]);
                return Ok(tabela_de(lua, &c)?.unwrap_or(Value::Nil));
            }
        }
        // Um número sem limite (`View::Unbounded`) é `null` no valor serde e `math.huge` no Luau.
        (FieldKind::Scalar, Json::Null) => return Ok(Value::Number(f64::INFINITY)),
        _ => {}
    }
    plain(lua, v)
}

fn plain(lua: &Lua, v: &Json) -> mlua::Result<Value> {
    Ok(match v {
        Json::Null => Value::Nil,
        Json::Bool(b) => Value::Boolean(*b),
        Json::Number(n) => Value::Number(n.as_f64().unwrap_or(f64::NAN)),
        Json::String(s) => Value::String(lua.create_string(s)?),
        Json::Array(a) => {
            let t = lua.create_table()?;
            for x in a {
                t.raw_push(plain(lua, x)?)?;
            }
            Value::Table(t)
        }
        Json::Object(o) => {
            let t = lua.create_table()?;
            for (k, x) in o {
                t.raw_set(k.as_str(), plain(lua, x)?)?;
            }
            Value::Table(t)
        }
    })
}

/// O valor que o script deu, para o campo. `Err` = um valor que nenhum campo guarda.
pub(crate) fn from_lua(v: &Value) -> Result<Json, String> {
    Ok(match v {
        Value::Nil => Json::Null,
        Value::Boolean(b) => Json::Bool(*b),
        Value::Integer(i) => Json::from(*i),
        Value::Number(n) if *n == f64::INFINITY => Json::Null,
        Value::Number(n) => serde_json::Number::from_f64(*n)
            .map(Json::Number)
            .ok_or_else(|| format!("{n} is not a number"))?,
        Value::String(s) => Json::String(s.to_str().map_err(|e| e.to_string())?.to_owned()),
        Value::Table(t) => match tabela_tipada(t) {
            Some(ScriptValue::Vec2([x, y])) => serde_json::json!([x, y]),
            Some(ScriptValue::Color(c)) => serde_json::json!(c),
            _ if t.raw_len() > 0 => Json::Array(
                t.sequence_values::<Value>()
                    .map(|x| x.map_err(|e| e.to_string()).and_then(|x| from_lua(&x)))
                    .collect::<Result<_, _>>()?,
            ),
            _ => {
                let mut o = serde_json::Map::new();
                for pair in t.pairs::<String, Value>() {
                    let (k, x) = pair.map_err(|e| e.to_string())?;
                    o.insert(k, from_lua(&x)?);
                }
                Json::Object(o)
            }
        },
        other => return Err(format!("a {} is not a property value", other.type_name())),
    })
}

/// Pousa as escritas de propriedade validadas, no fim do tique e pela ordem da fila (HR-8). Uma que
/// já não pousa (o objecto morreu no mesmo tique) cai como a da pose cai.
pub(crate) fn apply(
    sim: &mut ph2d_ecs::SimWorld,
    doors: &mut dyn crate::lane::SceneDoors,
    writes: &[crate::io::EntityWrite],
    report: &mut crate::scene::SceneReport,
) {
    for w in writes {
        let Some(e) = ph2d_ecs::Entity::try_from_bits(w.entity) else {
            continue;
        };
        match &w.value {
            crate::io::WriteValue::Property(v) => {
                if doors.property_set(sim, e, &w.field, v, false).is_ok() {
                    report.wrote.push(e);
                }
            }
            crate::io::WriteValue::Health(r) => {
                let _ = doors.health_request(sim, e, *r, false);
            }
            crate::io::WriteValue::Pose(_) => {}
        }
    }
}
