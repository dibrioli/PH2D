#![forbid(unsafe_code)]
//! `ph2d-eval-motion` — the **Motion domain evaluator** (ADR-0034).
//!
//! Cooks a motion graph (pull, at the playhead) via [`ph2d_nodegraph::cook`]
//! and **lowers** the resulting instance [`Stream`] to `ph2d-render`
//! [`RenderInstance`]s. The data path (`graph → Vec<RenderInstance>`) is
//! **headless** — it constructs plain POD instances and needs no GPU; the
//! upload + draw (`InstanceBuffer::upload`) is the shell's job (the visual
//! smoke). This is the pull-side, HR-5-exempt presentation path (motion is
//! purely visual; only the gameplay domain is bound by determinism).
//!
//! ## Instance stream convention
//!
//! A Motion instance stream carries named columns (SoA); a missing column
//! falls back to a sensible default, so a node only writes what it changes:
//!
//! | column    | type   | → RenderInstance field   | default                    |
//! |-----------|--------|--------------------------|----------------------------|
//! | `P`       | Vec2   | `world_pos`              | `[0,0]`                    |
//! | `size`    | Vec2   | `size`                   | caller's `default_size`    |
//! | `rot`     | Scalar | `basis` (rotation **deg**) | `0` → identity            |
//! | `tint`    | Vec4   | `tint` (rgba)            | `[1,1,1,1]`                |
//! | `uv_rect` | Vec4   | `atlas_uv` (u0,v0,u1,v1) | caller's `default_uv_rect` |
//!
//! `anchor`/`texture_id`/`premultiplied` use the shared-atlas defaults. A
//! stream with no `uv_rect` column takes the caller-supplied `default_uv_rect`
//! — the shell passes the atlas tile the default document should sample (a
//! single opaque tile, so the raw M0 output reads as clean solid quads rather
//! than a whole-atlas thumbnail), while a headless caller passes the
//! whole-atlas rect. Richer cloners can extend the convention later.
//!
//! Producer coverage so far: the W2 Motion vertical (`motion.grid` →
//! `transform` → `clone`) emits only `P`; `size`/`rot`/`tint`/`uv_rect` are
//! reserved columns of the convention with no producer node yet (a framing node
//! that writes `uv_rect`/`cell` lands in M1). Their lowering is covered by the
//! unit tests below. A later node that sets them needs no change here — the
//! lowering already reads them.

use ph2d_nodegraph::attr::{Column, PAR_THRESHOLD, Stream};
use ph2d_nodegraph::cook::{Cook, CookError, OpResolver, TimeScopes};
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_render::RenderInstance;

mod checkpoint;
pub use checkpoint::{CPU_RING_BYTES, CheckpointRing, RECENT_DENSE};

mod lower;
pub use lower::{
    evaluate_motion, evaluate_motion_into, lower_to_instances, lower_to_instances_into,
    lower_to_instances_onto,
};

/// Per-frame Motion cook driver (plan §1.8). Owns the persistent [`Cook`] (its
/// memo + `pre` feedback must survive across frames) and the reused instance
/// buffer, and re-cooks the sink **only when the frame is dirty**: the transport
/// tick advanced (playing / scrub), the graph was edited ([`Self::mark_dirty`]),
/// or it is the first cook. A paused, unchanged frame skips the cook entirely,
/// so it touches no heap — the [`Cook`] otherwise re-evaluates each node every
/// call (it is NOT playhead-memoized for a `Pure` graph). Gated by
/// `tests/paused_no_alloc.rs` (M0.T12).
pub struct MotionCookPump {
    /// Persistent cook — its memo + `pre`-edge feedback are carried across
    /// frames, so it must NOT be re-created each frame.
    pub cook: Cook,
    /// Reused per-frame lowering buffer — zero-alloc in steady state.
    pub instances: Vec<RenderInstance>,
    /// The transport tick `instances` currently reflects (`None` = never cooked).
    last_cooked_tick: Option<u64>,
    /// Set by a graph edit so the next frame re-cooks even at the same tick.
    dirty: bool,
    /// The last cook error, if this frame's cook refused a sink. Kept so the
    /// shell can explain a dark scene instead of leaving the artist guessing.
    last_error: Option<CookError>,
    /// Backwards-scrub cache (M2.N2): one checkpoint per forward tick. A scrub
    /// to an earlier tick restores from here instead of reading the marching
    /// future. Cleared on a graph edit (the cached sim is invalid for the new
    /// graph). See [`Self::scrub_to_scoped`].
    ring: CheckpointRing,
    /// The output streams of the nodes cooked by
    /// [`Self::advance_or_scrub_to_nodes_scoped`] (the GPU/M5 boundary hand-off),
    /// held so the caller can upload them after the tick march. Empty until that
    /// method cooks; NOT touched by the sink pump (which lowers to `instances`).
    ///
    /// **Plural, and labelled by node.** A plan's staged region can have two
    /// uncovered inputs on two ports, which leaves two boundaries — and the GPU
    /// sequencer validates the set it is handed against `plan.boundaries`, so
    /// which stream came from which node is part of the answer, not bookkeeping.
    boundary_streams: Vec<(NodeId, Stream)>,
}

