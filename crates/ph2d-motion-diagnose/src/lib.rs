#![forbid(unsafe_code)]
//! **The setup DIAGNOSER for the Motion graph** (ADR-0155).
//!
//! The Motion graph has a class of error that produces no error. The canonical
//! case the Enio named: a `force.*` node is `Pure` and accumulates into the
//! transient column `accel`; the only nodes that consume `accel` are the
//! integrators (`motion.integrate`, `sim.step`). A force wired toward the sink
//! with **no integrator on the path** writes `accel`, nothing reads it, and the
//! scene stays static — with no error and no warning. `Graph::validate` checks
//! only port types and membranes; it has no reachability analysis.
//!
//! [`diagnose`] is that analysis, and it does NOT keep a parallel hand-written
//! table of which node touches which column. It **derives** the produces/consumes
//! roles from the node's OWN declaration — the same one the cook trusts:
//!
//! - **The GPU `ColumnBinding` set** (`reg.gpu_kernel(ty).bindings`) is what the
//!   device sequencer reads to bind buffers, so it cannot drift from the cook. A
//!   binding that WRITES a column produces it; a binding that READS but does not
//!   write it consumes it. This is the gold-standard "derive, don't declare"
//!   (Blender geometry-node field dependency; Houdini's cook read/create/modify)
//!   and it covers every GPU-resident node — the whole `falloff` family of
//!   fields/forces/deformers — for free, with zero annotation.
//! - **The `Coupling` side-channel** (`reg.couplings(ty)`) covers the handful of
//!   CPU-only nodes that read a column only at eval runtime (no GPU kernel to
//!   derive from) — the `fx.*`/`motion.step`/… falloff consumers.
//!
//! The **re-producer rule** is the load-bearing subtlety: a node that reads AND
//! writes a column (a force accumulating `accel`, a `field.combine` composing
//! `falloff`) is a producer, NOT a consumer. That is exactly what keeps two
//! forces with no integrator inert — the second force reads `accel` but re-writes
//! it, so it does not "save" the first.
//!
//! Only [`TRANSIENT_COLUMNS`] are subject to the producer analysis: the columns NOT
//! lowered to instances, whose producers are inert without a graph-internal consumer.
//! `P`/`size`/`rot`/`tint` are consumed by the output, so wiring them to it is
//! never inert.
//!
//! The mirror analysis is [`REQUIRED_UPSTREAM`]: a column a node READS to do its job
//! (`P` — a deformer/force with no points is a silent no-op). A reader of one with
//! NOTHING wired into it is source-less, reported a [`Deficit::MissingSource`] OFFER —
//! WHICH source (grid / emitter / object) is a creative choice, never guessed. TWO kinds
//! of node are NOT source-less. A pure `Write` generator (grid / emitter) does not
//! [`reads`](ColumnAccess::reads), so it is never judged to need a column upstream. A
//! node fed by a **DELAYED** edge — a STATEFUL source reading its own previous `P`
//! through the `pre` self-loop (`boids` / `verlet_rope` / `soft_body` / `spring` /
//! `integrate`, which mint the initial cloud themselves), **or** the head of a force
//! chain fed by an integrator's `pre` — is likewise not source-less: it HAS a stream,
//! it is just last tick's. Both are exempt via [`fed_by_a_delayed_edge`].
//!
//! ⚠️ **Essa segunda metade custou um badge errado em toda cadeia de força.** A regra
//! era «a aresta atrasada do PRÓPRIO nó» (`seeds_own_state`), e o laço canónico é
//! `integrate ⟿pre⟿ força ⟿fwd⟿ integrate.forces` — a aresta vem de OUTRO nó. Medido em
//! 2026-08-20: **seis** cenas da conferência marcadas, todas correctas, e o mesmo badge
//! aparecia no grafo que o próprio AUTO-HEAL constrói.
//!
//! A re-producer that genuinely needs an upstream stream (a deformer's `ReadWrite` on
//! `P`, read from its DATA port with no incoming edge at all) READS it and is left to
//! warn — the same re-producer rule the produces/consumes analysis rests on, on the
//! required-upstream axis. The test is "**no incoming edge, delayed or not**", not a
//! port index or a backward source-search: a P-reader with none is unambiguously
//! source-less, and a reader that IS fed is left alone — the safe, under-warning
//! direction the "Node Help" toggle backstops.
//!
//! The per-PORT twin is [`Deficit::MissingInput`]: a node declares an input port REQUIRED
//! (`NodeRegistry::required_inputs`, e.g. `motion.duplicator`'s `shape`/`points`) and that
//! port has no edge. This one is DECLARED, not derived — required-vs-optional is semantic
//! (an integrator's `forces` port is optional: no forces is a valid static integration),
//! so it cannot be read off a binding — the port-level twin of a `Coupling::Requires`.
//!
//! For every producer of a transient column, [`diagnose`] walks forward
//! (non-`delayed`) edges: if some reachable node consumes the column, the producer
//! is healthy; if not, it is **inert**, and the [`Fix`] says whether the cure is to
//! insert the canonical plumbing ([`Fix::Insert`], the AUTO-HEAL case), to reorder
//! against an off-path consumer ([`Fix::Reorder`]), or to merely offer a choice
//! with no canonical answer ([`Fix::Offer`], the `falloff`/`inv_mass` advisory).

