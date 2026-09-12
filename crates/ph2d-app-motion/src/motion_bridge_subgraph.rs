//! Subgraph intents (Motion Nodes doc 57) — the shell half of the graph panel's
//! nesting: group, ungroup, enter, walk out, and the decode that turns a card's
//! socket back into the real port it stands for.
//!
//! **Grouping is semantically inert.** The graph stays flat; only membership moves.
//! So — exactly like the backdrops, and for the same reason — **nothing here calls
//! `mark_dirty`** except the paths that genuinely destroy nodes (deleting a card).
//! The gate that pins this down is `grouping_never_changes_the_cook`: fold the whole
//! rain into a group and the cooked instance buffer is **byte-identical**.
//!
//! The one thing that would silently break the illusion is a node minted while the
//! artist is inside a group and left at the root — it would vanish the instant it
//! was created. [`adopt_new`] is why that cannot happen: it runs from the ONE
//! reconcile every structural edit already goes through.

use super::{MotionState, fold};
use ph2d_motion_doc::{Subgraph, subgraph};
use ph2d_nodegraph::graph::{Graph, NodeId, Pos};
use std::collections::BTreeSet;

pub(super) use fold::{card_ports, subgraph_of, view_id};

// The nesting clipboard (Ctrl+D / Ctrl+V of a group) lives in a child module for the
// LOC cap; re-exported so callers keep saying `subgraph::duplicate_nesting`/`paste_nesting`.
#[path = "motion_bridge_subgraph_clipboard.rs"]
mod clipboard;
pub(super) use clipboard::{duplicate_nesting, paste_nesting};

/// What a view id names. The panel speaks in view ids (a card and a node are both
/// just cards on the canvas); the shell must know which is which before it touches
/// the document, and this is the only place that decides.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum Target {
    Node(NodeId),
    Card(u32),
}

pub(super) fn target(view: u32) -> Target {
    match subgraph_of(view) {
        Some(sid) => Target::Card(sid),
        None => Target::Node(NodeId(view)),
    }
}

/// **The real `(node, port)` a wire endpoint names.** A node's socket is itself; a
/// card's socket is the crossing port it was derived from ([`fold::card_ports`] —
/// the SAME derivation that drew it). `None` when the slot no longer exists (the
/// crossing wire was cut in the same frame the intent was queued).
pub(super) fn resolve_port(
    motion: &MotionState,
    view: u32,
    port: u16,
    input: bool,
) -> Option<(NodeId, u16)> {
    match target(view) {
        Target::Node(n) => Some((n, port)),
        Target::Card(sid) => {
            let ports = card_ports(motion, sid);
            let slots = if input { ports.inputs } else { ports.outputs };
            slots.get(port as usize).copied()
        }
    }
}

/// **The parameter an input-side port index names**, or `None` when it names a real port
/// (doc 58).
///
/// A node's declared inputs come first, then one socket per driven param in sorted order —
/// the SAME derivation `snapshot_from` used to draw them (`param_source::param_at`). A second
/// derivation here is exactly how a socket comes to mean a different parameter than the one
/// it drew.
pub(super) fn param_at(motion: &MotionState, node: NodeId, port: u16) -> Option<String> {
    param_at_in(&motion.doc.graph, &motion.registry, node, port)
}

/// The same, against a bare graph (the rewire path edits a TRIAL clone).
pub(super) fn param_at_in(
    graph: &ph2d_nodegraph::graph::Graph,
    registry: &ph2d_node_registry::NodeRegistry,
    node: NodeId,
    port: u16,
) -> Option<String> {
    let manifest = fold::manifest_in(graph, registry, node)?;
    let k = (port as usize).checked_sub(manifest.inputs.len())?;
    let sources = graph.param_sources(node)?;
    ph2d_nodegraph::param_source::param_at(sources, k).map(str::to_string)
}

