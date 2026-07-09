#![forbid(unsafe_code)]
//! `ph2d-nodegraph` — the unified node substrate (ADR-0030 / ADR-0032).
//!
//! This crate is the shared **contract** every node crate depends on, across
//! all domains (motion / shader / sound / gameplay). It is deliberately thin
//! and **stable**: its surface is capped by an arch-gate, and changes to it
//! are rare, Coordenador-only events — because every node in the engine
//! depends on it. Design: `docs/Migracao/2026-05-node-centric-architecture.md`
//! and ADR-0030..0038 in `docs/architecture/decisions/`.
//!
//! 🔒 **FROZEN at W2.T4 (ADR-0039, 2026-05-22).** The Motion vertical proved
//! the contract end to end (audited, gated, smoke-confirmed) and per-instance
//! param overrides closed the last authoring gap, so the surface is now frozen:
//! the arch-gate caps (`tests/architecture_contract_surface.rs`) are pinned to
//! the current surface, and any change is a deliberate Coordenador-only event
//! (bump the cap + write an ADR). Fan-out node crates build against this.
//!
//! What is **unified** here (the plural evaluators live in `ph2d-eval-*`):
//! - algebraic port types carrying domain + dimensionality + clock ([`port`]),
//! - the effect system / membrane rule ([`effect`]),
//! - the node contract ([`node`]),
//! - the columnar attribute stream ([`attr`]),
//! - the acyclic-by-construction graph with `pre` delay; [`graph::Graph::validate`]
//!   enforces the membrane + port types ([`graph`]),
//! - the demand-driven incremental cook engine ([`cook`]),
//! - the diffable textual format ([`format`]).
//!
//! Not yet built (separate crates per the plan): per-node `lowerings` →
//! WGSL|Luau (`ph2d-expr`, ADR-0033), the node registry + codegen
//! (`ph2d-node-registry`, ADR-0031), and the domain evaluators (`ph2d-eval-*`,
//! ADR-0034).

pub mod attr;
pub mod cook;
pub mod effect;
pub mod format;
pub mod graph;
pub mod node;
pub mod port;
pub mod time;
pub mod value;
