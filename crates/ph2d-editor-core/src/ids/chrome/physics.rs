//! Physics world panel widget NodeIds (PHYSICS_*).
use super::{NodeId, hash_node_id};

// ── Physics world panel (ADR-0131 D8 — docked `ph2d-panel-physics`) ──────────
// The WORLD half of physics authoring: gravity, solver, damping, sleep. The
// per-BODY half (type, density, restitution, friction, collider shape) is the
// Inspector's "Physics Body" section and uses the separate `INSP_PHYS_*` family
// — two owners, and mixing them is the error D8 names.
//
// Dotted slug family (`physics.*`), like the vector-era ids. Distinct from
// `INSP_PHYS_*`, which hashes from `insp_phys_*`.

/// Physics panel outer rect id (for `z_order` + hit-barrier).
pub const PHYSICS_PANEL: NodeId = hash_node_id("physics.panel");

// ── Interaction tool (W-Hand) ───────────────────────────────────────────────
// What the POINTER does to a running scene: the Hand (hold a body), the Blast
// (a radial impulse) and the Pull (a sustained field). Its own section because
// it is the only part of this panel that is NOT a property of the world — it is
// a property of the tool, and it is therefore runtime-only (never persisted).

// ── Joint tool (W-JointTools) ───────────────────────────────────────────────
// What the pointer does to an articulated CHAIN, with the clock stopped. Its
// own section because the Interaction one above is about a scene that is
// RUNNING — the two families need opposite transport states, and a single radio
// would make every consumer ask which kind it was holding.
