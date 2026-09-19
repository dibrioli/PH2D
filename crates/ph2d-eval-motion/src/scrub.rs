//! **O SCRUB DA BOMBA** — irmão por RESPONSABILIDADE (HR-18) do [`crate::MotionCookPump`], de
//! onde saiu quando o `lib.rs` passou o tecto de LOC.
//!
//! ⚠️ **O corte é o ASSUNTO e não o tamanho:** avançar um tique e SALTAR para um são leis
//! diferentes — a primeira compõe sobre o estado que está no ar, a segunda tem de o reconstruir a
//! partir de um checkpoint. O `mod scrub_tests` já era irmão destes métodos antes de eles terem
//! ficheiro próprio.
//!
//! ⚠️⚠️ **E o corte foram DUAS fatias, não uma:** o `boundary_streams` vive no meio da família e
//! **não é dela** — *um corte por número de linha teria levado um método de outro assunto*.

use super::*;

impl MotionCookPump {
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
            self.substep_declared_zones(graph, ops, playhead);
            self.cook_target_into(graph, ops, target, playhead, scopes);
            if !target.has_work() {
                break;
            }
            // Advance the `pre` feedback exactly as the forward pump does — so
            // after rendering the target the cook is left ready for `target+1`,
            // and resumed playback continues bit-exact (no off-by-one).
            let _ = self
                .cook
                .advance_tick_fanned(graph, ops, playhead, scopes, &self.fans);
            if t == target_tick {
                break;
            }
            t += 1;
        }
        // ⚠️ Com `t`, não com `target_tick`: depois do laço `t` É o último tique COZIDO, e os
        // dois só coincidem quando o alvo tem trabalho. Ver [`Self::record_tap_fires`].
        self.record_tap_fires(t);
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
}