use ph2d_node_registry::{Coupling, NodeRegistry};
use ph2d_nodegraph::column::ColumnAccess;
use ph2d_nodegraph::gpu::KernelResolver;
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::node::NodeTypeId;
use std::collections::BTreeSet;

/// The columns that are **not lowered to instances** — the transient-channel
/// convention (`ph2d_nodegraph::column`). A producer of one of these is inert
/// unless a graph-internal node consumes it; a producer of a LOWERED column
/// (`P`/`size`/`rot`/`tint`, read by the output) is never subject to this
/// analysis, which is why the set is a filter and not "every produced column".
///
/// `accel`/`inv_mass` are the columns some node **`Consume`-drops** (a gate proves
/// this set covers every such column, so a new transient column cannot be added
/// without landing here); `falloff` is the modulation weight — pure-read by
/// forces/deformers, never dropped, never lowered, so it is named directly.
const TRANSIENT_COLUMNS: &[&str] = &["accel", "falloff", "inv_mass"];

/// The columns a node READS to have anything to act on — the mirror of
/// [`TRANSIENT_COLUMNS`]. `P` is the position: a deformer/force with nothing feeding it
/// has no stream, so it is a silent no-op. A reader of one of these with NO incoming edge
/// is reported a [`Deficit::MissingSource`] (an OFFER — WHICH source is a creative choice).
///
/// Disjoint from [`TRANSIENT_COLUMNS`] by construction (a column is either a
/// producer-inert transient or a read-required stream, never both; a gate pins it): the
/// two analyses do not touch the same column. `vel` (for `force.drag`/`force.buoyancy`,
/// which only exists after an integrator runs) is a later low-priority member.
const REQUIRED_UPSTREAM: &[&str] = &["P"];

/// One diagnosed defect: a node whose placement makes its output inert.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    /// The offending producer.
    pub node: NodeId,
    /// What is wrong.
    pub deficit: Deficit,
    /// How to fix it (and how aggressively the editor may act — see [`Fix`]).
    pub fix: Fix,
}

