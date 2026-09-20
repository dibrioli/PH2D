//! **Setup auto-heal** (ADR-0155 W2) — sibling of `motion_bridge_adapt` (shell LOC
//! cap). `super` is `render_loop::motion_bridge`.
//!
//! The Adapter heals a *refused* wire; this heals a *successful-but-inert* graph.
//! A `force.*` accumulates `accel`; only an integrator consumes it. A force wired
//! toward the sink with no integrator writes `accel`, nothing reads it, and the
//! scene stays static — with no error (`ph2d_motion_diagnose` is the analysis that
//! finds it). After a CONSTRUCTIVE gesture, [`heal_setup`] wires every inert force
//! chain that reaches a sink through the integrator it forgot, as its own undo step
//! (the artist can undo just the fix), with a toast + selection so it is never
//! silent.
//!
//! ⚠️ **The cure is a RESTRUCTURE, not a splice** (doc 87 §3.4). `motion.integrate`
//! reads the base positions on `rest` (port 0) and the force-chain output on
//! `forces` (port 1, the feedback port `reconcile` owns). A force cannot live in the
//! horizontal `grid → force → output` chain: the healed graph is
//! `source → integrate.rest`, `last_force → integrate.forces`, `integrate → consumer`,
//! and `reconcile` plumbs `integrate.out ⟿pre⟿ chain_head.in0`. Healing therefore
//! REROUTES the artist's `source → force` edge onto the integrator — the only wiring
//! that actually moves the points.

use super::{MotionState, reconcile};
use ph2d_editor_core::{Toast, ToastQueue};
use ph2d_motion_diagnose::{Deficit, Diagnostic, Fix, diagnose};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};
use ph2d_nodegraph::node::NodeTypeId;
use ph2d_panel_motion_graph::GraphIntent;
use std::collections::BTreeSet;

/// A gesture that can COMPLETE a wrong setup (add a node / a wire). The heal fires
/// only after a batch that has one of these and no [`is_destructive`] intent — so
/// deleting the integrator to rewire never triggers a re-insert.
pub(super) fn is_constructive(i: &GraphIntent) -> bool {
    matches!(
        i,
        GraphIntent::Connect { .. }
            | GraphIntent::AddNode { .. }
            | GraphIntent::SpliceNode { .. }
            | GraphIntent::SpliceReroute { .. }
            | GraphIntent::SmartConnect { .. }
            | GraphIntent::SmartConnectBack { .. }
            | GraphIntent::Paste
            | GraphIntent::DuplicateSelection { .. }
    )
}

/// A gesture that REMOVES nodes/edges — never a reason to heal (the artist is taking
/// the graph apart, possibly to rewire it by hand).
pub(super) fn is_destructive(i: &GraphIntent) -> bool {
    matches!(
        i,
        GraphIntent::Disconnect { .. }
            | GraphIntent::DeleteSelection { .. }
            | GraphIntent::CutWires { .. }
            | GraphIntent::MoveWireEnd { .. }
            // ⚠️ **As duas largadas de 2026-09-19 são DESTRUTIVAS**, e a classificação é uma
            // decisão: as duas desligam fios de propósito (a troca reescreve TODAS as ligações
            // dos dois nós; o splice tira o nó da cadeia onde estava). *O artista está a refazer
            // a fiação à mão, e uma cura automática por cima disso desfaz o que ele acabou de
            // fazer* — a mesma leitura que o `MoveWireEnd` já carrega.
            | GraphIntent::SwapInChain { .. }
            | GraphIntent::SpliceExistingIntoWire { .. }
    )
}

/// The whole heal for one force chain: the ids to disconnect + the integrator to
/// splice, all resolved on the ORIGINAL graph so applying one chain's plan never
/// breaks the planning of another.
struct HealPlan {
    /// The chain's head force — its `in0` (freed from `source`) becomes the pre-loop.
    head: NodeId,
    /// The node feeding the head's `in0` + its output port. For [`HealKind::Insert`]
    /// it is the base positions (`grid`), fed into the NEW `integrate.rest`; for
    /// [`HealKind::Reuse`] it IS the existing integrator (its output fed the head).
    source: (NodeId, u16),
    /// The last force in the chain — feeds `integrate.forces`.
    last: NodeId,
    /// The node the last force feeds, and the input port (the sink side).
    consumer: (NodeId, u16),
    /// Create the integrator, or reuse the one already feeding the chain head.
    kind: HealKind,
    /// Where to drop a NEWLY-inserted integrator (ignored when reusing one).
    pos: Pos,
}

