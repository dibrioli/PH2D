//! A ABI C dos ganchos de depuração do Luau, atrás de funções seguras.
#![allow(unsafe_code)]

use mlua::{Function, Lua, ffi};

/// Um gancho do Luau (`debugbreak` / `debugstep`): corre no fio do script, com a pilha dele.
pub type RawHook = unsafe extern "C-unwind" fn(*mut ffi::lua_State, *mut ffi::lua_Debug);

/// Arma (ou desarma) um ponto de parada na `line` de `f` (e das funções dentro dela). Devolve a
/// linha onde ficou — a próxima com instruções —, ou `None` se não há nenhuma.
///
/// # Errors
/// O da VM ao empurrar `f`.
pub fn breakpoint(lua: &Lua, f: &Function, line: i32, on: bool) -> mlua::Result<Option<i32>> {
    let mut got = -1;
    // SAFETY: `exec_raw` empurra `f` (índice -1) e guarda a pilha; `lua_breakpoint` só lê esse
    // valor, que é uma função Luau (o `Function` do mlua nunca é uma função C aqui: vem de um
    // módulo carregado).
    unsafe {
        lua.exec_raw::<()>(f.clone(), |l| {
            got = ffi::lua_breakpoint(l, -1, line, i32::from(on));
        })?;
    }
    Ok((got >= 0).then_some(got))
}

/// Liga o passo-a-passo no fio em curso; as corrotinas criadas depois herdam-no (`lua_newthread`).
///
/// # Errors
/// O da VM.
pub fn single_step(lua: &Lua, on: bool) -> mlua::Result<()> {
    // SAFETY: só escreve o bit `singlestep` do estado em curso.
    unsafe { lua.exec_raw::<()>((), |l| ffi::lua_singlestep(l, i32::from(on))) }
}

/// Põe os ganchos de paragem e de passo (`None` = nenhum). Globais à VM.
///
/// # Errors
/// O da VM.
pub fn set_hooks(
    lua: &Lua,
    on_break: Option<RawHook>,
    on_step: Option<RawHook>,
) -> mlua::Result<()> {
    // SAFETY: `lua_callbacks` devolve a tabela de ganchos da VM, válida enquanto ela vive; só os
    // dois campos de depuração mudam (a `interrupt` da casa fica).
    unsafe {
        lua.exec_raw::<()>((), |l| {
            let cb = ffi::lua_callbacks(l);
            (*cb).debugbreak = on_break;
            (*cb).debugstep = on_step;
        })
    }
}