/// The kind of defect found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Deficit {
    /// This node writes the named transient column, and no node reachable
    /// downstream (via forward, non-`delayed` edges) consumes it — so it does
    /// nothing.
    InertProducer(&'static str),
    /// This node READS the named [required-upstream](REQUIRED_UPSTREAM) column (a
    /// deformer/force needs `P` to work on) but has NOTHING wired into it — so it has no
    /// stream to act on and is silently a no-op. Always a [`Fix::Offer`]: WHICH source
    /// (grid / emitter / object) is a creative choice.
    MissingSource(&'static str),
    /// This node declares the named input port REQUIRED (`NodeRegistry::required_inputs`,
    /// e.g. `motion.duplicator`'s `shape`/`points`) but that port has no edge — with
    /// nothing to copy, or nowhere to put it, the node is a silent no-op. Always a
    /// [`Fix::Offer`]: WHAT to wire into it is the artist's choice. Carries the PORT NAME.
    MissingInput(&'static str),
    /// **Outro nó do MESMO tipo já ocupa o único lugar que ele tem** — este é inerte.
    ///
    /// Os outros três déficits são sobre a POSIÇÃO de um nó no grafo; este é sobre a
    /// EXISTÊNCIA de um irmão. Ele existe para os nós que não são passos de um fluxo mas
    /// **configuram um passe de tela inteira**, lido uma vez a partir do grafo — para esses
    /// o segundo nó não compõe, ele é ignorado. Carrega o NOME DO TIPO.
    ///
    /// ⚠️ **Sempre [`Fix::Offer`]**: apagar qual dos dois é decisão do artista, e um
    /// auto-heal que apagasse um nó seria a única cura desta casa que destrói trabalho.
    Shadowed(&'static str),
    /// **Um BURACO no meio das portas de um roteador** — `in1` vazia com `in2` ligada.
    ///
    /// Um `value.switch` escolhe por índice: `clamp(round(select), 0, N−1)`. Uma porta
    /// vazia é um índice que existe e **lê `0.0`** — e `0` é um valor legítimo, então o
    /// artista não distingue *"esta ramificação está vazia"* de *"esta ramificação vale
    /// zero"*. Medido: com só `in0`/`in1` ligadas, `select = 2` e `select = 3` devolvem
    /// `0.000` sem sinal nenhum.
    ///
    /// ⚠️ **Só o buraco do MEIO é diagnosticado, e a distinção é o que impede o ruído:**
    /// deixar `in2`/`in3` vazias é como se escreve um mux de duas vias — legítimo, comum,
    /// e o `select` nem lá chega (o clamp para em `N−1`... e é justamente por o clamp
    /// parar em `N−1` e não no ÚLTIMO LIGADO que a cauda vazia também lê zero; isso é
    /// comportamento **documentado e deliberado** no doc-comment do kernel daquele nó, e
    /// mudá-lo seria outra função). Uma porta vazia **antes** de uma ligada não tem
    /// leitura inocente: o índice dela está no meio do alcance que o artista está a
    /// varrer.
    ///
    /// ⚠️ **Sempre [`Fix::Offer`]**: o que ligar ali é escolha do artista, e ligar por
    /// palpite é a única cura desta casa que INVENTA conteúdo. Carrega o NOME DA PORTA.
    DeadBranch(&'static str),
}

impl Deficit {
    /// **Um exemplar de CADA variante — a fonte de qualquer censo sobre esta lista.**
    ///
    /// ⚠️ **Ela existe porque um `enum` não se itera, e o consumidor que mais importa termina
    /// num `_ =>`:** o `explain` do shell escolhe a frase que o artista lê, e um variante novo
    /// cai no catch-all *em silêncio* — compila, corre, e diz ao artista uma coisa que não é o
    /// defeito dele. Uma lista escrita à mão do lado do gate teria o mesmo buraco um nível
    /// acima (é preciso lembrar de a estender); aqui ela mora **ao lado da definição**, onde
    /// quem acrescenta o variante já está.
    ///
    /// Os payloads são exemplos reais do repo, não placeholders: uma frase pode depender do
    /// conteúdo (`InertProducer("accel")` tem braço próprio), então um censo sobre nomes
    /// inventados provaria menos do que parece.
    pub const ALL: &'static [Deficit] = &[
        Deficit::InertProducer("accel"),
        Deficit::InertProducer("inv_mass"),
        Deficit::InertProducer("falloff"),
        Deficit::MissingSource("P"),
        Deficit::MissingInput("shape"),
        Deficit::Shadowed("fx.glow"),
        Deficit::DeadBranch("in1"),
    ];
}

/// Os tipos cujo nó configura um **passe de tela inteira**, lido UMA vez do grafo — para
/// eles o segundo nó não compõe, é ignorado.
///
/// ⚠️ **Medido em 2026-08-19, e é UM.** O `fx.glow` é o único `fx.*` com um `from_graph`
/// (`present.rs` chama-o por quadro); os irmãos `fx.drop_shadow` e `fx.rgb_split` fazem o
/// trabalho no `eval`, por nó, e por isso **compõem** — dois deles aplicam duas vezes.
/// A sonda `measure_second_glow` mostra o defeito: com 1, 2 ou 3 nós, o passe lê sempre o
/// primeiro (`intensity 1.0`), e os outros **pintam, aceitam clique, entram no undo e não
/// mudam um pixel**.
///
/// A lista existe em vez de o nome estar cravado porque o próximo passe de tela deste tipo
/// tem de ser **uma linha**, e não uma regra nova a redescobrir.
const SINGLETON_SCREEN_PASSES: &[&str] = &["fx.glow"];

/// The suggested cure, carrying how confidently the editor may apply it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fix {
    /// Insert this canonical consumer node type to make the producer live
    /// (`accel` → `motion.integrate`, or `sim.step` in a particle chain). The
    /// **AUTO-HEAL** candidate — unambiguous plumbing the artist forgot.
    Insert(&'static str),
    /// A consumer of this column exists in the graph, but not on the producer's
    /// forward path — the cure is to REORDER (put the producer upstream of it),
    /// never to insert a second one (one integrator applies). An **OFFER**.
    Reorder,
    /// The missing consumer is a creative choice with no canonical inserter
    /// (`inv_mass` needs *a* solver; `falloff` needs *a* force/deformer) — surface
    /// it, never guess. An **OFFER/AVISO**.
    Offer,
}

/// Walk the graph and report every node whose output is semantically inert
/// (ADR-0155). Pure: reads only the graph structure and the registry's derived
/// roles (GPU bindings + [`Coupling`] side-channel). A node that produces no
/// transient column is neutral and never diagnosed; a producer of a transient
/// column with a consumer reachable downstream is healthy and reported nowhere.
#[must_use]
pub fn diagnose(graph: &Graph, reg: &NodeRegistry) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let mut claimed: Vec<&str> = Vec::new();
    for inst in graph.nodes() {
        let ty = NodeTypeId::of(&inst.type_name);
        // Um passe de tela cujo lugar outro nó já tomou. ⚠️ Vem PRIMEIRO e com `continue`,
        // pela mesma lei dos dois déficits abaixo: um nó ignorado não tem output para ser
        // inerte, e dois avisos sobre o mesmo nó ensinam que há dois problemas.
        if let Some(&ty_name) = SINGLETON_SCREEN_PASSES
            .iter()
            .find(|t| **t == inst.type_name)
        {
            if claimed.contains(&ty_name) {
                out.push(Diagnostic {
                    node: inst.id,
                    deficit: Deficit::Shadowed(ty_name),
                    fix: Fix::Offer,
                });
                continue;
            }
            claimed.push(ty_name);
        }
        // A node that READS a required-upstream column with nothing wired into it has no
        // stream to act on — the ROOT cause, and it subsumes any inert output it might
        // otherwise produce (no input → no live output), so it is reported alone.
        if let Some(col) = missing_upstream(graph, reg, inst.id, ty) {
            out.push(Diagnostic {
                node: inst.id,
                deficit: Deficit::MissingSource(col),
                fix: Fix::Offer,
            });
            continue;
        }
        // A declared-required input port with no edge (`duplicator` missing `shape`/
        // `points`): nothing to work with. The root cause too — a node missing a required
        // input produces nothing — so it is reported alone.
        if let Some(port) = missing_input(graph, reg, inst.id, ty) {
            out.push(Diagnostic {
                node: inst.id,
                deficit: Deficit::MissingInput(port),
                fix: Fix::Offer,
            });
            continue;
        }
        // Um buraco no meio das portas de um roteador: o índice existe e lê zero em
        // silêncio. Reportado ANTES dos transientes pela mesma lei — a ramificação morta é
        // a causa, e a saída do nó não é "inerte", é *parcialmente* muda.
        if let Some(port) = dead_branch(graph, reg, inst.id, ty) {
            out.push(Diagnostic {
                node: inst.id,
                deficit: Deficit::DeadBranch(port),
                fix: Fix::Offer,
            });
            continue;
        }
        let param = param_reader(graph, reg, inst.id, ty);
        for &col in TRANSIENT_COLUMNS {
            if !produces(reg, ty, col, &param) {
                continue;
            }
            if consumer_reachable(graph, reg, inst.id, col) {
                continue; // healthy: something downstream reads it
            }
            let fix = if consumer_exists_anywhere(graph, reg, col) {
                // A consumer exists but off this producer's path: the answer is to
                // reorder, not to insert a *second* one.
                Fix::Reorder
            } else if let Some(consumer) =
                canonical_consumer(col, particle_upstream(graph, inst.id))
            {
                Fix::Insert(consumer)
            } else {
                Fix::Offer
            };
            out.push(Diagnostic {
                node: inst.id,
                deficit: Deficit::InertProducer(col),
                fix,
            });
        }
    }
    out
}

/// The canonical PLUMBING node that consumes `col`, when the cure is an
/// unambiguous insert. Only `accel` has one (the integrator); `inv_mass`/`falloff`
/// are creative choices with more than one reasonable consumer, so they return
/// `None` and become an [`Fix::Offer`]. The single door for "what heals this
/// column?" — the auto-heal (W2) asks the same function.
#[must_use]
pub fn canonical_consumer(col: &str, particle: bool) -> Option<&'static str> {
    match (col, particle) {
        ("accel", true) => Some("sim.step"),
        ("accel", false) => Some("motion.integrate"),
        _ => None,
    }
}

/// Does the node type `ty` **produce** `col` — write it to its output? United from
/// the two static, registry-queryable sources: a GPU `ColumnBinding` that writes
/// it, or a `Coupling::Produces`.
fn produces(reg: &NodeRegistry, ty: NodeTypeId, col: &str, param: &dyn Fn(&str) -> f32) -> bool {
    coupling_produces(reg, ty, col, param) || gpu_binding(reg, ty, col, is_producer)
}

/// **Como se lê um param de uma instância, com o default do manifesto por baixo.**
///
/// ⚠️ Sem o fallback ao default, um nó que nunca teve o param TOCADO leria `0.0`,
/// e um `0.0` que por acaso é o modo produtor faria o diagnóstico marcar toda
/// instância recém-criada. O `EvalCtx::param` do cook já resolve assim; esta é a
/// mesma escada, do lado de fora do cook.
fn param_reader<'a>(
    graph: &'a Graph,
    reg: &'a NodeRegistry,
    node: NodeId,
    ty: NodeTypeId,
) -> impl Fn(&str) -> f32 + 'a {
    move |name: &str| {
        graph
            .node_params()
            .get(&node)
            .and_then(|m| m.get(name))
            .copied()
            .unwrap_or_else(|| {
                reg.manifests()
                    .find(|m| m.id == ty)
                    .and_then(|m| m.params.iter().find(|p| p.name == name))
                    .map_or(0.0, |p| p.default)
            })
    }
}

