//! ⭐⭐ **As esperas** (`ph2d.wait`, CodeEditor item 6 — `docs/CodeEditor/05_api_que_funciona.md` §3.1):
//! cada gancho corre numa corrotina NOVA; uma que cede a marca da espera fica em [`Parked`] até ao
//! fim do tique em que o tempo chega.
//!
//! ⛔ **Corrotinas não se reaproveitam** (`Thread::reset`, 107 ns contra 198 de uma nova): um script
//! que guardasse `coroutine.running()` alcançaria o gancho de OUTRO objecto que herdasse a corrotina
//! (auditoria do fecho, 20/10).

use std::collections::BTreeMap;

use mlua::{Function, IntoLuaMulti, Lua, MultiValue, Thread, Value};
use ph2d_ecs::Entity;
use ph2d_script_doc::ScriptId;

use super::{Deadline, Instance, SceneReport, settle_call};
use crate::console::speaking;
use crate::io::EntityWrite;
use crate::lane::Lane;

/// Um gancho que parou num `ph2d.wait`.
pub(super) struct Parked {
    pub(super) entity: Entity,
    pub(super) script: ScriptId,
    thread: Thread,
    /// Microssegundos inteiros, como o `Timer` da casa: `round(s·1e6)`.
    target_us: u64,
    elapsed_us: u64,
}

/// O que uma chamada de gancho deu.
pub(super) enum Ran {
    Done,
    Waiting(Thread, u64),
}

/// A marca que o `ph2d.wait` cede junto com os segundos (uma cedência sem ela não é uma espera).
pub(crate) const WAIT_TOKEN: &str = "ph2d.wait";

/// Segundos → microssegundos inteiros (o relógio do `Timer` da casa).
pub(super) fn micros(seconds: f64) -> u64 {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "saturado: um número não-finito ou negativo vale 0, um enorme vale u64::MAX"
    )]
    let us = (seconds * 1e6).round().max(0.0) as u64;
    us
}

/// ⭐ **Corre um gancho numa corrotina nova.**
pub(super) fn run_hook(lua: &Lua, f: &Function, args: impl IntoLuaMulti) -> mlua::Result<Ran> {
    resume(lua, lua.create_thread(f.clone())?, args)
}

/// Retoma `th`.
fn resume(lua: &Lua, th: Thread, args: impl IntoLuaMulti) -> mlua::Result<Ran> {
    let out = th.resume::<MultiValue>(args);
    let yielded = th.status() == mlua::thread::ThreadStatus::Resumable;
    match out {
        Ok(vals) if yielded => match wait_of(lua, &vals) {
            Some(us) => Ok(Ran::Waiting(th, us)),
            None => Err(mlua::Error::RuntimeError(
                "coroutine.yield in a hook: use ph2d.wait(seconds) to pause".into(),
            )),
        },
        Ok(_) => Ok(Ran::Done),
        Err(e) => Err(e),
    }
}

/// Os microssegundos de uma cedência do `ph2d.wait` (`WAIT_TOKEN, segundos`).
fn wait_of(lua: &Lua, vals: &MultiValue) -> Option<u64> {
    let token: Value = lua.named_registry_value(WAIT_TOKEN).ok()?;
    let mut it = vals.iter();
    let first = it.next()?;
    if !first.equals(&token).unwrap_or(false) {
        return None;
    }
    match it.next() {
        Some(Value::Number(s)) => Some(micros(*s)),
        Some(Value::Integer(s)) => Some(micros(*s as f64)),
        _ => Some(0),
    }
}

/// Uma chamada que parou num `ph2d.wait` vai para as esperas; o resto segue como antes.
pub(super) fn park(
    ran: mlua::Result<Ran>,
    entity: Entity,
    script: ScriptId,
    parked: &mut Vec<Parked>,
) -> mlua::Result<()> {
    match ran? {
        Ran::Done => Ok(()),
        Ran::Waiting(thread, target_us) => {
            parked.push(Parked {
                entity,
                script,
                thread,
                target_us,
                elapsed_us: 0,
            });
            Ok(())
        }
    }
}

/// ⭐⭐ **As esperas que chegaram**, no fim do tique, depois dos ganchos (Godot): cada uma soma o
/// `dt` deste tique — também a que começou nele — e retoma se o acumulado chegou ao alvo. Uma
/// espera começada AQUI só conta a partir do tique seguinte (senão `wait(0)` num laço nunca
/// devolveria o tique). Objecto que morreu, trocou de script ou parou ⇒ a espera é esquecida.
#[allow(clippy::too_many_arguments)]
pub(super) fn resume_due(
    lua: &Lua,
    deadline: &Deadline,
    cell: &std::cell::RefCell<Lane<'_>>,
    instances: &mut BTreeMap<u64, Instance>,
    parked: &mut Vec<Parked>,
    dt_us: u64,
    writes: &mut Vec<EntityWrite>,
    report: &mut SceneReport,
) {
    let mut due = Vec::new();
    for mut p in std::mem::take(parked) {
        p.elapsed_us = p.elapsed_us.saturating_add(dt_us);
        if p.elapsed_us >= p.target_us {
            due.push(p);
        } else {
            parked.push(p);
        }
    }
    for p in due {
        let bits = p.entity.to_bits();
        if cell.borrow().is_dying(p.entity) {
            continue;
        }
        let Some(inst) = instances.get_mut(&bits) else {
            continue;
        };
        // Retomada por fora (o script guardou `coroutine.running()`)? Já não espera.
        if inst.script != p.script
            || inst.failed.is_some()
            || p.thread.status() != mlua::thread::ThreadStatus::Resumable
        {
            continue;
        }
        cell.borrow_mut().speaker = Some(p.entity);
        #[expect(
            clippy::cast_precision_loss,
            reason = "microssegundos de uma espera cabem em 2^53"
        )]
        let secs = p.elapsed_us as f64 / 1e6;
        report.calls += 1;
        let ran = speaking(lua, inst.script, Some(p.entity), || {
            deadline.within(|| resume(lua, p.thread, secs))
        });
        let ran = park(ran, p.entity, p.script, parked);
        settle_call(lua, p.entity, ran, inst, writes, report);
    }
}
