//! The two ECS components that make an entity a physics body.
//!
//! **Config only — never live solver state.** These are serialized into
//! the `WorldSnapshot` (undo + save), whose `canonicalize` sorts rows by
//! their component *bytes*. If a component held velocity / sleep / contact
//! state (which changes every physics tick), every frame would diff as a
//! new undo step and Ctrl+Z would misbehave — the exact bug `canonicalize`
//! exists to kill. The live world (`PhysicsBridge`) is transient and NOT
//! snapshotted; these components are the authored *rest* configuration
//! (ADR-0130 D2/D3). The body's live pose lives in `Transform`.

use ph2d_ecs::{Component, SimComponent};
use serde::{Deserialize, Serialize};

/// How the solver treats a body. **Append-only** (new variants at the END
/// — postcard encodes the discriminant positionally, so appending keeps
/// old saves readable). `Kinematic` lands with W2/W3.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BodyKind {
    /// Falls under gravity, collides, is pushed — the common case.
    #[default]
    Dynamic,
    /// Immovable (floor, wall). Infinite mass; the solver never moves it.
    Static,
}

/// The collider silhouette, in **world units (meters)** — the same unit as
/// `Transform` and the sprite's world size (there is no pixel↔meter
/// conversion at this boundary; the world is meter-native, ADR-0130 D4).
/// **Append-only** (Capsule/Triangle/Polygon land later).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ColliderShape {
    /// Circle of `radius` meters.
    Ball { radius: f32 },
    /// Axis-aligned box, `half_x`/`half_y` HALF-extents in meters.
    Cuboid { half_x: f32, half_y: f32 },
}

impl Default for ColliderShape {
    /// A half-meter ball — the byte-neutral default (matches the wrapper's
    /// `add_dynamic_circle` radius scale for a small body).
    fn default() -> Self {
        ColliderShape::Ball { radius: 0.5 }
    }
}

/// Marks an entity as a rigid body. Presence of this component (with a
/// [`Collider`]) is what puts the entity into the physics world; its
/// **absence is the "off"** — a plain sprite is never simulated.
///
/// W1 wires exactly [`RigidBody::kind`]. Damping / gravity-scale / ccd /
/// can-sleep are appended (and wired) in W2 — a field with no consumer is
/// an orphan (DIRETIVA §2), so it does not exist until the sync reads it.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RigidBody {
    pub kind: BodyKind,
}

impl SimComponent for RigidBody {}

/// The collider attached to a [`RigidBody`]'s entity. W1 wires `shape` and
/// `density`; restitution / friction / is_sensor append with W2.
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Collider {
    pub shape: ColliderShape,
    /// Mass density (kg/m² in 2D). Feeds rapier's mass computation for
    /// dynamic bodies; ignored for static. Default `1.0` matches the
    /// wrapper's `add_dynamic_circle` default.
    pub density: f32,
}

impl Default for Collider {
    fn default() -> Self {
        Collider {
            shape: ColliderShape::default(),
            density: 1.0,
        }
    }
}

impl SimComponent for Collider {}
