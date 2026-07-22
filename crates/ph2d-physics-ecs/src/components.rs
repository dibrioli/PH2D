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
/// **Append-only** (Triangle/Polygon land later).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ColliderShape {
    /// Circle of `radius` meters.
    Ball { radius: f32 },
    /// Axis-aligned box, `half_x`/`half_y` HALF-extents in meters.
    Cuboid { half_x: f32, half_y: f32 },
    /// **Y-aligned capsule** — the character collider of 2D. A straight middle
    /// segment of half-length `half_height`, capped by a half-disc of `radius`
    /// at each end, so the total half-extent along Y is `half_height + radius`.
    ///
    /// It exists because a box catches on tile seams and ramp corners while a
    /// capsule slides over them — the reason Unity and Godot both ship one as
    /// the default character shape.
    ///
    /// **Appended**, so the postcard discriminants of `Ball`/`Cuboid` are
    /// unchanged and every project saved before capsules existed still loads
    /// (the `BodyKind::Kinematic` precedent — appending a variant needs no
    /// `PROJECT_SCHEMA` bump; only reordering or inserting would).
    Capsule { half_height: f32, radius: f32 },
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
    /// **Collider offset** — the collider's centre relative to the body, in world
    /// units along the body's LOCAL axes (Unity's `Collider2D.offset`). `[0, 0]`
    /// (the default) centres it on the sprite; a non-zero value places it
    /// elsewhere — the feet of a character below its sprite, an off-centre hitbox.
    /// **Appended** (postcard positional, so it goes last); `[0, 0]` is what every
    /// collider did before this field existed. The bridge folds the body's scale
    /// into it on the way to rapier (a scaled sprite's offset scales, a flipped one
    /// mirrors), and the overlay draws the outline there so the offset is visible.
    pub offset: [f32; 2],
}

impl Collider {
    /// rapier's defaults, restated so the neutral point is one named place
    /// rather than a number repeated at each call site.
    pub const DEFAULT_RESTITUTION: f32 = 0.0;
    pub const DEFAULT_FRICTION: f32 = 0.5;

    /// The **auto mass** of this collider — `density × area` of the AUTHORED shape,
    /// the same `mass = density × area` rapier derives for a body without a
    /// [`MassOverride`]. Used to SEED the Manual-mode mass in the Inspector so it does
    /// not jump when the artist flips Auto→Manual (W-Mass).
    ///
    /// It ignores world scale (the seed is for the common unscaled body and is only a
    /// starting value the artist tunes); the true scaled mass would mean re-deriving
    /// rapier's computation in a second place. `min 1e-3` so a degenerate shape never
    /// seeds a zero mass.
    #[must_use]
    pub fn auto_mass(&self) -> f32 {
        use std::f32::consts::PI;
        let area = match self.shape {
            ColliderShape::Ball { radius } => PI * radius * radius,
            ColliderShape::Cuboid { half_x, half_y } => 4.0 * half_x * half_y,
            // A capsule: the straight rectangle (2·radius wide, 2·half_height tall)
            // plus the two caps, which together form one full disc of the cap radius.
            ColliderShape::Capsule {
                half_height,
                radius,
            } => 4.0 * half_height * radius + PI * radius * radius,
        };
        (self.density * area).max(1e-3)
    }
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
            offset: [0.0, 0.0],
        }
    }
}

impl SimComponent for Collider {}

/// The optional per-body overrides — one component per §11 control (gravity scale,
/// initial velocity, the constraint markers, mass, dominance, material combine,
/// damping, one-way, the force zone). Re-exported so `components::GravityScale` and
/// friends keep working: this is a LOC split, not a new address for the types.
mod overrides;

pub use overrides::{
    AreaBuoyancy, AreaDrag, AreaEffector, AreaFormDrag, Ccd, CombineRule, DampMode,
    DampingOverride, Dominance, GravityScale, InitialVelocity, LockPositionX, LockPositionY,
    LockRotation, MassOverride, MaterialCombine, OneWayPlatform,
};
