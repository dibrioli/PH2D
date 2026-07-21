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

/// **Authored initial velocity — an optional presence-override component (W9).**
///
/// Absent (the common case) means the body starts at rest, what every body did
/// before this existed. Present, `linvel` (m/s, WORLD axes) and `angvel` (rad/s,
/// CCW) are applied at SPAWN: a launched projectile, a ball kicked at t=0, a
/// spinning wheel. Until this, a body could only start still — which is why the
/// smoke scenes had to tilt gravity to fake a push.
///
/// It is the same idiom as [`GravityScale`], and its authored value is folded
/// into `BodyDesc` by the bridge so a rewind to t=0 re-arms the same launch —
/// initial velocity is part of the spawn recipe, not a per-frame force.
///
/// **World axes, not the parent's local frame**: the body spawns at its world
/// pose (the bridge composes the parent chain for position), so its launch is a
/// world-space vector — the convention Unity's `Rigidbody2D.linearVelocity` and
/// Godot's `linear_velocity` expose. Config, never live solver velocity (that
/// changes every tick and would make each frame an undo step — the rule this
/// whole file rests on).
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InitialVelocity {
    pub linvel: [f32; 2],
    pub angvel: f32,
}

impl InitialVelocity {
    /// The value an absent component stands for — a body at rest.
    pub const REST: InitialVelocity = InitialVelocity {
        linvel: [0.0, 0.0],
        angvel: 0.0,
    };

    /// Is this the rest value? Used to DETACH the component at rest, so an
    /// unmoved body carries none (the presence-override idiom, and a project file
    /// stays free of no-op zeros).
    #[must_use]
    pub fn is_rest(self) -> bool {
        self == Self::REST
    }
}

impl Default for InitialVelocity {
    fn default() -> Self {
        Self::REST
    }
}

impl SimComponent for InitialVelocity {}

/// **Continuous collision detection — a marker presence-override component (W-CCD).**
///
/// Its **presence IS the boolean**, exactly like `ph2d_ecs::Locked`: a body that
/// carries it uses continuous detection, one that does not uses discrete (rapier's
/// default). Unlike [`GravityScale`] and [`InitialVelocity`] there is no value to
/// carry — CCD is on or off — so the honest representation is a marker, not a
/// `bool` field that would always hold the same value when present. The Inspector
/// attaches it on "Continuous" and detaches it on "Discrete", so a project file is
/// never littered with an off-flag.
///
/// Why a body wants it: discrete detection only tests a body at each tick's END
/// pose, so a small fast one can pass clean THROUGH thin geometry between two ticks
/// (a bullet through a wall). CCD sweeps the motion and stops at the first impact.
/// It is meaningful only for a body the solver MOVES fast — the §11 row is offered
/// for Dynamic only, the same rule gravity and initial velocity follow.
///
/// The bridge reads its presence and folds the resulting flag into `BodyDesc.ccd`,
/// so — like the two components above — it rides the spawn recipe the world
/// rebuilds from and a rewind to t=0 re-arms it. Config, never live state.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ccd;

impl SimComponent for Ccd {}

/// **Lock rotation — a marker presence-override component (W-LockRot).**
///
/// Its **presence IS the boolean**, exactly like [`Ccd`] and `ph2d_ecs::Locked`:
/// a body that carries it has its orientation pinned (Unity's "Freeze Rotation",
/// Godot's `lock_rotation`), one that does not rotates freely (rapier's default).
/// A boolean has no value to carry, so a marker is the honest representation.
///
/// Why a body wants it: a free box tips and tumbles as it slides down a slope,
/// and a character falls over. Locking the angular DOF keeps it upright — it
/// still translates and collides, it just never rotates. It is meaningful only
/// for a body the solver MOVES under forces, so the §11 row is offered for
/// Dynamic only, the same rule gravity / initial velocity / CCD follow.
///
/// The bridge reads its presence and folds the flag into `BodyDesc.lock_rotation`,
/// so — like the components above — it rides the spawn recipe the world rebuilds
/// from and a rewind re-arms it. A locked body ignores any authored `angvel`
/// (`InitialVelocity`): with no angular DOF there is nothing to spin.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockRotation;

impl SimComponent for LockRotation {}