/// **Pull a wire off an input socket** — whichever KIND of socket it is (doc 58).
///
/// The three places that unplug a wire (Disconnect, the knife, a dragged wire-end) all go
/// through here, so none of them has to learn that a parameter exists — the same funnel
/// discipline that keeps `reconcile` from being forgotten at one of its seven call sites.
/// Returns whether anything was actually unplugged; the CALLER owns the undo bracket.
pub(super) fn unplug(motion: &mut MotionState, node: NodeId, port: u16) -> bool {
    let registry = &motion.registry;
    let name = param_at_in(&motion.doc.graph, registry, node, port);
    unplug_in(&mut motion.doc.graph, name.as_deref(), node, port)
}

/// The same, against a bare graph. `param` is what [`param_at_in`] said this socket is
/// (resolved BEFORE the borrow, since the answer lives in the graph we are about to edit).
pub(super) fn unplug_in(
    graph: &mut ph2d_nodegraph::graph::Graph,
    param: Option<&str>,
    node: NodeId,
    port: u16,
) -> bool {
    match param {
        Some(name) => graph.undrive_param(node, name).is_some(),
        None => graph.disconnect(node, port).is_some(),
    }
}

/// **Drive a param from a wire** (the `DriveParam` intent). One undo step; it re-cooks,
/// because unlike grouping this really does change what the graph computes.
pub(super) fn drive(
    motion: &mut MotionState,
    toasts: &mut ph2d_editor::ToastQueue,
    from: (NodeId, u16),
    to: NodeId,
    param: &str,
) {
    // ⛔⛔⛔ **A SEGUNDA RECUSA, que este doc dizia não existir** — report do Enio,
    // 2026-09-01: *«lfo nem oscillator conseguem atuar sobre Angle de Rotate»*.
    //
    // Um param conduzido lê a coluna `v` do condutor (`param_source::driven_value`), e só um
    // porto **escalar** a produz. O `Graph::drive_param` não tem registry, logo **não pode**
    // verificar isso: ele confere que os nós existem e que não se fecha um ciclo, e aceita
    // qualquer fonte. ⇒ ligar um `motion.oscillator` (que emite `Instances/Vec2`, uma
    // corrente por-elemento) a um param **é aceite e não faz nada**: medido em
    // `what_can_drive_the_rotate_angle`, o `rot` fica em `0` para sempre.
    //
    // ⚠️ *Um fio VISÍVEL que não age é pior que um knob morto* — o artista vê a ligação e
    // conclui que ela age. É a mesma frase que o `param_choices` já escrevera sobre os knobs
    // que o painel esconde, e a porta do GESTO não a honrava.
    //
    // ⚠️ **A regra é a MESMA que o menu usa** (`param_choices` declara o alvo como
    // `Instances/Scalar`): uma segunda formulação seria o defeito com o sinal trocado.
    if !source_can_drive(motion, from) {
        // ⭐⭐ **A recusa NOMEIA A CURA** (estudo do Mini Cavalry, doc 99 §10e). Ele resolve
        // isto convertendo em silêncio (23 conversões entre os 7 tipos dele) — o que inventa
        // um resultado que ninguém autorou: *de uma corrente para um número há várias
        // respostas (a média? o máximo? o primeiro?) e ele escolhe uma sem dizer*. Aqui o
        // artista escolhe, e o nó que converte fica **à vista e ajustável**.
        let cura = converter_from(motion, from).map_or_else(String::new, |ty| {
            format!(" — insert a `{ty}` to read one number from it")
        });
        toasts.push(ph2d_editor::Toast::warning(format!(
            "Can't drive: that output is a per-element stream, not a value{cura}"
        )));
        return;
    }
    let pre = motion.doc.clone();
    match motion.doc.graph.drive_param(to, param, from) {
        Ok(()) => {
            motion.history.push_undo(pre);
            motion.pump.mark_dirty();
        }
        // A outra recusa estrutural: fecharia um laço. (Um param não tem caso «já ligado» —
        // uma segunda fonte substitui a primeira, como re-plugar um socket de entrada.)
        Err(_) => {
            toasts.push(ph2d_editor::Toast::warning(
                "Can't drive: that would make a loop",
            ));
        }
    }
}

