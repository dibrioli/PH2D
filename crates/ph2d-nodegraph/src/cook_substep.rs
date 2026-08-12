//! **O SUBSTEP** — rodar `n` sub-passadas de um alvo dentro de um tique.
//!
//! Filho de [`super`] (o `#[path]` esta no `cook.rs`): o bracket mexe em `tick`,
//! `prev_playhead`, `prev_outputs` e `cache`, todos privados do modulo pai.

use super::{Cook, CookError, OpResolver, SCOPE_ROOT, TimeScopes};
use crate::graph::{Graph, NodeId};

impl Cook {
    /// The clock the last tick closed on, `None` before the first ever closed.
    ///
    /// It is the honest `frame_start` for a driver that marches from tick 0 — and the reason
    /// [`Self::substep`] takes that argument instead of reading this itself: the ENGINE cannot
    /// know where a frame began (there is no default for `None` that is not a guess), while a
    /// pump that owns the march does.
    pub fn prev_playhead(&self) -> Option<f64> {
        self.prev_playhead
    }
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

    /// The `pre` sources whose state belongs to `target`'s own circuit — [`upstream_cone`]
    /// intersected with the delayed-edge sources.
    fn pre_sources_feeding(
        &self,
        graph: &Graph,
        target: NodeId,
    ) -> std::collections::BTreeSet<NodeId> {
        let cone = upstream_cone(graph, target);
        graph
            .edges()
            .iter()
            .filter(|e| e.delayed && cone.contains(&e.from.0))
            .map(|e| e.from.0)
            .collect()
    }
}

/// Everything `target` pulls, crossing ordinary edges only — `target` included.
///
/// Not crossing delayed ones is the whole boundary: state arriving through a `pre` is *last
/// tick's*, and last tick's value is not this target's to advance.
pub fn upstream_cone(graph: &Graph, target: NodeId) -> std::collections::BTreeSet<NodeId> {
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
    cone
}

/// One coupled simulation and the rate it runs at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubstepIsland {
    /// The node to hand [`Cook::substep`] — the island's downstream-most declarer, whose cone
    /// covers the rest of it.
    pub root: NodeId,
    /// The rate the whole island runs at.
    pub substeps: u32,
}

/// **Partition the declaring nodes into ISLANDS, all on the graph's ONE clock.**
///
/// ⚠️ **As ilhas existem por CORREÇÃO, e a medição é que o disse.** Substepar cada declarante por
/// conta OVER-STEPA qualquer um que viva no cone de outro: o laço de baixo re-cozinha o cone
/// inteiro, então o de cima avança outra vez. Medido num par acoplado, o de cima ia de `4,876`
/// para **`15,094`**, quase o triplo. Uma raiz por ilha — a de baixo, cujo cone cobre o resto —
/// é o que dá a cada estado exactamente um avanço por sub-passada.
///
/// ⚠️ **E o RITMO é do GRAFO, não da ilha:** todas correm no maior que qualquer declarante pede.
/// Isso é a leitura das referências no nível do contêiner, e é o que o nosso contêiner passou a
/// ser — **cada objeto Motion tem o seu próprio grafo**, então um grafo é uma DOP Network do
/// Houdini / um System do Niagara, e os dois põem o substep exactamente aí (e o device já exige
/// um sink por documento, o que fecha a mesma fronteira do outro lado).
///
/// O que isso compra, e é o ponto: **os dois produtores concordam por CONSTRUÇÃO** e o device
/// nunca precisa recusar. Marchar o plano inteiro `n` vezes dá a cada ilha os mesmos `n`
/// sub-tiques que este bracket lhe daria — idêntico, não aproximado. Um ritmo por ilha exigiria
/// relógios por-ilha dentro do sequenciador de device (uma chamada de `cook` carrega UM playhead),
/// e compraria só o caso de duas simulações independentes **no mesmo objeto** em ritmos
/// diferentes — que com um grafo por objeto se autora como dois objetos.
///
/// A declaração é uma **convenção de manifesto**: um nó que oferece um param `substeps` diz *"o
/// meu interior sub-tica"*. É o mesmo param que o artista edita — um fato, um lugar.
pub fn substep_islands(graph: &Graph, ops: &dyn OpResolver) -> Vec<SubstepIsland> {
    let declarers: Vec<(NodeId, u32)> = graph
        .nodes()
        .iter()
        .filter_map(|inst| {
            let manifest = ops.resolve(inst.type_id())?.manifest();
            manifest.param_default(SUBSTEPS_PARAM)?;
            let n = graph
                .node_param_overrides(inst.id)
                .and_then(|m| m.get(SUBSTEPS_PARAM).copied())
                .or_else(|| manifest.param_default(SUBSTEPS_PARAM))
                .unwrap_or(1.0);
            // Um substep é uma CONTAGEM: arredonda, e o `<= 1` do bracket é o piso.
            Some((inst.id, n.round().clamp(1.0, f32::from(u16::MAX)) as u32))
        })
        .collect();

    let rate = graph_substeps_of(&declarers);
    let cones: Vec<_> = declarers
        .iter()
        .map(|(id, _)| upstream_cone(graph, *id))
        .collect();

    declarers
        .iter()
        .enumerate()
        .filter(|(i, (id, _))| {
            // Um declarante que vive no cone de OUTRO não é raiz: substepar a raiz já o cobre,
            // e substepá-lo à parte é exatamente o over-step medido acima.
            !declarers
                .iter()
                .enumerate()
                .any(|(j, _)| j != *i && cones[j].contains(id))
        })
        .map(|(_, (root, _))| SubstepIsland {
            root: *root,
            substeps: rate,
        })
        .collect()
}

/// **O ritmo deste grafo** — o maior que qualquer declarante pede, `1` se ninguém pede.
///
/// É a MESMA porta que o sequenciador de device pergunta para saber de quantas sub-passadas
/// marchar o plano; uma segunda cópia divergiria exactamente no documento em que as duas metades
/// têm de concordar.
pub fn graph_substeps(graph: &Graph, ops: &dyn OpResolver) -> u32 {
    let declarers: Vec<(NodeId, u32)> = graph
        .nodes()
        .iter()
        .filter_map(|inst| {
            let manifest = ops.resolve(inst.type_id())?.manifest();
            manifest.param_default(SUBSTEPS_PARAM)?;
            let n = graph
                .node_param_overrides(inst.id)
                .and_then(|m| m.get(SUBSTEPS_PARAM).copied())
                .or_else(|| manifest.param_default(SUBSTEPS_PARAM))
                .unwrap_or(1.0);
            Some((inst.id, n.round().clamp(1.0, f32::from(u16::MAX)) as u32))
        })
        .collect();
    graph_substeps_of(&declarers)
}

fn graph_substeps_of(declarers: &[(NodeId, u32)]) -> u32 {
    declarers.iter().map(|(_, n)| *n).max().unwrap_or(1)
}

/// O nome do param que declara *"o meu interior sub-tica"*.
pub const SUBSTEPS_PARAM: &str = "substeps";
