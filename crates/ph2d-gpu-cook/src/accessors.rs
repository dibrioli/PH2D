//! **What the host can READ about the last cook** — the query surface of
//! [`GpuCook`], split from the crate root at the LOC cap (blindagem Fase 3.2).
//!
//! These are all `&self`, all constant-time reads of state the walk left behind:
//! the instance buffer the renderer binds, the per-node shape the graph panel
//! draws, the sim's tick, the pool's allocation counters. The WALK that produces
//! them lives in `lib.rs`; this is only its readout.

use crate::{GpuCook, GpuInstances, shape};
use ph2d_nodegraph::graph::NodeId;

impl GpuCook {
    pub fn new() -> Self {
        Self::default()
    }

    /// The instance buffer the LAST [`GpuCook::cook`] produced, if any — what the
    /// renderer binds. `None` before the first cook.
    pub fn instances(&self) -> Option<&GpuInstances> {
        self.instances.as_ref()
    }

    /// What the last [`GpuCook::cook`] produced, per staged node — the graph
    /// panel's only window into a GPU-resident frame. See [`shape::CookShape`].
    pub fn shape(&self) -> &shape::CookShape {
        &self.shape
    }

    /// How many elements `node` carried on the last [`GpuCook::cook`].
    pub fn node_count(&self, node: NodeId) -> Option<u32> {
        self.shape.count(node)
    }

    /// The column names `node`'s output carried on the last [`GpuCook::cook`].
    pub fn node_columns(&self, node: NodeId) -> Option<&[String]> {
        self.shape.columns(node)
    }

    /// The fixed tick this sim's state ([`GpuCook::prev`]) belongs to — the GPU
    /// mirror of `MotionCookPump::last_cooked_tick`, and the caller's input for
    /// "how many ticks do I owe?". `None` before the first sequential cook.
    ///
    /// A sequential trajectory is the SUM of its steps, so a caller must cook
    /// EVERY owed tick rather than one big jump — the same law the CPU pump
    /// states (`ticks_owed`: "forward: every tick, never a skip"), for the same
    /// reason: otherwise the motion depends on the frame rate.
    pub fn last_cooked_tick(&self) -> Option<u64> {
        self.last_tick
    }

    /// Column buffers the pool has ever created — flat across a steady scene,
    /// which is the whole claim of the ping-pong (D1) and is otherwise
    /// unobservable from outside.
    pub fn pool_allocations(&self) -> usize {
        self.pool.allocations()
    }

    /// Column buffers something still holds — for a sim, last tick's state.
    pub fn pool_retained(&self) -> usize {
        self.pool.retained()
    }
}
