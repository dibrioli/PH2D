#![forbid(unsafe_code)]
//! ph2d-script — Luau scripting runtime + coroutine scheduler + messaging bus.
//!
//! S1 spike (C1): minimal ScriptRuntime + 60-frame loop. ✓
//! S2 spike (C3): Scheduler para coroutines com `ph2d.wait(seconds)`. ✓
//! S3 spike (C2.1): MessageBus estilo Defold. ✓ (sub-módulo `messaging`).
//!
//! Surface intencionalmente pequena; expande em S2 (hot reload integrado) e
//! S3 (LLM-centric API completa).

pub mod messaging;
pub use messaging::{EntityId, Handler, Message, MessageBus, MessageId};

use mlua::{Function, IntoLuaMulti, Lua, Thread, ThreadStatus, Value};

pub struct ScriptRuntime {
    lua: Lua,
}

impl ScriptRuntime {
    pub fn new() -> mlua::Result<Self> {
        let lua = Lua::new();
        let setup = r#"
ph2d = ph2d or {}
function ph2d.wait(seconds)
    coroutine.yield(seconds)
end
"#;
        lua.load(setup).exec()?;
        Ok(Self { lua })
    }

    pub fn eval(&self, source: &str) -> mlua::Result<()> {
        self.lua.load(source).exec()
    }

    pub fn lua(&self) -> &Lua {
        &self.lua
    }

    /// Wrap a Luau function as a coroutine thread.
    pub fn spawn(&self, body: Function) -> mlua::Result<Thread> {
        self.lua.create_thread(body)
    }
}

/// Cooperative scheduler for Luau coroutines that yield `seconds`-typed waits.
///
/// Design: each `tick(dt)` increments accumulated time on every pending task.
/// When a task's accumulated time meets its target, it is `resume()`d. If the
/// task yields again, it is re-queued with the new target (and elapsed=0).
/// If the task finishes (returns), it is dropped.
///
/// Per-task `elapsed_at_finish` is captured for measurement (C3).
pub struct Scheduler {
    tasks: Vec<Task>,
}

struct Task {
    thread: Thread,
    elapsed: f64,
    target: f64,
}

/// Outcome of `tick`: per-task wall-clock at finish, useful for C3 timing measurement.
#[derive(Default)]
pub struct TickReport {
    pub finished_elapsed: Vec<f64>,
    pub still_pending: usize,
}

impl Scheduler {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn pending(&self) -> usize {
        self.tasks.len()
    }

    /// Add a fresh thread. Performs an initial resume (passing `init_args`) to
    /// extract the first wait target.
    /// Returns `true` if thread parked (waiting), `false` if it finished immediately.
    pub fn add(&mut self, thread: Thread, init_args: impl IntoLuaMulti) -> mlua::Result<bool> {
        let value: Value = thread.resume(init_args)?;
        match thread.status() {
            ThreadStatus::Resumable => {
                let target = value_as_seconds(&value);
                self.tasks.push(Task {
                    thread,
                    elapsed: 0.0,
                    target,
                });
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Advance time by `dt` seconds for all pending tasks; resume those whose target met.
    pub fn tick(&mut self, dt: f64) -> mlua::Result<TickReport> {
        let mut report = TickReport::default();
        let mut i = 0;
        while i < self.tasks.len() {
            self.tasks[i].elapsed += dt;
            if self.tasks[i].elapsed >= self.tasks[i].target {
                let task = self.tasks.swap_remove(i);
                let elapsed_at_resume = task.elapsed;
                let value: Value = task.thread.resume(())?;
                match task.thread.status() {
                    ThreadStatus::Resumable => {
                        let new_target = value_as_seconds(&value);
                        self.tasks.push(Task {
                            thread: task.thread,
                            elapsed: 0.0,
                            target: new_target,
                        });
                    }
                    _ => {
                        report.finished_elapsed.push(elapsed_at_resume);
                    }
                }
            } else {
                i += 1;
            }
        }
        report.still_pending = self.tasks.len();
        Ok(report)
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

fn value_as_seconds(v: &Value) -> f64 {
    match v {
        Value::Number(n) => *n,
        Value::Integer(i) => *i as f64,
        _ => 0.0,
    }
}
