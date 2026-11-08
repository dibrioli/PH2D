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
use ph2d_script_debug::{Debugger, StopKind};
use ph2d_script_doc::ScriptId;

use super::{Deadline, Instance, SceneReport, settle_call};
use crate::console::speaking;
use crate::io::EntityWrite;
use crate::lane::Lane;

/// Um gancho que parou num `ph2d.wait` — ou no depurador.
pub(super) struct Parked {
    pub(super) entity: Entity,
    pub(super) script: ScriptId,
    pub(super) thread: Thread,
    pub(super) kind: Park,
}

pub(super) enum Park {
    /// Microssegundos inteiros, como o `Timer` da casa: `round(s·1e6)`.
    Wait { target_us: u64, elapsed_us: u64 },
    /// ⭐ **Parado no depurador** (plano 13 Onda D): não anda com o tempo; sai por Continue / Next
    /// line. Os ganchos que o objecto devia correr enquanto está parado esperam em `then`, por ordem.
    Held {
        why: StopKind,
        then: Vec<(Function, MultiValue)>,
    },
}

impl Parked {
    pub(super) fn is_held(&self) -> bool {
        matches!(self.kind, Park::Held { .. })
    }
}

/// O que uma chamada de gancho deu.
pub(super) enum Ran {
    Done,
    Waiting(Thread, u64),
    Held(Thread, StopKind),
}

/// O depurador da VM (instalado pelo `ScriptHost`).
pub(crate) fn debugger(lua: &Lua) -> Option<Debugger> {
    lua.app_data_ref::<Debugger>().map(|d| d.clone())
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

/// ⭐ **Corre um gancho numa corrotina nova.** `watch` = o chunk, se este gancho é do objecto cujos
/// valores ao vivo a aba mostra.
pub(super) fn run_hook(
    lua: &Lua,
    f: &Function,
    args: impl IntoLuaMulti,
    watch: Option<&str>,
) -> mlua::Result<Ran> {
    let th = lua.create_thread(f.clone())?;
    if let (Some(chunk), Some(d)) = (watch, debugger(lua)) {
        d.watch(lua, &th, chunk)?;
    }
    resume(lua, th, args)
}

/// Retoma `th`.
pub(super) fn resume(lua: &Lua, th: Thread, args: impl IntoLuaMulti) -> mlua::Result<Ran> {
    let dbg = debugger(lua);
    let out = match &dbg {
        Some(d) => d.running(&th, || th.resume::<MultiValue>(args)),
        None => th.resume::<MultiValue>(args),
    };
    let yielded = th.status() == mlua::thread::ThreadStatus::Resumable;
    match out {
        Ok(vals) if yielded => match wait_of(lua, &vals) {
            Some(us) => Ok(Ran::Waiting(th, us)),
            None => match dbg.as_ref().and_then(|d| d.took_stop(&th)) {
                Some(why) => Ok(Ran::Held(th, why)),
                None => Err(mlua::Error::RuntimeError(
                    "coroutine.yield in a hook: use ph2d.wait(seconds) to pause".into(),
                )),
            },
        },
        Ok(_) => Ok(Ran::Done),
        Err(e) => {
            // Pause on Error: a pilha da corrotina que errou ainda está de pé — lê-se AGORA.
            if let Some(d) = dbg.as_ref().filter(|d| d.pause_on_error())
                && let Ok(Some(frame)) = d.frame(lua, &th)
            {
                d.stash_error_frame(frame);
            }
            Err(e)
        }
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
                kind: Park::Wait {
                    target_us,
                    elapsed_us: 0,
                },
            });
            Ok(())
        }
        Ran::Held(thread, why) => {
            parked.push(Parked {
                entity,
                script,
                thread,
                kind: Park::Held {
                    why,
                    then: Vec::new(),
                },
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
        let Park::Wait {
            target_us,
            elapsed_us,
        } = &mut p.kind
        else {
            parked.push(p); // parado no depurador: não anda com o tempo
            continue;
        };
        *elapsed_us = elapsed_us.saturating_add(dt_us);
        if *elapsed_us >= *target_us {
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
            || !inst.runs()
            || p.thread.status() != mlua::thread::ThreadStatus::Resumable
        {
            continue;
        }
        cell.borrow_mut().speaker = Some(p.entity);
        #[expect(
            clippy::cast_precision_loss,
            reason = "microssegundos de uma espera cabem em 2^53"
        )]
        let secs = match p.kind {
            Park::Wait { elapsed_us, .. } => elapsed_us as f64 / 1e6,
            Park::Held { .. } => 0.0,
        };
        report.calls += 1;
        let ran = speaking(lua, inst.script, Some(p.entity), || {
            deadline.within(|| resume(lua, p.thread, secs))
        });
        let ran = park(ran, p.entity, p.script, parked);
        settle_call(lua, p.entity, ran, inst, writes, report);
    }
}