/// What one pump cook produces this frame — the two consumers of the SAME
/// forward/scrub tick march + `pre` feedback, kept as one path so a "fast
/// preview" boundary cook can never drift from the sink cook
/// ([[feedback_two_doors_to_the_same_question_diverge]]).
enum CookTarget<'a> {
    /// The render sinks, lowered into `instances` (the CPU render path).
    Sinks {
        sinks: &'a [NodeId],
        default_uv_rect: [f32; 4],
        default_size: [f32; 2],
    },
    /// Each named node's raw output stream, stored in `boundary_streams` (the
    /// CPU→GPU boundary: the lowering is the GPU's job now, so the CPU stops at
    /// the stream).
    Boundaries(&'a [NodeId]),
}

impl CookTarget<'_> {
    /// Is there work to cook? Drives the once-per-frame `pre`-feedback advance
    /// (a boundary node is always work; an empty sink list is not).
    fn has_work(&self) -> bool {
        match self {
            CookTarget::Sinks { sinks, .. } => !sinks.is_empty(),
            CookTarget::Boundaries(nodes) => !nodes.is_empty(),
        }
    }
}

impl Default for MotionCookPump {
    fn default() -> Self {
        Self::new()
    }
}

impl MotionCookPump {
    /// A fresh pump: empty cook + buffer, marked dirty so the first
    /// [`Self::pump`] cooks.
    pub fn new() -> Self {
        Self {
            cook: Cook::new(),
            instances: Vec::new(),
            last_cooked_tick: None,
            dirty: true,
            last_error: None,
            ring: CheckpointRing::new(),
            boundary_streams: Vec::new(),
        }
    }

    /// Force a re-cook on the next [`Self::pump`], even at the same tick (call
    /// after editing the graph while paused). The M1 graph-edit path. Also
    /// **invalidates the scrub cache**: the recorded checkpoints are the sim of
    /// the OLD graph, so a later scrub must re-sim from the tick-0 seed under the
    /// edited graph (Blender/Houdini "edit invalidates the cache").
    /// Retune the scrub ring's byte budget (default `CPU_RING_BYTES`) — the
    /// mirror of `GpuCook::set_ring_budget`, and what lets a gate squeeze the
    /// ring into the eviction regime a real heavy scene lives in (ADR-0137).
    pub fn set_ring_budget(&mut self, bytes: usize) {
        self.ring.set_budget(bytes);
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
        self.ring.clear();
    }

    /// Whether the next [`Self::pump`] will re-cook. Exposed so an edit that must
    /// NOT re-cook can prove it: the editor's decoration (the graph panel's group
    /// backdrops) is document state but cannot change what the graph cooks, and a
    /// stray `mark_dirty` on a backdrop drag would re-cook the whole graph every
    /// frame of the gesture. A test asserts this stays `false` across such an edit.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Re-cook every sink in `sinks` into `instances` at `playhead` **iff** the
    /// frame is dirty (the `tick` changed since the last cook,
    /// [`Self::mark_dirty`] was called, or this is the first cook). A clean,
    /// unchanged frame (paused, no edit) leaves the buffer untouched — zero
    /// allocation. Returns `true` if it cooked. Reuse the SAME pump across
    /// frames for the steady state.
    ///
    /// **Several sinks compose into one draw**: each `motion.output` node in the
    /// document lowers onto the shared buffer, in the order given (the shell
    /// scans by node id, so it is deterministic). A document may therefore hold
    /// independent scenes — a grid rig and a particle fountain — without a
    /// stream-merging node. An empty `sinks` clears the buffer (nothing renders).
    #[allow(clippy::too_many_arguments)] // graph + resolver + sinks + tick + playhead + 2 defaults
    pub fn pump(
        &mut self,
        graph: &Graph,
        ops: &dyn OpResolver,
        sinks: &[NodeId],
        tick: u64,
        playhead: f64,
        default_uv_rect: [f32; 4],
        default_size: [f32; 2],
    ) -> bool {
        self.pump_scoped(
            graph,
            ops,
            sinks,
            tick,
            playhead,
            default_uv_rect,
            default_size,
            &TimeScopes::new(),
        )
    }

