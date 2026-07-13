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
    // Dissolving the room you are standing in puts you where the room was.
    if motion.level == Some(sid) {
        set_level(motion, parent);
    }
    motion.history.push_undo(pre);
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

/// **A duplicated card duplicates its contents** (Unreal, on collapsed graphs: *"if
/// you copy the collapsed node, it duplicates the internal graph"*). The nodes are
/// copied by the ordinary duplicate; the NESTING is rebuilt here, mirroring the
/// subtree so the copies land in copies of the groups that held them.
///
/// Returns the new selection: the copied cards, plus the copies of the loose nodes.
pub(super) fn duplicate_nesting(
    motion: &mut MotionState,
    cards: &[u32],
    copy_of: &std::collections::BTreeMap<NodeId, NodeId>,
) -> Vec<u32> {
    let level = motion.level;
    let mut selection = Vec::new();
    // Mint a new subgraph for every one in the copied subtrees, remembering the
    // mapping so the parent links can be rebuilt among the copies.
    let mut new_of: std::collections::BTreeMap<u32, u32> = Default::default();
    for root in cards {
        for old in subgraph::descendants(&motion.doc.subgraphs, *root) {
            let Some(s) = subgraph::find(&motion.doc.subgraphs, old) else {
                continue;
            };
            let (title, x, y) = (s.title.clone(), s.x, s.y);
            let id = subgraph::next_id(&motion.doc.subgraphs);
            new_of.insert(old, id);
            motion.doc.subgraphs.push(Subgraph {
                id,
                parent: None, // fixed below, once every id exists
                x: x + DUP_OFFSET,
                y: y + DUP_OFFSET,
                title,
            });
        }
    }
    for (old, new) in &new_of {
        let old_parent = subgraph::find(&motion.doc.subgraphs, *old).and_then(|s| s.parent);
        // A parent inside the copied subtree maps to its copy; the outermost cards
        // hang from the level the artist is standing in.
        let parent = match old_parent {
            Some(p) => new_of.get(&p).copied().or(level),
            None => level,
        };
        if let Some(s) = motion.doc.subgraphs.iter_mut().find(|s| s.id == *new) {
            s.parent = parent;
        }
        if cards.contains(old) {
            selection.push(view_id(*new));
        }
    }
    // Each copied node joins the copy of the group its source was in.
    for (src, dst) in copy_of {
        match motion.doc.members.get(src).copied() {
            Some(old_sub) => {
                if let Some(new_sub) = new_of.get(&old_sub) {
                    motion.doc.members.insert(*dst, *new_sub);
                } else if let Some(l) = level {
                    // Source was a loose node at this level (adopt_new already did
                    // this, but be explicit rather than rely on order).
                    motion.doc.members.insert(*dst, l);
                }
                if !new_of.contains_key(&old_sub) {
                    selection.push(dst.0);
                }
            }
            None => selection.push(dst.0),
        }
    }
    selection
}

/// How far a duplicated card lands from its source (mirrors the node duplicate's).
const DUP_OFFSET: f32 = 40.0; // LITERAL-PX-OK: duplicate offset (graph space)

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
        rows: vec![ParamRow::Text(TextRow {
            name: "title",
            label: "Name".to_string(),
            value: s.title.clone(),
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
