//! The two ECS components that make an entity a physics body.
//!
//! **Config only — never live solver state.** These are serialized into
//! the `WorldSnapshot` (undo + save), whose `canonicalize` sorts rows by
//! their component *bytes*. If a component held velocity / sleep / contact
//! state (which changes every physics tick), every frame would diff as a
//! new undo step and Ctrl+Z would misbehave — the exact bug `canonicalize`
//! exists to kill. The live world (`PhysicsBridge`) is transient and NOT
//! snapshotted; these components are the authored *rest* configuration
//! (ADR-0131 D2/D3). The body's live pose lives in `Transform`.

use ph2d_ecs::{Component, SimComponent};
use serde::{Deserialize, Serialize};

/// How the solver treats a body. **Append-only** (new variants at the END
/// — postcard encodes the discriminant positionally, so appending keeps
/// old saves readable).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BodyKind {
    /// Falls under gravity, collides, is pushed — the common case.
    #[default]
    Dynamic,
    /// Immovable (floor, wall). Infinite mass; the solver never moves it.
    Static,
    /// **Driven by its `Transform`, and it pushes.** The solver never moves
    /// it; every tick the bridge aims it at wherever the entity's `Transform`
    /// now says, and rapier derives the velocity it needed to get there — so
    /// a dynamic body resting on it is carried, and one in its way is shoved.
    ///
    /// This is the kind an ANIMATED body has: a platform on a timeline track,
    /// and — the reason it lands in W4 — a body whose motion has been **baked**
    /// out of the simulation into curves. Baking is exactly the moment a pose
    /// stops being an output of the solver and becomes an input to it, and
    /// that change of direction is what this variant names.
    ///
    /// It is NOT `Static`. A static body teleported along a curve arrives with
    /// zero velocity, so contacts are discovered already overlapping and the
    /// penetration solver squirts things out sideways; and `readback` skips
    /// both kinds, so on screen the two would look identical right up to the
    /// first touch. See `PhysicsWorld::set_next_kinematic_pose`.
    ///
    /// Appended — the discriminant is a frozen wire value (`Dynamic` 0,
    /// `Static` 1, `Kinematic` 2), and `PhysicsFieldEdit::Kind` carries the
    /// same tags.
    Kinematic,
}

impl BodyKind {
    /// Does the SOLVER own this body's pose, or does the scene?
    ///
    /// One door, because two halves of the bridge ask it from opposite sides:
    /// `readback` writes `Transform` only for a body the solver owns, and the
    /// kinematic drive writes rapier only for a body it does not. A body that
    /// both sides claimed would have its pose written twice per tick, and the
    /// one that landed second would win in silence.
    #[must_use]
    pub fn solver_owns_pose(self) -> bool {
        matches!(self, BodyKind::Dynamic)
    }

    /// The `u8` this kind travels as across the UI boundary
    /// (`InspectorPhysicsInfo.kind_tag`, `PhysicsFieldEdit::Kind`), and back.
    ///
    /// One door, both directions. The tag was previously produced by a `match`
    /// in the snapshot builder and consumed by an `if tag == 1 { Static } else
    /// { Dynamic }` in the edit handler — two spellings of one mapping, and the
    /// consuming one folded **every** unrecognised tag onto `Dynamic`. With two
    /// variants that was merely redundant; the moment a third exists it is a
    /// chip the artist can click that quietly selects a different kind.
    #[must_use]
    pub fn tag(self) -> u8 {
        match self {
            BodyKind::Dynamic => 0,
            BodyKind::Static => 1,
            BodyKind::Kinematic => 2,
        }
    }

    /// Recover a kind from its tag. `None` for a tag no variant claims — the
    /// caller decides what to do about a value it did not expect, rather than
    /// being handed a plausible one.
    #[must_use]
    pub fn from_tag(tag: u8) -> Option<BodyKind> {
        match tag {
            0 => Some(BodyKind::Dynamic),
            1 => Some(BodyKind::Static),
            2 => Some(BodyKind::Kinematic),
            _ => None,
        }
    }
}

/// The collider silhouette, in **world units (meters)** — the same unit as
/// `Transform` and the sprite's world size (there is no pixel↔meter
/// conversion at this boundary; the world is meter-native, ADR-0131 D4).
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