/// ⭐⭐⭐ **O NÓ QUE CONVERTE esta saída num valor — DERIVADO do registry, nunca uma tabela.**
///
/// ⛔⛔ **É a diferença com ele:** o Mini Cavalry tem **23 conversões escritas à mão**
/// (`shape->point`, `value->color`, …) — uma segunda lista, que pode discordar do catálogo no
/// dia em que um nó mudar de portas. Esta **procura no registry** um tipo cuja entrada aceita o
/// que a fonte emite e cuja saída é um escalar. *A resposta não pode nomear um nó que não faça
/// aquilo, porque é o manifesto dele que a produz.*
///
/// ⚠️ **A ORDEM do catálogo decide o empate**, e é determinística (o registry é um `BTreeMap`):
/// entre dois conversores igualmente válidos sai sempre o mesmo, em toda máquina — que é o que
/// faz a mensagem ser a mesma no ecrã do artista e no gate.
///
/// `None` quando nenhum tipo converte — e aí a recusa fica só com o «porquê», que continua a
/// ser mais do que o silêncio que havia antes de 01/09.
///
/// ⚠️⚠️ **A cerca do TIPO DE ENTRADA é hoje INFALSIFICÁVEL, e isso está declarado em vez de
/// escondido:** a mutação que a apaga (`i.ty == saida` → `is_some()`) **SOBREVIVE** ao gate.
/// Não é redundância — é que o catálogo inteiro deste módulo só tem **uma** forma
/// não-conduzível (`Instances/Vec2`: o censo diz `Domain::Instances` em 100% das 138 portas),
/// então não há um segundo caso contra o qual ela possa discriminar. Ela fica porque é
/// correcta, e torna-se falsificável no dia em que um porto de `Vector`/`Field`/`Signal`
/// entrar num grafo de Motion. *Uma mutação sobrevivente tem três leituras, não duas: falta um
/// gate, a linha é redundante, ou o mundo ainda não tem o caso que a distingue.*
pub(super) fn converter_from(motion: &MotionState, from: (NodeId, u16)) -> Option<&'static str> {
    use ph2d_nodegraph::port::Dim;
    let saida = super::fold::manifest_of(motion, from.0)?
        .outputs
        .get(from.1 as usize)?
        .ty;
    motion
        .registry
        .manifests()
        .find(|m| {
            m.inputs.first().is_some_and(|i| i.ty == saida)
                && m.outputs.first().is_some_and(|o| o.ty.dim == Dim::Scalar)
        })
        .map(|m| m.name)
}

/// **Esta saída produz um VALOR?** — a pergunta que separa um condutor de uma corrente.
///
/// ⚠️ **`Dim::Scalar` é o proxy DECLARADO** do que o cook exige (a coluna `v`): é o mesmo
/// que o `param_choices` põe no alvo que oferece, então o menu e o gesto respondem pela
/// mesma régua. Um nó desconhecido ou uma porta que não existe respondem **não** — o
/// default recusa, que é o lado seguro (um fio que não aparece é uma ausência; um que
/// aparece e não age é uma mentira).
pub(super) fn source_can_drive(motion: &MotionState, from: (NodeId, u16)) -> bool {
    super::fold::manifest_of(motion, from.0).is_some_and(|m| {
        m.outputs
            .get(from.1 as usize)
            .is_some_and(|p| p.ty.dim == ph2d_nodegraph::port::Dim::Scalar)
    })
}

/// The node a readout should point at when the artist probes a CARD: what the group
/// emits (its first output's source). A group with no output emits nothing, and
/// there is nothing to read.
pub(super) fn probe_target(motion: &MotionState, view: u32) -> Option<NodeId> {
    match target(view) {
        Target::Node(n) => Some(n),
        Target::Card(sid) => card_ports(motion, sid).outputs.first().map(|(n, _)| *n),
    }
}