/// Whether the inert chain needs a NEW integrator or already has one feeding its head.
enum HealKind {
    /// No integrator on the chain: create one of this type (`motion.integrate`, or
    /// `sim.step` in a particle chain), and `source` (the base positions) feeds its
    /// `rest`. The Enio's canonical case — a force in the horizontal chain.
    Insert(&'static str),
    /// An integrator already feeds the chain head (`grid → integrate → force → …` — a
    /// force spliced DOWNSTREAM of a working integrator): reuse it (`source.0`), whose
    /// `rest` is already wired, and only the force-chain and sink sides move.
    Reuse,
}

/// **Heal the setup after a constructive gesture.** For every force chain that is
/// inert (ADR-0155), reaches a sink, and has a clean linear shape, wire it through
/// the integrator it forgot — one undo step for the whole heal. Returns how many
/// integrators were inserted (0 = nothing to do). The caller gates this on a
/// constructive-only batch.
pub(super) fn heal_setup(motion: &mut MotionState, toasts: &mut ToastQueue) -> usize {
    if !motion.node_help_enabled {
        return 0; // node help off (ADR-0155): the toolbar chip turned the system off
    }
    // Plan every inert insert-fixable producer that reaches a sink, on the original
    // graph, deduped by chain head (many inert forces in one chain heal once).
    let mut plans: Vec<HealPlan> = Vec::new();
    let mut heads: BTreeSet<u32> = BTreeSet::new();
    for d in diagnose(&motion.doc.graph, &motion.registry) {
        if !reaches_output(&motion.doc.graph, d.node) {
            continue; // a dangling mid-build force (no sink) is not a completed setup
        }
        // `plan_heal` is the ONE door on "is there a canonical cure?": it returns `Some`
        // only for `Fix::Insert` and the reuse case of `Fix::Reorder` (an integrator already
        // feeds the head), `None` for everything else (a pin's `Offer`, an indirect `Reorder`
        // with a transform between the integrator and the force — those are W3-advisory badges).
        // No `matches!(d.fix, ...)` pre-filter: it would be a redundant policy layer.
        if let Some(plan) = plan_heal(&motion.doc.graph, &motion.registry, d.node, d.fix)
            && heads.insert(plan.head.0)
        {
            plans.push(plan);
        }
    }
    if plans.is_empty() {
        return 0;
    }

    // Apply every plan to one trial clone; commit only if it validates.
    let mut trial = motion.doc.graph.clone();
    let mut inserted: Vec<NodeId> = Vec::new();
    for plan in &plans {
        if let Some(node) = apply_heal(&mut trial, plan) {
            inserted.push(node);
        }
    }
    if inserted.is_empty() || trial.validate(&motion.registry).is_err() {
        return 0; // never ship a heal the cook would then reject
    }

    let pre = motion.doc.clone();
    motion.doc.graph = trial;
    // `reconcile` plumbs `integrate.out ⟿pre⟿ chain_head.in0` (the head's in0 is now
    // free), so the force reads last tick's integrated state — the loop the user
    // never draws.
    reconcile(motion, &pre.graph);
    motion.history.push_undo(pre);
    motion.pump.mark_dirty();
    ph2d_panel_motion_graph::request_graph_selection(inserted.iter().map(|n| n.0).collect());
    toasts.push(Toast::info(if inserted.len() == 1 {
        ph2d_i18n::tr(
            "app.motion.motion_bridge_heal.wired_the_force_through_integrate_so_it_moves_th",
        )
    } else {
        ph2d_i18n::tr(
            "app.motion.motion_bridge_heal.wired_the_forces_through_integrate_so_they_move",
        )
    }));
    inserted.len()
}

/// The node ids that get a ⚠ inert badge (ADR-0155): every producer the diagnoser found
/// dead whose output nonetheless REACHES a sink — a completed-but-inert setup, the thing the
/// artist cannot see is wrong. A mid-build force that reaches no output yet is unfinished, not
/// wrong, so it gets no badge — the same [`reaches_output`] filter the auto-heal uses to wait
/// for the wiring to finish. Computed fresh from the current graph, so it is always what the
/// wiring now says (the panel paints from this, via the snapshot).
pub(super) fn inert_reaching_output(motion: &MotionState) -> BTreeSet<u32> {
    if !motion.node_help_enabled {
        return BTreeSet::new(); // node help off (ADR-0155): no ⚠ badges
    }
    let mut out: BTreeSet<u32> = diagnose(&motion.doc.graph, &motion.registry)
        .into_iter()
        .filter(|d| reaches_output(&motion.doc.graph, d.node))
        .map(|d| d.node.0)
        .collect();
    // ⚠️ **A segunda espécie, e ela NÃO passa pelo `reaches_output`.** Aquele filtro
    // existe para não marcar um produtor a meio de montagem (*"ainda não liguei o
    // integrador"* é inacabado, não errado). Um nome que não resolve é errado
    // AGORA — o nó já tem fonte, já cozeu, e já está a ler zeros; esperar que a
    // cadeia alcance uma saída atrasaria o aviso para depois de o artista ter
    // construído em cima do engano.
    out.extend(unresolved_names(motion).into_iter().map(|u| u.node.0));
    out
}

/// Os nomes que não resolvem, com a stream VIVA por trás da resposta (a porta
/// `columns`, que sabe ler o memo do cook e a tomada de GPU).
///
/// ⚠️ Este é o único diagnóstico do editor que precisa de um COOK — os outros são
/// estruturais —, e é por isso que a regra mora num módulo próprio da crate e não
/// no `Deficit`.
fn unresolved_names(motion: &MotionState) -> Vec<ph2d_motion_diagnose::UnresolvedRead> {
    // ⚠️ **DEFESA EM CAMADAS, e a mutação mediu-a:** os DOIS chamadores de hoje
    // (`inert_reaching_output` e `heal_one`) já saem cedo com o chip desligado, então
    // apagar esta linha **não sangra** nenhum gate de comportamento. Ela fica — e
    // ganha gate PRÓPRIO — porque é ela que torna este helper seguro de chamar de
    // qualquer sítio: o terceiro chamador não pode nascer a analisar um grafo cujo
    // dono pediu para não ser analisado.
    if !motion.node_help_enabled {
        return Vec::new(); // node help off (ADR-0155): sem badges
    }
    ph2d_motion_diagnose::unresolved_reads(&motion.doc.graph, &motion.registry, &|n, p| {
        super::columns::names_at(motion, n, p)
    })
}

/// Apply the fix under a ⚠ badge the artist clicked (ADR-0155). A CANONICAL fix — the
/// integrator a chain forgot ([`Fix::Insert`]) or the one already feeding the chain head
/// ([`Fix::Reorder`], the reuse case) — is applied, one undo step, with a toast + selection.
/// ⚠️ This fires even after a DESTRUCTIVE gesture (unlike [`heal_setup`], which never re-inserts
/// on its own): the click IS the request, so re-adding an integrator the artist just deleted
/// honours them instead of fighting them. A case with NO canonical fix (a pin that needs *some*
/// solver, a force with a transform between it and the integrator) is EXPLAINED + selected —
/// never guessed, the ADR-0155 law.
pub(super) fn heal_one(motion: &mut MotionState, toasts: &mut ToastQueue, node: NodeId) {
    if !motion.node_help_enabled {
        return; // node help off (ADR-0155): a stale badge click is a clean no-op
    }
    // Re-diagnose: the badge was painted from LAST frame's graph, so a stale click on a
    // node that already healed itself (or is no longer inert) is a clean no-op.
    let Some(d) = diagnose(&motion.doc.graph, &motion.registry)
        .into_iter()
        .find(|d| d.node == node)
    else {
        // ⚠️ Não é um produtor inerte — pode ser a OUTRA espécie. Um nome que não
        // resolve nunca tem cura canônica (qual coluna o artista queria é escolha
        // dele, e adivinhar é exactamente o que o ADR-0155 proíbe): selecionar e
        // EXPLICAR, citando o nome que ele escreveu.
        if let Some(u) = unresolved_names(motion)
            .into_iter()
            .find(|u| u.node == node)
        {
            ph2d_panel_motion_graph::request_graph_selection(vec![node.0]);
            toasts.push(Toast::info(ph2d_i18n::tr_with(
                "app.motion.motion_bridge_heal.nothing_upstream_carries_a_column_called_so_this",
                &[("column", &(u.column))],
            )));
        }
        return;
    };
    // `plan_heal` is the ONE door on "does this fix have a canonical cure?" — it returns
    // `Some` only for `Fix::Insert` and the reuse case of `Fix::Reorder` (an integrator
    // already feeds the head); everything else (a pin's `Offer`, an indirect `Reorder`)
    // returns `None`. A second `matches!(d.fix, ...)` guard here would be a redundant policy
    // layer that no single mutation could flip (plan_heal always catches what it would).
    if reaches_output(&motion.doc.graph, node)
        && let Some(plan) = plan_heal(&motion.doc.graph, &motion.registry, node, d.fix)
    {
        let mut trial = motion.doc.graph.clone();
        if let Some(integ) = apply_heal(&mut trial, &plan)
            && trial.validate(&motion.registry).is_ok()
        {
            let pre = motion.doc.clone();
            motion.doc.graph = trial;
            // `reconcile` plumbs `integrate.out ⟿pre⟿ chain_head.in0` — the loop the artist
            // never draws — exactly as `heal_setup` does.
            reconcile(motion, &pre.graph);
            motion.history.push_undo(pre);
            motion.pump.mark_dirty();
            ph2d_panel_motion_graph::request_graph_selection(vec![integ.0]);
            toasts.push(Toast::info(ph2d_i18n::tr(
                "app.motion.motion_bridge_heal.wired_the_force_through_integrate_so_it_moves_th",
            )));
            return;
        }
    }
    // No canonical fix: point the artist at the node and say what it needs. Guessing a
    // solver / a reorder here is exactly what the ADR forbids.
    ph2d_panel_motion_graph::request_graph_selection(vec![node.0]);
    toasts.push(Toast::info(explain(&d)));
}

/// The English advisory for an inert producer with no canonical fix (app UI, HR-15).
/// Returns `String` because [`Deficit::MissingInput`] names the PORT it needs in the text.
fn explain(d: &Diagnostic) -> String {
    match (d.deficit, d.fix) {
        // A force downstream of an integrator, with a NON-integrator between them: reusing that
        // integrator would double-integrate a moving base, so the artist must place the fix.
        (Deficit::InertProducer("accel"), Fix::Reorder) => ph2d_i18n::tr(
            "app.motion.motion_bridge_heal.wire_this_force_upstream_of_the_integrator_so_it",
        )
        .into(),
        // A pin_constraint's inv_mass with no solver: WHICH solver is a creative choice.
        (Deficit::InertProducer("inv_mass"), _) => ph2d_i18n::tr(
            "app.motion.motion_bridge_heal.this_constraint_needs_a_solver_integrate_sim_ste",
        )
        .into(),
        // A field's falloff read by no force/deformer: WHICH modulator is a creative choice.
        (Deficit::InertProducer("falloff"), _) => ph2d_i18n::tr(
            "app.motion.motion_bridge_heal.this_field_shapes_a_falloff_that_no_force_or_def",
        )
        .into(),
        // A deformer/force with nothing wired into it: it reads P but has no stream — the
        // ROOT cause. WHICH source (grid / emitter / object) is a creative choice.
        (Deficit::MissingSource("P"), _) => ph2d_i18n::tr(
            "app.motion.motion_bridge_heal.this_node_has_no_points_to_work_on_wire_a_source",
        )
        .into(),
        (Deficit::MissingSource(_), _) => ph2d_i18n::tr(
            "app.motion.motion_bridge_heal.this_node_has_nothing_wired_into_it_so_it_has_no",
        )
        .into(),
        // A required input port (a duplicator's shape/points) with nothing wired in: WHAT to
        // wire is the artist's choice, so it is named and offered, never guessed.
        (Deficit::MissingInput(port), _) => ph2d_i18n::tr_with(
            "app.motion.motion_bridge_heal.this_node_needs_a_stream_wired_into_its_input",
            &[("port", &port)],
        ),
        // ⭐⭐ **O sujeito do nó é escolhido por NOME e ninguém o escolheu.** A frase nomeia o
        // gesto (*escolher*) e não o param, porque o que o artista tem de FAZER é escolher um
        // caminho — e diz o que o nó está a fazer entretanto, que é a metade que faltava: sem
        // ela ele vê uma cadeia completa a não mudar nada e conclui que o nó está avariado.
        (Deficit::MissingChoice(_), _) => ph2d_i18n::tr(
            "app.motion.motion_bridge_heal.this_node_has_no_path_chosen_yet_so_it_passes_th",
        )
        .into(),
        // ⚠️ **Um irmão já ocupa o passe de tela.** A mensagem nomeia o TIPO porque é o que
        // o artista tem de procurar no grafo, e diz qual dos dois manda (o primeiro) — sem
        // isso ele apaga o errado e o efeito muda de aparência.
        (Deficit::Shadowed(ty), _) => ph2d_i18n::tr_with(
            "app.motion.motion_bridge_heal.another_already_drives_this_screen_pass_only_the",
            &[("ty", &ty)],
        ),
        // ⚠️ **Uma ramificação morta no meio do roteador.** A mensagem nomeia a PORTA (é o
        // que ele tem de ligar) e diz o que o índice dela devolve hoje — sem isso ele lê um
        // zero e não sabe se a ramificação está vazia ou se ela vale zero.
        (Deficit::DeadBranch(port), _) => ph2d_i18n::tr_with(
            "app.motion.motion_bridge_heal.input_is_empty_but_a_later_one_is_wired_that_bra",
            &[("port", &port)],
        ),
        // ⭐⭐⭐ **Só posições, e nada a jusante que as vista.** A frase diz as TRÊS coisas que
        // o artista precisa: o que ele está a ver AGORA (marcas), o que falta (um duplicador) e
        // o que o duplicador quer (uma forma) — sem a primeira ele lê «não funciona», e sem a
        // terceira ele põe o duplicador e volta a ficar sem nada.
        (Deficit::SemQuemVista, _) => ph2d_i18n::tr(
            "app.motion.motion_bridge_heal.this_node_only_gives_positions_add_a_duplicator",
        )
        .into(),
        _ => ph2d_i18n::tr(
            "app.motion.motion_bridge_heal.this_node_produces_data_nothing_downstream_consu",
        )
        .into(),
    }
}

/// Plan the restructure for the chain containing `force`. **The ONE authority on "can this
/// fix be auto-applied?"** — both [`heal_one`] (a badge click) and [`heal_setup`] (the
/// post-gesture sweep) ask it, and neither pre-filters `d.fix`, because a `Some` here already
/// means the fix is `Insert` or a reusable `Reorder`. `None` for: a fix with no `HealKind`
/// (`Offer`, an indirect `Reorder`), a dangling chain (head has no source, or the last force
/// has no single consumer) — those are advisory badges the artist resolves, not auto-heals.
fn plan_heal(
    g: &Graph,
    reg: &ph2d_node_registry::NodeRegistry,
    force: NodeId,
    fix: Fix,
) -> Option<HealPlan> {
    // Walk back through accel-producers to the head; its in0 feeder is the source.
    let mut head = force;
    let source = loop {
        let feed = feeder(g, head, 0)?; // no feeder → dangling head → cannot heal
        if produces_accel(g, reg, feed.0) {
            head = feed.0;
        } else {
            break feed;
        }
    };
    // Insert makes a NEW integrator (`source` is the base positions); Reorder reuses
    // the one that must feed the head DIRECTLY. If a non-integrator (a transform) sits
    // between an existing integrator and the force, reusing would double-integrate (its
    // `rest` would be a moving, transformed base) — leave it for a W3 badge.
    let kind = match fix {
        Fix::Insert(integ) => HealKind::Insert(integ),
        Fix::Reorder if consumes_accel(g, reg, source.0) => HealKind::Reuse,
        _ => return None,
    };
    // Walk forward through accel-producers to the last force; its single forward edge
    // is the consumer (the sink side).
    let mut last = force;
    let consumer = loop {
        let outs = forward_edges(g, last);
        let [only] = outs.as_slice() else {
            // Zero (dangling) or a branch we do not auto-restructure in W1/W2.
            return None;
        };
        if produces_accel(g, reg, only.to.0) {
            last = only.to.0;
        } else {
            break (only.to.0, only.to.1);
        }
    };

    let pos = match (g.pos(last), g.pos(consumer.0)) {
        (Some(a), Some(b)) => Pos {
            x: (a.x + b.x) * 0.5,
            y: (a.y + b.y) * 0.5,
        },
        (Some(a), None) | (None, Some(a)) => a,
        (None, None) => Pos { x: 0.0, y: 0.0 },
    };
    Some(HealPlan {
        head,
        source,
        last,
        consumer,
        kind,
        pos,
    })
}

/// Apply one plan to the graph, returning the integrator now driving the chain (or
/// `None` if a connect unexpectedly fails). Frees the head from its source and the last
/// force from its consumer, then wires `last → integ.forces`, `integ → consumer`. The
/// integrator is either NEW (fed the base positions on `rest`) or the one that already
/// fed the head (its `rest` is already wired — just its self-loop is cleared so the
/// force chain can take `forces`).
fn apply_heal(g: &mut Graph, plan: &HealPlan) -> Option<NodeId> {
    g.disconnect(plan.head, 0); // free source → head (becomes the pre-loop)
    g.disconnect(plan.consumer.0, plan.consumer.1); // free last → consumer
    let integ = match plan.kind {
        HealKind::Insert(ty) => {
            let n = g.add_node(ty);
            g.set_pos(n, plan.pos);
            g.connect(Edge {
                from: plan.source,
                to: (n, 0), // rest ← the base positions (grid)
                delayed: false,
            })
            .ok()?;
            n
        }
        HealKind::Reuse => {
            // The integrator already feeds the head (`source.0`); its `rest` is wired.
            // Clear the engine's self-loop on `forces` so the force chain can take it —
            // `reconcile` re-plumbs the pre-loop to the freed head afterward.
            let n = plan.source.0;
            g.disconnect(n, 1);
            n
        }
    };
    g.connect(Edge {
        from: (plan.last, 0),
        to: (integ, 1), // forces ← the force-chain output
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (integ, 0),
        to: plan.consumer, // integrate → the sink side
        delayed: false,
    })
    .ok()?;
    Some(integ)
}

/// Does `node`'s output reach a `motion.output` (the render sink) via forward
/// (non-`delayed`) edges? A completed wrong setup reaches the output; a dangling
/// mid-build force does not — so healing waits for the artist to finish wiring.
fn reaches_output(g: &Graph, node: NodeId) -> bool {
    let mut seen = BTreeSet::new();
    seen.insert(node);
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        if g.node(n).is_some_and(|i| i.type_name == "motion.output") {
            return true;
        }
        for e in g.edges() {
            if e.from.0 == n && !e.delayed && seen.insert(e.to.0) {
                stack.push(e.to.0);
            }
        }
    }
    false
}

