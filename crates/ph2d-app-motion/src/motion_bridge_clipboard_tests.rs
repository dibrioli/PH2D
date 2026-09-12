//! Seam tests do CLIPBOARD do grafo F2 (Ctrl+C / Ctrl+V).
//!
//! Modulo FILHO do `motion_bridge_edit_tests` — nao um irmao — de proposito: o
//! `scene()` que monta o grafo de fixture e UMA porta, e um irmao teria de
//! reconstrui-lo ou importa-lo por um caminho que envelhece. `use super::*`
//! alcanca `scene`, `edit` e os tipos que o pai ja importou.
//!
//! O corte e por ASSUNTO (o cap de 600 LOC do HR-18): o pai guarda os verbos que
//! EDITAM o grafo no lugar (duplicate, faca, smart-connect, probe, delete-heal);
//! aqui mora o que sai do documento e volta.

use super::*;

// ───────────────────────────── Copy / Paste (Ctrl+C / Ctrl+V) ─────────────────────────────

/// **The clipboard round-trip** — Ctrl+C then Ctrl+V re-creates the copied nodes with
/// their params, and the wire that ran BETWEEN them, as FRESH nodes offset from the
/// originals; the pastes come back as the pending selection so the drag that follows
/// moves them. FALSIFIED four ways: params dropped (a factory-reset paste) · the
/// internal wire dropped (a pasted chain in pieces) · an EXTERNAL wire copied (the
/// paste spliced into somebody else's upstream) · the copies not handed back as the
/// selection (the drag would move whatever was selected before the paste).
#[test]
fn copy_then_paste_recreates_the_chain_with_params_and_internal_wires_only() {
    let mut motion = MotionState::new();
    let (grid, mv, out, _tint) = scene(&mut motion);
    motion.doc.graph.set_param(mv, "dx", 3.5);
    let before = motion.doc.graph.nodes().len();

    edit::copy_selection(&mut motion, vec![grid.0, mv.0], vec![]);
    edit::paste(&mut motion);

    let g = &motion.doc.graph;
    assert_eq!(
        g.nodes().len(),
        before + 2,
        "two fresh nodes from the clipboard"
    );

    // The pastes are the selection, and their ids are NEW — copy is decoupled from
    // the originals (a stored id would have collided or dangled).
    let pasted: Vec<NodeId> = ph2d_panel_motion_graph::pending_graph_selection()
        .expect("the pastes were handed back as the selection")
        .into_iter()
        .map(NodeId)
        .collect();
    assert_eq!(pasted.len(), 2, "and both are selected");
    assert!(
        !pasted.contains(&grid) && !pasted.contains(&mv),
        "the pastes are fresh ids, distinct from the originals"
    );

    let paste_of_move = pasted
        .iter()
        .find(|id| g.node(**id).unwrap().type_name == "motion.move")
        .expect("the move was pasted");
    assert_eq!(
        g.node_param_overrides(*paste_of_move).unwrap().get("dx"),
        Some(&3.5),
        "the params rode through the clipboard"
    );
    let paste_of_grid = pasted.iter().find(|id| *id != paste_of_move).unwrap();
    assert_eq!(
        g.input_edge(*paste_of_move, 0).map(|(from, _, _)| from),
        Some(*paste_of_grid),
        "the internal wire was re-created between the pastes"
    );
    assert!(
        !g.edges()
            .iter()
            .any(|e| e.from.0 == *paste_of_move && e.to.0 == out),
        "an external wire is never copied"
    );
}

/// A pasted node's text param (a `motion.expression` formula) survives the clipboard —
/// the text channel is a SECOND map, and capturing only the f32 params would paste a
/// formula-less expression node.
#[test]
fn copy_carries_the_text_param_through_the_clipboard() {
    let mut motion = MotionState::new();
    motion.doc.graph = Graph::new();
    let ex = motion.doc.graph.add_node("motion.expression");
    motion.doc.graph.set_text_param(ex, "expr", "sin(t) * a");

    edit::copy_selection(&mut motion, vec![ex.0], vec![]);
    edit::paste(&mut motion);

    let paste = motion
        .doc
        .graph
        .nodes()
        .iter()
        .map(|n| n.id)
        .find(|id| *id != ex)
        .expect("a paste was minted");
    assert_eq!(
        motion
            .doc
            .graph
            .node_text_param_overrides(paste)
            .and_then(|m| m.get("expr"))
            .map(String::as_str),
        Some("sin(t) * a"),
        "the formula rode through the clipboard"
    );
}

