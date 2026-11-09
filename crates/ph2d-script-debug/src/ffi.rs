//! A ABI C dos ganchos de depuração do Luau, atrás de funções seguras.
//!
//! ⚠️ Um gancho corre DENTRO da VM: não chama Lua (nada de `__tostring`), não entra em pânico, e
//! só cede (`lua_yield`) o fio de gancho da cena que está a correr e quando o Luau o permite.
#![allow(unsafe_code)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ffi::{CStr, c_char};
use std::rc::{Rc, Weak};

use mlua::{Function, Lua, Thread, ffi};

use crate::{LiveFrame, State, StopFrame, StopKind, Var};

/// Um gancho do Luau (`debugbreak` / `debugstep`): corre no fio do script, com a pilha dele.
pub type RawHook = unsafe extern "C-unwind" fn(*mut ffi::lua_State, *mut ffi::lua_Debug);

const MAX_TEXT: usize = 32;
const MAX_THIS: usize = 24;
const MAX_STACK: usize = 8;

thread_local! {
    /// Os depuradores vivos, pela VM (o `lua_mainthread`).
    static VMS: RefCell<BTreeMap<usize, Weak<RefCell<State>>>> = const { RefCell::new(BTreeMap::new()) };
}

pub(crate) fn register(key: usize, state: &Rc<RefCell<State>>) {
    VMS.with(|m| m.borrow_mut().insert(key, Rc::downgrade(state)));
}

pub(crate) fn unregister(key: usize) {
    VMS.with(|m| m.borrow_mut().remove(&key));
}

fn lookup(l: *mut ffi::lua_State) -> Option<Rc<RefCell<State>>> {
    // SAFETY: `l` é um fio vivo da VM (o Luau chama o gancho com ele).
    let key = unsafe { ffi::lua_mainthread(l) } as usize;
    VMS.with(|m| m.try_borrow().ok()?.get(&key)?.upgrade())
}

/// A chave da VM de `lua`.
pub(crate) fn vm_key(lua: &Lua) -> mlua::Result<usize> {
    let mut key = 0;
    // SAFETY: só lê o fio principal do estado em curso.
    unsafe { lua.exec_raw::<()>((), |l| key = ffi::lua_mainthread(l) as usize)? };
    Ok(key)
}

