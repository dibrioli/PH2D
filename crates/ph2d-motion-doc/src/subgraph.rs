//! Subgraphs — **nesting as a fold in the VIEW, over a graph that stays FLAT**
//! (Motion Nodes doc 57).
//!
//! A [`Subgraph`] collapses a set of nodes into ONE card in its parent view;
//! double-clicking the card enters it and a breadcrumb walks back out. The nodes
//! themselves never move, never change id, and never stop being ordinary members
//! of the one flat [`Graph`] — **the cook cannot tell that a subgraph exists**.
//!
//! ## Why the fold, and not a node
//!
//! The obvious design — a `motion.subgraph` NODE whose `eval` cooks a nested
//! graph — is **impossible here, and the frozen contract is what forbids it**
//! (ADR-0039):
//!
//! - [`ph2d_nodegraph::node::NodeManifest::inputs`] is `&'static [PortSpec]`. A
//!   subgraph's ports are *dynamic* (they are whatever crosses its boundary), and
//!   a `&'static` list cannot be per-instance. Adding a 9th field would break the
//!   `NodeManifest = 8` cap and every one of the ~120 node crates at once.
//! - `NodeOp::eval(&self, ctx: &mut EvalCtx)` gets no `Cook` and no `OpResolver`,
//!   so a node cannot evaluate a sub-graph even if it wanted to. `NodeOp = 2` is
//!   capped too.
//! - A nested `Graph` would carry its own `next_id`, so two nodes in two nests
//!   would share a `NodeId` — and the cook memoizes on `(NodeId, ScopeKey)`. They
//!   would alias.
//!
//! So the graph stays flat and the *editor* folds. This is exactly Unreal's
//! **collapsed graph** (purely organizational — the compiler inlines it), and it
//! buys the strongest guarantee this feature could have: **grouping is a byte-
//! identical no-op on the cook** (gate: `grouping_never_changes_the_cook`).
//!
//! What we give up is the other half of the industry pattern — a *reusable*
//! subgraph (Blender's node-group datablock, a Nuke Gizmo, a Houdini HDA), where
//! one definition instances in many places. That needs a second id space and a
//! real evaluator entry point, i.e. the frozen contract. It is deliberately NOT
//! in this design (doc 57 §6).
//!
//! ## The boundary is DERIVED, never declared
//!
//! Blender/Nuke/Houdini materialize a subgraph's interface with proxy nodes
//! inside it (Group Input / `Input1..N` / indirect inputs). We cannot: a proxy is
//! a node, and a node needs a static manifest. So a card's ports are **derived
//! from the edges that actually cross its boundary** — an edge into a member from
//! outside IS an input; a member port feeding outside IS an output (one slot per
//! source port, however many outside targets it feeds). Deterministic, and it
//! costs the document nothing.
//!
//! The consequence, stated plainly: a collapsed card's input slots are always
//! occupied (they are the crossing wires), so a *new* wire into a group is
//! authored from **inside** it — where the outside nodes that touch the boundary
//! are drawn as read-only **ghosts**, precisely so that wiring across the seam is
//! possible without a lie about what is connected to what.

use ph2d_nodegraph::format::ParseError;
use ph2d_nodegraph::graph::{Graph, NodeId};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

/// A collapsed group of nodes: one card in the parent view, a canvas of its own
/// when entered. Semantically inert — the cook never sees it (module docs).
#[derive(Clone, Debug, PartialEq)]
pub struct Subgraph {
    /// Stable id, unique within the document (serialized; drives membership,
    /// selection and navigation).
    pub id: u32,
    /// The subgraph this one is nested in, or `None` for the root canvas. The
    /// parent chain is acyclic (enforced at parse and by construction).
    pub parent: Option<u32>,
    /// Top-left of the COLLAPSED CARD, in graph space, in the PARENT's canvas.
    pub x: f32,
    pub y: f32,
    /// Free-text label (the trailing field of the `g` record, so spaces survive).
    pub title: String,
}

/// Where a node sits relative to the level being viewed — the one question the
/// view fold has to answer for every endpoint of every edge.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Holder {
    /// A direct member of the level being viewed: drawn as itself.
    Direct,
    /// Inside a subgraph nested (however deep) in this level: drawn as THAT
    /// child's collapsed card, which is where its wires terminate.
    Card(u32),
    /// Not in this level's subtree at all: it is across the boundary. Drawn as a
    /// read-only ghost when it touches the level, and not at all otherwise.
    Outside,
}

/// The membership map: node → the subgraph that holds it. A node absent from the
/// map lives on the root canvas. (A node is in at most one subgraph — the map's
/// shape says so, which is why it is a map and not a list on each subgraph.)
pub type Members = BTreeMap<NodeId, u32>;

/// backdrop → the subgraph whose canvas it decorates (absent = the root canvas).
pub type BackdropMembers = BTreeMap<u32, u32>;

/// Everything the `[subgraph]` section carries: the groups, and who is in them.
type Section = (Vec<Subgraph>, Members, BackdropMembers);

