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
use crate::io::{
    EntityWrite, InputSnapshot, NameSnapshot, ReadSnapshot, SpawnCommand, SpawnQueue, WriteQueue,
};
use crate::lateral::{PodValue, StateTable};
use mlua::Function;

/// Public host facade. One per app. Held by the shell; passed
/// `provide_read` / `tick` / `drain_writes` from the per-frame loop.
///
/// M14.2 additions:
/// - [`SpawnQueue`] receives `ph2d.spawn` / `despawn` / `attach_script`.
/// - [`NameSnapshot`] is populated by the host each frame so
///   `ph2d.find_by_name(name)` resolves without crossing the world
///   handle.
/// - [`StateTable`] holds per-entity lateral state (HR-16). Survives
///   `load_script` resets — that's the whole point.
pub struct ScriptHost {
    runtime: ScriptRuntime,
    last_source_hash: Option<[u8; 32]>,
    /// Number of resets we've performed — useful for tests that
    /// assert reset+restore actually happened on a source change.
    reset_count: u64,
    write_queue: WriteQueue,
    read_snapshot: ReadSnapshot,
    input_snapshot: InputSnapshot,
    spawn_queue: SpawnQueue,
    name_snapshot: NameSnapshot,
    state_table: StateTable,
}

impl ScriptHost {
    pub fn new() -> mlua::Result<Self> {
        let write_queue = WriteQueue::new();
        let read_snapshot = ReadSnapshot::new();
        let input_snapshot = InputSnapshot::new();
        let spawn_queue = SpawnQueue::new();
        let name_snapshot = NameSnapshot::new();
        let state_table = StateTable::new();
        let runtime = build_runtime(
            &write_queue,
            &read_snapshot,
            &input_snapshot,
            &spawn_queue,
            &name_snapshot,
            &state_table,
        )?;
        Ok(Self {
            runtime,
            last_source_hash: None,
            reset_count: 0,
            write_queue,
            read_snapshot,
            input_snapshot,
            spawn_queue,
            name_snapshot,
            state_table,
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
        // The shared Arc-backed handles (WriteQueue, ReadSnapshot,
        // InputSnapshot, SpawnQueue, NameSnapshot, StateTable) all
        // survive the reset — that's how per-entity state stays put
        // across a Luau hot reload (HR-16 + ADR-0025 M14.2 contract).
        self.runtime = build_runtime(
            &self.write_queue,
            &self.read_snapshot,
            &self.input_snapshot,
            &self.spawn_queue,
            &self.name_snapshot,
            &self.state_table,
        )?;
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

    /// Drain pending `ph2d.spawn` / `despawn` / `attach_script`
    /// commands queued by Luau callbacks. Host applies these to the
    /// SimWorld between ticks.
    pub fn drain_spawns(&self) -> Vec<SpawnCommand> {
        self.spawn_queue.drain()
    }

    /// Inject one `(name → entity_id)` mapping into the snapshot
    /// `ph2d.find_by_name` reads. Host re-populates each frame
    /// before tick (analogous to `provide_read`).
    pub fn provide_name(&self, name: &str, entity: u64) {
        self.name_snapshot.set(name, entity);
    }

    /// Drop every name from the snapshot. Call before re-populating
    /// per-frame so stale mappings from the previous tick don't leak.
    pub fn clear_names(&self) {
        self.name_snapshot.clear();
    }

    /// Set a single field on `lateral_key`'s state table directly
    /// from Rust (e.g. when the host wants to seed initial state
    /// for an entity it just attached a script to).
    pub fn provide_state(&self, lateral_key: u64, field: &str, value: PodValue) {
        self.state_table.set(lateral_key, field, value);
    }

    /// Read a single field from the lateral state table. Returns
    /// `None` if the entity has no entry for that field.
    pub fn read_state(&self, lateral_key: u64, field: &str) -> Option<PodValue> {
        self.state_table.get(lateral_key, field)
    }

    /// Borrow the lateral state table (e.g. for snapshot / inspection).
    pub fn state_table(&self) -> &StateTable {
        &self.state_table
    }

    /// Borrow the spawn queue (for tests + debug overlays).
    pub fn spawn_queue(&self) -> &SpawnQueue {
        &self.spawn_queue
    }

    /// Borrow the name snapshot (for tests + debug overlays).
    pub fn name_snapshot(&self) -> &NameSnapshot {
        &self.name_snapshot
    }
}

/// Build a fresh `ScriptRuntime` with the `ph2d.*` Luau bindings
/// wired against the supplied bridge handles, then activate
/// `Lua::sandbox(true)`. Order is load-bearing: sandbox freezes the
/// globals + the `ph2d` namespace, so the API must be installed
/// FIRST. Activating sandbox last makes the surface read-only to
/// user scripts (HR-9).
fn build_runtime(
    write_queue: &WriteQueue,
    read_snapshot: &ReadSnapshot,
    input_snapshot: &InputSnapshot,
    spawn_queue: &SpawnQueue,
    name_snapshot: &NameSnapshot,
    state_table: &StateTable,
) -> mlua::Result<ScriptRuntime> {
    let runtime = ScriptRuntime::new()?;
    wire_ph2d_api(
        &runtime,
        write_queue,
        read_snapshot,
        input_snapshot,
        spawn_queue,
        name_snapshot,
        state_table,
    )?;
    runtime.lua().sandbox(true)?;
    Ok(runtime)
}

fn wire_ph2d_api(
    runtime: &ScriptRuntime,
    write_queue: &WriteQueue,
    read_snapshot: &ReadSnapshot,
    input_snapshot: &InputSnapshot,
    spawn_queue: &SpawnQueue,
    name_snapshot: &NameSnapshot,
    state_table: &StateTable,
) -> mlua::Result<()> {
    let lua = runtime.lua();
    // Stash clones of the SHARED handles in app-data so the closures
    // below can grab them without capturing — `Lua::create_function`
    // requires `Send + 'static` bodies; app-data sidesteps the
    // closure-capture lifetimes. The clones share inner Arc storage,
    // so values the host pushes via `provide_read` / `provide_input`
    // / `provide_name` / `provide_state` are observable here even
    // after a VM rebuild.
    lua.set_app_data(write_queue.clone());
    lua.set_app_data(read_snapshot.clone());
    lua.set_app_data(input_snapshot.clone());
    lua.set_app_data(spawn_queue.clone());
    lua.set_app_data(name_snapshot.clone());
    lua.set_app_data(state_table.clone());

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

    // M14.2 additions ----------------------------------------------------

    // ph2d.spawn() — queue an empty entity spawn. Returns nil; the
    // assigned entity id appears in `ph2d.find_by_name(name)` next
    // tick if the spawn was named via `ph2d.spawn_named`.
    //
    // Prefab-aware variant `ph2d.spawn(prefab_id)` arrives in M14.3
    // when `Asset::Prefab` exists.
    let spawn_fn = lua.create_function(|lua, ()| -> mlua::Result<()> {
        let q = lua.app_data_ref::<SpawnQueue>().ok_or_else(|| {
            mlua::Error::RuntimeError("ph2d.spawn: SpawnQueue not registered".into())
        })?;
        q.push(SpawnCommand::SpawnEmpty)
            .map_err(|e| mlua::Error::RuntimeError(format!("ph2d.spawn: {e}")))
    })?;

    // ph2d.spawn_named(name) — queue a spawn that will get a Name
    // component on application. Same caveats as `spawn`.
    let spawn_named_fn = lua.create_function(|lua, name: String| -> mlua::Result<()> {
        let q = lua.app_data_ref::<SpawnQueue>().ok_or_else(|| {
            mlua::Error::RuntimeError("ph2d.spawn_named: SpawnQueue not registered".into())
        })?;
        q.push(SpawnCommand::SpawnNamed { name })
            .map_err(|e| mlua::Error::RuntimeError(format!("ph2d.spawn_named: {e}")))
    })?;

    // ph2d.despawn(entity) — queue an entity despawn.
    let despawn_fn = lua.create_function(|lua, entity: f64| -> mlua::Result<()> {
        let q = lua.app_data_ref::<SpawnQueue>().ok_or_else(|| {
            mlua::Error::RuntimeError("ph2d.despawn: SpawnQueue not registered".into())
        })?;
        q.push(SpawnCommand::Despawn {
            entity: entity as u64,
        })
        .map_err(|e| mlua::Error::RuntimeError(format!("ph2d.despawn: {e}")))
    })?;

    // ph2d.attach_script(entity, asset_id_hex) — queue a LuauScript
    // component attachment. Accepts the 64-char hex form of the
    // bytecode AssetId (the only stable, script-visible form);
    // host parses it back to a 32-byte digest on apply.
    let attach_script_fn = lua.create_function(
        |lua, (entity, asset_hex): (f64, String)| -> mlua::Result<()> {
            let q = lua.app_data_ref::<SpawnQueue>().ok_or_else(|| {
                mlua::Error::RuntimeError("ph2d.attach_script: SpawnQueue not registered".into())
            })?;
            if asset_hex.len() != 64 {
                return Err(mlua::Error::RuntimeError(
                    "ph2d.attach_script: bytecode id must be 64-char hex".into(),
                ));
            }
            let mut digest = [0u8; 32];
            for (i, chunk) in asset_hex.as_bytes().chunks_exact(2).enumerate() {
                let s = std::str::from_utf8(chunk).map_err(|_| {
                    mlua::Error::RuntimeError(
                        "ph2d.attach_script: bytecode id is not ASCII hex".into(),
                    )
                })?;
                digest[i] = u8::from_str_radix(s, 16).map_err(|_| {
                    mlua::Error::RuntimeError(
                        "ph2d.attach_script: bytecode id is not valid hex".into(),
                    )
                })?;
            }
            q.push(SpawnCommand::AttachScript {
                entity: entity as u64,
                bytecode: digest,
            })
            .map_err(|e| mlua::Error::RuntimeError(format!("ph2d.attach_script: {e}")))
        },
    )?;

    // ph2d.find_by_name(name) — resolve a Name component to an
    // entity id. Reads from a NameSnapshot the host populates each
    // tick.
    let find_by_name_fn =
        lua.create_function(|lua, name: String| -> mlua::Result<Option<f64>> {
            let s = lua.app_data_ref::<NameSnapshot>().ok_or_else(|| {
                mlua::Error::RuntimeError("ph2d.find_by_name: NameSnapshot not registered".into())
            })?;
            Ok(s.get(&name).map(|e| e as f64))
        })?;

    // ph2d.state_get(entity, field) — read per-instance lateral state.
    // Returns nil if either the entity or field is absent.
    let state_get_fn = lua.create_function(
        |lua, (entity, field): (f64, String)| -> mlua::Result<mlua::Value> {
            let s = lua.app_data_ref::<StateTable>().ok_or_else(|| {
                mlua::Error::RuntimeError("ph2d.state_get: StateTable not registered".into())
            })?;
            match s.get(entity as u64, &field) {
                Some(v) => v.to_lua(lua),
                None => Ok(mlua::Value::Nil),
            }
        },
    )?;

    // ph2d.state_set(entity, field, value) — write per-instance
    // lateral state. Value is type-checked HR-16 POD; non-POD
    // (function, thread, userdata) returns a Luau error.
    let state_set_fn = lua.create_function(
        |lua, (entity, field, value): (f64, String, mlua::Value)| -> mlua::Result<()> {
            let s = lua.app_data_ref::<StateTable>().ok_or_else(|| {
                mlua::Error::RuntimeError("ph2d.state_set: StateTable not registered".into())
            })?;
            let pod = PodValue::from_lua(value, 0)?;
            s.set(entity as u64, &field, pod);
            Ok(())
        },
    )?;

    // ph2d.state_keys(entity) — alphabetically-sorted list of fields
    // for the given entity's lateral state. Returns an empty table
    // if the entity has no state.
    let state_keys_fn = lua.create_function(|lua, entity: f64| -> mlua::Result<mlua::Table> {
        let s = lua.app_data_ref::<StateTable>().ok_or_else(|| {
            mlua::Error::RuntimeError("ph2d.state_keys: StateTable not registered".into())
        })?;
        let keys = s.keys(entity as u64);
        let out = lua.create_table_with_capacity(keys.len(), 0)?;
        for (i, k) in keys.into_iter().enumerate() {
            // Luau tables are 1-indexed.
            out.set(i + 1, k)?;
        }
        Ok(out)
    })?;

    // ph2d.state_clear(entity) — drop every field for the given
    // entity. Idempotent.
    let state_clear_fn = lua.create_function(|lua, entity: f64| -> mlua::Result<()> {
        let s = lua.app_data_ref::<StateTable>().ok_or_else(|| {
            mlua::Error::RuntimeError("ph2d.state_clear: StateTable not registered".into())
        })?;
        s.clear(entity as u64);
        Ok(())
    })?;

    let ph2d: mlua::Table = lua.globals().get("ph2d")?;
    ph2d.set("set", set_fn)?;
    ph2d.set("get", get_fn)?;
    ph2d.set("input", input_fn)?;
    ph2d.set("spawn", spawn_fn)?;
    ph2d.set("spawn_named", spawn_named_fn)?;
    ph2d.set("despawn", despawn_fn)?;
    ph2d.set("attach_script", attach_script_fn)?;
    ph2d.set("find_by_name", find_by_name_fn)?;
    ph2d.set("state_get", state_get_fn)?;
    ph2d.set("state_set", state_set_fn)?;
    ph2d.set("state_keys", state_keys_fn)?;
    ph2d.set("state_clear", state_clear_fn)?;

    Ok(())
}
