//! **The reverse channel** (Motion Nodes M1.E10) — what the panel asks the shell to
//! change. Sibling of `snapshot` (panel LOC cap): that module is the shell → panel
//! view; this one is the panel → shell edit.
//!
//! The rule that keeps the two apart: **ephemeral view state (pan / zoom / selection /
//! drag / the open menu) is NEVER an intent** — it lives in `Panel::State` and dies
//! with the panel. Only a mutation of the DOCUMENT crosses.
/// **What a rename is pointing at** (doc 61). Three id spaces meet in the graph view — nodes,
/// subgraph cards and backdrops — and they overlap: there is routinely a node 3 AND a subgraph 3
/// AND a backdrop 3. A bare `u32` would be a coin toss about which one you just renamed, so the
/// target says which space it lives in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenameTarget {
    /// A plain node card — the name lands in `Graph::set_label` (the `t` record).
    Node(u32),
    /// A collapsed group card — `Subgraph::title` (which already existed; nothing could set it).
    Subgraph(u32),
    /// A backdrop's title strip — `Backdrop::title` (likewise).
    Backdrop(u32),
}

/// An edit the panel asks the shell to apply to the document (M1.E10, reverse of
/// [`GraphViewSnapshot`]). Ephemeral view state (pan / zoom / selection / drag)
/// stays in `Panel::State` and is NEVER an intent; only doc mutations are.
#[derive(Clone, Debug, PartialEq)]
pub enum GraphIntent {
    /// Open a one-undo-step bracket (the shell snapshots the doc). Pushed on the
    /// first movement of a node drag.
    BeginDrag,
    /// Move nodes by an INCREMENTAL graph-space delta — applied live each frame
    /// so the node tracks the cursor (no end-jump); the whole drag is one undo
    /// step (bracketed by [`Self::BeginDrag`]/[`Self::EndDrag`]).
    MoveNodes { nodes: Vec<u32>, dx: f32, dy: f32 },
    /// Close the bracket (the shell commits the undo step). Pushed on release.
    EndDrag,
    /// Connect an output port to an input port (drag socket→socket). The shell is
    /// the authority: it runs `Graph::connect` (cycle / occupied-input structure)
    /// then `Graph::validate` (typing / membrane) on a trial clone and only keeps
    /// it when the new edge is legal — else it raises a refusal toast. One undo
    /// step; re-cooks (`mark_dirty`).
    /// **Drive a parameter from a wire** (doc 58) — the artist dropped a wire on a node's
    /// body and picked one of its params. The shell calls `Graph::drive_param`; the socket
    /// then appears because the wire exists, not the other way round.
    ///
    /// A `Connect` cannot carry this: it names a port INDEX, and a param has no port. The
    /// name is `&'static` (it comes off the node's own manifest, through the shell).
    DriveParam {
        from_node: u32,
        from_port: u16,
        to_node: u32,
        param: &'static str,
    },
    Connect {
        from_node: u32,
        from_port: u16,
        to_node: u32,
        to_port: u16,
    },
    /// Remove the edge feeding an input port (alt-click a wire). Identified by its
    /// unique target `(to_node, to_port)`. One undo step; re-cooks.
    Disconnect { to_node: u32, to_port: u16 },
    /// Add a node of the given canonical type at a graph-space position (add-node
    /// menu). `type_name` is a `&'static` canonical name from the published
    /// [`NodeChoice`] catalog. One undo step; re-cooks.
    AddNode {
        type_name: &'static str,
        x: f32,
        y: f32,
    },
    /// Delete the selected nodes and their orphaned incident edges (Delete key).
    /// A no-op (no undo step) when `nodes` is empty or none still exist — safe
    /// against the double key-dispatch (M0 focus gate + shell cursor push). One
    /// undo step; re-cooks.
    DeleteSelection { nodes: Vec<u32> },
    /// Set the viewport⟂graph split fraction (divider drag, E9). Absolute `t` in
    /// the center band; the shell clamps it to the split's legal range. UI-only
    /// (never touches the cook / undo).
    SetSplit { t: f32 },
    /// Flip the split orientation (SplitH/SplitV chips, E9): `true` = vertical
    /// (scene left, graph right), `false` = horizontal (scene top, graph bottom).
    /// UI-only.
    SetSplitVertical { vertical: bool },
    /// Toggle transport play/pause (Space) so time-driven behaviours animate.
    /// The shell owns the transport; UI-only w.r.t. the doc (no undo step).
    TogglePlay,
    /// Add a group backdrop covering the given graph-space rect (the toolbar's
    /// Backdrop chip). The panel computes the rect — wrapping the selection when
    /// there is one, a default block at the view centre otherwise — and the shell
    /// mints the id (the document owns id allocation). One undo step.
    ///
    /// Backdrops are **UI-only**: applying one never re-cooks (`mark_dirty` is
    /// not called), because nothing about the cook can depend on decoration.
    AddBackdrop { x: f32, y: f32, w: f32, h: f32 },
    /// Move a backdrop by an incremental graph-space delta (header drag). The
    /// nodes it frames move in the SAME frame via a companion
    /// [`Self::MoveNodes`], both inside one [`Self::BeginDrag`]/[`Self::EndDrag`]
    /// bracket — so carrying a group is a single undo step, and the panel needs no
    /// second kind of node-move.
    MoveBackdrop { id: u32, dx: f32, dy: f32 },
    /// Grow/shrink a backdrop from one of its bottom corners (`left` = the
    /// bottom-LEFT gripper), by the pointer's incremental graph-space delta. The
    /// corner the artist did NOT grab stays put: dragging the left one moves the
    /// region's `x` and shrinks its `w`, holding the right edge — which is what a
    /// resize from that side means. The shell owns the minimum-size clamp, and
    /// anchors it to that same fixed edge. Bracketed like a move.
    ResizeBackdrop {
        id: u32,
        left: bool,
        dx: f32,
        dy: f32,
    },
    /// Delete a backdrop (Delete with one selected). The nodes it framed stay —
    /// a backdrop owns nothing, it only draws around things. One undo step.
    DeleteBackdrop { id: u32 },
    /// **Name a thing** (doc 61) — a card, a group or a backdrop, from the one inline box F2
    /// opens over whichever of them is selected. An EMPTY name is not a name: it clears the
    /// label, and the thing goes back to being called what it is (its type's display name, or
    /// the backdrop/group default). One undo step.
    ///
    /// One intent for three targets, because it is one gesture. (`SetBackdropTitle` used to
    /// live here as a separate variant, with a doc comment claiming the params panel had a
    /// Title row — it did not, and nothing ever emitted the intent. Dead plumbing with a
    /// confident comment on it: [[feedback_stale_comment_and_dead_code_lie]].)
    Rename { target: RenameTarget, name: String },
    /// Re-tint a backdrop, 0-based into `graph-backdrop-1..8` — from the palette that R-clicking
    /// its header opens (doc 62). One undo step, and no re-cook: a colour is decoration, and a
    /// cook that depended on it would be a cook that depended on taste.
    SetBackdropColor { id: u32, color: u8 },
    /// Duplicate the selected nodes (Ctrl+D) — with their params, their text
    /// params and the wires **between them**, offset so the copies are visible.
    /// Wires coming from OUTSIDE the selection are not copied: a duplicate is a
    /// new thing to place, not a second consumer silently spliced into the
    /// upstream (Blender / Nuke both copy internal links only).
    ///
    /// The shell mints the ids and hands the copies back as the new selection
    /// (via [`request_graph_selection`]) — so Ctrl+D then drag moves the COPIES,
    /// which is the whole point of the gesture. One undo step.
    DuplicateSelection { nodes: Vec<u32> },
    /// **Smart-connect**: add a node of `to_type` (the add-menu pick that followed
    /// a wire dropped in empty space) and wire the dragged output into its FIRST
    /// compatible input — one undo step with the add. The panel names the target by
    /// TYPE, not by id: the shell is the one minting ids, and it resolves the type
    /// to the node it just created.
    SmartConnect {
        from_node: u32,
        from_port: u16,
        to_type: &'static str,
        x: f32,
        y: f32,
    },
    /// Point the probe at a node (or clear it). The shell samples that node's
    /// output every tick and publishes the readout + the ring of recent samples
    /// back on the snapshot. UI-only: it never edits the document, so no undo step.
    SetProbe { node: Option<u32> },
    /// Cut every wire the knife stroke crossed, identified by target input
    /// `(to_node, to_port)`. **One undo step for the whole stroke** — a knife that
    /// cut five wires and needed five Ctrl+Z would be a trap.
    CutWires { targets: Vec<(u32, u16)> },
    /// **Move a wire's END from one input to another** (grab an occupied input socket and
    /// drag, doc 45). `new_to = None` means it was dropped on empty canvas — the wire is
    /// simply unplugged.
    ///
    /// A grabbed end **moves**; it does not copy. So this is ONE intent and ONE undo step
    /// (unplug + plug), not a `Disconnect` followed by a `Connect` — which would need two
    /// Ctrl+Z to put back, and would leave the graph half-rewired if the second half were
    /// refused. The shell tries both halves on a trial clone and keeps the ORIGINAL wire if
    /// the new landing is illegal (a refused move must not destroy the wire it moved).
    MoveWireEnd {
        from_node: u32,
        from_port: u16,
        old_to_node: u32,
        old_to_port: u16,
        new_to: Option<(u32, u16)>,
    },
    /// **Collapse the selection into a subgraph** (Ctrl+G / the Group chip, doc 57)
    /// — Blender's and Nuke's gesture, and their name for it.
    ///
    /// The nodes do not move and do not change id: the graph stays FLAT and the
    /// group is a fold in the VIEW, so this **cannot change what the cook produces**
    /// (gate: `grouping_never_changes_the_cook`). A card in the selection is
    /// re-parented, which is how nesting happens. The shell mints the id and hands
    /// the new card back as the selection. One undo step; it does NOT re-cook.
    GroupSelection { nodes: Vec<u32> },
    /// **Dissolve a subgraph** (Ctrl+Alt+G — the chord Blender AND Nuke both use for
    /// this verb), lifting its members into its parent level. Nothing is deleted and
    /// no wire is lost. One undo step; no re-cook.
    Ungroup { id: u32 },
    /// **Enter** a collapsed card (double-click it — Houdini's gesture). Navigation,
    /// not a document edit: no undo step, no re-cook. `id` is the SUBGRAPH id
    /// (untagged — the panel strips the tag on the way out).
    EnterSubgraph { id: u32 },
    /// **Walk the breadcrumb** to a level (`None` = the root canvas). Same
    /// navigation channel as [`Self::EnterSubgraph`], in the other direction.
    GoToLevel { level: Option<u32> },
    /// **Splice a reroute node into a wire** (double-click it) — the dot that bends the
    /// wire AND branches from it (doc 45). The panel says WHICH wire and WHERE; the shell
    /// picks the reroute type that fits the wire's own port type, inserts it, and re-wires
    /// source → dot → target. One undo step; it re-cooks (a reroute is a NODE — it is in the
    /// graph, and the graph changed).
    SpliceReroute {
        to_node: u32,
        to_port: u16,
        x: f32,
        y: f32,
    },
    /// **Auto-arrange the whole document** (the Hierarchy chip) — lay every node out
    /// in a left-to-right layered flow: a chain becomes one straight horizontal line,
    /// branches stack, and disconnected components go in their own bands (the
    /// Sugiyama layout already proven in `ph2d-nodegraph::layout`). It rewrites only
    /// POSITIONS, which are UI-only (they never touch the cook), so the shell applies
    /// it as ONE undo step with no `mark_dirty` — the same class as the backdrops.
    ArrangeLayout,
}