/// The parent chain of `id`, from its parent up to the root (excludes `id`).
/// Empty for a root-level subgraph. Bounded by the subgraph count even if the
/// stored parents were somehow cyclic (they cannot be — [`validate`] rejects it —
/// but a walk that can loop is a hang, and a hang is not a diagnosis).
pub fn ancestors(subgraphs: &[Subgraph], id: u32) -> Vec<u32> {
    let mut out = Vec::new();
    let mut cur = find(subgraphs, id).and_then(|s| s.parent);
    let mut budget = subgraphs.len();
    while let Some(p) = cur {
        if budget == 0 {
            break;
        }
        budget -= 1;
        out.push(p);
        cur = find(subgraphs, p).and_then(|s| s.parent);
    }
    out
}

/// `id` and every subgraph nested below it (any depth), in ascending id order.
pub fn descendants(subgraphs: &[Subgraph], id: u32) -> BTreeSet<u32> {
    let mut out = BTreeSet::new();
    out.insert(id);
    // Fixed-point over the (small) subgraph list: a subgraph joins when its
    // parent is already in. Bounded by the list length, so a corrupt parent chain
    // cannot spin.
    for _ in 0..subgraphs.len() {
        let before = out.len();
        for s in subgraphs {
            if s.parent.is_some_and(|p| out.contains(&p)) {
                out.insert(s.id);
            }
        }
        if out.len() == before {
            break;
        }
    }
    out
}

/// Every node held by `id` or by anything nested below it.
pub fn member_nodes_deep(subgraphs: &[Subgraph], members: &Members, id: u32) -> Vec<NodeId> {
    let sub = descendants(subgraphs, id);
    members
        .iter()
        .filter(|(_, sid)| sub.contains(sid))
        .map(|(n, _)| *n)
        .collect()
}

/// Where `node` sits relative to `level` (the subgraph being viewed; `None` = the
/// root canvas). This is the whole fold in one function: the view draws a
/// [`Holder::Direct`] node as itself, terminates a wire touching a
/// [`Holder::Card`] on that card's socket, and shows a [`Holder::Outside`]
/// endpoint as a ghost.
pub fn holder_at(
    subgraphs: &[Subgraph],
    members: &Members,
    node: NodeId,
    level: Option<u32>,
) -> Holder {
    let owner = members.get(&node).copied();
    if owner == level {
        return Holder::Direct;
    }
    // Walk up from the node's owner: the first subgraph whose PARENT is the level
    // is the card that stands in for it here.
    let Some(owner) = owner else {
        // A root-level node, viewed from inside a subgraph: across the boundary.
        return Holder::Outside;
    };
    let mut cur = owner;
    let mut budget = subgraphs.len();
    loop {
        let Some(s) = find(subgraphs, cur) else {
            return Holder::Outside; // dangling membership (parse rejects it; be total anyway)
        };
        if s.parent == level {
            return Holder::Card(s.id);
        }
        let Some(p) = s.parent else {
            return Holder::Outside; // walked to the root without meeting `level`
        };
        if budget == 0 {
            return Holder::Outside;
        }
        budget -= 1;
        cur = p;
    }
}

pub fn find(subgraphs: &[Subgraph], id: u32) -> Option<&Subgraph> {
    subgraphs.iter().find(|s| s.id == id)
}

/// The next free subgraph id (`max + 1`, never reused within a session, like the
/// backdrops'). Saturating: a stale id can only miss, never hit another group.
pub fn next_id(subgraphs: &[Subgraph]) -> u32 {
    subgraphs
        .iter()
        .map(|s| s.id)
        .max()
        .map_or(0, |m| m.saturating_add(1))
}

// ── Serialization: the `[subgraph]` section ─────────────────────────────────
//
// Sibling of `[backdrop]`, appended LAST, and absent entirely from a document
// with no subgraphs — so every graph written before this feature still loads
// byte-for-byte the same (gate: `a_doc_without_subgraphs_writes_no_section`).
//
//   [subgraph]
//   g <id> <parent|-> <x> <y> <title...>     one per subgraph
//   m <node_id> <subgraph_id>                node membership
//   bm <backdrop_id> <subgraph_id>           backdrop membership (decoration has a level too)

/// Write the `[subgraph]` section, or nothing at all when the document has no
/// subgraphs (see the module docs on back-compat).
pub(crate) fn write_section(
    out: &mut String,
    subgraphs: &[Subgraph],
    members: &Members,
    backdrop_members: &BTreeMap<u32, u32>,
) {
    if subgraphs.is_empty() {
        return;
    }
    out.push_str("[subgraph]\n");
    let mut sorted: Vec<&Subgraph> = subgraphs.iter().collect();
    sorted.sort_by_key(|s| s.id);
    for s in sorted {
        let parent = match s.parent {
            Some(p) => p.to_string(),
            None => "-".to_string(),
        };
        // `title` is the trailing free-text field (single-space separated), so a
        // title with interior spaces round-trips exactly — same convention as the
        // backdrop's `b` record and the graph's `x` record.
        let _ = writeln!(out, "g {} {} {} {} {}", s.id, parent, s.x, s.y, s.title);
    }
    // BTreeMaps iterate in key order → deterministic bytes.
    for (n, sid) in members {
        let _ = writeln!(out, "m {} {}", n.0, sid);
    }
    for (b, sid) in backdrop_members {
        let _ = writeln!(out, "bm {b} {sid}");
    }
}