/// Does the node type `ty` **consume** `col` — read it without re-producing it (a
/// pure read, including `Consume`)? A re-producer (read + write, like a force
/// accumulating `accel` or a field composing `falloff`) is NOT a consumer — that
/// is what keeps two forces with no integrator inert.
fn consumes(reg: &NodeRegistry, ty: NodeTypeId, col: &str) -> bool {
    coupling_consumes(reg, ty, col) || gpu_binding(reg, ty, col, |a| a.reads() && !is_producer(a))
}

/// The first [`REQUIRED_UPSTREAM`] column `ty` READS while `node` has nothing wired into
/// it — a deformer/force floating with no points to work on. `None` if the node is fed
/// (any incoming edge) or reads no required column. Returns the column so [`diagnose`]
/// can name it in the [`Deficit::MissingSource`].
fn missing_upstream(
    graph: &Graph,
    reg: &NodeRegistry,
    node: NodeId,
    ty: NodeTypeId,
) -> Option<&'static str> {
    if has_input(graph, node) {
        return None; // fed: not source-less (the safe, under-warning direction)
    }
    if fed_by_a_delayed_edge(graph, node) {
        // ⚠️ **QUALQUER aresta atrasada é um stream**, e não só a auto-alça.
        //
        // A regra era `seeds_own_state` — *"a aresta atrasada do próprio nó para si"* —,
        // escrita para as sims que se semeiam (boids / verlet / soft-body / spring /
        // integrate). Ela é estreita DEMAIS, e o preço foi medido em 2026-08-20: numa
        // cadeia de força o laço é `integrate ⟿pre⟿ força ⟿fwd⟿ integrate.forces`, então
        // a força é alimentada por uma aresta atrasada que vem de OUTRO nó. Ela tinha
        // stream, e o diagnoser dizia que não.
        //
        // ⚠️ **E o badge era VISÍVEL:** `inert_badges` filtra por `reaches_output`, e uma
        // força no laço alcança. Toda cadeia de força correctamente montada exibia um ⚠
        // a dizer *"este nó não tem nada ligado"* — inclusive a que o próprio AUTO-HEAL
        // constrói (`heal.rs`: *"reconcile plumbs integrate.out ⟿pre⟿ chain_head.in0"*).
        // ⛔ *O diagnoser acusava a cura que a casa aplica.*
        //
        // Medido pela sonda `which_conference_scenes_the_diagnoser_flags`: **seis** cenas
        // da conferência (`=3`, `=31`, `=38`, `=57`, `=61`, `=71`), todas com a mesma
        // forma, todas correctas.
        //
        // ⚠️ A direcção continua a ser a segura (SOB-avisar): um nó com aresta nenhuma
        // segue reportado, que é o defeito real que este déficit existe para pegar.
        return None;
    }
    REQUIRED_UPSTREAM
        .iter()
        .copied()
        .find(|&col| reads_column(reg, ty, col))
}

