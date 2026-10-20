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
use crate::console::{Console, ConsoleLine};
use crate::io::{Devices, Edge, EntityWrite, InputSnapshot, WriteQueue};
use crate::lane::{NoDoors, SceneDoors};
use crate::lateral::{PodValue, StateTable};
use crate::scene::{Deadline, EmitQueue, SceneReport, SceneScripts, ScriptInfo, ScriptReload};
use ph2d_ecs::{Entity, SimWorld, World};
use ph2d_input::{Key, MouseButton};

/// Public host facade. One per app. Held by the shell; passed
/// `provide_read` / `tick` / `drain_writes` from the per-frame loop.
///
/// - [`StateTable`] holds per-entity lateral state (HR-16). Survives
///   `load_script` resets — that's the whole point — and is cleared by a rewind.
/// - `ph2d.get` / `find_by_name` / `spawn` / `despawn` / `attach_script` read and change the world
///   through the scene lane (`lane.rs`), open only during a tick.
pub struct ScriptHost {
    runtime: ScriptRuntime,
    last_source_hash: Option<[u8; 32]>,
    /// Number of resets we've performed — useful for tests that
    /// assert reset+restore actually happened on a source change.
    reset_count: u64,
    write_queue: WriteQueue,
    input_snapshot: InputSnapshot,
    /// ⭐ O teclado e o rato dos scripts (CodeEditor item 6).
    devices: Devices,
    state_table: StateTable,
    /// ⭐ TOP-20 #16 — os sinais que `ph2d.emit` empurra (drenados por gancho).
    emit_queue: EmitQueue,
    /// O prazo da chamada em curso, partilhado com a interrupção da VM.
    deadline: Deadline,
    /// Os scripts da cena — um módulo por ficheiro, um `self` por objecto.
    scene: SceneScripts,
    /// ⭐ CodeEditor E10: o `print`, os erros de runtime e as recusas de recarga.
    console: Console,
}

