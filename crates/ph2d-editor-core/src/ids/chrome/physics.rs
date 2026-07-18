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
/// Physics panel close (X) button.
pub const PHYSICS_CLOSE: NodeId = hash_node_id("physics.close");

// ── Section headers ─────────────────────────────────────────────────────────
// Collapsible, and they have to be: `paint_section_header` ALWAYS paints the
// chevron, so a header without a live id would draw a "click me to fold"
// affordance that does nothing.
/// World section (gravity).
pub const PHYSICS_SEC_WORLD: NodeId = hash_node_id("physics.sec.world");
/// Solver section (sub-steps, iterations, contact frequency).
pub const PHYSICS_SEC_SOLVER: NodeId = hash_node_id("physics.sec.solver");
/// Air-drag section (the size-aware model).
pub const PHYSICS_SEC_AIR: NodeId = hash_node_id("physics.sec.air");
/// Damping section (the uniform model).
pub const PHYSICS_SEC_DAMPING: NodeId = hash_node_id("physics.sec.damping");
/// Sleep section.
pub const PHYSICS_SEC_SLEEP: NodeId = hash_node_id("physics.sec.sleep");
/// Debug section (collider overlay + readouts + reset).
pub const PHYSICS_SEC_DEBUG: NodeId = hash_node_id("physics.sec.debug");

// ── World ───────────────────────────────────────────────────────────────────
/// Gravity X (m/s²).
pub const PHYSICS_GRAVITY_X: NodeId = hash_node_id("physics.gravity_x");
/// Chip linked to [`PHYSICS_GRAVITY_X`].
pub const PHYSICS_GRAVITY_X_NUM: NodeId = hash_node_id("physics.gravity_x_num");
/// Gravity Y (m/s²). Y-up, so Earth is negative.
pub const PHYSICS_GRAVITY_Y: NodeId = hash_node_id("physics.gravity_y");
/// Chip linked to [`PHYSICS_GRAVITY_Y`].
pub const PHYSICS_GRAVITY_Y_NUM: NodeId = hash_node_id("physics.gravity_y_num");

// ── Solver ──────────────────────────────────────────────────────────────────
/// Integration sub-steps per tick.
pub const PHYSICS_SUBSTEPS: NodeId = hash_node_id("physics.substeps");
/// Chip linked to [`PHYSICS_SUBSTEPS`].
pub const PHYSICS_SUBSTEPS_NUM: NodeId = hash_node_id("physics.substeps_num");
/// Solver iterations per step.
pub const PHYSICS_ITERATIONS: NodeId = hash_node_id("physics.iterations");
/// Chip linked to [`PHYSICS_ITERATIONS`].
pub const PHYSICS_ITERATIONS_NUM: NodeId = hash_node_id("physics.iterations_num");
/// Contact spring frequency (Hz).
pub const PHYSICS_CONTACT_HZ: NodeId = hash_node_id("physics.contact_hz");
/// Chip linked to [`PHYSICS_CONTACT_HZ`].
pub const PHYSICS_CONTACT_HZ_NUM: NodeId = hash_node_id("physics.contact_hz_num");

// ── Air drag ────────────────────────────────────────────────────────────────
/// Air density — the size-aware drag coefficient.
pub const PHYSICS_AIR_DRAG: NodeId = hash_node_id("physics.air_drag");
/// Chip linked to [`PHYSICS_AIR_DRAG`].
pub const PHYSICS_AIR_DRAG_NUM: NodeId = hash_node_id("physics.air_drag_num");

// ── Damping ─────────────────────────────────────────────────────────────────
/// World linear drag.
pub const PHYSICS_LINEAR_DAMPING: NodeId = hash_node_id("physics.linear_damping");
/// Chip linked to [`PHYSICS_LINEAR_DAMPING`].
pub const PHYSICS_LINEAR_DAMPING_NUM: NodeId = hash_node_id("physics.linear_damping_num");
/// World angular drag.
pub const PHYSICS_ANGULAR_DAMPING: NodeId = hash_node_id("physics.angular_damping");
/// Chip linked to [`PHYSICS_ANGULAR_DAMPING`].
pub const PHYSICS_ANGULAR_DAMPING_NUM: NodeId = hash_node_id("physics.angular_damping_num");

// ── Sleep ───────────────────────────────────────────────────────────────────
/// Speed below which a body may sleep.
pub const PHYSICS_SLEEP_SPEED: NodeId = hash_node_id("physics.sleep_speed");
/// Chip linked to [`PHYSICS_SLEEP_SPEED`].
pub const PHYSICS_SLEEP_SPEED_NUM: NodeId = hash_node_id("physics.sleep_speed_num");
/// Spin below which a body may sleep.
pub const PHYSICS_SLEEP_SPIN: NodeId = hash_node_id("physics.sleep_spin");
/// Chip linked to [`PHYSICS_SLEEP_SPIN`].
pub const PHYSICS_SLEEP_SPIN_NUM: NodeId = hash_node_id("physics.sleep_spin_num");
/// Seconds under both thresholds before a body sleeps.
pub const PHYSICS_SLEEP_DELAY: NodeId = hash_node_id("physics.sleep_delay");
/// Chip linked to [`PHYSICS_SLEEP_DELAY`].
pub const PHYSICS_SLEEP_DELAY_NUM: NodeId = hash_node_id("physics.sleep_delay_num");

// ── Debug + commands ────────────────────────────────────────────────────────
/// "Show Colliders" toggle.
///
/// ⚠️ It **reads the shell's `App.show_colliders`** — the same flag the `B` key
/// toggles — rather than carrying its own. Two doors to one question diverge,
/// and here the divergence would be visible: the key and the checkbox
/// disagreeing about whether the outlines are on.
pub const PHYSICS_SHOW_COLLIDERS: NodeId = hash_node_id("physics.show_colliders");
/// Restore every world setting to the engine defaults.
pub const PHYSICS_RESET_DEFAULTS: NodeId = hash_node_id("physics.reset_defaults");
