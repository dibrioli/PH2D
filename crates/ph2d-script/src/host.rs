//! [`ScriptHost`] — the canonical M7 entry point for engine ↔ Luau.
//!
//! Wraps [`crate::ScriptRuntime`] with:
//! - The `ph2d.set` / `ph2d.get` Lua bindings, both backed by
//!   [`crate::WriteQueue`] / [`crate::ReadSnapshot`] (HR-8 keeps the
//!   FFI boundary pure-data — no live ECS handles cross into Lua).
//! - `Lua::sandbox(true)` activation so user scripts can't reach
//!   `os.execute`, `io.open`, `package.loadlib`, etc. (HR-9).
//! - Reset-and-restore hot reload: `load_script` blake3-hashes the
//!   source and only rebuilds the VM when the hash actually changes.
//!   The cost is one Lua VM construction — measured in spike C4 at
//!   well under the budget — and ECS state is untouched (it's the
//!   canonical store, scripts are derived behavior).

use crate::ScriptRuntime;
use crate::io::{EntityWrite, InputSnapshot, ReadSnapshot, WriteQueue};
use mlua::Function;

/// Public host facade. One per app. Held by the shell; passed
/// `provide_read` / `tick` / `drain_writes` from the per-frame loop.
pub struct ScriptHost {
    runtime: ScriptRuntime,
    last_source_hash: Option<[u8; 32]>,
    /// Number of resets we've performed — useful for tests that
    /// assert reset+restore actually happened on a source change.
    reset_count: u64,
    write_queue: WriteQueue,
    read_snapshot: ReadSnapshot,
    input_snapshot: InputSnapshot,
}

impl ScriptHost {
    pub fn new() -> mlua::Result<Self> {
        let write_queue = WriteQueue::new();
        let read_snapshot = ReadSnapshot::new();
        let input_snapshot = InputSnapshot::new();
        let runtime = build_runtime(&write_queue, &read_snapshot, &input_snapshot)?;
        Ok(Self {
            runtime,
            last_source_hash: None,
            reset_count: 0,
            write_queue,
            read_snapshot,
            input_snapshot,
        })
    }

    /// Load (or reload) the user script. Reset-and-restore: if the
    /// blake3 of `source` matches the previous load, this is a no-op
    /// (returns `Ok(false)`). Otherwise the VM is rebuilt and the
    /// script re-executed (returns `Ok(true)`).
    ///
    /// HR-16: hot-reload is content-addressed, so re-saving an
    /// unchanged file doesn't churn coroutines. The shell can call
    /// `load_script` every frame without cost.
    ///
    /// `WriteQueue` and `ReadSnapshot` survive the reset — they're
    /// the engine↔script bridge, not script state — so values the
    /// host pushed via `provide_read` before reload remain visible to
    /// the freshly-loaded script.
    pub fn load_script(&mut self, source: &str) -> mlua::Result<bool> {
        let hash = *blake3::hash(source.as_bytes()).as_bytes();
        if self.last_source_hash == Some(hash) {
            return Ok(false);
        }
        // Source actually changed → tear down + rebuild. The previous
        // VM (with its coroutines, globals, GC heap) is dropped here.
        self.runtime = build_runtime(&self.write_queue, &self.read_snapshot, &self.input_snapshot)?;
        self.runtime.eval(source)?;
        self.last_source_hash = Some(hash);
        self.reset_count += 1;
        Ok(true)
    }

    /// Inject one ECS field value into the read snapshot. Call once
    /// per (entity, field) before the per-frame `tick` so `ph2d.get`
    /// resolves to current state.
    pub fn provide_read(&self, entity: u32, field: &str, value: f64) {
        self.read_snapshot.set(entity, field, value);
    }

    /// Clear the read snapshot. Call before re-populating a new frame
    /// of values, otherwise stale entries from the previous tick leak.
    pub fn clear_reads(&self) {
        self.read_snapshot.clear();
    }

    /// Push one input key/value into the per-frame snapshot consulted
    /// by `ph2d.input(key)` from Luau. Shells (e.g. shells/desktop
    /// gilrs poll) call this for each held button / axis value after
    /// `clear_input()`.
    pub fn provide_input(&self, key: &str, value: f64) {
        self.input_snapshot.set(key, value);
    }

    /// Drop every key from the input snapshot. Must be called once
    /// per frame BEFORE re-populating, otherwise a button released
    /// last frame would still show as held.
    pub fn clear_input(&self) {
        self.input_snapshot.clear();
    }

    pub fn input_snapshot(&self) -> &InputSnapshot {
        &self.input_snapshot
    }

