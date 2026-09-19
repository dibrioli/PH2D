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
/// Os LEQUES de tempo desta marcha (ADR-0163) — o acessor e o porquê da assimetria.
#[path = "fans.rs"]
mod fans;
/// O achador de substeps — a convenção de manifesto que o pump lê antes de cada cook.
#[path = "substep_zones.rs"]
mod substep_zones;

use ph2d_nodegraph::cook::{Cook, CookError, OpResolver, TimeScopes};
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_render::RenderInstance;

mod checkpoint;
pub use checkpoint::{CPU_RING_BYTES, CheckpointRing, RECENT_DENSE};

mod sink_style;
pub use sink_style::{
    SINK_BLEND_PARAM, SINK_COLLIDE_ITERATIONS_DEFAULT, SINK_COLLIDE_ITERATIONS_MAX,
    SINK_COLLIDE_ITERATIONS_PARAM, SINK_COLLIDE_PARAM, SINK_FILTER_PARAM, SINK_PIVOT_LIMIT,
    SINK_PIVOT_X_PARAM, SINK_PIVOT_Y_PARAM, SINK_SORT_PARAM, o_que_o_sink_desenha, sink_blend_tag,
    sink_collide_sweeps, sink_style, so_com_forma_por_ordem,
};

mod lower;
pub use lower::{
    MediaColumns, RowMedium, VECTOR_PASS_COLUMN, VectorInstance, evaluate_motion,
    evaluate_motion_into, lower_to_instances, lower_to_instances_into, lower_to_instances_onto,
    lower_to_vector_instances_onto, row_medium, tem_aparencia,
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
    /// **Os LEQUES DE TEMPO desta marcha** — ver o módulo `fans`. Vazio por omissão.
    fans: ph2d_nodegraph::cook::TimeFans,
    /// Reused per-frame lowering buffer — zero-alloc in steady state.
    pub instances: Vec<RenderInstance>,
    /// The cooked VECTOR shape instances this frame (ADR-0154) — the other side
    /// of the `geometry_id` convention, drawn by the shell through
    /// `ph2d-vec-render` into the Vello scene. Reused per-frame like `instances`
    /// (zero-alloc in steady state); empty for any graph without a shape source.
    pub vector_instances: Vec<VectorInstance>,
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
    /// The raw output streams of this frame's TAPS — mid-graph nodes the caller
    /// named on the sink march because it wants to READ them, not draw them.
    ///
    /// ⚠️ **Separado do `boundary_streams` de propósito.** Aquele é o hand-off
    /// CPU→GPU e é validado contra `plan.boundaries`; este é o que um consumidor do
    /// host lê para saber que algo aconteceu no grafo. Um nome, um significado — os
    /// dois numa lista só obrigariam todo leitor a perguntar de qual metade veio.
    tap_streams: Vec<(NodeId, Stream)>,
    /// **As TOMADAS são estado da BOMBA, não argumento de chamada** — e a diferença é a
    /// única razão de este campo existir.
    ///
    /// ⚠️ Como argumento, elas só existiam na porta que o chamador lembrasse de usar: a rota
    /// **híbrida** marcha por [`Self::advance_or_scrub_to_nodes_scoped`] e ficava MUDA, com
    /// os gates verdes (eles dirigem a porta de sinks) e o produto sem gritar nada. Aqui a
    /// tomada cavalga QUALQUER marcha, e uma rota nova nasce coberta.
    taps: Vec<NodeId>,
    /// O que as tomadas disseram em cada tique MARCHADO deste quadro — o livro-razão que o
    /// host lê UMA vez, depois de qualquer rota.
    ///
    /// ⚠️ **Um tique por PEDIDO, não por re-simulação:** um scrub re-cozinha o intervalo
    /// inteiro por dentro, e registrar cada passo dele faria um wrap de loop gritar a volta
    /// toda de uma vez.
    tap_fires: Vec<(u64, NodeId, Stream)>,
    /// ⭐⭐⭐ **Este cozimento vai ser DESENHADO?** (report do dono, 2026-09-18: *«189 objetos,
    /// Sweeps 1024 = 3 FPS»*).
    ///
    /// ⚠️⚠️ **A shell cozinha UM QUADRO POR TIQUE EM DÍVIDA** (`ticks_owed`): um quadro lento
    /// recupera vários tiques de simulação de uma vez, e **só o ÚLTIMO é desenhado** — os
    /// anteriores enchem o `instances`/`vector_instances` e são logo sobrescritos.
    ///
    /// ⇒ o passe de separação, que é um **ACABAMENTO SOBRE O QUE SE DESENHA** e não uma lei de
    /// simulação (não realimenta nada — doc 115 §3 W0), estava a correr `N` vezes para desenhar
    /// **uma**. Com `1024` varreduras num quadro que recupera 11 tiques, isso é onze vezes o preço
    /// da separação para um desenho só.
    ///
    /// ⚠️ Nasce `true`: quem não sabe se está a recuperar (todo chamador que cozinha um tique só)
    /// continua a separar, exactamente como antes.
    separa_o_desenho: bool,
    /// Quantas vezes a separação de facto correu — o readout que torna a lei acima OBSERVÁVEL.
    ///
    /// ⚠️ **Sem ele a economia é invisível:** as duas rotas entregam o mesmo desenho (o tique
    /// intermédio ia ser sobrescrito), logo nenhum gate de valor, de bits ou de pixel a vê.
    separacoes: u64,
    /// Ver [`MotionCookPump::ultimo_relatorio`].
    ultimo_relatorio: ph2d_contact::passe::Relatorio,
    /// ⭐⭐⭐ **A LEI DO DONO, COMO DADO** — *«nós como Grid, rope, etc, não passam de posições do
    /// espaço, sem nenhuma capacidade de gerar pixels na tela»* (ver
    /// [`crate::sink_style::so_com_forma_por_ordem`], que é de onde ela nasce).
    ///
    /// ⚠️⚠️ **Ela é um CAMPO e não uma leitura do ambiente lá dentro, e a razão está medida:** ao
    /// ligar a lei por omissão, **doze** gates desta casa passaram a ler zero — *e nenhum deles é
    /// sobre a lei*. Eles medem o cozimento (o realinhamento de um fio, o healing de uma força, o
    /// carimbo do cartão) pelo canal do DESENHO, e com a lei ligada o desenho de uma corrente sem
    /// forma é, por construção, vazio. ⇒ *um gate que mede outra coisa desliga-a numa linha*, e a
    /// auditoria do doc 115 §31 já pedia exactamente isto: **uma lei só alcançável pelo ambiente
    /// não é gateável.**
    ///
    /// ⛔⛔⛔ **ELA NASCE DESLIGADA, E ISSO É A LEI — não uma omissão conservadora.** Até
    /// 2026-09-19 ela nascia com a porta do ambiente, e a varredura impactada devolveu **onze**
    /// vermelhos: o emissor de PARTÍCULAS (`ph2d-particles`) constrói uma bomba PRÓPRIA, e uma
    /// partícula é uma posição pura — sem `uv_rect`, sem `geometry_id` —, logo a lei cortava-a e
    /// **o emissor desaparecia do produto**, com o oráculo do Godot a ler zero instâncias.
    ///
    /// ⇒ *a lei é do MÓDULO MOTION — o grafo que o artista edita —, e não de toda a gente que
    /// usa esta bomba.* Quem a liga é o [`MotionState`], que é o dono da ordem do dono; a bomba é
    /// a peça NEUTRA, exactamente como o [`ph2d_render::SinkStyle::PLAIN`] já declara por escrito
    /// para o estilo. ⚠️ **Um segundo consumidor desta bomba herda o neutro por construção**, e é
    /// isso que o gate `uma_bomba_nasce_neutra_e_o_motion_e_que_liga` afirma.
    so_com_forma: bool,
}

