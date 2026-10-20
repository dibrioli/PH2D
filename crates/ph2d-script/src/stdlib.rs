//! ⭐ **As curas da biblioteca padrão do Luau** (CodeEditor item 6 (g),
//! `docs/CodeEditor/05_api_que_funciona.md` §2 e §3.7) — instaladas antes da sandbox.
//!
//! 1. **A família do inteiro.** O Luau 736 converte índices e contagens por *cast*
//!    (`luai_num2int(i, d) ((i) = (int)(d))`, `lnumutils.h`): fora de ±2³¹ o x86 dá `INT_MIN` e a
//!    resposta MENTE (`string.sub('abcdef', 2, 1e10) == ""`, `#string.rep('x', 1e10) == 0`); o ARM
//!    satura e acerta. As funções abaixo saturam o argumento antes do Luau o ver — a resposta do ARM
//!    em toda máquina; o lugar de um `table.insert` não (saturá-lo seria outra mentira): é erro. As que já respondem com erro honesto ficam como estão (censo:
//!    `scene_api_tests::a_familia_do_inteiro_satura`). ⚠️ O portão corre cada caso como SCRIPT: no
//!    ambiente global o compilador usa a versão interna da função (`FASTCALL`) e salta a cura; no
//!    ambiente de um script, que é onde o código do artista corre, não.
//! 2. **O `loadstring` de cada script** compila no ambiente DESSE script (medido 20/10: o do Luau
//!    escrevia num ambiente partilhado — dois scripts trocavam globais por ali — e não via os do
//!    próprio script). Um erro de sintaxe devolve `nil, mensagem`, a lei do Lua 5.1. O `_G` não
//!    mudou: é a tabela partilhada e TRANCADA dos nomes de base (escrever nela é erro).

use mlua::{Lua, Table, Value};

const SATURATE_LUAU: &str = r##"
local MAXI, MINI = 2147483647, -2147483648
local function sat(v)
  if type(v) ~= "number" then return v end
  if v ~= v then error("bad argument: number has no integer representation (nan)", 3) end
  if v > MAXI then return MAXI end
  if v < MINI then return MINI end
  return v
end
local function rot(v)
  if type(v) ~= "number" or v ~= v then return sat(v) end
  return v % 32
end
local s, t, u, m, b = string, table, utf8, math, bit32
local sub, byte, rep, find, match, gsub, unpack_ = s.sub, s.byte, s.rep, s.find, s.match, s.gsub, s.unpack
s.sub = function(x, i, j) return sub(x, sat(i), sat(j)) end
s.byte = function(x, i, j) return byte(x, sat(i), sat(j)) end
s.rep = function(x, n) return rep(x, sat(n)) end
s.find = function(x, p, init, plain) return find(x, p, sat(init), plain) end
s.match = function(x, p, init) return match(x, p, sat(init)) end
s.gsub = function(x, p, r, n) return gsub(x, p, r, sat(n)) end
s.unpack = function(f, x, pos) return unpack_(f, x, sat(pos)) end
local tunpack, concat, move, remove = t.unpack, t.concat, t.move, t.remove
t.unpack = function(x, i, j) return tunpack(x, sat(i), sat(j)) end
t.concat = function(x, sep, i, j) return concat(x, sep, sat(i), sat(j)) end
t.move = function(a1, f, e, to, a2) return move(a1, sat(f), sat(e), sat(to), a2) end
t.remove = function(x, pos) if pos == nil then return remove(x) end return remove(x, sat(pos)) end
-- O lugar de um insert não satura: pôr em 2^31-1 o que foi pedido em 2^32+1 seria outra mentira.
local insert = t.insert
t.insert = function(x, ...)
  if select("#", ...) >= 2 then
    local pos = ...
    if type(pos) == "number" and (pos ~= pos or pos > MAXI or pos < MINI) then
      error("invalid argument #2 to 'insert' (position out of bounds)", 2)
    end
  end
  return insert(x, ...)
end
local ulen, codepoint, offset = u.len, u.codepoint, u.offset
u.len = function(x, i, j) return ulen(x, sat(i), sat(j)) end
u.codepoint = function(x, i, j) return codepoint(x, sat(i), sat(j)) end
u.offset = function(x, n, i) return offset(x, sat(n), sat(i)) end
local ldexp = m.ldexp
m.ldexp = function(x, e) return ldexp(x, sat(e)) end
local lrotate, rrotate = b.lrotate, b.rrotate
b.lrotate = function(x, n) return lrotate(x, rot(n)) end
b.rrotate = function(x, n) return rrotate(x, rot(n)) end
"##;

/// A chave no registo da VM da função que liga o `loadstring` a um ambiente.
const BIND_KEY: &str = "ph2d.loadstring_bind";

/// O `loadstring` de um ambiente — Luau, para não haver um ciclo Rust↔tabela que o GC não vê.
const BIND_LUAU: &str = r#"
local load_in = ...
return function(env)
  rawset(env, "loadstring", function(source, name)
    return load_in(source, name, env)
  end)
end
"#;

/// Instala as curas (antes da sandbox).
pub(crate) fn wire(lua: &Lua) -> mlua::Result<()> {
    lua.load(SATURATE_LUAU).set_name("=ph2d").exec()?;
    let load_in = lua.create_function(
        |lua, (source, name, env): (mlua::LuaString, Option<String>, Table)| {
            // O nome por omissão é o próprio texto (`[string "…"]`), como no Lua e no Luau.
            let name = name.unwrap_or_else(|| source.to_string_lossy());
            let chunk = lua
                .load(source.as_bytes().to_vec())
                .set_name(name)
                .set_environment(env)
                .into_function();
            Ok(match chunk {
                Ok(f) => (Value::Function(f), None),
                Err(e) => (Value::Nil, Some(e.to_string())),
            })
        },
    )?;
    let bind: mlua::Function = lua.load(BIND_LUAU).set_name("=ph2d").call(load_in)?;
    lua.set_named_registry_value(BIND_KEY, bind)
}

/// Dá ao ambiente `env` de um script o seu `loadstring` (sem a VM do host, nada a fazer).
pub(crate) fn bind_env(lua: &Lua, env: &Table) -> mlua::Result<()> {
    match lua.named_registry_value::<Option<mlua::Function>>(BIND_KEY)? {
        Some(bind) => bind.call(env.clone()),
        None => Ok(()),
    }
}
