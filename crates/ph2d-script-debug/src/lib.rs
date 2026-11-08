//! **O depurador dos scripts** — a porta segura para os ganchos de depuração do Luau (plano 13,
//! Onda D: `docs/CodeEditor/13_plano_bancada.md`).
//!
//! O Luau não tem o `lua_sethook` do Lua 5: tem o ponto de parada que REESCREVE a instrução da
//! linha (`LOP_BREAK`, custo zero nas outras linhas) e o passo-a-passo por fio, que troca o
//! interpretador desse fio por um mais lento enquanto armado. A `interrupt` é da casa (o prazo de
//! cada chamada, `ph2d-script`); estes ganchos são os outros campos de `lua_Callbacks`.
//!
//! Hoje só a porta e a sonda que mede o custo (`tests/sonda_gancho_de_linha.rs`); o depurador
//! constrói-se depois da proposta ao dono.

mod ffi;

pub use ffi::{RawHook, breakpoint, set_hooks, single_step};
pub use mlua::ffi::{lua_Debug, lua_State};