/// Arma (ou desarma) um ponto de parada na `line` de `f` (e das funções dentro dela). Devolve a
/// linha onde ficou — a próxima com instruções —, ou `None` se não há nenhuma.
///
/// # Errors
/// O da VM ao empurrar `f`.
pub fn breakpoint(lua: &Lua, f: &Function, line: i32, on: bool) -> mlua::Result<Option<i32>> {
    let mut got = -1;
    // SAFETY: `exec_raw` empurra `f` (índice -1) e guarda a pilha; `lua_breakpoint` só lê esse
    // valor, que é uma função Luau (vem de um módulo carregado, nunca é uma função C).
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

/// O fio cru de `th`, validado contra a VM de `lua`.
fn raw_thread(lua: &Lua, th: &Thread) -> mlua::Result<*mut ffi::lua_State> {
    let mut out = std::ptr::null_mut();
    // SAFETY: `exec_raw` empurra o fio; `lua_tothread` devolve o estado dele (vivo enquanto `th`).
    unsafe { lua.exec_raw::<()>(th.clone(), |l| out = ffi::lua_tothread(l, -1))? };
    Ok(out)
}

/// O passo-a-passo de UM fio (que não está a correr).
pub(crate) fn thread_single_step(lua: &Lua, th: &Thread, on: bool) -> mlua::Result<()> {
    let t = raw_thread(lua, th)?;
    // SAFETY: `t` é o fio de `th`, suspenso ou por começar; só o bit dele muda.
    unsafe { ffi::lua_singlestep(t, i32::from(on)) };
    Ok(())
}

/// A linha e a profundidade da 1.ª função Luau de `th`.
pub(crate) fn where_is(lua: &Lua, th: &Thread) -> mlua::Result<(i32, i32)> {
    let t = raw_thread(lua, th)?;
    // SAFETY: `t` está suspenso (não corre) — ler a pilha dele não a muda.
    unsafe {
        let depth = ffi::lua_stackdepth(t);
        let Some((level, ar)) = first_lua_level(t, depth) else {
            return Ok((-1, depth));
        };
        Ok((ar.currentline, depth - level))
    }
}

/// O sítio e os valores de `th` (morto por um erro: o `savedpc` dele é o normal).
pub(crate) fn frame_of(lua: &Lua, th: &Thread) -> mlua::Result<Option<StopFrame>> {
    let t = raw_thread(lua, th)?;
    // SAFETY: `t` não corre.
    Ok(unsafe { frame_raw(t) })
}

/// O sítio e os valores de `t`, a partir da 1.ª função Luau da pilha.
///
/// # Safety
/// `t` é um fio vivo: parado, morto, ou o que corre um gancho (que tem `LUA_MINSTACK` livre).
unsafe fn frame_raw(t: *mut ffi::lua_State) -> Option<StopFrame> {
    // SAFETY: cada valor empurrado em `t` sai antes do próximo (pilha igual à entrada), e há
    // espaço (`lua_checkstack`).
    unsafe {
        if ffi::lua_checkstack(t, 4) == 0 {
            return None;
        }
        let depth = ffi::lua_stackdepth(t);
        let (level, ar) = first_lua_level(t, depth)?;
        let locals = locals_at(t, level);
        let mut this = Vec::new();
        let mut n = 1;
        loop {
            let name = ffi::lua_getlocal(t, level, n);
            if name.is_null() {
                break;
            }
            if CStr::from_ptr(name).to_bytes() == b"self" && ffi::lua_type(t, -1) == ffi::LUA_TTABLE
            {
                this = fields(t, -1);
            }
            ffi::lua_settop(t, -2);
            n += 1;
        }
        let mut stack = Vec::new();
        for lv in level..depth.min(level + i32::try_from(MAX_STACK).unwrap_or(8)) {
            let mut a = zeroed();
            if ffi::lua_getinfo(t, lv, c"sln".as_ptr(), &raw mut a) == 0 {
                break;
            }
            if is_c(&a) {
                continue;
            }
            stack.push((text_of(a.name), u32::try_from(a.currentline).unwrap_or(0)));
        }
        Some(StopFrame {
            line: u32::try_from(ar.currentline).unwrap_or(0),
            function: text_of(ar.name),
            locals,
            this,
            stack,
        })
    }
}

/// Fotografa o sítio do fio que o gancho vai ceder (a linha é a que o gancho recebeu).
unsafe fn snap(l: *mut ffi::lua_State, line: i32) -> Option<StopFrame> {
    // SAFETY: `l` corre o gancho.
    let mut f = unsafe { frame_raw(l) }?;
    let at = u32::try_from(line).unwrap_or(0);
    f.line = at;
    if let Some(top) = f.stack.first_mut() {
        top.1 = at;
    }
    Some(f)
}

/// O 1.º nível com uma função Luau (o erro de uma função C deixa-a no topo).
unsafe fn first_lua_level(t: *mut ffi::lua_State, depth: i32) -> Option<(i32, ffi::lua_Debug)> {
    for level in 0..depth {
        let mut ar = zeroed();
        // SAFETY: `level < depth`; `ar` é nosso.
        if unsafe { ffi::lua_getinfo(t, level, c"sln".as_ptr(), &raw mut ar) } == 0 {
            return None;
        }
        // SAFETY: `what` aponta para uma cadeia estática do Luau.
        if !unsafe { is_c(&ar) } {
            return Some((level, ar));
        }
    }
    None
}

fn zeroed() -> ffi::lua_Debug {
    // SAFETY: `lua_Debug` é POD (ponteiros nulos, inteiros, um buffer).
    unsafe { std::mem::zeroed() }
}

unsafe fn is_c(ar: &ffi::lua_Debug) -> bool {
    // SAFETY: `what` é nulo ou uma cadeia estática do Luau.
    !ar.what.is_null() && unsafe { CStr::from_ptr(ar.what) }.to_bytes() == b"C"
}

fn text_of(p: *const c_char) -> String {
    if p.is_null() {
        return String::new();
    }
    // SAFETY: cadeia terminada em nulo do Luau, viva durante esta leitura.
    unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned()
}

/// As locais visíveis no `level`, sem as temporárias do Luau (`(for index)`).
unsafe fn locals_at(t: *mut ffi::lua_State, level: i32) -> Vec<Var> {
    let mut out = Vec::new();
    let mut n = 1;
    loop {
        // SAFETY: cada `lua_getlocal` bem-sucedido empurra 1 valor, tirado a seguir.
        let name = unsafe { ffi::lua_getlocal(t, level, n) };
        if name.is_null() {
            break;
        }
        // SAFETY: o nome é uma cadeia do Luau; o valor está no topo.
        unsafe {
            let name = CStr::from_ptr(name).to_string_lossy();
            if !name.starts_with('(') {
                out.push(Var {
                    name: name.into_owned(),
                    value: value_text(t, -1),
                });
            }
            ffi::lua_settop(t, -2);
        }
        n += 1;
    }
    out
}

/// Os campos de texto da tabela em `idx` (um nível, sem metamétodos), por nome.
unsafe fn fields(t: *mut ffi::lua_State, idx: i32) -> Vec<Var> {
    let mut out = Vec::new();
    // SAFETY: `lua_next` cru sobre a tabela em `idx`; a chave e o valor saem a cada volta.
    unsafe {
        let table = ffi::lua_absindex(t, idx);
        ffi::lua_pushnil(t);
        while ffi::lua_next(t, table) != 0 {
            if ffi::lua_type(t, -2) == ffi::LUA_TSTRING && out.len() < MAX_THIS * 4 {
                let mut len = 0;
                let k = ffi::lua_tolstring(t, -2, &raw mut len);
                let k = String::from_utf8_lossy(std::slice::from_raw_parts(k.cast::<u8>(), len));
                out.push(Var {
                    name: k.into_owned(),
                    value: value_text(t, -1),
                });
            }
            ffi::lua_settop(t, -2);
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out.truncate(MAX_THIS);
    out
}

/// O valor em `idx` em texto curto, sem chamar Lua.
unsafe fn value_text(t: *mut ffi::lua_State, idx: i32) -> String {
    // SAFETY: `idx` é um índice válido de `t`; só leituras.
    unsafe {
        match ffi::lua_type(t, idx) {
            ffi::LUA_TNIL => "nil".into(),
            ffi::LUA_TBOOLEAN => (ffi::lua_toboolean(t, idx) != 0).to_string(),
            ffi::LUA_TNUMBER | ffi::LUA_TINTEGER => {
                number_text(ffi::lua_tonumberx(t, idx, std::ptr::null_mut()))
            }
            ffi::LUA_TVECTOR => {
                let v = ffi::lua_tovector(t, idx);
                if v.is_null() {
                    "vector".into()
                } else {
                    let v = std::slice::from_raw_parts(v, 3);
                    format!(
                        "({}, {}, {})",
                        number_text(f64::from(v[0])),
                        number_text(f64::from(v[1])),
                        number_text(f64::from(v[2]))
                    )
                }
            }
            ffi::LUA_TSTRING => {
                let mut len = 0;
                let p = ffi::lua_tolstring(t, idx, &raw mut len);
                let s = String::from_utf8_lossy(std::slice::from_raw_parts(p.cast::<u8>(), len));
                let mut out: String = s.chars().take(MAX_TEXT).collect();
                if s.chars().count() > MAX_TEXT {
                    out.push('…');
                }
                format!("\"{out}\"")
            }
            ffi::LUA_TTABLE => "table".into(),
            ffi::LUA_TFUNCTION => "function".into(),
            ffi::LUA_TTHREAD => "coroutine".into(),
            ffi::LUA_TBUFFER => "buffer".into(),
            _ => "userdata".into(),
        }
    }
}

/// `412` · `0.5` · `-3.142` — três casas no máximo, sem zeros à direita.
pub(crate) fn number_text(x: f64) -> String {
    if !x.is_finite() {
        return if x.is_nan() {
            "nan".into()
        } else if x > 0.0 {
            "inf".into()
        } else {
            "-inf".into()
        };
    }
    let s = format!("{x:.3}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" { "0".into() } else { s.to_owned() }
}

/// ⭐ **O gancho do ponto de parada.**
pub(crate) unsafe extern "C-unwind" fn on_break(l: *mut ffi::lua_State, ar: *mut ffi::lua_Debug) {
    let Some(st) = lookup(l) else { return };
    let Ok(mut s) = st.try_borrow_mut() else {
        return;
    };
    // SAFETY: o Luau entrega um `ar` válido durante o gancho.
    let line = unsafe { (*ar).currentline };
    let me = l as usize;
    if s.suppress.get(&me) == Some(&line) {
        // A própria paragem a voltar a bater: gasta-se aqui (um laço numa linha pára de novo).
        s.suppress.remove(&me);
        return;
    }
    if s.quiet > 0 {
        return;
    }
    // SAFETY: `l` é o fio que corre.
    if me != s.current || unsafe { ffi::lua_isyieldable(l) } == 0 {
        s.blocked += 1;
        return;
    }
    s.stop_pending = Some((me, StopKind::Breakpoint));
    // SAFETY: `l` corre este gancho.
    if let Some(f) = unsafe { snap(l, line) } {
        s.stopped.insert(me, f);
    }
    s.step.remove(&me);
    s.suppress.remove(&me);
    drop(s);
    // SAFETY: o fio pode ceder (verificado); o Luau retoma-o na mesma instrução.
    unsafe { ffi::lua_yield(l, 0) };
}

/// ⭐ **O gancho do passo-a-passo** — só fios armados um a um o chamam.
pub(crate) unsafe extern "C-unwind" fn on_step(l: *mut ffi::lua_State, ar: *mut ffi::lua_Debug) {
    let Some(st) = lookup(l) else { return };
    let Ok(mut s) = st.try_borrow_mut() else {
        return;
    };
    // SAFETY: o Luau entrega um `ar` válido durante o gancho.
    let line = unsafe { (*ar).currentline };
    let me = l as usize;
    if me != s.watched && !s.step.contains_key(&me) && !s.suppress.contains_key(&me) {
        // Uma corrotina que herdou o passo-a-passo de um fio vigiado ou a andar: desliga-o.
        // SAFETY: `l` corre este gancho; só o bit dele muda.
        unsafe { ffi::lua_singlestep(l, 0) };
        return;
    }
    if s.suppress.get(&me).is_some_and(|at| *at != line) {
        s.suppress.remove(&me);
    }
    if let Some(step) = s.step.get(&me).copied()
        && step.line != line
        && s.quiet == 0
    {
        // SAFETY: `l` é o fio que corre.
        let depth = unsafe { ffi::lua_stackdepth(l) };
        if depth <= step.depth {
            s.step.remove(&me);
            // SAFETY: idem.
            if me == s.current && unsafe { ffi::lua_isyieldable(l) } != 0 {
                s.stop_pending = Some((me, StopKind::Step));
                // SAFETY: `l` corre este gancho.
                if let Some(f) = unsafe { snap(l, line) } {
                    s.stopped.insert(me, f);
                }
                drop(s);
                // SAFETY: o fio pode ceder (verificado).
                unsafe { ffi::lua_yield(l, 0) };
                return;
            }
            s.blocked += 1;
        }
    }
    if me == s.watched {
        // SAFETY: `l` é o fio vigiado, a correr.
        unsafe { live_step(l, &mut s, line) };
    }
}

/// Os valores ao vivo: a linha que acabou leva as locais que mudou.
unsafe fn live_step(l: *mut ffi::lua_State, s: &mut State, line: i32) {
    // SAFETY: `l` corre; `lua_stackdepth` só lê.
    let depth = unsafe { ffi::lua_stackdepth(l) };
    while s.frames.last().is_some_and(|f| f.depth > depth) {
        s.frames.pop();
    }
    if s.frames
        .last()
        .is_some_and(|f| f.depth == depth && f.line == line)
    {
        return;
    }
    let mut ar = zeroed();
    // SAFETY: nível 0 = a função em curso.
    if unsafe { ffi::lua_getinfo(l, 0, c"s".as_ptr(), &raw mut ar) } == 0
        || text_of(ar.source) != s.watched_chunk
    {
        return;
    }
    // SAFETY: o gancho tem `LUA_MINSTACK` livre; cada valor sai a seguir.
    let now = unsafe { locals_at(l, 0) };
    let at = u32::try_from(line).unwrap_or(0);
    match s.frames.last_mut() {
        Some(top) if top.depth == depth => {
            let changed: Vec<Var> = now
                .iter()
                .filter(|v| !top.snapshot.contains(v))
                .cloned()
                .collect();
            let done = u32::try_from(top.line).unwrap_or(0);
            if !changed.is_empty() {
                s.live.entry(done).or_default().values = changed;
            }
            top.line = line;
            top.snapshot = now;
        }
        _ => s.frames.push(LiveFrame {
            depth,
            line,
            snapshot: now,
        }),
    }
    s.live.entry(at).or_default();
    *s.ran.entry(at).or_default() += 1;
}