/// **O nó recebe um stream por uma aresta ATRASADA?** — a auto-alça de uma sim que se
/// semeia (`motion.integrate` / `spring` / `boids` / `verlet_rope` / `soft_body`) **ou** o
/// `pre` que um integrador manda à cabeça de uma cadeia de força.
///
/// Nos dois casos o `P` que ele lê é o do tique anterior, e nos dois casos ele TEM
/// stream — que é a única coisa que este déficit pergunta. ⚠️ Exigir que a aresta fosse
/// do próprio nó era a versão estreita desta regra, e ela acusava toda cadeia de força
/// (ver [`missing_upstream`]).
fn fed_by_a_delayed_edge(graph: &Graph, node: NodeId) -> bool {
    graph.edges().iter().any(|e| e.delayed && e.to.0 == node)
}

/// Does `ty` READ `col` on its input — a deformer/force that needs the column present to
/// do anything? Derived from a `reads()` GPU binding, or a `Coupling::Requires` (the
/// declared half, for a CPU-only reader with no GPU kernel — the symmetric twin of
/// [`coupling_produces`]). A pure `Write` (a source) does NOT `reads()`, so a generator is
/// never judged to need a column upstream.
fn reads_column(reg: &NodeRegistry, ty: NodeTypeId, col: &str) -> bool {
    coupling_requires(reg, ty, col) || gpu_binding(reg, ty, col, ColumnAccess::reads)
}