/// **Lock translation on X — a marker presence-override component (W-LockPos).**
///
/// Its **presence IS the boolean**, exactly like [`LockRotation`] and [`Ccd`]: a
/// body that carries it has its horizontal position pinned (Unity's "Freeze
/// Position X", Godot's `axis_lock_linear_x`), one that does not moves freely on X
/// (rapier's default). A boolean has no value to carry, so a marker is the honest
/// representation.
///
/// Why a body wants it: an elevator that only travels vertically, a side-scroller
/// actor held to a single lane, a platform on a rail. The body still falls,
/// rotates and collides — the solver simply can never move it sideways. It is
/// meaningful only for a body the solver MOVES under forces, so the §11 row is
/// offered for Dynamic only, the same rule the other constraints follow.
///
/// The bridge reads its presence and folds the flag into `BodyDesc.lock_x`, which
/// ORs into the same `LockedAxes` bitmask as [`LockRotation`]; so — like the
/// markers above — it rides the spawn recipe the world rebuilds from and a rewind
/// re-arms it. A body with an authored X [`InitialVelocity`] has that component
/// dropped by the lock (there is no X DOF to carry it).
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockPositionX;

impl SimComponent for LockPositionX {}

/// **Lock translation on Y — a marker presence-override component (W-LockPos).**
///
/// The vertical sibling of [`LockPositionX`]: its presence pins the body's Y
/// position (Unity's "Freeze Position Y"), so gravity cannot pull it down — a
/// floating platform, a hovering pickup that still slides horizontally. Same
/// idiom, same `LockedAxes` fold (`BodyDesc.lock_y`), same rewind behaviour, same
/// Dynamic-only offer. The two axes are independent — a body can lock either, both
/// or neither.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockPositionY;

impl SimComponent for LockPositionY {}

/// **Explicit mass override — an optional VALUED presence-override component (W-Mass).**
///
/// Absent (the common case) means *auto mass*: the body weighs `density × area`,
/// exactly what every body did before this existed. Present, its `f32` is the mass
/// in **kilograms**, set directly on the collider (`ColliderBuilder::mass`) and
/// ignoring density — Unity's manual `Rigidbody2D.mass`, the opposite of
/// `useAutoMass`. The angular inertia is still derived from the shape, so a heavy
/// crate rotates like a crate.
///
/// **Why a valued component and not a `Collider` field** (which would append and
/// bump the schema): it is the same idiom as [`GravityScale`] — an *override* that
/// most bodies do not carry, so absent = the engine default and a project file
/// stays free of the no-op. Being a newly registered component keyed by its own
/// type-name hash, it needs **no `PROJECT_SCHEMA` bump** (the marker precedent).
///
/// **Density and mass are the SAME quantity by two roads** (`mass = density × area`),
/// so the Inspector never shows both live: absent = density is the source (the
/// Density row), present = mass is (the Mass row) — the Auto/Manual mode. It rides
/// the `BodyDesc` the world rebuilds from, so a rewind re-arms it, and it only bites
/// a Dynamic body (a Static/Kinematic one has infinite mass — rapier ignores both).
/// Config, never live solver state, like every component in this file.
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MassOverride(pub f32);

impl SimComponent for MassOverride {}

/// **Dominance group — an optional VALUED presence-override component (W-Dominance).**
///
/// Absent (the common case) means the neutral group `0`, exactly what every body did
/// before this existed. Present, its `i8` is a collision PRIORITY: when two bodies
/// collide, the STRICTLY higher-dominance one is treated as infinite mass by the
/// lower — it bulldozes through and is never pushed back, while still falling under
/// gravity and colliding normally with equal-or-higher peers (rapier's
/// `dominance_group`, Box2D's dominance).
///
/// **It is orthogonal to mass**: a LIGHT body with high dominance shoves a HEAVY one
/// with the default — the unstoppable mover, the boss, the player that pushes debris
/// but is never shoved by it. It expresses a middle ground no body KIND can: unlike a
/// Static/Kinematic body (which also pushes everything) a high-dominance Dynamic body
/// still falls and reacts to its peers. Static/Kinematic bodies sit at the maximum, so
/// a dynamic body never shoves them — consistent with their infinite mass.
///
/// Same idiom as [`GravityScale`] — a valued override most bodies do not carry, so
/// absent = the neutral default and the Inspector detaches it at `0` (a project file
/// stays free of the no-op). A newly registered component keyed by its type-name hash,
/// so **no `PROJECT_SCHEMA` bump**. It rides the `BodyDesc` the world rebuilds from, so
/// a rewind re-arms it, and it bites only a Dynamic body (a non-dynamic one is already
/// at the max), which is why the §11 row is Dynamic-only. Config, never live state.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dominance(pub i8);

