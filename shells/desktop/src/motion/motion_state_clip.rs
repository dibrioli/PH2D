//! **O CLIPBOARD do grafo** — o que viaja num Ctrl+C/Ctrl+V entre níveis.
//!
//! Irmão pelo cap de 600 LOC do shell (HR-18), cortado por ASSUNTO — o precedente é
//! o `motion_state_fixture.rs`, ao lado: o que sobra no pai é *o que o app FAZ com um
//! documento*; o que sai aqui é *o que uma cópia CARREGA*.
//!
//! ⚠️ **A família inteira é NodeId-free de propósito**, e é isso que a torna um
//! assunto separável: ela não menciona `MotionState` nem nó nenhum do manifesto — só
//! `BTreeMap` e `Pos`, os dois por caminho completo.

/// A **portable snapshot of copied nodes** — the graph clipboard (Ctrl+C/Ctrl+V).
///
/// **NodeId-free on purpose.** Each node carries its type, params, text params and
/// position; the wires BETWEEN the copied nodes are stored by INDEX into
/// [`Self::nodes`]. So a paste re-creates them in whatever graph the artist is
/// standing in — a different level, or after the originals were deleted — where a
/// stored NodeId would dangle. `pre` (feedback) wires are never stored: the paste
/// re-plumbs the copies' self-loops through `reconcile`, exactly as a dropped node.
#[derive(Clone, Default)]
pub(crate) struct GraphClip {
    pub(crate) nodes: Vec<ClipNode>,
    pub(crate) edges: Vec<ClipEdge>,
    /// The **groups the copied set lived in**, portably (Ctrl+C on a collapsed card).
    /// Empty for a copy of loose nodes — the common case, and byte-identical to the
    /// pre-nesting clip. Like the edges, this is NodeId-free: a subgraph's parent is
    /// an INDEX into this list (`None` = a top-level group of the clip), so the paste
    /// rebuilds the fold in whatever level it lands in. It mirrors what `duplicate`
    /// keeps with `subgraph::duplicate_nesting`, so Ctrl+V matches Ctrl+D on a group.
    pub(crate) subgraphs: Vec<ClipSubgraph>,
    /// How many times this clip has already been pasted. Each paste cascades one
    /// offset further down-right (`copy_selection` resets it to 0), so pasting the
    /// same clip repeatedly does not stack every copy on one spot.
    pub(crate) pastes: u32,
}

/// One node in a [`GraphClip`] — the same three things `duplicate` copies (type +
/// f32 params + text params), plus the source position the paste offsets from.
#[derive(Clone)]
pub(crate) struct ClipNode {
    pub(crate) type_name: String,
    pub(crate) params: std::collections::BTreeMap<String, f32>,
    pub(crate) texts: std::collections::BTreeMap<String, String>,
    pub(crate) pos: ph2d_nodegraph::graph::Pos,
    /// The group this node belongs to, as an INDEX into [`GraphClip::subgraphs`]
    /// (`None` = loose at the clip's top level). Id-free like everything else in the
    /// clip, so the paste re-homes the copy into the group it mints.
    pub(crate) subgraph: Option<usize>,
}

/// One group in a [`GraphClip`] — the portable, NodeId-free record the paste rebuilds
/// the collapsed cards from. `parent` is an INDEX into [`GraphClip::subgraphs`]
/// (`None` = a top-level group of the clip, which hangs from whatever level the paste
/// lands in); `x`/`y` are the card's position, offset by the paste like a node's.
#[derive(Clone)]
pub(crate) struct ClipSubgraph {
    pub(crate) title: String,
    pub(crate) parent: Option<usize>,
    pub(crate) x: f32,
    pub(crate) y: f32,
}

/// A wire with BOTH ends among the copied nodes, endpoints as `(index, port)` into
/// [`GraphClip::nodes`] — id-free, so it re-points at the pasted copies.
#[derive(Clone)]
pub(crate) struct ClipEdge {
    pub(crate) from: (usize, u16),
    pub(crate) to: (usize, u16),
}