/// Does any non-`delayed` edge feed `node` (on any port)? A node with none is a floating
/// head — nothing upstream to bring it a stream.
fn has_input(graph: &Graph, node: NodeId) -> bool {
    graph.edges().iter().any(|e| !e.delayed && e.to.0 == node)
}

/// The first declared-required input PORT of `ty` (`NodeRegistry::required_inputs`) that
/// has no incoming non-`delayed` edge — a `motion.duplicator` missing `shape` or `points`.
/// Returns the port NAME so [`diagnose`] can name it. Unlike [`missing_upstream`] (a
/// column read with no stream at all), this is a per-PORT structural requirement the node
/// declares, because required-vs-optional is semantic (an integrator's `forces` is
/// optional) and not derivable.
fn missing_input(
    graph: &Graph,
    reg: &NodeRegistry,
    node: NodeId,
    ty: NodeTypeId,
) -> Option<&'static str> {
    let required = reg.required_inputs(ty)?;
    let manifest = reg.manifests().find(|m| m.id == ty)?;
    required.iter().copied().find(|&name| {
        // Resolve the port name to its index (a stale name the manifest lacks is skipped,
        // never a crash), then check that port has no edge.
        manifest
            .inputs
            .iter()
            .position(|p| p.name == name)
            .is_some_and(|idx| {
                let idx = idx as u16;
                !graph
                    .edges()
                    .iter()
                    .any(|e| !e.delayed && e.to == (node, idx))
            })
    })
}

