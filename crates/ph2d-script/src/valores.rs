//! ⭐⭐⭐ **A FORMA de um `vec2` e de uma `color` do lado do Luau — UMA, com dois chamadores.**
//!
//! # ⚠️⚠️ Porque a forma é UMA
//!
//! O artista escreve a declaração com um construtor e lê o valor em `self`:
//!
//! ```luau
//! ph2d.property("offset", ph2d.vec2(0, 1))
//! -- …
//! function update(dt) self.x = self.x + self.offset.x * dt end
//! ```
//!
//! ⛔ **Se a declaração fosse uma LISTA (`{0, 1}`) e o `self` um REGISTO (`{x=…, y=…}`), o mesmo
//! valor teria duas formas** — e o artista que copiasse uma para a outra escreveria um default que
//! o painel não sabe pintar. *Duas formas para a mesma coisa é a mesma família de duas respostas
//! para a mesma pergunta.* ⇒ [`tabela_de`] tem **dois** chamadores (o construtor e o `to_lua` da
//! cena) e [`tabela_tipada`] lê exactamente o que ela escreve, com gate de ida-e-volta.
//!
//! # ⚠️ A desambiguação é EXPLÍCITA, nunca pelo comprimento
//!
//! Uma tabela só é um default se ela se **declarar** (`__kind`). ⛔ Adivinhar pelo tamanho — *três
//! números são uma cor, dois são uma posição* — é um palpite, e um `{1, 0, 0}` escrito à mão pelo
//! artista lê-se igual a uma cor vermelha e a uma posição com lixo no fim.
//!
//! # ⛔ O construtor NÃO valida, e é deliberado
//!
//! Quem recusa uma declaração é o [`crate::props::check_decl`] — a porta única, cuja queixa já
//! chega ao painel pelo caminho do `DeclError`. Uma segunda conferência dentro do construtor seria
//! a segunda resposta a *«esta cor é legal?»*, e as duas divergiriam no dia em que uma mudasse.

use mlua::{Function, Lua, Table, Value};

use crate::props::ScriptValue;

/// A chave que diz de que tipo a tabela é.
pub(crate) const MARCA: &str = "__kind";
/// O valor da [`MARCA`] numa posição.
pub(crate) const VEC2: &str = "vec2";
/// O valor da [`MARCA`] numa cor.
pub(crate) const COR: &str = "color";

/// **A tabela que o Luau vê** — `None` para os valores que não são tabela (número, sim/não, texto).
///
/// # Errors
/// O que o `mlua` devolver ao construir a tabela.
pub(crate) fn tabela_de(lua: &Lua, v: &ScriptValue) -> mlua::Result<Option<Value>> {
    let t = match v {
        ScriptValue::Vec2([x, y]) => {
            let t = lua.create_table()?;
            t.set(MARCA, VEC2)?;
            t.set("x", *x)?;
            t.set("y", *y)?;
            t
        }
        ScriptValue::Color([r, g, b, a]) => {
            let t = lua.create_table()?;
            t.set(MARCA, COR)?;
            t.set("r", *r)?;
            t.set("g", *g)?;
            t.set("b", *b)?;
            t.set("a", *a)?;
            t
        }
        ScriptValue::Number(_) | ScriptValue::Bool(_) | ScriptValue::Text(_) => return Ok(None),
    };
    Ok(Some(Value::Table(t)))
}

/// **O valor que uma tabela declara ser** — `None` se ela não se declarar ou se faltar um campo.
pub(crate) fn tabela_tipada(t: &Table) -> Option<ScriptValue> {
    let marca: String = t.get(MARCA).ok()?;
    match marca.as_str() {
        VEC2 => Some(ScriptValue::Vec2([numero(t, "x")?, numero(t, "y")?])),
        COR => Some(ScriptValue::Color([
            numero(t, "r")?,
            numero(t, "g")?,
            numero(t, "b")?,
            numero(t, "a")?,
        ])),
        _ => None,
    }
}

/// Um campo numérico da tabela.
///
/// ⚠️ **Ele lê o `Value` e faz o `match` à mão**, e não um `get::<f64>`: o `FromLua` do `f64`
/// **COAGE uma string numérica**, logo `{x = "1"}` passaria — e é o mesmo idioma que o
/// `read_decl` já usa para o `min`/`max`/`step`.
fn numero(t: &Table, chave: &str) -> Option<f64> {
    match t.get::<Value>(chave).ok()? {
        Value::Number(n) => Some(n),
        #[expect(clippy::cast_precision_loss, reason = "o Luau guarda números como f64")]
        Value::Integer(i) => Some(i as f64),
        _ => None,
    }
}

/// ⭐ **`ph2d.vec2(x, y)`** — uma posição.
pub(crate) fn vec2_binding(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, (x, y): (f64, f64)| {
        // O `unwrap_or` nunca arma: a [`tabela_de`] só devolve `None` para os três tipos simples.
        Ok(tabela_de(lua, &ScriptValue::Vec2([x, y]))?.unwrap_or(Value::Nil))
    })
}

/// ⭐ **`ph2d.color(r, g, b, a?)`** — uma cor; sem o quarto argumento ela é opaca.
pub(crate) fn color_binding(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|lua, (r, g, b, a): (f64, f64, f64, Option<f64>)| {
        let v = ScriptValue::Color([r, g, b, a.unwrap_or(1.0)]);
        Ok(tabela_de(lua, &v)?.unwrap_or(Value::Nil))
    })
}

#[cfg(test)]
#[path = "valores_tests.rs"]
mod tests;
