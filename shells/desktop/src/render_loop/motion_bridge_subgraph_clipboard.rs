//! **Nesting clipboard** — rebuild a subtree of groups when the selection is DUPLICATED (Ctrl+D)
//! or PASTED (Ctrl+V). Split from `motion_bridge_subgraph` for the shell LOC cap; declared there
//! as a nested `#[path]` module and re-exported, so callers keep saying `subgraph::duplicate_nesting`
//! / `subgraph::paste_nesting`. `super` is `subgraph`; `super::super` is `render_loop::motion_bridge`.

use super::super::MotionState;
use super::view_id;
use ph2d_motion_doc::{Subgraph, subgraph};
use ph2d_nodegraph::graph::NodeId;

/// How far a duplicated card lands from its source (mirrors the node duplicate's).
const DUP_OFFSET: f32 = 40.0; // LITERAL-PX-OK: duplicate offset (graph space)

/// **A duplicated card duplicates its contents** (Unreal, on collapsed graphs: *"if
/// you copy the collapsed node, it duplicates the internal graph"*). The nodes are
/// copied by the ordinary duplicate; the NESTING is rebuilt here, mirroring the
/// subtree so the copies land in copies of the groups that held them.
///
/// Returns the new selection: the copied cards, plus the copies of the loose nodes.
pub(crate) fn duplicate_nesting(
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

/// **Rebuild a pasted clip's nesting** (Ctrl+V of a copied group) — the sibling of
/// [`duplicate_nesting`], but sourced from the PORTABLE clip instead of the live doc,
/// so it works after the originals are gone or in another level. It mints one subgraph
/// per clip subgraph, rebuilds the parent chain among the copies (a clip TOP-LEVEL
/// group — `parent: None` — hangs from the level the paste landed in), and re-homes
/// each pasted node into the group its clip entry named, OVERWRITING the current-level
/// membership `reconcile`/`adopt_new` just set (so this must run AFTER reconcile).
///
/// Returns the new selection: the top-level pasted cards + the loose pasted nodes (a
/// node inside a group is represented by its card, not selected on its own). An empty
/// `clip.subgraphs` (a copy of loose nodes) mints nothing and returns every node —
/// byte-identical to the pre-nesting paste.
pub(crate) fn paste_nesting(
    motion: &mut MotionState,
    clip: &crate::motion_state::GraphClip,
    new_ids: &[NodeId],
    ox: f32,
    oy: f32,
) -> Vec<u32> {
    let level = motion.level;
    // Mint one new subgraph per clip subgraph; `new_of[i]` is the copy of
    // `clip.subgraphs[i]`, so the parent/member references (indices) re-point at it.
    let new_of: Vec<u32> = clip
        .subgraphs
        .iter()
        .map(|cs| {
            let id = subgraph::next_id(&motion.doc.subgraphs);
            motion.doc.subgraphs.push(Subgraph {
                id,
                parent: None, // fixed below, once every id exists
                x: cs.x + ox,
                y: cs.y + oy,
                title: cs.title.clone(),
            });
            id
        })
        .collect();
    // Parent chain: a clip parent index maps to its copy; a top-level clip group hangs
    // from the level the paste is standing in.
    for (i, cs) in clip.subgraphs.iter().enumerate() {
        let parent = match cs.parent {
            Some(p) => Some(new_of[p]),
            None => level,
        };
        if let Some(s) = motion.doc.subgraphs.iter_mut().find(|s| s.id == new_of[i]) {
            s.parent = parent;
        }
    }
    // Re-home each pasted node into its clip group; a loose node stays at the level.
    for (i, cn) in clip.nodes.iter().enumerate() {
        if let Some(ci) = cn.subgraph {
            motion.doc.members.insert(new_ids[i], new_of[ci]);
        }
    }
    // Selection: the top-level pasted cards + the loose pasted nodes.
    let mut selection: Vec<u32> = clip
        .subgraphs
        .iter()
        .enumerate()
        .filter(|(_, cs)| cs.parent.is_none())
        .map(|(i, _)| view_id(new_of[i]))
        .collect();
    for (i, cn) in clip.nodes.iter().enumerate() {
        if cn.subgraph.is_none() {
            selection.push(new_ids[i].0);
        }
    }
    selection
}
