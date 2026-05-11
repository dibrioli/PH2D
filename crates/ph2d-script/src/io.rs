//! Buffered write queue + read snapshot — the only legal way for Luau
//! scripts to talk to ECS state.
//!
//! Why buffered: scripts run inside `Lua::exec`, which holds the
//! single mlua VM lock. Letting Luau callbacks reach into bevy_ecs
//! mid-tick is racy and crosses the sim/script boundary. Instead:
//! - Before the script tick, the host writes the current snapshot
//!   (entity → field → value) into `ReadSnapshot`.
//! - During the tick, `ph2d.get` reads from the snapshot, `ph2d.set`
//!   pushes an `EntityWrite` into `WriteQueue`.
//! - After the tick, the host drains the queue and applies the writes
//!   to ECS in one pass.
//!
//! This is the same pattern Defold and Roblox use for script ↔ engine
//! marshalling (HR-8 keeps scripts pure-data at the FFI boundary).
//!
//! ### Backpressure / DoS resistance
//!
//! [`WriteQueue::push`] enforces [`WriteQueue::DEFAULT_CAP`] (250k
//! pending writes). A malicious or runaway script doing
//! `while true do ph2d.set(...) end` triggers [`QueueFull`] instead
//! of OOMing the host. The `ph2d.set` Lua binding surfaces this as
//! a Luau-level error so the script fails fast and the host stays
//! responsive.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq)]
pub struct EntityWrite {
    pub entity: u32,
    pub field: String,
    pub value: f64,
}

/// Returned by [`WriteQueue::push`] when the cap is reached. Carried
/// across the FFI boundary as a Luau runtime error in `ph2d.set`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueueFull {
    pub cap: usize,
}

impl std::fmt::Display for QueueFull {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "WriteQueue full ({} pending writes); script must yield or batch fewer ph2d.set calls per tick",
            self.cap
        )
    }
}

impl std::error::Error for QueueFull {}

/// Write-side: Luau `ph2d.set` enqueues here. Cloning is cheap (Arc),
/// so the host stashes one clone in `Lua::set_app_data` and keeps
/// another for `drain` after the tick.
#[derive(Clone)]
pub struct WriteQueue {
    inner: Arc<Mutex<Vec<EntityWrite>>>,
    cap: usize,
}

impl WriteQueue {
    /// Default ceiling on pending writes. 250k = 2500 entities × 100
    /// fields, comfortably above realistic per-tick budgets and
    /// 5-10× below the OOM threshold on a typical desktop.
    pub const DEFAULT_CAP: usize = 250_000;

    pub fn new() -> Self {
        Self::with_cap(Self::DEFAULT_CAP)
    }

    pub fn with_cap(cap: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Vec::new())),
            cap,
        }
    }

    /// Enqueue a write. Returns [`QueueFull`] if the cap is reached.
    pub fn push(&self, w: EntityWrite) -> Result<(), QueueFull> {
        let mut g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        if g.len() >= self.cap {
            return Err(QueueFull { cap: self.cap });
        }
        g.push(w);
        Ok(())
    }

    /// Take all pending writes; the queue is empty after this call.
    pub fn drain(&self) -> Vec<EntityWrite> {
        std::mem::take(&mut *self.inner.lock().unwrap_or_else(|p| p.into_inner()))
    }

    pub fn len(&self) -> usize {
        self.inner.lock().unwrap_or_else(|p| p.into_inner()).len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .is_empty()
    }

    pub fn cap(&self) -> usize {
        self.cap
    }
}

impl Default for WriteQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Read-side: the host populates this with the current ECS snapshot
/// before each script tick. `ph2d.get` reads from it. Same Arc-clone
/// pattern as `WriteQueue`. BTreeMap (not HashMap) per ADR-0022 —
/// ph2d-script feeds sim systems, so it inherits the ban.
///
/// Poison recovery: mutex acquisitions use `unwrap_or_else(into_inner)`
/// so a panic in another thread holding the lock degrades gracefully
/// (potentially-stale snapshot, but engine keeps running) instead of
/// cascade-crashing every subsequent `ph2d.get`.
#[derive(Clone, Default)]
pub struct ReadSnapshot {
    inner: Arc<Mutex<BTreeMap<(u32, String), f64>>>,
}

impl ReadSnapshot {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&self, entity: u32, field: &str, value: f64) {
        self.inner
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert((entity, field.to_owned()), value);
    }

    pub fn get(&self, entity: u32, field: &str) -> Option<f64> {
        self.inner
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(&(entity, field.to_owned()))
            .copied()
    }

    pub fn clear(&self) {
        self.inner.lock().unwrap_or_else(|p| p.into_inner()).clear();
    }

    pub fn len(&self) -> usize {
        self.inner.lock().unwrap_or_else(|p| p.into_inner()).len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .is_empty()
    }
}