/// **Copy is a READ** — Ctrl+C fills the clipboard and leaves the document
/// byte-identical, with no undo step (a copy that pushed one would leave a stray
/// Ctrl+Z that undoes nothing). FALSIFIED by a copy that mutates the doc or pushes undo.
#[test]
fn copy_does_not_touch_the_document_or_the_undo_history() {
    let mut motion = MotionState::new();
    let (grid, mv, _out, _tint) = scene(&mut motion);
    let pre = motion.doc.clone();

    edit::copy_selection(&mut motion, vec![grid.0, mv.0], vec![]);

    assert_eq!(motion.doc, pre, "copy left the document byte-identical");
    assert!(motion.clip.is_some(), "but the clipboard filled");
    assert!(
        !motion.history.can_undo(),
        "and pushed no undo step (copy is a read, not an edit)"
    );
}

/// **Paste is ONE undo step**, however many nodes it minted — a paste of a two-node
/// chain that needed two Ctrl+Z would be a trap. FALSIFIED by a paste that pushes undo
/// per node.
#[test]
fn paste_is_one_undo_step() {
    let mut motion = MotionState::new();
    let (grid, mv, _out, _tint) = scene(&mut motion);
    let before = motion.doc.graph.nodes().len();

    edit::copy_selection(&mut motion, vec![grid.0, mv.0], vec![]);
    edit::paste(&mut motion);
    assert_eq!(
        motion.doc.graph.nodes().len(),
        before + 2,
        "two pastes landed"
    );

    let back = motion.history.undo(&motion.doc).expect("one step");
    assert_eq!(
        back.graph.nodes().len(),
        before,
        "one Ctrl+Z takes the whole paste back"
    );
    // ...and it was the ONLY step: a paste that pushed one per node would leave the
    // per-node steps under the top, invisible to a single `undo` — so this is what
    // actually pins "one step", not the restore above.
    assert!(
        !motion.history.can_undo(),
        "the paste was a single undo step, nothing left under it"
    );
}

/// **Paste-many from ONE copy** — the clipboard survives a paste, and each paste is an
/// INDEPENDENT copy cascaded one offset further (so they do not stack on one spot).
/// Distinct from Ctrl+D, which re-duplicates the last copies. FALSIFIED two ways: the
/// clipboard cleared after the first paste (the second is inert) · the cascade dropped
/// (both pastes land at the same position, one hidden under the other).
#[test]
fn paste_many_from_one_copy_makes_independent_cascaded_copies() {
    let mut motion = MotionState::new();
    motion.doc.graph = Graph::new();
    let grid = motion.doc.graph.add_node("motion.grid");
    motion
        .doc
        .graph
        .set_pos(grid, ph2d_nodegraph::graph::Pos { x: 10.0, y: 20.0 });
    let before = motion.doc.graph.nodes().len();

    edit::copy_selection(&mut motion, vec![grid.0], vec![]);
    edit::paste(&mut motion);
    let first = NodeId(ph2d_panel_motion_graph::pending_graph_selection().unwrap()[0]);
    edit::paste(&mut motion);
    let second = NodeId(ph2d_panel_motion_graph::pending_graph_selection().unwrap()[0]);

    assert_eq!(
        motion.doc.graph.nodes().len(),
        before + 2,
        "the clipboard survived — a second paste minted a second copy"
    );
    assert_ne!(first, second, "and they are two distinct nodes");
    let p1 = motion.doc.graph.pos(first).unwrap();
    let p2 = motion.doc.graph.pos(second).unwrap();
    assert!(
        p2.x > p1.x && p2.y > p1.y,
        "the second paste cascaded further out ({p1:?} then {p2:?}), not stacked on the first"
    );
}

