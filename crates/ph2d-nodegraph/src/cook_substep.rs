//! **O SUBSTEP** — rodar `n` sub-passadas de um alvo dentro de um tique.
//!
//! Filho de [`super`] (o `#[path]` esta no `cook.rs`): o bracket mexe em `tick`,
//! `prev_playhead`, `prev_outputs` e `cache`, todos privados do modulo pai.

use super::{Cook, CookError, OpResolver, SCOPE_ROOT, TimeScopes};
use crate::graph::{Graph, NodeId};

impl Cook {
    /// Run `n` sub-passes of `target` inside one tick — the **substep**.
    ///
    /// A substep is a property of the **clock**, not a parameter of the step: a
    /// stateful node takes `dt` from `playhead − <its own clock column>`, so
    /// subdividing the playhead subdivides the integration with no change to any
    /// kernel. Chaining the integrator twice in the interior cannot work and the
    /// reason is the same fact from the other side — the first pass writes the
    /// clock column to `playhead`, so the second reads `dt = 0` and multiplies
    /// every term by it.
    ///
    /// **Only `target`'s own `pre` sources are refreshed**, and that is the whole
    /// point: [`Self::advance_tick`] refreshes *every* `pre` source in the graph,
    /// so driving substeps through it makes an unrelated neighbour pay `n` times
    /// (measured: 1,83× / 3,95× / 8,13× / 16,36× for n = 2/4/8/16 on a graph whose
    /// other half has nothing to do with the target).
    ///
    /// Two clock facts decide the shape, and getting either wrong is silent:
    ///
    /// * `prev_playhead` **advances** through the loop, because a count law reads
    ///   its `dt` from there — a birth rate handed the whole frame on every
    ///   sub-pass emits the cumulative count `n` times. Advancing it is also why
    ///   the totals do not move: `floor(rate·t) − floor(rate·(t−dt))` **telescopes**
    ///   across the slices, so a frame births exactly what it births at `n = 1`.
    /// * and it is **restored** at the end, because the frame's own cook and
    ///   [`Self::advance_tick`] run afterwards for the rest of the graph — leaving
    ///   it at the last slice would hand every other temporal node a fraction of a
    ///   frame as if it were the frame.
    ///
    /// The last sub-pass lands exactly on `playhead`, so the frame's own
    /// `cook(target, playhead)` **hits the memo** rather than stepping again.
    /// `n <= 1` is not a special case to remember: the loop does not execute and
    /// the tick is byte-identical to one that never called this.
    ///
    /// ⚠️ **`frame_start` is the caller's to supply, and it is not decoration.**
    /// The obvious shortcut — read it from the engine's own `prev_playhead` — is
    /// wrong on the very first tick, where that is `None` and there is no honest
    /// default: `playhead` collapses the span to zero, so the first frame runs
    /// **coarse whatever `n` says**, and a coarse first frame is a whole frame of
    /// lag that never comes back. Measured on a body under constant acceleration:
    /// the error stops halving and **saturates at −0,64** (against −0,0625 at
    /// n = 16) — the substep looks like it converges and then stops, which is the
    /// worst shape a defect can have. The clock is the driver's; the engine does
    /// not guess it.
    pub fn substep(
        &mut self,
        graph: &Graph,
        ops: &dyn OpResolver,
        target: NodeId,
        frame_start: f64,
        playhead: f64,
        n: u32,
    ) -> Result<(), CookError> {
        if n <= 1 {
            return Ok(());
        }
        let span = playhead - frame_start;
        let mine = self.pre_sources_feeding(graph, target);
        let scopes = TimeScopes::new();
        // Guardado como estava, `None` incluído: restaurar um `Some` onde havia `None` diria
        // ao resto do grafo que um tique já fechou, e o primeiro `dt` de todo mundo mudaria.
        let clock_before = self.prev_playhead;
        for k in 1..=n {
            // ⚠️ **Um substep é um sub-TIQUE, e sem isto ele é um no-op silencioso.** O
            // fingerprint de um nó que consome `pre` inclui `self.tick` — o memo existe
            // precisamente para que um circuito sequencial avance *uma vez por tique* —,
            // então as passadas 2..n bateriam no memo e não fariam nada: medido, o alvo
            // ficava meio quadro atrás e a convergência caía de 2,0× para 1,2×.
            // A primeira passada NÃO bumpa: ela é o primeiro avanço deste tique, que o
            // `advance_tick` do quadro anterior já abriu.
            if k > 1 {
                self.tick += 1;
            }
            let t_k = frame_start + span * (f64::from(k) / f64::from(n));
            self.cook_node(graph, ops, target, t_k, SCOPE_ROOT, &scopes)?;
            for &src in &mine {
                self.cook_node(graph, ops, src, t_k, SCOPE_ROOT, &scopes)?;
                if let Some(c) = self.cache.get(&(src, SCOPE_ROOT)) {
                    self.prev_outputs.insert(src, c.outputs.clone());
                }
            }
            self.prev_playhead = Some(t_k);
        }
        self.prev_playhead = clock_before;
        Ok(())
    }

    /// The `pre` sources whose state belongs to `target`'s own circuit: a node
    /// that feeds a delayed edge **and** sits in `target`'s upstream cone (or is
    /// `target` itself — a simulation zone is the source of its own feedback).
    ///
    /// The walk crosses ordinary edges only. Not crossing delayed ones is what
    /// keeps a neighbouring circuit out: its state reaches here through a `pre`,
    /// which is precisely the edge that says *"last tick's value"*, and last
    /// tick's value is not this target's to advance.
    fn pre_sources_feeding(
        &self,
        graph: &Graph,
        target: NodeId,
    ) -> std::collections::BTreeSet<NodeId> {
        let mut cone: std::collections::BTreeSet<NodeId> = Default::default();
        let mut stack = vec![target];
        while let Some(n) = stack.pop() {
            if !cone.insert(n) {
                continue;
            }
            for e in graph.edges() {
                if e.to.0 == n && !e.delayed {
                    stack.push(e.from.0);
                }
            }
        }
        graph
            .edges()
            .iter()
            .filter(|e| e.delayed && cone.contains(&e.from.0))
            .map(|e| e.from.0)
            .collect()
    }
}