/// Per-frame input snapshot consulted by `ph2d.input(key)` from Luau.
/// Distinct from [`ReadSnapshot`] because keys are flat strings
/// (e.g. `"gamepad.held.south"`, `"gamepad.axis.left_stick_x"`)
/// rather than `(entity, field)` pairs — input isn't entity-scoped.
///
/// The host (shells/desktop) populates this each frame from
/// `ph2d_input::InputState`. M8 ships gamepad + Pencil-stub keys;
/// future devices (e.g. mouse wheel, IMU) just add new keys.
#[derive(Clone, Default)]
pub struct InputSnapshot {
    inner: Arc<Mutex<BTreeMap<String, f64>>>,
}

impl InputSnapshot {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&self, key: &str, value: f64) {
        self.inner
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(key.to_owned(), value);
    }

    pub fn get(&self, key: &str) -> Option<f64> {
        self.inner
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(key)
            .copied()
    }

    /// Drop every key. Call before re-populating per-frame so stale
    /// "pressed" edges from the previous frame don't leak.
    pub fn clear(&self) {
        self.inner.lock().unwrap_or_else(|p| p.into_inner()).clear();
    }

    pub fn len(&self) -> usize {
        self.inner.lock().unwrap_or_else(|p| p.into_inner()).len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .is_empty()
    }
}

/// One queued spawn/despawn/attach command from a Luau callback.
///
/// `ph2d.spawn` / `ph2d.despawn` / `ph2d.attach_script` push into the
/// shared [`SpawnQueue`]; the host drains and applies after the tick
/// finishes. Same backpressure model as [`WriteQueue`] — keeps the
/// FFI boundary pure-data (HR-8) and the engine in charge of when
/// world mutations happen.
#[derive(Clone, Debug, PartialEq)]
pub enum SpawnCommand {
    /// Spawn an empty entity (placeholder until M14.3 Prefab assets
    /// arrive — that milestone replaces this variant with
    /// `SpawnPrefab { prefab: AssetId, transform: Transform }`).
    SpawnEmpty,
    /// Spawn an empty entity and tag it with a `Name(name)` so the
    /// next-frame `ph2d.find_by_name(name)` lookup can locate it.
    /// The `lateral_key` rendezvous slot used to publish the assigned
    /// entity id back to scripts is left for a later milestone — for
    /// now the script tags by name and re-queries.
    SpawnNamed { name: String },
    /// Despawn an entity by `Entity::to_bits()` value. Silently
    /// ignored if the entity no longer exists.
    Despawn { entity: u64 },
    /// Attach a `LuauScript` to an existing entity. `lateral_key` is
    /// derived deterministically on the host side via
    /// `LuauScript::derive_lateral_key` so the state survives reload.
    AttachScript {
        entity: u64,
        bytecode: [u8; 32], // raw blake3 digest — AssetId postcard tail
    },
}

/// Backpressure error returned by [`SpawnQueue::push`] when the cap
/// is hit. Same shape as [`QueueFull`] but distinct so the script
/// error message is specific.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnQueueFull {
    pub cap: usize,
}

impl std::fmt::Display for SpawnQueueFull {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SpawnQueue full ({} pending commands); script must yield before next spawn",
            self.cap
        )
    }
}

impl std::error::Error for SpawnQueueFull {}

/// Queue of pending world-mutation commands emitted by Luau bindings
/// (`ph2d.spawn`, `ph2d.despawn`, `ph2d.attach_script`). Mirrors the
/// [`WriteQueue`] pattern: cloned `Arc<Mutex<...>>` shared between
/// the host and the Lua `app_data` slot; drained by the host every
/// tick.
///
/// Cap default = 1024 pending commands — comfortably above the
/// per-tick spawn rate of any reasonable scene; a "spawn bomb"
/// script bumps into this instead of OOM-ing the host.
#[derive(Clone)]
pub struct SpawnQueue {
    inner: Arc<Mutex<Vec<SpawnCommand>>>,
    cap: usize,
}

impl SpawnQueue {
    /// Default cap. Override with [`SpawnQueue::with_cap`] for tests.
    pub const DEFAULT_CAP: usize = 1024;

    pub fn new() -> Self {
        Self::with_cap(Self::DEFAULT_CAP)
    }

    pub fn with_cap(cap: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Vec::new())),
            cap,
        }
    }

    pub fn push(&self, cmd: SpawnCommand) -> Result<(), SpawnQueueFull> {
        let mut g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        if g.len() >= self.cap {
            return Err(SpawnQueueFull { cap: self.cap });
        }
        g.push(cmd);
        Ok(())
    }

    pub fn drain(&self) -> Vec<SpawnCommand> {
        std::mem::take(&mut *self.inner.lock().unwrap_or_else(|p| p.into_inner()))
    }

    pub fn len(&self) -> usize {
        self.inner.lock().unwrap_or_else(|p| p.into_inner()).len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .is_empty()
    }

    pub fn cap(&self) -> usize {
        self.cap
    }
}