    /// [`Self::pump`] under time scopes (M2.N1): every `motion.time_remap` node
    /// rewrites the clock of its upstream subtree. Build `scopes` with
    /// `ph2d_node_motion_time_remap::time_scopes` — the substrate keys scopes by
    /// `NodeId` and knows no node types, so the caller owns that translation.
    /// An empty map behaves exactly like [`Self::pump`].
    #[allow(clippy::too_many_arguments)] // `pump` + the scopes; one shell call site
    pub fn pump_scoped(
        &mut self,
        graph: &Graph,
        ops: &dyn OpResolver,
        sinks: &[NodeId],
        tick: u64,
        playhead: f64,
        default_uv_rect: [f32; 4],
        default_size: [f32; 2],
        scopes: &TimeScopes,
    ) -> bool {
        self.pump_target_scoped(
            graph,
            ops,
            &CookTarget::Sinks {
                sinks,
                default_uv_rect,
                default_size,
            },
            tick,
            playhead,
            scopes,
        )
    }

    /// The forward cook for either target (sinks-lower or boundary-stream) — the
    /// one place the dirty gate, the ring record and the `pre`-feedback advance
    /// live, so both callers march identically.
    fn pump_target_scoped(
        &mut self,
        graph: &Graph,
        ops: &dyn OpResolver,
        target: &CookTarget,
        tick: u64,
        playhead: f64,
        scopes: &TimeScopes,
    ) -> bool {
        if !self.dirty && self.last_cooked_tick == Some(tick) {
            return false; // paused + unchanged → reuse the buffer, no heap traffic
        }
        // Record the checkpoint that reproduces THIS frame — captured before the
        // cook, only on a genuine forward tick change (a same-tick re-cook after
        // an edit holds the next tick's state, not this one). This is the ring a
        // backwards scrub restores from (M2.N2).
        if self.last_cooked_tick != Some(tick) {
            self.ring.record(tick, self.cook.checkpoint());
        }
        self.cook_target_into(graph, ops, target, playhead, scopes);
        if target.has_work() {
            // Advance the 1-tick `pre` feedback once per cooked frame — ONCE for
            // the whole graph, not per sink (each sink's `pre` sources are
            // snapshotted by the same call).
            let _ = self.cook.advance_tick_scoped(graph, ops, playhead, scopes);
        }
        self.last_cooked_tick = Some(tick);
        self.dirty = false;
        true
    }

    /// Cook one target at `playhead`: either every sink lowered into `instances`
    /// (clears + refills, keeping capacity) or one node's stream into
    /// `boundary_stream`. Shared by the forward [`Self::pump_target_scoped`] and
    /// the backwards [`Self::scrub_target_scoped`] so both take the IDENTICAL cook
    /// path — a "fast preview" scrub that diverged from playback is the classic
    /// determinism trap (the state-of-the-art survey's warning).
    fn cook_target_into(
        &mut self,
        graph: &Graph,
        ops: &dyn OpResolver,
        target: &CookTarget,
        playhead: f64,
        scopes: &TimeScopes,
    ) {
        self.last_error = None;
        match *target {
            CookTarget::Sinks {
                sinks,
                default_uv_rect,
                default_size,
            } => {
                self.instances.clear();
                for &sink in sinks {
                    // A sink that fails to cook (an unknown type mid-edit, or a
                    // sequential node caught inside a remapped time scope)
                    // contributes nothing; the others still draw. The error is
                    // kept for the shell.
                    match self.cook.cook_scoped(graph, ops, sink, playhead, scopes) {
                        Ok(outputs) => {
                            if let Some(v) = outputs.first() {
                                lower_to_instances_onto(
                                    v.as_stream(),
                                    default_uv_rect,
                                    default_size,
                                    &mut self.instances,
                                );
                            }
                        }
                        Err(e) => self.last_error = Some(e),
                    }
                }
            }
            CookTarget::Boundaries(nodes) => {
                // The CPU prefix's hand-off to the GPU: cook each boundary node on
                // the SAME canonical `Cook` (memo + `pre` feedback), then hold the
                // output streams for the caller to upload. No lowering — that runs
                // on the GPU.
                //
                // **The loop is inside the consume, so the march outside it runs
                // ONCE.** Advancing the clock per boundary is the trap that kept
                // this singular; the `pre` feedback and the forward/scrub decision
                // live in the caller, and only this step is per target.
                //
                // Cooking node B after node A at the SAME playhead hits the memo on
                // everything they share: `Fingerprint` carries
                // `tick: consumes_pre.then_some(self.tick)` and `self.tick` only
                // moves in `advance_tick_scoped`, which the march calls once per
                // frame, outside here. So a shared sequential prefix is simulated
                // once, not once per boundary (gated: the eval COUNT, not a timer).
                self.boundary_streams.clear();
                for &node in nodes {
                    // `plan.boundaries` pushes per PORT, so one CPU node feeding two
                    // staged nodes is named twice. Hand it over once: the GPU keys
                    // its uploads by node, and the duplicate would at best waste an
                    // upload.
                    if self.boundary_streams.iter().any(|(n, _)| *n == node) {
                        continue;
                    }
                    // A boundary that fails to cook (an unknown type mid-edit)
                    // contributes nothing and the others still hand over — dropping
                    // all of them would turn one bad node into a black frame. The
                    // caller sees a short set, disagrees with `plan.boundaries`, and
                    // falls back cleanly.
                    match self.cook.cook_scoped(graph, ops, node, playhead, scopes) {
                        Ok(outputs) => {
                            if let Some(v) = outputs.first() {
                                self.boundary_streams.push((node, v.as_stream().clone()));
                            }
                        }
                        Err(e) => self.last_error = Some(e),
                    }
                }
            }
        }
    }