/// **Paste with an empty clipboard is inert** — before any copy, Ctrl+V does nothing:
/// no node minted, no undo step. FALSIFIED by a paste that runs against a `None` clip.
#[test]
fn paste_with_an_empty_clipboard_is_inert() {
    let mut motion = MotionState::new();
    let (_grid, _mv, _out, _tint) = scene(&mut motion);
    let pre = motion.doc.clone();
    assert!(motion.clip.is_none(), "nothing copied yet");

    edit::paste(&mut motion);

    assert_eq!(motion.doc, pre, "an empty-clipboard paste changed nothing");
    assert!(!motion.history.can_undo(), "and pushed no undo step");
}

/// **A copied GROUP pastes AS a group** — Ctrl+V rebuilds the collapsed card and re-homes
/// the pasted nodes into it, matching Ctrl+D (which never exploded it). The clip is
/// portable, so this holds even though `paste` replays the CLIP, not the live group.
/// FALSIFIED three ways: no nesting rebuild (the pastes land loose, member of nothing) ·
/// re-homing into the ORIGINAL group id (a stored id instead of a fresh copy) · the title
/// dropped. The loose-node paste above is the control (empty `cards` → flat, unchanged).
#[test]
fn paste_rebuilds_the_copied_groups_nesting() {
    use ph2d_motion_doc::subgraph::Subgraph;
    let mut motion = MotionState::new();
    let (grid, mv, _out, _tint) = scene(&mut motion);
    // `scene` resets the graph but not the demo doc's groups — clear them so the
    // "new subgraph" below is unambiguously the pasted one.
    motion.doc.subgraphs.clear();
    motion.doc.members.clear();
    // Collapse grid+move into subgraph 1, titled "Rig", at the root level.
    motion.doc.subgraphs.push(Subgraph {
        id: 1,
        parent: None,
        x: 10.0,
        y: 20.0,
        title: "Rig".into(),
    });
    motion.doc.members.insert(grid, 1);
    motion.doc.members.insert(mv, 1);
    let before_subs = motion.doc.subgraphs.len();
    let orig: std::collections::BTreeSet<NodeId> =
        motion.doc.graph.nodes().iter().map(|n| n.id).collect();
    let before_nodes = orig.len();

    // Copy the CARD (subgraph 1) with its members, then paste.
    edit::copy_selection(&mut motion, vec![grid.0, mv.0], vec![1]);
    edit::paste(&mut motion);

    assert_eq!(
        motion.doc.graph.nodes().len(),
        before_nodes + 2,
        "two nodes pasted"
    );
    assert_eq!(
        motion.doc.subgraphs.len(),
        before_subs + 1,
        "and a FRESH group to hold them (not re-using the original)"
    );

    // The new group is the one that is not the original id 1.
    let (new_id, new_title, new_parent) = {
        let s = motion
            .doc
            .subgraphs
            .iter()
            .find(|s| s.id != 1)
            .expect("a new subgraph");
        (s.id, s.title.clone(), s.parent)
    };
    assert_eq!(new_title, "Rig", "the copied group's title rode the clip");
    assert_eq!(
        new_parent, None,
        "a top-level clip group hangs from the current level (root)"
    );

    // Both pasted nodes are members of the NEW group, not the original.
    let pasted: Vec<NodeId> = motion
        .doc
        .graph
        .nodes()
        .iter()
        .map(|n| n.id)
        .filter(|id| !orig.contains(id))
        .collect();
    assert_eq!(pasted.len(), 2);
    for p in &pasted {
        assert_eq!(
            motion.doc.members.get(p),
            Some(&new_id),
            "the paste re-homed the copy into the COPY of the group, not the original id 1"
        );
    }

    // The selection is the pasted CARD (one view id), not the two loose member nodes.
    let sel = ph2d_panel_motion_graph::pending_graph_selection().expect("a selection");
    assert_eq!(
        sel.len(),
        1,
        "a group is grabbed by its card, not by its members"
    );
    assert!(
        !sel.contains(&pasted[0].0) && !sel.contains(&pasted[1].0),
        "the selection is the card view id, not a member node id"
    );
}
