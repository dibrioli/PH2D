#![forbid(unsafe_code)]
//! ph2d-script — Luau scripting runtime + coroutine scheduler + messaging bus.
//!
//! Spike (S1-S3) skeleton: minimal `ScriptRuntime`, `Scheduler` for
//! `ph2d.wait`-style coroutines, Defold-style `MessageBus`. ✓
//!
//! M7 layered the canonical engine ↔ Luau bridge on top:
//! - [`ScriptHost`] — public facade with reset+restore hot reload via
//!   blake3 source-content hashing (HR-16). VM rebuild costs one
//!   constructor call; ECS state stays untouched (canonical store).
//! - [`WriteQueue`] / [`ReadSnapshot`] — the only legal data path
//!   between Luau callbacks and ECS, per HR-8 ("scripts are pure
//!   data at the FFI boundary"). Same pattern Defold/Roblox use.
//! - `ph2d.set(entity, field, value)` and `ph2d.get(entity, field)`
//!   wired into the Lua globals; both go through the queues above.
//! - `Lua::sandbox(true)` activated so user scripts can't reach
//!   `os.execute`, `io.open`, `package.loadlib`, etc. (HR-9).

pub mod host;
pub mod io;
pub mod messaging;

pub use host::ScriptHost;
pub use io::{EntityWrite, QueueFull, ReadSnapshot, WriteQueue};
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