/// **Collapse the selection into a new subgraph** (Ctrl+G). The nodes do not move
/// and do not change id — only membership does — so the cook cannot tell.
///
/// A card in the selection is re-parented rather than dissolved: that is how a nest
/// gets a second storey. The new card lands at the centre of what it swallowed, and
/// arrives SELECTED (the shell mints the id, so only it can say what to select).
pub(super) fn group(motion: &mut MotionState, views: Vec<u32>) {
    let level = motion.level;
    let mut nodes: Vec<NodeId> = Vec::new();
    let mut cards: Vec<u32> = Vec::new();
    for v in views {
        match target(v) {
            Target::Node(n) if motion.doc.graph.node(n).is_some() => nodes.push(n),
            Target::Card(sid) if subgraph::find(&motion.doc.subgraphs, sid).is_some() => {
                cards.push(sid)
            }
            // A stale id from an in-flight gesture: it names nothing, and grouping
            // nothing into something is not an edit.
            _ => {}
        }
    }
    if nodes.is_empty() && cards.is_empty() {
        return;
    }
    let pre = motion.doc.clone();
    let sid = subgraph::next_id(&motion.doc.subgraphs);

    // Land the card at the centre of the cluster it folds.
    let mut sum = (0.0f32, 0.0f32);
    let mut n = 0.0f32;
    for id in &nodes {
        if let Some(p) = motion.doc.graph.pos(*id) {
            sum = (sum.0 + p.x, sum.1 + p.y);
            n += 1.0;
        }
    }
    for c in &cards {
        if let Some(s) = subgraph::find(&motion.doc.subgraphs, *c) {
            sum = (sum.0 + s.x, sum.1 + s.y);
            n += 1.0;
        }
    }
    let (x, y) = if n > 0.0 {
        (sum.0 / n, sum.1 / n)
    } else {
        (0.0, 0.0)
    };

    motion.doc.subgraphs.push(Subgraph {
        id: sid,
        parent: level,
        x,
        y,
        title: fold::DEFAULT_TITLE.to_string(),
    });
    for id in nodes {
        motion.doc.members.insert(id, sid);
    }
    for c in cards {
        if let Some(s) = motion.doc.subgraphs.iter_mut().find(|s| s.id == c) {
            s.parent = Some(sid);
        }
    }
    motion.history.push_undo(pre);
    // NO mark_dirty: the graph is untouched. (If this line ever appears here, the
    // feature has stopped being a fold and become a lie about the cook.)
    ph2d_panel_motion_graph::request_graph_selection(vec![view_id(sid)]);
}

/// **Dissolve a subgraph** (Ctrl+Alt+G), lifting its members into its parent. Nothing
/// is deleted and no wire is lost — Blender: *"Removes the group and places the
/// individual nodes into your editor workspace. No internal connections are lost."*
pub(super) fn ungroup(motion: &mut MotionState, sid: u32) {
    let Some(parent) = subgraph::find(&motion.doc.subgraphs, sid).map(|s| s.parent) else {
        return;
    };
    let pre = motion.doc.clone();
    // What the card was holding, in the view ids of the level it is about to spill onto
    // — the DIRECT members plus the nested cards, which is exactly what becomes visible.
    // Read BEFORE the mutation, handed back as the selection AFTER it: dissolving a group
    // and finding nothing selected loses the artist the cluster they had in their hand
    // (Enio, smoke 2026-07-13), and it is also what Blender's Ungroup leaves selected.
    let mut freed: Vec<u32> = motion
        .doc
        .members
        .iter()
        .filter(|(_, s)| **s == sid)
        .map(|(n, _)| n.0)
        .collect();
    freed.extend(
        motion
            .doc
            .subgraphs
            .iter()
            .filter(|s| s.parent == Some(sid))
            .map(|s| view_id(s.id)),
    );
    // Members rise one level (to the root when there is no parent).
    motion.doc.members.retain(|_, s| {
        if *s == sid {
            match parent {
                Some(p) => {
                    *s = p;
                    true
                }
                None => false,
            }
        } else {
            true
        }
    });
    motion.doc.backdrop_members.retain(|_, s| {
        if *s == sid {
            match parent {
                Some(p) => {
                    *s = p;
                    true
                }
                None => false,
            }
        } else {
            true
        }
    });
    for s in &mut motion.doc.subgraphs {
        if s.parent == Some(sid) {
            s.parent = parent;
        }
    }
    motion.doc.subgraphs.retain(|s| s.id != sid);
    // The group ceases to exist, so its unit-bypass goes with it — a dangling id would emit a
    // `yg` record the loader rejects. The members already carry their own node-bypass, untouched.
    motion.doc.set_subgraph_bypassed(sid, false);
    // Dissolving the room you are standing in puts you where the room was.
    if motion.level == Some(sid) {
        // …which CLEARS the selection, so the hand-back below has to come after it.
        set_level(motion, parent);
    }
    motion.history.push_undo(pre);
    ph2d_panel_motion_graph::request_graph_selection(freed);
}

