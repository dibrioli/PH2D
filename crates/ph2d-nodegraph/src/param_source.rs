//! **Driven parameters** (Motion Nodes doc 58) — a wire that lands on a *parameter*
//! instead of a port.
//!
//! A node's ports are frozen: `NodeManifest.inputs` is `&'static [PortSpec]` (ADR-0039), so
//! a node **cannot grow a port**, and the original plan deferred param-to-socket promotion
//! saying it "requires a dynamic port in the model". It does not. It requires an **edge the
//! manifest does not know about** — and an edge is not a manifest, it is document state,
//! which is precisely where the text-param channel already lives (`Graph::set_text_param`,
//! doc 32). Same trick, second use: put it in the [`Graph`](crate::graph::Graph).
//!
//! So a driven param is:
//!
//! - **a dependency to the cook** — resolved and cooked exactly like an input edge, in the
//!   same recursion, with its revision in the same fingerprint (so the memo stays correct);
//! - **a socket to the view** — the shell appends one per driven param, after the manifest's
//!   declared inputs. The graph does not have that port. Neither does a subgraph card have
//!   the ports it draws (doc 57 §3): *the view may show what the graph does not have, as long
//!   as one derivation both draws it and resolves it back.*
//! - **one number to the node** — `EvalCtx::param` returns the driver's first value, and
//!   every node reads its params through that one funnel, so **all 86 node types become
//!   drivable without a line of change in any of them.**
//!
//! **There is no separate "promote" state, and that is deliberate.** Cavalry and Houdini make
//! you promote a parameter and *then* wire it; here the wire IS the promotion (drop it on the
//! node's body and pick the parameter), and pulling it off takes the socket away. A socket
//! that exists only to be filled is a state the artist has to maintain, and the derived
//! interface of doc 57 already proved we do not need one.

use crate::graph::NodeId;
use std::collections::BTreeMap;

/// Which node port drives a parameter: `(source node, source output port)`.
pub type Source = (NodeId, u16);

/// The driven params of one node, **sorted by name** — so socket *k* means the same
/// parameter on every frame, in every session, on every machine (the same determinism rule
/// the subgraph card's slots obey).
pub type Sources = BTreeMap<String, Source>;

/// Every driven param in the document.
pub type All = BTreeMap<NodeId, Sources>;

/// The socket index a driven param occupies on the card: `inputs.len() + k`, where *k* is
/// its position in the sorted map. `None` if the param is not driven.
///
/// The one derivation the view and the intent decode share — a second one is exactly how a
/// socket comes to mean a different parameter than the one it drew
/// ([[feedback_derived_coordinate_seed_must_match_sample]]).
pub fn slot_of(sources: &Sources, param: &str) -> Option<usize> {
    sources.keys().position(|k| k == param)
}

/// The parameter socket *k* stands for.
pub fn param_at(sources: &Sources, k: usize) -> Option<&str> {
    sources.keys().nth(k).map(String::as_str)
}