/// Parse the `[subgraph]` section (the text AFTER the header line).
pub(crate) fn parse_section(part: &str) -> Result<Section, ParseError> {
    let mut subgraphs: Vec<Subgraph> = Vec::new();
    let mut members: Members = BTreeMap::new();
    let mut backdrop_members: BTreeMap<u32, u32> = BTreeMap::new();
    let mut seen = BTreeSet::new();

    for line in part.lines().map(str::trim).filter(|l| !l.is_empty()) {
        if let Some(rest) = line.strip_prefix("g ") {
            // 5 fields max: the title is free text and keeps its interior spaces.
            let parts: Vec<&str> = rest.splitn(5, ' ').collect();
            if parts.len() < 4 {
                return Err(ParseError::BadLine(line.into()));
            }
            let id: u32 = parse(parts[0], line)?;
            if !seen.insert(id) {
                return Err(ParseError::BadLine(line.into())); // duplicate subgraph id
            }
            let parent = match parts[1] {
                "-" => None,
                p => Some(parse::<u32>(p, line)?),
            };
            let x = finite(parse(parts[2], line)?, line)?;
            let y = finite(parse(parts[3], line)?, line)?;
            let title = parts
                .get(4)
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            subgraphs.push(Subgraph {
                id,
                parent,
                x,
                y,
                title,
            });
        } else if let Some(rest) = line.strip_prefix("bm ") {
            let mut tok = rest.split_whitespace();
            let b: u32 = parse(tok.next().unwrap_or_default(), line)?;
            let sid: u32 = parse(tok.next().unwrap_or_default(), line)?;
            if tok.next().is_some() || backdrop_members.insert(b, sid).is_some() {
                return Err(ParseError::BadLine(line.into()));
            }
        } else if let Some(rest) = line.strip_prefix("m ") {
            let mut tok = rest.split_whitespace();
            let n: u32 = parse(tok.next().unwrap_or_default(), line)?;
            let sid: u32 = parse(tok.next().unwrap_or_default(), line)?;
            if tok.next().is_some() || members.insert(NodeId(n), sid).is_some() {
                return Err(ParseError::BadLine(line.into())); // trailing junk / duplicate member
            }
        } else if line != "[subgraph]" {
            return Err(ParseError::BadLine(line.into()));
        }
    }
    Ok((subgraphs, members, backdrop_members))
}

/// Reject a corrupt nesting at the BOUNDARY, where a bad file becomes an error
/// instead of a wrong picture: an unknown parent, a **cycle** in the parent chain
/// (which would hang or lie in every walk above), a member naming a node or a
/// subgraph that does not exist, and a backdrop bound to a subgraph that does not.
pub(crate) fn validate(
    graph: &Graph,
    subgraphs: &[Subgraph],
    members: &Members,
    backdrop_members: &BTreeMap<u32, u32>,
    backdrop_ids: &BTreeSet<u32>,
) -> Result<(), ParseError> {
    let ids: BTreeSet<u32> = subgraphs.iter().map(|s| s.id).collect();
    for s in subgraphs {
        if let Some(p) = s.parent
            && !ids.contains(&p)
        {
            return Err(ParseError::BadLine(format!("subgraph {} parent {p}", s.id)));
        }
        // **Cycle**: walk up from `s` and refuse to meet an id twice. A cycle would
        // hang or lie in every walk above it (a nest that contains itself has no
        // root canvas to draw), so it dies at the boundary. Self-parenting is the
        // same defect and the same check: `s.id` is already in `seen`.
        let mut seen = BTreeSet::from([s.id]);
        let mut cur = s.parent;
        while let Some(p) = cur {
            if !seen.insert(p) {
                return Err(ParseError::BadLine(format!("subgraph {} cycle", s.id)));
            }
            cur = find(subgraphs, p).and_then(|x| x.parent);
        }
    }
    for (n, sid) in members {
        if graph.node(*n).is_none() {
            return Err(ParseError::BadLine(format!("member node {}", n.0)));
        }
        if !ids.contains(sid) {
            return Err(ParseError::BadLine(format!("member subgraph {sid}")));
        }
    }
    for (b, sid) in backdrop_members {
        if !backdrop_ids.contains(b) {
            return Err(ParseError::BadLine(format!("backdrop member {b}")));
        }
        if !ids.contains(sid) {
            return Err(ParseError::BadLine(format!("backdrop subgraph {sid}")));
        }
    }
    Ok(())
}

fn parse<T: std::str::FromStr>(s: &str, line: &str) -> Result<T, ParseError> {
    s.parse().map_err(|_| ParseError::BadLine(line.into()))
}

fn finite(v: f32, line: &str) -> Result<f32, ParseError> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(ParseError::BadLine(line.into()))
    }
}

#[cfg(test)]
#[path = "subgraph_tests.rs"]
mod tests;
