#![forbid(unsafe_code)]
//! `ph2d-nodegraph` — the unified node substrate (ADR-0030 / ADR-0032).
//!
//! This crate is the shared **contract** every node crate depends on, across
//! all domains (motion / shader / sound / gameplay). It is deliberately thin
//! and **stable**: its surface is capped by an arch-gate, and changes to it
//! are rare, Coordenador-only events — because every node in the engine
//! depends on it. See `docs/Migracao/2026-05-node-centric-architecture.md`
//! and the funnel plan (`~/.claude/plans/...iridescent-bumblebee.md`).
//!
//! What is **unified** here (the plural evaluators live in `ph2d-eval-*`):
//! - algebraic port types carrying domain + dimensionality + clock ([`port`]),
//! - the effect system + type-checked membrane ([`effect`]),
//! - the node contract ([`node`]),
//! - the acyclic-by-construction graph with `pre` delay ([`graph`]).
//!
//! Still to land (tracked in the plan): the textual diffable format, the
//! generic cook engine, and the attribute stream / `EvalCtx`.

pub mod attr;
pub mod cook;
pub mod effect;
pub mod format;
pub mod graph;
pub mod node;
pub mod port;
