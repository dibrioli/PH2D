//! **Where a wire can land that has no socket to aim at** — the drop-target channel
//! (Motion Nodes doc 57 §5 + doc 58). Sibling of `snapshot` (panel LOC cap).
//!
//! Two different holes, one question. A **collapsed card** hides the ports inside it: its
//! sockets are the wires that already cross, so a NEW wire has nowhere to aim. An ordinary
//! **node** hides its parameters: a param has no port and never will (`NodeManifest.inputs`
//! is frozen, ADR-0039). Either way the artist drops the wire on the body and the editor
//! asks the one question it cannot answer for itself: *which one?*
//!
//! It rides beside the snapshot rather than inside it for the same reason the node catalog
//! does: these are the rows of a MENU, not something the paint draws. A card that showed its
//! hidden ports as sockets would not be hiding them.

use super::PortView;
use ph2d_node_registry::NodeUiCategory;
use std::cell::RefCell;
use std::collections::BTreeMap;

/// **A port INSIDE a collapsed group that the card does not expose** (doc 57 §5) — one
/// row of the menu a wire dropped on a card's body opens.
///
/// It names a REAL node and a REAL port (untagged ids): the fold is a view, the graph is
/// flat, so the connection the artist picks is an ordinary edge and everything downstream
/// — the connect authority, the plumbing, the undo — goes on speaking about real nodes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortChoice {
    pub node: u32,
    /// A real port, or a **parameter** (doc 58).
    ///
    /// An enum, not a `port: u16` with a sentinel: a parameter has no port index, and the
    /// first thing a sentinel `0` did was collide with real port 0 in a gate that had been
    /// green for a day. The two kinds live in ONE list because to the artist they are one
    /// question — *where does this wire go?* — but they are not the same thing, and the type
    /// should not pretend they are.
    pub target: ChoiceTarget,
    /// `"<node display name>: <port name>"` — the node has to be named, because a group
    /// holds many and "Value" alone would not say whose.
    pub label: String,
    /// The MEMBER's category, so the row's dot reads like the card it lives on.
    pub category: NodeUiCategory,
    /// The port's type — how the drop filters the list to what this wire can actually feed.
    pub port_type: PortView,
}

/// What a menu row would connect to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChoiceTarget {
    /// A real input/output port of the node (doc 57 §5 — hidden inside a group).
    Port(u16),
    /// A **parameter**, named (doc 58). It has no port and never will, so picking it does
    /// not connect an edge — it drives the param, and the socket appears BECAUSE the wire
    /// exists.
    Param(&'static str),
}

/// **Everything a wire could land on that has no socket to aim at** — published per view id
/// by the shell's fold, and the rows of the menu a wire dropped on a body opens.
///
/// For a **card** (doc 57 §5): the ports inside the group that no wire crosses to.
/// For an ordinary **node** (doc 58): its PARAMETERS — a param has no port and never will
/// (`NodeManifest.inputs` is frozen), so the only way a wire reaches one is to ask.
///
/// **The two sides are not symmetric, and the asymmetry is the graph's own:** an input
/// holds at most ONE edge, so an input that is already fed cannot take this wire (it is
/// hidden AND unavailable — you would have to go in and move the wire). An output fans
/// out freely, so every member output is offerable **unless the card already exposes it**,
/// in which case the artist should drop on the socket that is right there.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HiddenPorts {
    pub inputs: Vec<PortChoice>,
    pub outputs: Vec<PortChoice>,
}

thread_local! {
    static CARD_PORTS: RefCell<BTreeMap<u32, HiddenPorts>> = const { RefCell::new(BTreeMap::new()) };
}

/// Publish what each collapsed card is HIDING, keyed by its view id (shell fold → panel,
/// doc 57 §5). Rebuilt every frame by the fold, which is the only code that can see
/// through a card.
///
/// It rides beside the snapshot rather than inside it for the same reason the node
/// catalog does: these are the rows of a MENU, not something the paint draws. A card
/// that showed its hidden ports as sockets would not be hiding them.
pub fn set_card_hidden_ports(cards: BTreeMap<u32, HiddenPorts>) {
    CARD_PORTS.with(|c| *c.borrow_mut() = cards);
}

/// What card `view` is hiding — empty for anything that is not a collapsed card.
///
/// **Public so the shell's seam gate can read back what its fold published.** Deriving the
/// hidden ports perfectly and never publishing them would compile, pass every unit test on
/// the shell side, and hand the artist an empty menu.
pub fn card_hidden_ports(view: u32) -> HiddenPorts {
    CARD_PORTS.with(|c| c.borrow().get(&view).cloned().unwrap_or_default())
}