/// Does the node `n` produce `accel` (is it a force)?
fn produces_accel(g: &Graph, reg: &ph2d_node_registry::NodeRegistry, n: NodeId) -> bool {
    g.node(n)
        .and_then(|inst| reg.couplings(NodeTypeId::of(&inst.type_name)))
        .is_some_and(|cs| {
            cs.iter()
                .any(|c| matches!(c, ph2d_node_registry::Coupling::Produces(x) if *x == "accel"))
        })
}

/// Does the node `n` consume `accel` (is it an integrator)? Distinguishes a chain head
/// fed DIRECTLY by an integrator (the Reorder/reuse case) from one fed by a plain
/// source (the Insert case).
fn consumes_accel(g: &Graph, reg: &ph2d_node_registry::NodeRegistry, n: NodeId) -> bool {
    g.node(n)
        .and_then(|inst| reg.couplings(NodeTypeId::of(&inst.type_name)))
        .is_some_and(|cs| {
            cs.iter()
                .any(|c| matches!(c, ph2d_node_registry::Coupling::Consumes(x) if *x == "accel"))
        })
}

/// The node feeding `(node, port)` through a forward (non-`pre`) edge.
fn feeder(g: &Graph, node: NodeId, port: u16) -> Option<(NodeId, u16)> {
    g.edges()
        .iter()
        .find(|e| !e.delayed && e.to == (node, port))
        .map(|e| e.from)
}

/// The forward (non-`delayed`) edges leaving `node`.
fn forward_edges(g: &Graph, node: NodeId) -> Vec<Edge> {
    g.edges()
        .iter()
        .filter(|e| !e.delayed && e.from.0 == node)
        .copied()
        .collect()
}

#[cfg(all(test, feature = "panel-motion-graph"))]
#[path = "motion_bridge_heal_tests.rs"]
mod tests;

// A SEGUNDA espécie de diagnóstico, em arquivo próprio: os gates dela BOMBEIAM
// antes de perguntar (a resposta só existe depois de um cook), e o irmão acima está
// a 569 das 600 linhas do teto.
#[cfg(all(test, feature = "panel-motion-graph"))]
#[path = "motion_bridge_unresolved_tests.rs"]
mod unresolved_tests;