impl Default for SpawnQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Per-frame snapshot of `(name -> entity_id)` so Luau can resolve
/// `ph2d.find_by_name("Player")` without crossing the world handle.
/// Populated by the host from `Query<(Entity, &Name)>` before tick.
#[derive(Clone, Default)]
pub struct NameSnapshot {
    inner: Arc<Mutex<BTreeMap<String, u64>>>,
}

impl NameSnapshot {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&self, name: &str, entity: u64) {
        self.inner
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(name.to_owned(), entity);
    }

    pub fn get(&self, name: &str) -> Option<u64> {
        self.inner
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(name)
            .copied()
    }

    pub fn clear(&self) {
        self.inner.lock().unwrap_or_else(|p| p.into_inner()).clear();
    }

    pub fn len(&self) -> usize {
        self.inner.lock().unwrap_or_else(|p| p.into_inner()).len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ew(e: u32, f: &str, v: f64) -> EntityWrite {
        EntityWrite {
            entity: e,
            field: f.into(),
            value: v,
        }
    }

    #[test]
    fn write_queue_drain_resets() {
        let q = WriteQueue::new();
        q.push(ew(1, "x", 1.0)).unwrap();
        q.push(ew(2, "y", 2.0)).unwrap();
        let drained = q.drain();
        assert_eq!(drained.len(), 2);
        assert!(q.is_empty());
    }

    #[test]
    fn read_snapshot_round_trips() {
        let s = ReadSnapshot::new();
        s.set(7, "x", 7.25);
        assert_eq!(s.get(7, "x"), Some(7.25));
        assert_eq!(s.get(7, "y"), None);
        assert_eq!(s.get(8, "x"), None);
        s.clear();
        assert_eq!(s.get(7, "x"), None);
    }

    #[test]
    fn clones_share_inner_storage() {
        let q1 = WriteQueue::new();
        let q2 = q1.clone();
        q1.push(ew(0, "z", 9.0)).unwrap();
        assert_eq!(q2.len(), 1);
    }

    #[test]
    fn input_snapshot_round_trips() {
        let s = InputSnapshot::new();
        s.set("gamepad.held.south", 1.0);
        s.set("gamepad.axis.left_stick_x", -0.42);
        assert_eq!(s.get("gamepad.held.south"), Some(1.0));
        assert!((s.get("gamepad.axis.left_stick_x").unwrap() + 0.42).abs() < 1e-6);
        assert_eq!(s.get("gamepad.held.north"), None);
        s.clear();
        assert!(s.is_empty());
    }

    #[test]
    fn push_returns_queue_full_at_cap() {
        let q = WriteQueue::with_cap(3);
        assert!(q.push(ew(1, "x", 1.0)).is_ok());
        assert!(q.push(ew(2, "x", 2.0)).is_ok());
        assert!(q.push(ew(3, "x", 3.0)).is_ok());
        let err = q.push(ew(4, "x", 4.0)).unwrap_err();
        assert_eq!(err, QueueFull { cap: 3 });
        // Drain re-opens capacity.
        assert_eq!(q.drain().len(), 3);
        assert!(q.push(ew(5, "x", 5.0)).is_ok());
    }

    #[test]
    fn spawn_queue_drain_resets() {
        let q = SpawnQueue::new();
        q.push(SpawnCommand::SpawnEmpty).unwrap();
        q.push(SpawnCommand::Despawn { entity: 7 }).unwrap();
        let drained = q.drain();
        assert_eq!(drained.len(), 2);
        assert!(q.is_empty());
        assert_eq!(drained[0], SpawnCommand::SpawnEmpty);
        assert_eq!(drained[1], SpawnCommand::Despawn { entity: 7 });
    }

    #[test]
    fn spawn_queue_caps() {
        let q = SpawnQueue::with_cap(2);
        assert!(q.push(SpawnCommand::SpawnEmpty).is_ok());
        assert!(
            q.push(SpawnCommand::SpawnNamed { name: "x".into() })
                .is_ok()
        );
        let err = q.push(SpawnCommand::SpawnEmpty).unwrap_err();
        assert_eq!(err, SpawnQueueFull { cap: 2 });
    }

    #[test]
    fn name_snapshot_round_trip() {
        let s = NameSnapshot::new();
        s.set("Player", 1234);
        assert_eq!(s.get("Player"), Some(1234));
        assert_eq!(s.get("Enemy"), None);
        s.clear();
        assert_eq!(s.get("Player"), None);
    }
}
