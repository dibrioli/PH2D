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

use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, CookError, OpResolver, TimeScopes};
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_render::RenderInstance;

mod checkpoint;
pub use checkpoint::{CheckpointRing, RECENT_CAPACITY};

/// Lower a cooked instance stream **into `out`** (one instance per element),
/// reusing `out`'s capacity: `out` is cleared and refilled, so a steady stream
/// count frame-to-frame allocates nothing (M0.T11 — the per-frame bridge path;
/// zero-alloc gated by M0.T12). Pure + headless: no GPU.
///
/// `default_uv_rect` / `default_size` are the `atlas_uv` / `size` for an
/// instance whose stream lacks the matching column (the M0 case — no framing
/// node yet). The shell passes a single opaque atlas tile plus a `size` below
/// the grid spacing so the raw default document renders as clean, distinct
/// quads; a headless caller passes the whole-atlas rect `[0,0,1,1]` and unit
/// size `[1,1]`.
pub fn lower_to_instances_into(
    stream: &Stream,
    default_uv_rect: [f32; 4],
    default_size: [f32; 2],
    out: &mut Vec<RenderInstance>,
) {
    out.clear();
    lower_to_instances_onto(stream, default_uv_rect, default_size, out);
}

/// Like [`lower_to_instances_into`] but **appends** — `out` keeps whatever it
/// already holds. This is how several render sinks compose into one draw: the
/// pump clears once, then each `motion.output` node's stream lowers onto the
/// same buffer (still zero-alloc in steady state — capacity is retained).
pub fn lower_to_instances_onto(
    stream: &Stream,
    default_uv_rect: [f32; 4],
    default_size: [f32; 2],
    out: &mut Vec<RenderInstance>,
) {
    let n = stream.count();
    let p = stream.get("P");
    let size = stream.get("size");
    let rot = stream.get("rot");
    let tint = stream.get("tint");
    let uv_rect = stream.get("uv_rect");
    out.reserve(n);
    for i in 0..n {
        // ADR-0070-amendment-4: RenderInstance carries the 2×2 world
        // basis, not a rotation scalar. A Motion stream emits only a
        // rotation (no skew), so the basis is a pure rotation matrix
        // `[cos, sin, -sin, cos]`. RenderInstance is PresentWorld-only
        // (HR-5 exempt), so std `sin_cos` is fine here.
        //
        // The `rot` column is in **degrees** — the app's authored-angle unit
        // (the Painter's `*_angle_deg` fields, the Inspector's `deg` boxes).
        // Radians live nowhere in the Motion authoring surface; only this
        // conversion, at the very edge where the basis is built.
        let (sin_r, cos_r) = scalar_at(rot, i, 0.0).to_radians().sin_cos();
        out.push(RenderInstance {
            world_pos: vec2_at(p, i, [0.0, 0.0]),
            size: vec2_at(size, i, default_size),
            atlas_uv: vec4_at(uv_rect, i, default_uv_rect),
            tint: vec4_at(tint, i, [1.0, 1.0, 1.0, 1.0]),
            basis: [cos_r, sin_r, -sin_r, cos_r],
            premultiplied: 0.0,
            anchor: [0.0, 0.0],
            // Sprite-Inspector-v2 v4 ABI fields: a Motion node stream has
            // no per-corner/opacity/flip authoring surface, so they take
            // their identity values (white gradient, full opacity, no
            // flip) — byte-identical render to the pre-v4 path.
            per_corner_tint: [[1.0; 4]; 4],
            opacity: 1.0,
            flip_uv: 0,
            texture_id: 0,
            // Node-graph emit doesn't have a hierarchy slot — every
            // motion node's instances share `z_order = 0`. Renderer's
            // tiebreaker (`texture_id`) groups them into one run.
            z_order: 0,
            sampling: 0,
            uv_xform: RenderInstance::IDENTITY_UV_XFORM,
            // Node-graph emit has no hierarchy → no clip silhouette.
            clip_group: RenderInstance::CLIP_GROUP_NONE,
            clip_meta: 0,
        });
    }
}

/// Lower a cooked instance stream to render instances (one per element).
/// Pure + headless. Allocates a fresh `Vec`; the per-frame path uses
/// [`lower_to_instances_into`] to reuse a buffer instead. Uses the whole-atlas
/// UV `[0,0,1,1]` for any instance without a `uv_rect` column (the shell path
/// supplies a real tile via [`lower_to_instances_into`]'s `default_uv_rect`).
pub fn lower_to_instances(stream: &Stream) -> Vec<RenderInstance> {
    let mut out = Vec::new();
    lower_to_instances_into(stream, [0.0, 0.0, 1.0, 1.0], [1.0, 1.0], &mut out);
    out
}

/// Cook `target` at `playhead` and lower its **output port 0** to render
/// instances. Reuse the same [`Cook`] across frames for incremental cheapness.
///
/// Lowering a single port is intentional: a Motion render target is one
/// instance stream. A target with several output ports has only port 0 lowered
/// here (a multi-port target would select the port at the call site — not
/// needed by any Motion node today, all of which have exactly one output). A
/// target that legitimately declares **zero** outputs yields an empty `Vec`;
/// note the cook itself already rejects a node that *declares* an output but
/// emits none ([`CookError::OutputCountMismatch`]), so an empty result here
/// means "no output port", never a dropped stream.
pub fn evaluate_motion(
    cook: &mut Cook,
    graph: &Graph,
    ops: &dyn OpResolver,
    target: NodeId,
    playhead: f64,
) -> Result<Vec<RenderInstance>, CookError> {
    let mut out = Vec::new();
    evaluate_motion_into(
        cook,
        graph,
        ops,
        target,
        playhead,
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 1.0],
        &mut out,
    )?;
    Ok(out)
}