    /// Scrub to `target_tick`: render the exact simulation state of that frame
    /// even when it is BEHIND the current playhead (plan §1.4, M2.N2). A plain
    /// forward cook would read the marching-future `pre` state; this restores the
    /// newest checkpoint ≤ target from the ring (or the tick-0 seed) and re-cooks
    /// forward to the target — bit-exact, because the re-sim walks the identical
    /// cook path as playback (GGPO save/load/advance). `playhead_of(tick)` maps a
    /// tick to its seconds (the transport's `tick × fixed_dt`).
    ///
    /// Recent scrubs are an `O(1)` restore with zero re-sim (the dense window);
    /// a target older than the window re-sims from the seed. Returns `true` once
    /// it has rendered `target_tick` into `instances`.
    #[allow(clippy::too_many_arguments)]
    pub fn scrub_to_scoped(
        &mut self,
        graph: &Graph,
        ops: &dyn OpResolver,
        sinks: &[NodeId],
        target_tick: u64,
        playhead_of: impl Fn(u64) -> f64,
        default_uv_rect: [f32; 4],
        default_size: [f32; 2],
        scopes: &TimeScopes,
    ) -> bool {
        self.scrub_target_scoped(
            graph,
            ops,
            &CookTarget::Sinks {
                sinks,
                default_uv_rect,
                default_size,
            },
            target_tick,
            playhead_of,
            scopes,
        )
    }

    /// The backwards re-sim for either target — restores the newest checkpoint
    /// ≤ target and re-cooks forward, the same GGPO path for sinks and boundary.
    fn scrub_target_scoped(
        &mut self,
        graph: &Graph,
        ops: &dyn OpResolver,
        target: &CookTarget,
        target_tick: u64,
        playhead_of: impl Fn(u64) -> f64,
        scopes: &TimeScopes,
    ) -> bool {
        let (anchor, cp) = self.ring.anchor_at_or_before(target_tick);
        self.cook.restore(&cp);
        let mut t = anchor;
        loop {
            let playhead = playhead_of(t);
            // Record the state that reproduces frame `t` (before its cook), so a
            // re-sim past the window rebuilds the ring; a within-window tick is
            // already covered and the deep clone is skipped.
            if self.ring.should_record(t) {
                self.ring.record(t, self.cook.checkpoint());
            }
            self.cook_target_into(graph, ops, target, playhead, scopes);
            if !target.has_work() {
                break;
            }
            // Advance the `pre` feedback exactly as the forward pump does — so
            // after rendering the target the cook is left ready for `target+1`,
            // and resumed playback continues bit-exact (no off-by-one).
            let _ = self.cook.advance_tick_scoped(graph, ops, playhead, scopes);
            if t == target_tick {
                break;
            }
            t += 1;
        }
        self.last_cooked_tick = Some(target_tick);
        self.dirty = false;
        true
    }