/// **Delete a card and everything inside it** — the members, the nests below them,
/// and the decoration that lived in there. A collapsed card IS its contents (Nuke:
/// *"the original nodes are replaced with the Group node"*), so deleting it deletes
/// them; the undo step is the caller's. Returns whether the graph changed (i.e.
/// whether the cook must be re-run).
pub(super) fn delete_deep(motion: &mut MotionState, sid: u32) -> bool {
    if subgraph::find(&motion.doc.subgraphs, sid).is_none() {
        return false;
    }
    let dead_subs = subgraph::descendants(&motion.doc.subgraphs, sid);
    let dead_nodes = subgraph::member_nodes_deep(&motion.doc.subgraphs, &motion.doc.members, sid);
    let mut changed = false;
    for n in &dead_nodes {
        changed |= motion.doc.graph.remove_node(*n);
    }
    motion.doc.forget_nodes(&dead_nodes);
    motion.doc.backdrops.retain(|b| {
        !motion
            .doc
            .backdrop_members
            .get(&b.id)
            .is_some_and(|s| dead_subs.contains(s))
    });
    motion
        .doc
        .backdrop_members
        .retain(|_, s| !dead_subs.contains(s));
    motion.doc.subgraphs.retain(|s| !dead_subs.contains(&s.id));
    // The deleted groups take their unit-bypass with them (as the members and decoration go).
    motion
        .doc
        .bypassed_subgraphs
        .retain(|s| !dead_subs.contains(s));
    // Standing inside a card that was just deleted (from a parent level) is not a
    // place: fall back to the root.
    if motion.level.is_some_and(|l| dead_subs.contains(&l)) {
        set_level(motion, None);
    }
    changed
}

/// **Move a card** — and everything it holds, at every depth. The members never
/// stopped being where they are; if the card moved without them, entering it would
/// land the artist on empty canvas a screen away from the card they just dragged.
pub(super) fn translate(motion: &mut MotionState, sid: u32, dx: f32, dy: f32) {
    if let Some(s) = motion.doc.subgraphs.iter_mut().find(|s| s.id == sid) {
        s.x += dx;
        s.y += dy;
    } else {
        return;
    }
    for n in subgraph::member_nodes_deep(&motion.doc.subgraphs, &motion.doc.members, sid) {
        if let Some(p) = motion.doc.graph.pos(n) {
            motion.doc.graph.set_pos(
                n,
                Pos {
                    x: p.x + dx,
                    y: p.y + dy,
                },
            );
        }
    }
    // Nested cards ride along too (they are drawn inside, at their own coordinates).
    let inner: Vec<u32> = subgraph::descendants(&motion.doc.subgraphs, sid)
        .into_iter()
        .filter(|s| *s != sid)
        .collect();
    for s in &mut motion.doc.subgraphs {
        if inner.contains(&s.id) {
            s.x += dx;
            s.y += dy;
        }
    }
}

/// **Navigation** — enter a card, or walk the breadcrumb back out. Not a document
/// edit: no undo step, no re-cook. The selection is dropped, because a node selected
/// in the room you just left is not a subject the params panel can show.
pub(super) fn set_level(motion: &mut MotionState, level: Option<u32>) {
    let valid = level.is_none_or(|l| subgraph::find(&motion.doc.subgraphs, l).is_some());
    let next = if valid { level } else { None };
    if motion.level != next {
        motion.level = next;
        ph2d_panel_motion_graph::request_graph_selection(Vec::new());
    }
}

