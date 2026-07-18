//! Widget `NodeId`s for the Physics world panel.
//!
//! Like every other panel crate, the ids stay defined in editor-core
//! (`ph2d_editor_core::ids`) — the layout, the z-order walk and the
//! `node_id_collisions` arch test all reference them, and re-defining them here
//! would fork the source of truth. This is a convenience re-export.

pub use ph2d_editor_core::ids::{
    PHYSICS_ANGULAR_DAMPING, PHYSICS_ANGULAR_DAMPING_NUM, PHYSICS_CLOSE, PHYSICS_CONTACT_HZ,
    PHYSICS_CONTACT_HZ_NUM, PHYSICS_GRAVITY_X, PHYSICS_GRAVITY_X_NUM, PHYSICS_GRAVITY_Y,
    PHYSICS_GRAVITY_Y_NUM, PHYSICS_ITERATIONS, PHYSICS_ITERATIONS_NUM, PHYSICS_LINEAR_DAMPING,
    PHYSICS_LINEAR_DAMPING_NUM, PHYSICS_PANEL, PHYSICS_RESET_DEFAULTS, PHYSICS_SHOW_COLLIDERS,
    PHYSICS_SLEEP_DELAY, PHYSICS_SLEEP_DELAY_NUM, PHYSICS_SLEEP_SPEED, PHYSICS_SLEEP_SPEED_NUM,
    PHYSICS_SLEEP_SPIN, PHYSICS_SLEEP_SPIN_NUM, PHYSICS_SUBSTEPS, PHYSICS_SUBSTEPS_NUM,
};