/// Cook `target` at `playhead` and lower its output port 0 **into `out`**,
/// reusing the buffer's capacity (M0.T11 — the per-frame bridge entry). Same
/// single-port semantics as [`evaluate_motion`]; a target with no output port
/// leaves `out` empty. Reuse the same [`Cook`] AND the same `out` across frames
/// for the zero-alloc steady state (gated by M0.T12).
///
/// `default_uv_rect` / `default_size` are the `atlas_uv` / `size` fallbacks for
/// a stream without the matching column (see [`lower_to_instances_into`]).
#[allow(clippy::too_many_arguments)] // cook + graph + resolver + target + playhead + 2 defaults + out
pub fn evaluate_motion_into(
    cook: &mut Cook,
    graph: &Graph,
    ops: &dyn OpResolver,
    target: NodeId,
    playhead: f64,
    default_uv_rect: [f32; 4],
    default_size: [f32; 2],
    out: &mut Vec<RenderInstance>,
) -> Result<(), CookError> {
    let outputs = cook.cook(graph, ops, target, playhead)?;
    // A cooked output port is a `CookValue`; a Motion target's port 0 is an
    // instance stream (ADR-0058-amendment-1). A non-stream value lowers to no
    // instances (its `as_stream()` is empty).
    match outputs.first() {
        Some(v) => lower_to_instances_into(v.as_stream(), default_uv_rect, default_size, out),
        None => out.clear(),
    }
    Ok(())
}

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
        }
    }

    /// Force a re-cook on the next [`Self::pump`], even at the same tick (call
    /// after editing the graph while paused). The M1 graph-edit path. Also
    /// **invalidates the scrub cache**: the recorded checkpoints are the sim of
    /// the OLD graph, so a later scrub must re-sim from the tick-0 seed under the
    /// edited graph (Blender/Houdini "edit invalidates the cache").
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
        self.ring.clear();
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
        self.cook_sinks_into(
            graph,
            ops,
            sinks,
            playhead,
            default_uv_rect,
            default_size,
            scopes,
        );
        if !sinks.is_empty() {
            // Advance the 1-tick `pre` feedback once per cooked frame — ONCE for
            // the whole graph, not per sink (each sink's `pre` sources are
            // snapshotted by the same call).
            let _ = self.cook.advance_tick_scoped(graph, ops, playhead, scopes);
        }
        self.last_cooked_tick = Some(tick);
        self.dirty = false;
        true
    }

    /// Cook every sink into `instances` at `playhead` (clears + refills the
    /// buffer, keeping capacity). Shared by the forward [`Self::pump_scoped`] and
    /// the backwards [`Self::scrub_to_scoped`] so both take the IDENTICAL cook
    /// path — a "fast preview" scrub that diverged from playback is the classic
    /// determinism trap (the state-of-the-art survey's warning).
    #[allow(clippy::too_many_arguments)]
    fn cook_sinks_into(
        &mut self,
        graph: &Graph,
        ops: &dyn OpResolver,
        sinks: &[NodeId],
        playhead: f64,
        default_uv_rect: [f32; 4],
        default_size: [f32; 2],
        scopes: &TimeScopes,
    ) {
        self.instances.clear();
        self.last_error = None;
        for &sink in sinks {
            // A sink that fails to cook (an unknown type mid-edit, or a
            // sequential node caught inside a remapped time scope) contributes
            // nothing; the others still draw. The error is kept for the shell.
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
            self.cook_sinks_into(
                graph,
                ops,
                sinks,
                playhead,
                default_uv_rect,
                default_size,
                scopes,
            );
            if sinks.is_empty() {
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
        let forward = match self.last_cooked_tick {
            None => tick == 0,
            Some(last) => tick == last || tick == last + 1,
        };
        if forward {
            let playhead = playhead_of(tick);
            self.pump_scoped(
                graph,
                ops,
                sinks,
                tick,
                playhead,
                default_uv_rect,
                default_size,
                scopes,
            )
        } else {
            self.scrub_to_scoped(
                graph,
                ops,
                sinks,
                tick,
                playhead_of,
                default_uv_rect,
                default_size,
                scopes,
            )
        }
    }

    /// The error from the last cook, if a sink refused. `None` when every sink
    /// cooked. Lets the shell explain a dark scene rather than leave the artist
    /// staring at an empty viewport.
    #[must_use]
    pub fn last_error(&self) -> Option<&CookError> {
        self.last_error.as_ref()
    }
}

fn scalar_at(c: Option<&Column>, i: usize, default: f32) -> f32 {
    match c {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(default),
        _ => default,
    }
}
fn vec2_at(c: Option<&Column>, i: usize, default: [f32; 2]) -> [f32; 2] {
    match c {
        Some(Column::Vec2(v)) => v.get(i).copied().unwrap_or(default),
        _ => default,
    }
}
fn vec4_at(c: Option<&Column>, i: usize, default: [f32; 4]) -> [f32; 4] {
    match c {
        Some(Column::Vec4(v)) => v.get(i).copied().unwrap_or(default),
        _ => default,
    }
}

#[cfg(test)]
#[path = "eval_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "scrub_tests.rs"]
mod scrub_tests;