/// The level the artist is standing in may STOP EXISTING under their feet — an undo
/// that unmakes the group, a delete from a parent level. Re-checked every frame; a
/// vanished level falls back to the root rather than showing an empty canvas that
/// nothing can leave.
pub(super) fn clamp_level(motion: &mut MotionState) {
    if motion
        .level
        .is_some_and(|l| subgraph::find(&motion.doc.subgraphs, l).is_none())
    {
        set_level(motion, None);
    }
}

/// **Every node minted while inside a group belongs to that group.** Called from the
/// ONE reconcile that every structural edit runs (`motion_bridge::reconcile`), so a
/// new add / smart-connect / spliced reroute / duplicate cannot land at the root and
/// vanish the moment it is created.
pub(super) fn adopt_new(motion: &mut MotionState, before: &Graph) {
    let Some(level) = motion.level else {
        return; // at the root, membership is the absence of an entry
    };
    let old: BTreeSet<NodeId> = before.nodes().iter().map(|n| n.id).collect();
    let fresh: Vec<NodeId> = motion
        .doc
        .graph
        .nodes()
        .iter()
        .map(|n| n.id)
        .filter(|id| !old.contains(id))
        .collect();
    for id in fresh {
        motion.doc.members.entry(id).or_insert(level);
    }
}

/// Rename (params panel Title row). No undo push — the params bridge brackets the
/// typing session, so a rename is ONE step and not one per keystroke.
pub(super) fn set_title(motion: &mut MotionState, sid: u32, title: String) {
    if let Some(s) = motion.doc.subgraphs.iter_mut().find(|s| s.id == sid) {
        s.title = title;
    }
}

/// The ONE selected card, or `None` (nothing, several things, or a node). Read from
/// the live selection rather than from an intent's `node` field: an intent left over
/// from a previous frame carries an id from the old subject, and a stale rename must
/// not land on whatever group happens to share the number.
pub(super) fn selected_card() -> Option<u32> {
    let sel = ph2d_panel_motion_graph::current_graph_selection();
    let [only] = sel[..] else { return None };
    subgraph_of(only)
}

/// The params-panel rows for a selected CARD: its name, and what is inside it. A
/// subgraph has no manifest — it is not a node — so its properties are hand-built
/// here, exactly as a backdrop's are. Without this a group could never be named, and
/// a wall of cards all reading "Group" is the wall we set out to remove.
pub(super) fn params_snapshot(
    motion: &MotionState,
) -> Option<ph2d_panel_motion_params::ParamsSnapshot> {
    use ph2d_panel_motion_params::{ParamRow, ParamsSnapshot, TextRow};
    let sid = selected_card()?;
    let only = view_id(sid);
    let s = subgraph::find(&motion.doc.subgraphs, sid)?;
    let inside = subgraph::member_nodes_deep(&motion.doc.subgraphs, &motion.doc.members, sid);
    Some(ParamsSnapshot {
        node: only,
        title: format!("Group ({} nodes)", inside.len()),
        modified: Default::default(),
        sections: Vec::new(),
        folded_by_default: std::collections::BTreeSet::new(),
        rows: vec![ParamRow::Text(TextRow {
            name: "title",
            label: "Name".to_string(),
            value: s.title.clone(),
            problem: None,
            // O título de um subgrafo é texto livre: não há alfabeto que explicar.
            help: None,
        })],
    })
}

/// Route one params-panel edit to the selected card.
pub(super) fn apply_param_intent(
    motion: &mut MotionState,
    view: u32,
    intent: ph2d_panel_motion_params::MotionParamIntent,
) {
    use ph2d_panel_motion_params::MotionParamIntent as I;
    if let Some(sid) = subgraph_of(view)
        && let I::SetTextParam {
            param: "title",
            value,
            ..
        } = intent
    {
        set_title(motion, sid, value);
    }
}