impl SimComponent for Dominance {}

/// **How two colliders' friction/restitution combine on contact** (Unity's
/// `PhysicMaterial` combine, rapier's `CoefficientCombineRule`). The serde-safe
/// mirror of that rapier enum — the physics crate carries the rapier type in
/// `BodyDesc`, and `scale::body_desc` maps this to it, exactly as `BodyKind`
/// maps to `RigidBodyType` (this crate stays serde-native; rapier stays over the
/// fence).
///
/// **Append-only** — the discriminant is the wire value (postcard encodes it
/// positionally), and it deliberately matches rapier's numbering
/// (`Average` 0, `Min` 1, `Multiply` 2, `Max` 3) so the §11 segmented control's
/// index is the tag with no remap.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombineRule {
    /// The two coefficients are averaged — rapier's own default.
    #[default]
    Average,
    /// The smaller of the two is used.
    Min,
    /// The two are multiplied.
    Multiply,
    /// The larger of the two is used — a `Max` superball bounces off any floor.
    Max,
}

impl CombineRule {
    /// The `u8` this rule travels as across the UI boundary (the §11 segmented
    /// control's index) and back. One door, both directions.
    #[must_use]
    pub fn tag(self) -> u8 {
        match self {
            CombineRule::Average => 0,
            CombineRule::Min => 1,
            CombineRule::Multiply => 2,
            CombineRule::Max => 3,
        }
    }

    /// Recover a rule from its tag. `None` for a tag no variant claims — the
    /// caller decides what to do with a value it did not expect, rather than
    /// being handed a plausible one (the `BodyKind::from_tag` discipline).
    #[must_use]
    pub fn from_tag(tag: u8) -> Option<CombineRule> {
        match tag {
            0 => Some(CombineRule::Average),
            1 => Some(CombineRule::Min),
            2 => Some(CombineRule::Multiply),
            3 => Some(CombineRule::Max),
            _ => None,
        }
    }
}

/// **Collision-material combine policy — an optional presence-override component
/// (W-Material).**
///
/// Absent (the common case) means both rules are `Average`, exactly what every
/// body did before this existed — the material basics (`Collider.restitution`
/// and `.friction`) shipped combining by `Average`, and there was no way to
/// author anything else. Present, it names how THIS collider's coefficients
/// combine with another's on contact.
///
/// **The higher-priority rule of the two colliders wins** (rapier combines with
/// `rule1.max(rule2)`), so a superball set to `Max` bounces off ANY floor,
/// regardless of the floor's rule — which is the whole point: under `Average`,
/// a Bounce = 1.0 ball on a dead floor returns to only a quarter of its drop
/// height, and nothing on the ball alone could fix it.
///
/// Same idiom as [`GravityScale`] — an override most bodies do not carry — so it
/// is a newly registered component keyed by its type-name hash (**no
/// `PROJECT_SCHEMA` bump**) and the Inspector DETACHES it when both rules return
/// to `Average` ([`Self::is_neutral`]), keeping a project file free of the no-op.
/// It rides the `BodyDesc` the world rebuilds from, so a rewind re-arms it.
///
/// Unlike the flags in this file it is a COLLIDER material property, not a
/// rigid-body one, so — like `restitution`/`friction` — it applies to any body
/// kind (a static floor's rule matters), which is why the §11 rows are NOT
/// Dynamic-only. Config, never live solver state, like every component here.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialCombine {
    /// How this collider's restitution (bounciness) combines with another's.
    pub restitution: CombineRule,
    /// How this collider's friction combines with another's.
    pub friction: CombineRule,
}

impl MaterialCombine {
    /// Both rules `Average` (rapier's default) — the value an absent component
    /// stands for. Used to DETACH the component so a neutral body carries none.
    #[must_use]
    pub fn is_neutral(self) -> bool {
        self == Self::default()
    }
}

impl SimComponent for MaterialCombine {}
