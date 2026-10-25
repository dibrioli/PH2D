//! **O `self` de cada objecto** — nascer com o `id` e receber os números do Inspector (§3.5 do plano
//! 13). Saiu do `scene.rs` por tecto de LOC (CodeEditor item 6).

use std::collections::BTreeMap;

use mlua::{Lua, Value};

use super::{Actor, Instance, library_id};
use crate::component::LuauScript;
use crate::props::{PropDecl, ScriptValue, resolve};

/// O valor como o script o lê em `self`.
///
/// ⚠️⚠️ **A tabela de um `vec2`/`color` sai da MESMA porta que o construtor `ph2d.vec2` usa**
/// ([`crate::valores::tabela_de`]): o que o artista escreve na declaração e o que ele lê em `self`
/// têm de ser a mesma forma, senão copiar uma para a outra produz um default que o painel não sabe
/// pintar. Há gate de ida-e-volta.
/// Um objecto lê-se como a entidade com esse nome (`objects`), `nil` se nenhuma.
fn to_lua(lua: &Lua, v: &ScriptValue, objects: &mut Objects<'_>) -> mlua::Result<Value> {
    if let ScriptValue::Object(nome) = v {
        return Ok(objects(nome).map_or(Value::Nil, Value::Number));
    }
    if let Some(t) = crate::valores::tabela_de(lua, v)? {
        return Ok(t);
    }
    Ok(match v {
        ScriptValue::Number(n) => Value::Number(*n),
        ScriptValue::Bool(b) => Value::Boolean(*b),
        ScriptValue::Text(s) => Value::String(lua.create_string(s)?),
        // A [`crate::valores::tabela_de`] já os devolveu acima.
        ScriptValue::Vec2(_) | ScriptValue::Color(_) | ScriptValue::Object(_) => {
            unreachable!("a porta ja' os cobriu")
        }
    })
}

/// O `self` de um objecto acabado de nascer: só o `id`. Os números entram pelo [`apply_props`].
pub(super) fn born(lua: &Lua, a: &Actor, bits: u64, chunk: String) -> mlua::Result<Instance> {
    let this = lua.create_table()?;
    #[expect(
        clippy::cast_precision_loss,
        reason = "os bits de uma entidade cabem em 2^53 enquanto a geração for < 2^21 — e \
                  valem só dentro da sessão"
    )]
    this.set("id", bits as f64)?;
    Ok(Instance {
        script: library_id(&a.cfg).unwrap_or_default(),
        chunk,
        this,
        applied: BTreeMap::new(),
        failed: None,
        gone: false,
        connections: Vec::new(),
        sid: a.sid,
    })
}

/// Escreve no `self` os números que MUDARAM desde a última vez (§3.5): a mão do artista manda,
/// mas só onde mexeu.
/// A entidade (`id` do Luau) de um objecto pelo nome.
pub(super) type Objects<'a> = dyn FnMut(&str) -> Option<f64> + 'a;

pub(super) fn apply_props(
    lua: &Lua,
    inst: &mut Instance,
    decls: &[PropDecl],
    cfg: &LuauScript,
    objects: &mut Objects<'_>,
) {
    let r = resolve(Some(decls), &cfg.own);
    for p in &r.values {
        if inst.applied.get(&p.name) == Some(&p.value) {
            continue;
        }
        // Um objecto que ainda não existe fica por aplicar: tenta-se no quadro seguinte.
        if let Ok(v) = to_lua(lua, &p.value, objects)
            && !(v.is_nil() && matches!(p.value, ScriptValue::Object(_)))
            && inst.this.raw_set(p.name.as_str(), v).is_ok()
        {
            inst.applied.insert(p.name.clone(), p.value.clone());
        }
    }
    // Uma declaração que SAIU do script leva o campo com ela.
    let gone: Vec<String> = inst
        .applied
        .keys()
        .filter(|k| !r.values.iter().any(|p| p.name == **k))
        .cloned()
        .collect();
    for k in gone {
        let _ = inst.this.raw_set(k.as_str(), Value::Nil);
        inst.applied.remove(&k);
    }
}
