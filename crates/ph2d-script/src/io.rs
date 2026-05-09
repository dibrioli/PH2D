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
}
