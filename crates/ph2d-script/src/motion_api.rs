//! ⭐ **A tabela `motion` do Luau** (ciclo 19, doc 128 §3.6) — o script escreve as entradas que o
//! grafo do Motion lê (`value.input`):
//!
//! ```lua
//! motion.set("vida", 0.3)          -- um número (ou true/false)
//! motion.color("time", 1, 0.2, 0)  -- uma cor, RGBA linear (alpha 1 se faltar)
//! motion.fire("dano")              -- um disparo de gatilho
//! ```
//!
//! As escritas seguem a lei das do `ph2d.set`: ficam numa fila da VM, drenada no fim de CADA
//! gancho — um gancho que falha perde as dele — e aplicadas no fim do tique ao recurso
//! [`ph2d_ecs::MotionInputs`], antes de o Motion cozer no mesmo quadro.

use mlua::{Lua, Value};
use ph2d_ecs::MotionInputs;
use ph2d_ecs::motion_inputs::{Escritor, Valor};
use std::sync::{Arc, Mutex};

/// Uma escrita do script na tabela `motion`.
#[derive(Clone, Debug, PartialEq)]
pub enum MotionWrite {
    Set(String, f32),
    Color(String, [f32; 4]),
    Fire(String),
}

#[derive(Clone, Default)]
struct MotionQueue(Arc<Mutex<Vec<MotionWrite>>>);

fn push(lua: &Lua, w: MotionWrite) -> mlua::Result<()> {
    let q = lua
        .app_data_ref::<MotionQueue>()
        .ok_or_else(|| mlua::Error::RuntimeError("motion: queue not registered".into()))?;
    q.0.lock().unwrap_or_else(|p| p.into_inner()).push(w);
    Ok(())
}

/// ⚠️ Vazio é erro, não silêncio (a lei do `ph2d.emit`): um nome vazio é uma variável por
/// preencher, e o painel tem de o dizer.
fn nome(f: &str, n: String) -> mlua::Result<String> {
    let n = n.trim().to_string();
    if n.is_empty() {
        return Err(mlua::Error::RuntimeError(format!(
            "motion.{f}: the input needs a name"
        )));
    }
    Ok(n)
}

fn numero(f: &str, v: f64) -> mlua::Result<f32> {
    if !v.is_finite() {
        return Err(mlua::Error::RuntimeError(format!(
            "motion.{f}: got {v} — not a number"
        )));
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "o grafo do Motion é f32; o número do Luau é f64"
    )]
    Ok(v as f32)
}

/// Instala a tabela global `motion` (ANTES do sandbox, que congela os globais). A fila nasce com a
/// VM: entre ganchos ela está sempre vazia, então reconstruir a VM não perde nada.
pub(crate) fn wire(lua: &Lua) -> mlua::Result<()> {
    lua.set_app_data(MotionQueue::default());
    let set = lua.create_function(|lua, (n, v): (String, Value)| -> mlua::Result<()> {
        let n = nome("set", n)?;
        let x = match v {
            Value::Boolean(b) => f32::from(u8::from(b)),
            Value::Integer(i) => numero("set", i as f64)?,
            Value::Number(x) => numero("set", x)?,
            other => {
                return Err(mlua::Error::RuntimeError(format!(
                    "motion.set: '{n}' got a {} — a number or true/false",
                    other.type_name()
                )));
            }
        };
        push(lua, MotionWrite::Set(n, x))
    })?;
    let color = lua.create_function(
        |lua, (n, r, g, b, a): (String, f64, f64, f64, Option<f64>)| -> mlua::Result<()> {
            let n = nome("color", n)?;
            let c = [
                numero("color", r)?,
                numero("color", g)?,
                numero("color", b)?,
                numero("color", a.unwrap_or(1.0))?,
            ];
            push(lua, MotionWrite::Color(n, c))
        },
    )?;
    let fire = lua.create_function(|lua, n: String| -> mlua::Result<()> {
        let n = nome("fire", n)?;
        push(lua, MotionWrite::Fire(n))
    })?;
    let t = lua.create_table()?;
    t.set("set", set)?;
    t.set("color", color)?;
    t.set("fire", fire)?;
    lua.globals().set("motion", t)
}

/// As escritas do gancho que acabou.
pub(crate) fn drain(lua: &Lua) -> Vec<MotionWrite> {
    lua.app_data_ref::<MotionQueue>()
        .map(|q| std::mem::take(&mut *q.0.lock().unwrap_or_else(|p| p.into_inner())))
        .unwrap_or_default()
}

/// Aplica as escritas do tique ao recurso (criado na primeira).
pub(crate) fn apply(world: &mut ph2d_ecs::World, writes: &[MotionWrite]) {
    if writes.is_empty() {
        return;
    }
    let mut m = world.get_resource_or_insert_with(MotionInputs::default);
    for w in writes {
        match w {
            MotionWrite::Set(n, x) => m.set(n, Escritor::Script, Valor::Numero(*x)),
            MotionWrite::Color(n, c) => m.set(n, Escritor::Script, Valor::Cor(*c)),
            MotionWrite::Fire(n) => m.fire(n, Escritor::Script),
        }
    }
}