/// **A primeira porta VAZIA que ainda tem uma porta LIGADA depois dela**, num nó que
/// roteia por ÍNDICE.
///
/// ⚠️ **A regra é DERIVADA da forma do manifesto, não de uma lista de nomes** — um nó que
/// oferece uma porta `select` **e** portas `in0`, `in1`, … roteia por índice, e é isso que
/// torna uma porta vazia um índice morto. A alternativa (uma const com `"value.switch"`
/// dentro, como a `SINGLETON_SCREEN_PASSES`) era possível e é pior aqui: ali a lista existe
/// porque *"ser um passe de tela"* não se lê do manifesto; aqui lê-se. Um roteador novo com
/// esta forma **nasce coberto**.
///
/// ⚠️ **E a forma exclui os vizinhos que também têm `in0..in3`:** o `motion.combine`
/// concatena o que estiver ligado e não tem `select` — nele um buraco é inofensivo, e um
/// aviso ali seria o falso positivo que faz o artista desligar os avisos.
fn dead_branch(
    graph: &Graph,
    reg: &NodeRegistry,
    node: NodeId,
    ty: NodeTypeId,
) -> Option<&'static str> {
    let manifest = reg.manifests().find(|m| m.id == ty)?;
    if !manifest.inputs.iter().any(|p| p.name == "select") {
        return None;
    }
    let wired = |idx: usize| {
        graph
            .edges()
            .iter()
            .any(|e| !e.delayed && e.to == (node, idx as u16))
    };
    let routed: Vec<(usize, &'static str)> = manifest
        .inputs
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            p.name
                .strip_prefix("in")
                .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        })
        .map(|(i, p)| (i, p.name))
        .collect();
    let last_wired = routed.iter().rposition(|(i, _)| wired(*i))?;
    routed[..last_wired]
        .iter()
        .find(|(i, _)| !wired(*i))
        .map(|(_, name)| *name)
}

/// Any access that can WRITE the column to the node's output. `ReadWriteExisting`
/// writes only when the input carries the column, but it is still a producer of it
/// (never a pure-read consumer) — the distinction between conditional and
/// unconditional writing does not matter for the transient columns, none of which
/// is touched by a `ReadWriteExisting` binding.
fn is_producer(a: ColumnAccess) -> bool {
    matches!(
        a,
        ColumnAccess::Write | ColumnAccess::ReadWrite | ColumnAccess::ReadWriteExisting
    )
}

/// Does `ty` declare a GPU `ColumnBinding` for `col` whose access satisfies `pred`?
/// This is the derived, drift-proof half of a role: the binding set is the same one
/// the device sequencer reads, so it can never disagree with the cook.
fn gpu_binding(
    reg: &NodeRegistry,
    ty: NodeTypeId,
    col: &str,
    pred: impl Fn(ColumnAccess) -> bool,
) -> bool {
    reg.gpu_kernel(ty)
        .is_some_and(|k| k.bindings.iter().any(|b| b.column == col && pred(b.access)))
}