mod cook_target;
mod scrub;
/// A caixa de saída dos TAPS — o readout inline do doc 43, num irmão.
mod taps;
use cook_target::CookTarget;

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
            fans: ph2d_nodegraph::cook::TimeFans::new(),
            instances: Vec::new(),
            vector_instances: Vec::new(),
            last_cooked_tick: None,
            dirty: true,
            last_error: None,
            ring: CheckpointRing::new(),
            boundary_streams: Vec::new(),
            tap_streams: Vec::new(),
            taps: Vec::new(),
            tap_fires: Vec::new(),
            separa_o_desenho: true,
            separacoes: 0,
            ultimo_relatorio: ph2d_contact::passe::Relatorio::default(),
            so_com_forma: false,
        }
    }

    /// Liga ou desliga a lei do dono NESTA bomba — ver [`Self::so_com_forma`].
    ///
    /// ⚠️ **Chamada pelo PRODUTO** — o [`MotionState`] liga-a com a porta do ambiente, porque a
    /// lei é do módulo Motion e não desta bomba (ver [`Self::so_com_forma`]). Gates e sondas que
    /// medem OUTRA coisa desligam-na aqui numa linha.
    pub fn define_a_lei(&mut self, so_com_forma: bool) {
        self.so_com_forma = so_com_forma;
    }

    /// A lei que esta bomba carrega — ver [`Self::so_com_forma`]. Lida por quem SUBSTITUI a
    /// bomba e tem de a levar consigo.
    #[must_use]
    pub fn a_lei(&self) -> bool {
        self.so_com_forma
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
        self.substep_declared_zones(graph, ops, playhead);
        self.cook_target_into(graph, ops, target, playhead, scopes);
        if target.has_work() {
            // Advance the 1-tick `pre` feedback once per cooked frame — ONCE for
            // the whole graph, not per sink (each sink's `pre` sources are
            // snapshotted by the same call).
            let _ = self
                .cook
                .advance_tick_fanned(graph, ops, playhead, scopes, &self.fans);
        }
        self.record_tap_fires(tick);
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
        // Içado antes do laço: lá dentro o `&mut self.instances` já está tomado.
        let so_com_forma = self.so_com_forma;
        match *target {
            CookTarget::Sinks {
                sinks,
                default_uv_rect,
                default_size,
            } => {
                self.instances.clear();
                self.vector_instances.clear();
                // ⭐⭐⭐ **LIMPA AQUI, e não à porta do laço das tomadas** (report do dono, 18/09:
                // *«com 1024 FPS cai para 7»*). O gizmo do colisor pede o PRÓPRIO sink como tomada
                // (`collider_gizmo::taps_for`), logo o sink estava a ser separado **DUAS VEZES por
                // quadro**: uma para desenhar e outra para a tomada. ⚠️ O 2.º cozimento é barato
                // (bate no memo, como o comentário do laço das tomadas explica) — o passe do fim
                // **não é memoizado**, e a `1024` varreduras ele era metade do quadro.
                //
                // ⇒ quem desenha PUBLICA o que separou, e o laço das tomadas salta-o pela cerca de
                // duplicado que ele já tinha. A corrente é a mesma; o que desaparece é a 2.ª conta.
                self.tap_streams.clear();
                for &sink in sinks {
                    // A sink that fails to cook (an unknown type mid-edit, or a
                    // sequential node caught inside a remapped time scope)
                    // contributes nothing; the others still draw. The error is
                    // kept for the shell.
                    match self
                        .cook
                        .cook_scoped_fanned(graph, ops, sink, playhead, scopes, &self.fans)
                    {
                        Ok(outputs) => {
                            if let Some(v) = outputs.first() {
                                let cozido = v.as_stream();
                                // ⭐⭐⭐ **O PASSE AUTOMÁTICO** (doc 115 W5): o acabamento que
                                // separa o que vai ser desenhado, armado pelo interruptor do
                                // próprio sink. ⚠️ **AQUI e não dentro de cada lowering**: as duas
                                // mídias lêem a MESMA corrente (a lei do `geometry_id`), e separar
                                // em dois sítios poria um vector e uma sprite do mesmo grupo em
                                // posições diferentes. ⛔ E ele devolve `None` — sem clonar nada —
                                // quando a corrente não declara colisor ou quando ninguém se mexeu,
                                // que é o que mantém toda cena de hoje byte-idêntica.
                                // ⭐ Só o cozimento que vai ser DESENHADO paga o acabamento — ver
                                // [`Self::separa_o_desenho`].
                                let separado = if self.separa_o_desenho {
                                    self.separacoes += 1;
                                    let (s, r) = o_que_o_sink_desenha(graph, sink, cozido);
                                    self.ultimo_relatorio = r;
                                    s
                                } else {
                                    None
                                };
                                let stream = separado.as_ref().unwrap_or(cozido);
                                // A tomada deste sink, se alguém a pediu — ver o `clear` acima.
                                if self.taps.contains(&sink) {
                                    self.tap_streams.push((sink, stream.clone()));
                                }
                                // The SAME cooked stream feeds both sides of the
                                // `geometry_id` convention (ADR-0154): textured-quad
                                // rows lower to `instances`, vector-shape rows to
                                // `vector_instances`. Disjoint by construction (each
                                // reads/skips on the same `geometry_id > 0` test), so
                                // a shape is drawn once, as vector.
                                // ⚠️ Uma leitura, dois lowerings: eles TÊM de receber o MESMO
                                // estilo, e perguntá-lo duas vezes é como um deles passa a ler
                                // outra coisa no dia em que a porta ganhar um campo.
                                let estilo = ph2d_render::SinkStyle {
                                    so_com_forma,
                                    ..sink_style(graph, sink)
                                };
                                lower_to_instances_onto(
                                    stream,
                                    default_uv_rect,
                                    default_size,
                                    // Per SINK, not per document: two Output nodes
                                    // may draw the same document in two modes, and
                                    // each lowers with its own tag.
                                    estilo,
                                    &mut self.instances,
                                );
                                lower_to_vector_instances_onto(
                                    stream,
                                    estilo,
                                    &mut self.vector_instances,
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
                // ⚠️ E as TOMADAS, que nesta rota não têm quem as publique — sem isto as tomadas
                // do quadro ANTERIOR sobreviveriam a um quadro híbrido. (A rota dos sinks limpa-as
                // ela própria, porque é ela quem publica o que desenhou.)
                self.tap_streams.clear();
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
                    match self
                        .cook
                        .cook_scoped_fanned(graph, ops, node, playhead, scopes, &self.fans)
                    {
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
        self.cozinha_as_tomadas(graph, ops, target, playhead, scopes);
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
    pub fn boundary_streams(&self) -> &[(NodeId, Stream)] {
        &self.boundary_streams
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

/// ⭐⭐⭐ O passe automático **percorrido pelo pump** (doc 115 W5) — ver o cabeçalho dele.
#[cfg(test)]
#[path = "passe_no_pump_tests.rs"]
mod passe_no_pump_tests;

#[cfg(test)]
#[path = "scrub_tests.rs"]
mod scrub_tests;

#[cfg(test)]
#[path = "tap_tests.rs"]
mod tap_tests;

#[cfg(test)]
#[path = "boundary_tests.rs"]
mod boundary_tests;

#[cfg(test)]
#[path = "lower_tests.rs"]
mod lower_tests;