    /// Render `tick` correctly whether it is a **forward step** or a **jump**
    /// (backwards scrub, a loop-wrap, a ruler seek): a contiguous forward tick
    /// (or a same-tick re-cook) takes the cheap forward [`Self::pump_scoped`]; a
    /// tick that moved backwards or skipped ahead restores from the ring and
    /// re-sims via [`Self::scrub_to_scoped`] (M2.N2). One entry point, so the
    /// shell never branches — a future timeline ruler that sets `transport.tick`
    /// is handled for free, and a `loop_range` wrap replays the sim from `lo`
    /// instead of showing the marching-future state. `playhead_of` maps a tick
    /// to seconds (`tick × fixed_dt`).
    #[allow(clippy::too_many_arguments)]
    pub fn advance_or_scrub_scoped(
        &mut self,
        graph: &Graph,
        ops: &dyn OpResolver,
        sinks: &[NodeId],
        tick: u64,
        playhead_of: impl Fn(u64) -> f64,
        default_uv_rect: [f32; 4],
        default_size: [f32; 2],
        scopes: &TimeScopes,
    ) -> bool {
        self.advance_or_scrub_target_scoped(
            graph,
            ops,
            &CookTarget::Sinks {
                sinks,
                default_uv_rect,
                default_size,
            },
            tick,
            playhead_of,
            scopes,
        )
    }

    /// Render `tick` into `boundary_streams` (NOT `instances`): cook the CPU
    /// prefix up to each of `nodes` on the persistent pump — marching every owed
    /// tick, so a sequential prefix node (`integrate`/`emitter`) sims correctly
    /// and a scrub is bit-exact — then leave their output streams in
    /// [`Self::boundary_streams`] for the GPU sequencer to upload (GPU/M5, the
    /// hybrid CPU-prefix / GPU-suffix cook). The forward/scrub decision, the ring
    /// and the `pre` feedback are the SAME as the sink pump; only the consume
    /// step differs (streams vs lowered instances).
    ///
    /// **One march, N hand-offs.** `nodes` is `plan.boundaries` — pass the whole
    /// set, never one node per call: calling this per boundary would advance the
    /// clock once per boundary and simulate the shared prefix once per boundary.
    /// Duplicates in the set are handed over once.
    #[allow(clippy::too_many_arguments)]
    pub fn advance_or_scrub_to_nodes_scoped(
        &mut self,
        graph: &Graph,
        ops: &dyn OpResolver,
        nodes: &[NodeId],
        tick: u64,
        playhead_of: impl Fn(u64) -> f64,
        scopes: &TimeScopes,
    ) -> bool {
        self.advance_or_scrub_target_scoped(
            graph,
            ops,
            &CookTarget::Boundaries(nodes),
            tick,
            playhead_of,
            scopes,
        )
    }

    /// The output streams of the last [`Self::advance_or_scrub_to_nodes_scoped`]
    /// cook, labelled by node — the boundary hand-off the GPU sequencer uploads.
    /// Empty before the first boundary cook; **shorter than the set asked for**
    /// when a boundary failed to cook, which the caller must treat as a fallback
    /// rather than upload a partial set.
    #[must_use]
    pub fn boundary_streams(&self) -> &[(NodeId, Stream)] {
        &self.boundary_streams
    }

    /// The shared forward/scrub dispatch for either target.
    fn advance_or_scrub_target_scoped(
        &mut self,
        graph: &Graph,
        ops: &dyn OpResolver,
        target: &CookTarget,
        tick: u64,
        playhead_of: impl Fn(u64) -> f64,
        scopes: &TimeScopes,
    ) -> bool {
        let forward = match self.last_cooked_tick {
            None => tick == 0,
            Some(last) => tick == last || tick == last + 1,
        };
        if forward {
            let playhead = playhead_of(tick);
            self.pump_target_scoped(graph, ops, target, tick, playhead, scopes)
        } else {
            self.scrub_target_scoped(graph, ops, target, tick, playhead_of, scopes)
        }
    }

    /// The tick this pump last rendered, if any — the ONE record of where the
    /// cook stands. A caller driving the pump from an external clock (the
    /// editor's `ph2d_core::Playhead`, W4.T7) reads this to know which ticks it
    /// still owes: every tick between here and its target must be simmed, because
    /// a sequential node's trajectory (`integrate`, `spring`, `verlet_rope`) is
    /// the sum of its steps and may not skip one.
    ///
    /// Exposed so the shell does **not** keep a tick of its own. It used to, and
    /// that copy was a second clock that could drift from the first.
    #[must_use]
    pub fn last_cooked_tick(&self) -> Option<u64> {
        self.last_cooked_tick
    }

    /// The error from the last cook, if a sink refused. `None` when every sink
    /// cooked. Lets the shell explain a dark scene rather than leave the artist
    /// staring at an empty viewport.
    #[must_use]
    pub fn last_error(&self) -> Option<&CookError> {
        self.last_error.as_ref()
    }
}

#[cfg(test)]
#[path = "eval_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "scrub_tests.rs"]
mod scrub_tests;

#[cfg(test)]
#[path = "boundary_tests.rs"]
mod boundary_tests;