/// The declared half of "produces": a `Coupling::Produces(col)` (the CPU-only
/// stragglers with no GPU kernel).
fn coupling_produces(
    reg: &NodeRegistry,
    ty: NodeTypeId,
    col: &str,
    param: &dyn Fn(&str) -> f32,
) -> bool {
    reg.couplings(ty).is_some_and(|cs| {
        cs.iter().any(|c| match c {
            Coupling::Produces(x) => *x == col,
            // ⚠️ O predicado é perguntado SÓ depois de a coluna casar: ele é
            // arbitrário e não deve correr para toda coluna transiente do repo.
            Coupling::ProducesWhen(x, when) => *x == col && when(param),
            _ => false,
        })
    })
}

/// The declared half of "consumes": a `Coupling::Consumes(col)`.
fn coupling_consumes(reg: &NodeRegistry, ty: NodeTypeId, col: &str) -> bool {
    reg.couplings(ty).is_some_and(|cs| {
        cs.iter()
            .any(|c| matches!(c, Coupling::Consumes(x) if *x == col))
    })
}

/// The declared half of "reads": a `Coupling::Requires(col)`. The GPU-resident
/// deformers/forces all declare a `reads()` binding, so no node needs this TODAY; it is
/// the door for a future CPU-only P-requirer, and it is what gives the `Coupling::Requires`
/// variant a consumer.
fn coupling_requires(reg: &NodeRegistry, ty: NodeTypeId, col: &str) -> bool {
    reg.couplings(ty).is_some_and(|cs| {
        cs.iter()
            .any(|c| matches!(c, Coupling::Requires(x) if *x == col))
    })
}

/// Is a node that consumes `col` reachable from `from` via forward (non-`delayed`)
/// edges? `from` itself is excluded — a node does not heal its own output.
fn consumer_reachable(graph: &Graph, reg: &NodeRegistry, from: NodeId, col: &str) -> bool {
    let mut seen = BTreeSet::new();
    seen.insert(from);
    let mut stack = vec![from];
    while let Some(n) = stack.pop() {
        for e in graph.edges() {
            if e.from.0 == n && !e.delayed && seen.insert(e.to.0) {
                if node_consumes(graph, reg, e.to.0, col) {
                    return true;
                }
                stack.push(e.to.0);
            }
        }
    }
    false
}

/// Does any node in the whole graph consume `col`? (Distinguishes "no consumer at
/// all → insert" from "a consumer exists but off my path → reorder".)
fn consumer_exists_anywhere(graph: &Graph, reg: &NodeRegistry, col: &str) -> bool {
    graph
        .nodes()
        .iter()
        .any(|inst| consumes(reg, NodeTypeId::of(&inst.type_name), col))
}

/// Does the node with this id consume `col`? (Resolves id → type_name → role.)
fn node_consumes(graph: &Graph, reg: &NodeRegistry, node: NodeId, col: &str) -> bool {
    graph
        .nodes()
        .iter()
        .find(|n| n.id == node)
        .is_some_and(|n| consumes(reg, NodeTypeId::of(&n.type_name), col))
}

/// Is a `sim.spawn` upstream of `node` (feeding it via forward edges)? A
/// `sim.spawn` chain wants `sim.step`; everything else wants `motion.integrate`.
fn particle_upstream(graph: &Graph, node: NodeId) -> bool {
    let spawn = NodeTypeId::of("sim.spawn");
    let mut seen = BTreeSet::new();
    seen.insert(node);
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        if graph
            .nodes()
            .iter()
            .any(|i| i.id == n && NodeTypeId::of(&i.type_name) == spawn)
        {
            return true;
        }
        for e in graph.edges() {
            if e.to.0 == n && !e.delayed && seen.insert(e.from.0) {
                stack.push(e.from.0);
            }
        }
    }
    false
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

// ⚠️ **A segunda espécie de diagnóstico, e ela mora num MÓDULO PRÓPRIO de propósito.**
// Tudo acima é PURO e estrutural (grafo + registry, sem cook); a regra do `reads` só
// pode ser respondida pelo stream COZIDO, e misturá-la no `Deficit` esconderia
// exactamente essa distinção — quem chama `diagnose` não precisa de um cook, quem
// chama `unresolved_reads` precisa.
pub mod reads;
pub use reads::{UnresolvedRead, unresolved_reads};