impl ScriptHost {
    pub fn new() -> mlua::Result<Self> {
        let write_queue = WriteQueue::new();
        let input_snapshot = InputSnapshot::new();
        let devices = Devices::default();
        let state_table = StateTable::new();
        let emit_queue = EmitQueue::default();
        let deadline = Deadline::default();
        let console = Console::default();
        let runtime = build_runtime(
            &write_queue,
            &input_snapshot,
            &devices,
            &state_table,
            &emit_queue,
            &deadline,
            &console,
        )?;
        Ok(Self {
            runtime,
            last_source_hash: None,
            reset_count: 0,
            write_queue,
            input_snapshot,
            devices,
            state_table,
            emit_queue,
            deadline,
            scene: SceneScripts::default(),
            console,
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
    /// `WriteQueue`, the input snapshot, the devices and the state
    /// table survive the reset — they're the engine↔script bridge, not
    /// script state.
    pub fn load_script(&mut self, source: &str) -> mlua::Result<bool> {
        let hash = *blake3::hash(source.as_bytes()).as_bytes();
        if self.last_source_hash == Some(hash) {
            return Ok(false);
        }
        // Source actually changed → tear down + rebuild. The previous
        // VM (with its coroutines, globals, GC heap) is dropped here.
        // The shared Arc-backed handles (WriteQueue, InputSnapshot,
        // Devices, StateTable) all survive the reset — that's how per-entity state stays put
        // across a Luau hot reload (HR-16 + ADR-0025 M14.2 contract).
        self.runtime = build_runtime(
            &self.write_queue,
            &self.input_snapshot,
            &self.devices,
            &self.state_table,
            &self.emit_queue,
            &self.deadline,
            &self.console,
        )?;
        // ⚠️ Os módulos e os `self` da cena são tabelas da VM VELHA — esquecê-los obriga o próximo
        // `scene_sync` a recarregar da biblioteca, em vez de chamar funções de um estado que morreu.
        self.scene.clear();
        self.runtime.eval(source)?;
        self.last_source_hash = Some(hash);
        self.reset_count += 1;
        Ok(true)
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

    /// ⭐ O teclado e o rato dos scripts — a shell alimenta-os uma vez por quadro
    /// ([`Devices::feed`]).
    pub fn devices(&self) -> &Devices {
        &self.devices
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

    // ── TOP-20 #16: os scripts da CENA ────────────────────────────────────────────────────────

    /// ⭐ **Uma vez por quadro, a correr ou não** — ver [`SceneScripts::sync`]. Os textos vêm do
    /// recurso [`crate::ProjectScripts`] do mundo (ausente = biblioteca vazia).
    pub fn scene_sync(&mut self, world: &mut World) {
        let epoch = world
            .get_resource::<crate::ProjectScripts>()
            .map(crate::ProjectScripts::epoch);
        if self.scene.follow_epoch(epoch) {
            // O que o projecto anterior imprimiu e ninguém leu nomeia os ids dele.
            let _ = self.console.drain();
        }
        let (lua, deadline, scene) = (self.runtime.lua(), &self.deadline, &mut self.scene);
        if world.contains_resource::<crate::ProjectScripts>() {
            world.resource_scope(
                |world, lib: bevy_ecs::change_detection::Mut<crate::ProjectScripts>| {
                    scene.sync(lua, deadline, world, &lib);
                },
            );
        } else {
            scene.sync(
                lua,
                deadline,
                world,
                &ph2d_script_doc::ScriptLibrary::default(),
            );
        }
    }

    /// ⭐⭐⭐ **Um passo fixo** sem portas para mestres ([`NoDoors`]) — os testes da crate; o
    /// produto passa as suas pelo [`Self::scene_tick_with`].
    pub fn scene_tick(&mut self, sim: &mut SimWorld, dt: f64) -> SceneReport {
        self.scene_tick_with(sim, dt, &mut NoDoors)
    }

    /// ⭐⭐⭐ **Um passo fixo** — ver [`SceneScripts::tick`]. Depois dele as bordas do teclado e do
    /// rato estão vistas, e o estado lateral de quem pediu para morrer sai.
    pub fn scene_tick_with(
        &mut self,
        sim: &mut SimWorld,
        dt: f64,
        doors: &mut dyn SceneDoors,
    ) -> SceneReport {
        let r = self
            .scene
            .tick(self.runtime.lua(), &self.deadline, sim, dt, doors);
        self.devices.edges_seen();
        self.forget_the_dead(&r);
        r
    }

    /// Os sinais deste quadro sem portas — ver [`Self::scene_hear_with`].
    pub fn scene_hear(&mut self, sim: &mut SimWorld, signals: &[&str]) -> SceneReport {
        self.scene_hear_with(sim, signals, &mut NoDoors)
    }

    /// Os sinais deste quadro — ver [`SceneScripts::hear`].
    pub fn scene_hear_with(
        &mut self,
        sim: &mut SimWorld,
        signals: &[&str],
        doors: &mut dyn SceneDoors,
    ) -> SceneReport {
        let r = self
            .scene
            .hear(self.runtime.lua(), &self.deadline, sim, signals, doors);
        self.forget_the_dead(&r);
        r
    }

    fn forget_the_dead(&self, r: &SceneReport) {
        for e in &r.despawned {
            self.state_table.clear(e.to_bits());
        }
    }

    /// ⭐ **Rebobinar é renascer** — devolve quantos objectos tinham vivo. As esperas, o estado
    /// lateral (é da corrida) e as bordas por ver vão com ela.
    pub fn scene_rewind(&mut self) -> usize {
        self.state_table.clear_all();
        self.devices.forget_edges();
        self.scene.rewind()
    }

    /// O que se sabe do script `id` — ver [`SceneScripts::info`].
    #[must_use]
    pub fn scene_info(&self, id: ph2d_script_doc::ScriptId) -> Option<&ScriptInfo> {
        self.scene.info(id)
    }

    /// Porque o objecto parou de correr o script — ver [`SceneScripts::failure`].
    #[must_use]
    pub fn scene_failure(&self, entity: Entity) -> Option<&crate::ScriptDiagnostic> {
        self.scene.failure(entity)
    }

    /// Os scripts da cena, para leitura (gates e diagnóstico).
    #[must_use]
    pub fn scene(&self) -> &SceneScripts {
        &self.scene
    }

    // ── CodeEditor E6/E10: a recarga e o console ─────────────────────────────────────────────

    /// ⭐ As recargas desde a última drenagem — ver [`SceneScripts::drain_reloads`].
    pub fn drain_reloads(&mut self) -> Vec<ScriptReload> {
        self.scene.drain_reloads()
    }

    /// Quantos objectos apontam `id` — ver [`SceneScripts::objects_using`].
    #[must_use]
    pub fn objects_using(&self, id: ph2d_script_doc::ScriptId) -> usize {
        self.scene.objects_using(id)
    }

    /// Recarrega `id` no próximo `scene_sync` mesmo sem mudança — ver
    /// [`SceneScripts::request_reload`].
    pub fn request_reload(&mut self, id: ph2d_script_doc::ScriptId) {
        self.scene.request_reload(id);
    }

    /// ⭐ As linhas do console por ler, da mais antiga à mais nova ([`crate::CONSOLE_LINES_MAX`]).
    pub fn drain_console(&self) -> Vec<ConsoleLine> {
        self.console.drain()
    }

    /// Quantas linhas do console o tecto deitou fora (desde o arranque).
    #[must_use]
    pub fn console_dropped(&self) -> u64 {
        self.console.dropped()
    }
}

/// Build a fresh `ScriptRuntime` with the `ph2d.*` Luau bindings
/// wired against the supplied bridge handles, then activate
/// `Lua::sandbox(true)`. Order is load-bearing: sandbox freezes the
/// globals + the `ph2d` namespace, so the API must be installed
/// FIRST. Activating sandbox last makes the surface read-only to
/// user scripts (HR-9).
#[allow(clippy::too_many_arguments)]
fn build_runtime(
    write_queue: &WriteQueue,
    input_snapshot: &InputSnapshot,
    devices: &Devices,
    state_table: &StateTable,
    emit_queue: &EmitQueue,
    deadline: &Deadline,
    console: &Console,
) -> mlua::Result<ScriptRuntime> {
    let runtime = ScriptRuntime::new()?;
    wire_ph2d_api(&runtime, write_queue, input_snapshot, state_table)?;
    wire_devices(runtime.lua(), devices)?;
    crate::lane::wire(runtime.lua())?;
    crate::stdlib::wire(runtime.lua())?;
    wire_scene_api(&runtime, emit_queue)?;
    crate::console::wire_print(runtime.lua(), console)?;
    install_deadline(runtime.lua(), deadline);
    runtime.lua().sandbox(true)?;
    Ok(runtime)
}

/// ⭐ TOP-20 #16 — `ph2d.emit(nome)`: um sinal no MESMO outbox de toda a casa.
///
/// ⚠️ **Vazio é erro, não silêncio**: a lei do produtor desta casa é *um produtor sem nome não
/// fala*, e num script um nome vazio é quase sempre uma variável por preencher — o painel tem de o
/// dizer, e um erro é a única forma de ele chegar lá.
fn wire_scene_api(runtime: &ScriptRuntime, emit_queue: &EmitQueue) -> mlua::Result<()> {
    let lua = runtime.lua();
    lua.set_app_data(emit_queue.clone());
    let emit_fn = lua.create_function(|lua, name: String| -> mlua::Result<()> {
        if name.trim().is_empty() {
            return Err(mlua::Error::RuntimeError(
                "ph2d.emit: the signal needs a name".into(),
            ));
        }
        let q = lua.app_data_ref::<EmitQueue>().ok_or_else(|| {
            mlua::Error::RuntimeError("ph2d.emit: EmitQueue not registered".into())
        })?;
        q.0.lock().unwrap_or_else(|p| p.into_inner()).push(name);
        Ok(())
    })?;
    let ph2d: mlua::Table = lua.globals().get("ph2d")?;
    ph2d.set("emit", emit_fn)?;
    crate::motion_api::wire(lua)
}

/// ⭐ **A interrupção que impede um laço sem saída de congelar o app** (§3.8 do plano).
///
/// ⚠️ **Lê o relógio uma vez em cada 256 interrupções**: o Luau interrompe em cada volta de laço
/// e em cada chamada, e um `Instant::now` por volta pagaria o relógio pela cena inteira. 256 voltas
/// de um laço vazio são microssegundos — o prazo continua a valer ao quadro.
fn install_deadline(lua: &mlua::Lua, deadline: &Deadline) {
    use std::sync::atomic::{AtomicU32, Ordering};
    let limit = deadline.0.clone();
    let ticks = std::sync::Arc::new(AtomicU32::new(0));
    lua.set_interrupt(move |lua| {
        let until = limit.load(Ordering::Relaxed);
        if until == u64::MAX || ticks.fetch_add(1, Ordering::Relaxed) & 0xFF != 0 {
            return Ok(mlua::VmState::Continue);
        }
        if crate::scene::now_nanos() > until {
            // A pilha só existe AQUI: o erro de uma interrupção volta sem ela (medido).
            let traceback = lua
                .traceback(None, 0)
                .map(|s| s.to_string_lossy())
                .unwrap_or_default();
            return Err(mlua::Error::external(crate::diagnostic::BudgetExceeded {
                traceback,
            }));
        }
        Ok(mlua::VmState::Continue)
    });
}

fn wire_ph2d_api(
    runtime: &ScriptRuntime,
    write_queue: &WriteQueue,
    input_snapshot: &InputSnapshot,
    state_table: &StateTable,
) -> mlua::Result<()> {
    let lua = runtime.lua();
    // Stash clones of the SHARED handles in app-data so the closures
    // below can grab them without capturing — `Lua::create_function`
    // requires `Send + 'static` bodies; app-data sidesteps the
    // closure-capture lifetimes. The clones share inner Arc storage,
    // so values the host pushes via `provide_input` / `provide_state`
    // are observable here even after a VM rebuild.
    lua.set_app_data(write_queue.clone());
    lua.set_app_data(input_snapshot.clone());
    lua.set_app_data(state_table.clone());

    let set_fn = lua.create_function(
        |lua, (entity, field, value): (u64, String, f64)| -> mlua::Result<()> {
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

    let input_fn = lua.create_function(|lua, key: String| -> mlua::Result<Option<f64>> {
        let s = lua.app_data_ref::<InputSnapshot>().ok_or_else(|| {
            mlua::Error::RuntimeError("ph2d.input: InputSnapshot not registered".into())
        })?;
        Ok(s.get(&key))
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
    // (function, thread, userdata) returns a Luau error; nil removes.
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
    ph2d.set("input", input_fn)?;
    ph2d.set("state_get", state_get_fn)?;
    ph2d.set("state_set", state_set_fn)?;
    ph2d.set("state_keys", state_keys_fn)?;
    ph2d.set("state_clear", state_clear_fn)?;
    // ⭐ TOP-20 #16 — as declarações de um script (só no topo; ver `crate::module`).
    ph2d.set("property", crate::module::property_binding(lua)?)?;
    // ⭐⭐ Os dois construtores de valor. ⚠️ Eles **não** são só para a declaração: a tabela que
    //    devolvem é a mesma que o script lê em `self` (ver `crate::valores`).
    ph2d.set("vec2", crate::valores::vec2_binding(lua)?)?;
    ph2d.set("color", crate::valores::color_binding(lua)?)?;

    Ok(())
}

/// O nome de uma tecla para o script, ou o erro que o diz (uma gralha não pode ler-se «solta»).
fn keys_of(func: &str, name: &str) -> mlua::Result<Vec<Key>> {
    Key::named_in_script(name).ok_or_else(|| {
        mlua::Error::RuntimeError(format!(
            "ph2d.{func}: unknown key '{name}' — names are lower case, like \"a\", \"space\", \"left\", \"shift\", \"f1\""
        ))
    })
}

fn button_of(func: &str, name: &str) -> mlua::Result<MouseButton> {
    MouseButton::from_script_name(name).ok_or_else(|| {
        mlua::Error::RuntimeError(format!(
            "ph2d.{func}: unknown mouse button '{name}' — \"left\", \"right\" or \"middle\""
        ))
    })
}

/// ⭐ **O teclado e o rato** (`docs/CodeEditor/05_api_que_funciona.md` §3.5).
fn wire_devices(lua: &mlua::Lua, devices: &Devices) -> mlua::Result<()> {
    let ph2d: mlua::Table = lua.globals().get("ph2d")?;
    for (func, edge) in [
        ("key_down", Edge::Down),
        ("key_pressed", Edge::Pressed),
        ("key_released", Edge::Released),
    ] {
        let d = devices.clone();
        let f = lua.create_function(move |_, name: String| {
            Ok(d.key(&keys_of(func, &name)?, edge))
        })?;
        ph2d.set(func, f)?;
    }
    for (func, edge) in [
        ("mouse_down", Edge::Down),
        ("mouse_pressed", Edge::Pressed),
        ("mouse_released", Edge::Released),
    ] {
        let d = devices.clone();
        let f = lua.create_function(move |_, name: String| {
            Ok(d.button(button_of(func, &name)?, edge))
        })?;
        ph2d.set(func, f)?;
    }
    let d = devices.clone();
    let pos = lua.create_function(move |_, ()| {
        Ok(match d.mouse_world() {
            Some([x, y]) => (Some(x), Some(y)),
            None => (None, None),
        })
    })?;
    ph2d.set("mouse_position", pos)
}