    /// Drain everything Luau wrote via `ph2d.set` since the last
    /// `drain_writes` call. The host applies these to ECS.
    pub fn drain_writes(&self) -> Vec<EntityWrite> {
        self.write_queue.drain()
    }

    pub fn reset_count(&self) -> u64 {
        self.reset_count
    }

    pub fn runtime(&self) -> &ScriptRuntime {
        &self.runtime
    }

    /// Run one GC step. Per HR-9 / M7 budget: must complete in
    /// ≤ 0.01 ms p99 — measured in `tests/gc_pause.rs`.
    pub fn gc_step(&self) -> mlua::Result<()> {
        self.runtime.lua().gc_step()?;
        Ok(())
    }

    /// Convenience: lookup a global Luau function by name and resume
    /// it as a coroutine via [`ScriptRuntime::spawn`]. Useful when a
    /// shell wants to start a long-running user task by name.
    pub fn spawn_named(&self, name: &str) -> mlua::Result<mlua::Thread> {
        let f: Function = self.runtime.lua().globals().get(name)?;
        self.runtime.spawn(f)
    }
}

/// Build a fresh `ScriptRuntime` with `ph2d.set`/`ph2d.get`/`ph2d.input`
/// wired against the supplied bridge handles, then activate
/// `Lua::sandbox(true)`. Order is load-bearing: sandbox freezes the
/// globals + the `ph2d` namespace, so the API must be installed
/// FIRST. Activating sandbox last makes the surface read-only to
/// user scripts (HR-9).
fn build_runtime(
    write_queue: &WriteQueue,
    read_snapshot: &ReadSnapshot,
    input_snapshot: &InputSnapshot,
) -> mlua::Result<ScriptRuntime> {
    let runtime = ScriptRuntime::new()?;
    wire_ph2d_api(&runtime, write_queue, read_snapshot, input_snapshot)?;
    runtime.lua().sandbox(true)?;
    Ok(runtime)
}

fn wire_ph2d_api(
    runtime: &ScriptRuntime,
    write_queue: &WriteQueue,
    read_snapshot: &ReadSnapshot,
    input_snapshot: &InputSnapshot,
) -> mlua::Result<()> {
    let lua = runtime.lua();
    // Stash clones of the SHARED handles in app-data so the closures
    // below can grab them without capturing — `Lua::create_function`
    // requires `Send + 'static` bodies; app-data sidesteps the
    // closure-capture lifetimes. The clones share inner Arc storage,
    // so values the host pushes via `provide_read` / `provide_input`
    // are observable here even after a VM rebuild.
    lua.set_app_data(write_queue.clone());
    lua.set_app_data(read_snapshot.clone());
    lua.set_app_data(input_snapshot.clone());

    let set_fn = lua.create_function(
        |lua, (entity, field, value): (u32, String, f64)| -> mlua::Result<()> {
            // app_data_ref panic was a known gap (audit MEDIUM): convert
            // missing-data into a script-visible error instead of a Rust
            // panic. This branch is unreachable in normal flow (we just
            // set the data in build_runtime), but defends against
            // mlua-version drift or mid-tick lua.remove_app_data calls.
            let q = lua.app_data_ref::<WriteQueue>().ok_or_else(|| {
                mlua::Error::RuntimeError("ph2d.set: WriteQueue not registered".into())
            })?;
            // Surface QueueFull as a Luau runtime error so the script
            // fails fast (HR-9 backpressure) instead of OOMing the
            // host. Caller can `pcall` to recover.
            q.push(EntityWrite {
                entity,
                field,
                value,
            })
            .map_err(|e| mlua::Error::RuntimeError(format!("ph2d.set: {e}")))
        },
    )?;

    let get_fn = lua.create_function(
        |lua, (entity, field): (u32, String)| -> mlua::Result<Option<f64>> {
            let s = lua.app_data_ref::<ReadSnapshot>().ok_or_else(|| {
                mlua::Error::RuntimeError("ph2d.get: ReadSnapshot not registered".into())
            })?;
            Ok(s.get(entity, &field))
        },
    )?;

    let input_fn = lua.create_function(|lua, key: String| -> mlua::Result<Option<f64>> {
        let s = lua.app_data_ref::<InputSnapshot>().ok_or_else(|| {
            mlua::Error::RuntimeError("ph2d.input: InputSnapshot not registered".into())
        })?;
        Ok(s.get(&key))
    })?;

    let ph2d: mlua::Table = lua.globals().get("ph2d")?;
    ph2d.set("set", set_fn)?;
    ph2d.set("get", get_fn)?;
    ph2d.set("input", input_fn)?;

    Ok(())
}