/// The collider attached to a [`RigidBody`]'s entity. W2 adds `restitution`
/// and `friction` — **appended**, because postcard encodes fields
/// positionally, and both are read by `body_desc` on the way to rapier in
/// the same commit (a field with no consumer is an orphan, DIRETIVA §2).
/// `is_sensor` waits for a consumer of its own.
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Collider {
    pub shape: ColliderShape,
    /// Mass density (kg/m² in 2D). Feeds rapier's mass computation for
    /// dynamic bodies; ignored for static. Default `1.0` matches the
    /// wrapper's `add_dynamic_circle` default.
    pub density: f32,
    /// Bounciness, `0..=1`. `0` = a beanbag (all energy absorbed), `1` = a
    /// superball that returns to the height it was dropped from. **Default
    /// `0.0` is rapier's own**, so every body authored before this field
    /// existed simulates byte-identically.
    pub restitution: f32,
    /// Coulomb friction coefficient. `0` = ice, `1` ≈ rubber on concrete;
    /// values above 1 are legal and mean "grips harder than it weighs".
    /// **Default `0.5` is rapier's own** — same byte-identity argument.
    pub friction: f32,
    /// Which collision layer this body is on (`0..MAX_LAYERS`). **Appended** —
    /// postcard is positional, so the field goes last and `0` must mean "what
    /// every body did before layers existed".
    ///
    /// Only the layer lives here. Whether two layers actually collide is a
    /// WORLD rule (`PhysicsSettings::layer_matrix`), authored once instead of
    /// re-typed on every body — see `ph2d_physics::world::layers` for why this
    /// engine takes Unity's matrix over Godot's per-body mask pair.
    pub layer: u8,
    /// A **sensor** (trigger): the collider passes through — no contact forces,
    /// nothing is pushed — but the solver still reports which bodies overlap it.
    /// **Appended** (postcard positional, so it goes last); default `false` is a
    /// solid collider, what every body did before triggers existed. The
    /// overlaps become a `PhysicsBridge` trigger state, made visible by the
    /// collider overlay + the Inspector (ADR-0131 W7). A sensor still respects
    /// the collision layers, so a trigger only detects the layers it is set to.
    pub is_sensor: bool,
}

impl Collider {
    /// rapier's defaults, restated so the neutral point is one named place
    /// rather than a number repeated at each call site.
    pub const DEFAULT_RESTITUTION: f32 = 0.0;
    pub const DEFAULT_FRICTION: f32 = 0.5;
}

impl Default for Collider {
    fn default() -> Self {
        Collider {
            shape: ColliderShape::default(),
            density: 1.0,
            restitution: Self::DEFAULT_RESTITUTION,
            friction: Self::DEFAULT_FRICTION,
            layer: 0,
            is_sensor: false,
        }
    }
}

impl SimComponent for Collider {}

/// **Per-body gravity multiplier — an optional presence-override component.**
///
/// Absent (the common case) means full world gravity, exactly what a body did
/// before this existed. Present, its `f32` scales the world gravity for THIS
/// body: `0.0` weightless (a bullet, a top-down actor), `< 0` floats up (a
/// balloon), `> 1` heavier. rapier applies gravity only to dynamic bodies, so
/// on a `Static`/`Kinematic` body it is a no-op — the §11 row is offered for
/// Dynamic only, where it can actually do something.
///
/// **Why its own component instead of a field on [`RigidBody`]** (which the W1
/// note anticipated): `RigidBody` is constructed as a bare `{ kind }` literal at
/// ~80 sites across every wave's fixtures, so appending a required field there
/// is a large, risky mechanical churn — and it would recur for each of
/// damping/ccd/can-sleep. An optional presence-override is the idiom the rest of
/// the Inspector already uses (`ZIndexOverride`, `YSort`, `BlendMode`,
/// `MaskInteraction`…): absent = engine default, present = override. It costs no
/// construction-site churn, and — being a newly *registered* component keyed by
/// its own type-name hash — it needs **no `PROJECT_SCHEMA` bump** (a new blob
/// key is additive; old files simply lack it → default; the `PhysicsJoint`/W3
/// precedent). Config, never live state, like every component in this file.
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GravityScale(pub f32);

impl GravityScale {
    /// The value an absent component stands for — rapier's own gravity scale.
    pub const NEUTRAL: f32 = 1.0;
}

impl Default for GravityScale {
    fn default() -> Self {
        GravityScale(Self::NEUTRAL)
    }
}

impl SimComponent for GravityScale {}
