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

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq)]
pub struct EntityWrite {
    pub entity: u32,
    pub field: String,
    pub value: f64,
}

/// Write-side: Luau `ph2d.set` enqueues here. Cloning is cheap (Arc),
/// so the host stashes one clone in `Lua::set_app_data` and keeps
/// another for `drain` after the tick.
#[derive(Clone, Default)]
pub struct WriteQueue {
    inner: Arc<Mutex<Vec<EntityWrite>>>,
}

impl WriteQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&self, w: EntityWrite) {
        self.inner.lock().expect("WriteQueue poisoned").push(w);
    }

    /// Take all pending writes; the queue is empty after this call.
    pub fn drain(&self) -> Vec<EntityWrite> {
        std::mem::take(&mut *self.inner.lock().expect("WriteQueue poisoned"))
    }

    pub fn len(&self) -> usize {
        self.inner.lock().expect("WriteQueue poisoned").len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.lock().expect("WriteQueue poisoned").is_empty()
    }
}

/// Read-side: the host populates this with the current ECS snapshot
/// before each script tick. `ph2d.get` reads from it. Same Arc-clone
/// pattern as `WriteQueue`. BTreeMap (not HashMap) per ADR-0022 —
/// ph2d-script feeds sim systems, so it inherits the ban.
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
            .expect("ReadSnapshot poisoned")
            .insert((entity, field.to_owned()), value);
    }

    pub fn get(&self, entity: u32, field: &str) -> Option<f64> {
        self.inner
            .lock()
            .expect("ReadSnapshot poisoned")
            .get(&(entity, field.to_owned()))
            .copied()
    }

    pub fn clear(&self) {
        self.inner.lock().expect("ReadSnapshot poisoned").clear();
    }

    pub fn len(&self) -> usize {
        self.inner.lock().expect("ReadSnapshot poisoned").len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.lock().expect("ReadSnapshot poisoned").is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_queue_drain_resets() {
        let q = WriteQueue::new();
        q.push(EntityWrite {
            entity: 1,
            field: "x".into(),
            value: 1.0,
        });
        q.push(EntityWrite {
            entity: 2,
            field: "y".into(),
            value: 2.0,
        });
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
        q1.push(EntityWrite {
            entity: 0,
            field: "z".into(),
            value: 9.0,
        });
        assert_eq!(q2.len(), 1);
    }
}
