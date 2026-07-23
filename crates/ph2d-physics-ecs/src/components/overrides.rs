//! **The optional per-body overrides** — one component per §11 control that most
//! bodies do not carry.
//!
//! Split from the parent `components` for the workspace's 700-LOC cap when the force
//! zone (W-Area) took it past. The cut is the one the file was already making on its
//! own: the parent answers *what makes an entity a physics body* (`RigidBody` +
//! `Collider`, both required), and this answers *what an artist can change about one*.
//!
//! Every type here follows the same idiom, and the idiom is the reason there are so
//! many of them:
//!
//! - **Absent is the engine default**, so a body that never touched the control is
//!   byte-identical to one authored before the component existed.
//! - **A marker's PRESENCE is its boolean** ([`Ccd`], [`LockRotation`],
//!   [`LockPositionX`]/[`LockPositionY`], [`OneWayPlatform`]); a valued override
//!   carries its number and DETACHES at neutral.
//! - **A newly registered component costs no `PROJECT_SCHEMA` bump** — it is keyed by
//!   its own type-name hash, so old files simply lack the blob and take the default.
//!   Appending a field to `Collider` is the opposite: postcard is POSITIONAL, which is
//!   what `layer`/`is_sensor`/`offset` each cost.
//! - **`RigidBody` is built as a bare `{ kind }` literal at ~80 sites**, so a required
//!   field there is a large mechanical churn that would recur for every wave.
//!
//! Config, never live solver state — see the parent module's header for why that
//! distinction is load-bearing for undo.

use ph2d_ecs::{Component, SimComponent};
use serde::{Deserialize, Serialize};

/// **Per-body gravity multiplier — an optional presence-override component.**
///
/// Absent (the common case) means full world gravity, exactly what a body did
/// before this existed. Present, its `f32` scales the world gravity for THIS
/// body: `0.0` weightless (a bullet, a top-down actor), `< 0` floats up (a
/// balloon), `> 1` heavier. rapier applies gravity only to dynamic bodies, so
/// on a `Static`/`Kinematic` body it is a no-op — the §11 row is offered for
/// Dynamic only, where it can actually do something.
///
/// **Why its own component instead of a field on [`RigidBody`](super::RigidBody)** (which the W1
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

/// **How a per-body damping override meets the world's default drag** (Godot's
/// `damp_mode`). `Combine` ADDS the override to the global `BodyDefaults` drag;
/// `Replace` IGNORES the global and uses the override outright.
///
/// **Append-only** — the discriminant is the wire value (postcard positional).
/// `Combine` is the default (Godot's own), and with the default global drag of `0.0`
/// it coincides with `Replace`, so the mode only bites once the artist authors a
/// world drag. The §11 mode toggle's index is the tag (no remap).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DampMode {
    /// Add the override to the world default drag.
    #[default]
    Combine,
    /// Use the override outright, ignoring the world default.
    Replace,
}

impl DampMode {
    /// The `u8` this mode travels as across the UI boundary and back (one door).
    #[must_use]
    pub fn tag(self) -> u8 {
        match self {
            DampMode::Combine => 0,
            DampMode::Replace => 1,
        }
    }

    /// Recover a mode from its tag. `None` for a tag no variant claims (the
    /// `BodyKind::from_tag` discipline).
    #[must_use]
    pub fn from_tag(tag: u8) -> Option<DampMode> {
        match tag {
            0 => Some(DampMode::Combine),
            1 => Some(DampMode::Replace),
            _ => None,
        }
    }
}

/// **Per-body damping override — an optional presence-override component (W-Damping).**
///
/// Absent (the common case) leaves the body on the world's `BodyDefaults` drag,
/// exactly what every body did before this existed. Present, `linear`/`angular` are
/// drag coefficients that decay the body's velocities each step (Unity's
/// `Rigidbody2D.linearDamping`/`angularDamping`, Godot's `linear_damp`/`angular_damp`),
/// and `mode` chooses whether they [`DampMode::Combine`] with or [`DampMode::Replace`]
/// the global. It only bites a Dynamic body (damping decays a velocity the solver owns),
/// so the §11 rows are Dynamic-only.
///
/// Same idiom as [`GravityScale`] — an override most bodies do not carry — so it is a
/// newly registered component keyed by its type-name hash (**no `PROJECT_SCHEMA`
/// bump**), and the Inspector DETACHES it at neutral ([`Self::is_neutral`]). It maps to
/// `ph2d_physics::DampingDesc` in `scale::body_desc` and rides the `BodyDesc` the world
/// rebuilds from, so a rewind re-arms it. ⚠️ Unlike the other overrides it is ALSO
/// re-stamped by the bridge each dispatch, so a change to the GLOBAL drag mid-play
/// cannot leave an override body wearing the world value (the `apply_to_all` clobber).
/// Config, never live solver state, like every component here.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DampingOverride {
    /// Linear drag coefficient (decays translation).
    pub linear: f32,
    /// Angular drag coefficient (decays rotation).
    pub angular: f32,
    /// Whether these combine with or replace the world default drag.
    pub mode: DampMode,
}

impl DampingOverride {
    /// Is this the neutral value — no effect on the body? Zero drag on both axes AND
    /// `Combine` mode: `Combine + 0` is the global drag, i.e. no override. `Replace + 0`
    /// is NOT neutral (it forces zero damping, ignoring a world drag — a real choice),
    /// so it keeps the component. Used to DETACH so a neutral body carries none.
    #[must_use]
    pub fn is_neutral(self) -> bool {
        self.linear == 0.0 && self.angular == 0.0 && matches!(self.mode, DampMode::Combine)
    }
}

impl SimComponent for DampingOverride {}

/// **One-way (jump-through) platform — a marker presence-override component (W-OneWay).**
///
/// Its **presence IS the boolean**, exactly like [`Ccd`] and [`LockRotation`]: a collider
/// that carries it is solid only from its **local +Y side**, so a body arriving from
/// below passes clean through and then LANDS on it coming back down. The iconic 2D
/// platformer collider (Godot's `one_way_collision`).
///
/// The direction is the collider's own local up, so a ROTATED platform is one-way along
/// its own axis — there is no separate direction field that could disagree with the
/// pose. It is realised by rapier's `update_as_oneway_platform` through the world's
/// `OneWayHooks`, which also owns the hysteresis that keeps a body from popping while it
/// straddles the surface.
///
/// **Why a marker and not a `Collider` field:** `Collider` is serialized POSITIONALLY by
/// postcard, so appending to it is a `PROJECT_SCHEMA` bump (what `layer`/`is_sensor`/
/// `offset` each cost). A newly registered component is keyed by its own type-name hash
/// and is purely additive — **no bump** — and a boolean has no value to carry, which is
/// what makes a marker the honest shape (the `Ccd` precedent).
///
/// It is a COLLIDER property, not a rigid-body one, so it applies to any body kind — and
/// a platform is usually **Static**, which is exactly why the §11 toggle is NOT
/// Dynamic-only. The bridge folds its presence into `BodyDesc.one_way`, so it rides the
/// spawn recipe the world rebuilds from and a rewind re-arms it. Config, never live state.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OneWayPlatform;

impl SimComponent for OneWayPlatform {}

/// **Force zone — an optional presence-override component (W-Area).**
///
/// Absent (the common case) is an ordinary body that pushes nothing, exactly what
/// every body did before this existed. Present, `force` is a constant force in
/// **newtons** (world axes) applied every substep to every DYNAMIC body overlapping
/// this collider: wind, an updraft, a conveyor, a current (Unity's `AreaEffector2D`,
/// Godot's `Area2D` overrides).
///
/// **A force, not an acceleration** — the impulse is resisted by the body's mass, so
/// a leaf is carried by a wind a crate barely feels. That asymmetry is the feature,
/// and it is the half that cannot be authored per body: an *acceleration* zone would
/// be a second answer to what [`GravityScale`] already says about one body.
///
/// ⚠️ **It only bites when the collider is a SENSOR** (`Collider::is_sensor`): the
/// narrow phase records an overlap only when one side is a sensor, and a solid
/// collider pushes bodies OUT rather than letting them in — an area you cannot enter
/// is not an area. The §11 Force rows are offered only for a sensor for that reason,
/// which makes this the first control in the section gated on another CONTROL rather
/// than on the body kind.
///
/// Same idiom as [`GravityScale`]/[`DampingOverride`] — an override most bodies do not
/// carry — so it is a newly registered component keyed by its type-name hash (**no
/// `PROJECT_SCHEMA` bump**), and the Inspector DETACHES it at neutral
/// ([`Self::is_neutral`]). The bridge folds it into `BodyDesc.effector`, so it rides
/// the spawn recipe the world rebuilds from and a rewind re-arms it. Config, never
/// live solver state, like every component here.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AreaEffector {
    /// The force in newtons, world axes. `[0, 0]` is neutral (see [`Self::is_neutral`]).
    pub force: [f32; 2],
}

impl AreaEffector {
    /// Is this the neutral value — a zone that pushes nothing? A zero force on both
    /// axes. Used to DETACH so a body with no push carries no component.
    #[must_use]
    pub fn is_neutral(self) -> bool {
        self.force[0] == 0.0 && self.force[1] == 0.0
    }
}

impl SimComponent for AreaEffector {}

/// **Area drag — the medium half of a force zone (W-AreaDrag).**
///
/// Absent (the common case) is an area that offers no resistance. Present, its `f32`
/// is a drag coefficient applied every substep to every DYNAMIC body overlapping this
/// collider: the difference between **wind** and **water**.
///
/// It is the same law as the world's default drag and the per-body [`DampingOverride`]
/// (`v /= 1 + d·dt`), so "drag" means one thing everywhere — and it damps **both** the
/// linear and the angular velocity, because a medium resists a spin too.
///
/// ⚠️ **Its own component rather than a field on [`AreaEffector`], deliberately.** The
/// wrapper bundles the two into one `AreaEffect` (that side is not serialized), but a
/// component's blob is postcard, which is POSITIONAL: appending a field to
/// `AreaEffector` would be a `PROJECT_SCHEMA` bump, and a bump **refuses every project
/// file already saved at the old number** — throwing away real work to avoid a second
/// component. A newly registered component is keyed by its own type-name hash and is
/// purely additive, so every existing file still loads. The two are independently
/// meaningful anyway: a wind that does not slow you, a pool of syrup that does not push.
///
/// Same coupling as its sibling: it only bites when the collider is a **SENSOR**, and
/// the §11 row is offered under exactly that condition. It rides the `BodyDesc` the
/// world rebuilds from, so a rewind re-arms it. Config, never live solver state.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AreaDrag(pub f32);

impl AreaDrag {
    /// Is this neutral — an area that resists nothing? Used to DETACH so a zone with
    /// no medium carries no component.
    #[must_use]
    pub fn is_neutral(self) -> bool {
        self.0 <= 0.0
    }
}

impl SimComponent for AreaDrag {}

/// **Empuxo de área — a densidade do FLUIDO (W-Buoyancy).**
///
/// Ausente (o caso comum) é uma área sem empuxo. Presente, seu `f32` é a densidade do
/// fluido em kg/m² (2D), e cada corpo submerso recebe `ρ·|g|·A_submersa` para cima,
/// aplicada no centroide da parte submersa.
///
/// ⚠️ **Não é uma `AreaEffector` para cima, e a diferença é o motivo da wave.** Uma
/// força constante não sabe onde a superfície está (o corpo leve é arremessado para fora
/// da piscina em vez de parar na linha d'água), é vencida pela MASSA e não pela densidade
/// (o número certo muda para cada objeto, quando a intuição é *madeira boia, pedra
/// afunda* — propriedade do MATERIAL), e não endireita nada. Arquimedes resolve os três
/// com um número só, e ele é **comparável ao `density` do `Collider`**: menor que ele o
/// corpo afunda, maior ele boia.
///
/// Terceiro componente da mesma área, pela terceira vez pela mesma razão: um blob de
/// componente é postcard POSICIONAL, então apendar campo no [`AreaEffector`] seria bump
/// de `PROJECT_SCHEMA` — e um bump **recusa todo projeto já salvo**. Mesmo acoplamento
/// dos irmãos: só morde num collider **SENSOR**, e a row da §11 é oferecida sob
/// exatamente essa condição. Config, nunca estado vivo.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AreaBuoyancy(pub f32);

impl AreaBuoyancy {
    /// É neutro — uma área sem empuxo? Usado para DESTACAR, como os irmãos.
    #[must_use]
    pub fn is_neutral(self) -> bool {
        self.0 <= 0.0
    }
}

impl SimComponent for AreaBuoyancy {}

/// **Arrasto de FORMA — a resistência que sabe para onde o corpo aponta (W-FormDrag).**
///
/// Ausente é uma área sem resistência de forma. Presente, cada aresta do corpo virada
/// para o escoamento é empurrada ao longo da própria normal, o que dá duas coisas que o
/// [`AreaDrag`] uniforme não pode dar: **resistência por secção** (o mesmo tronco sofre
/// 4× mais de través que de proa) e **freio de rotação pela FORMA** (um tronco comprido
/// resiste a girar muito mais que uma bola de mesma área).
///
/// Coexiste com o `AreaDrag` porque são mecanismos diferentes e ambos existem na
/// natureza: *Drag* é viscosidade, *Shape Drag* é resistência de forma. Quarto
/// componente da mesma área, pela quarta vez pela mesma razão (campo novo = bump de
/// `PROJECT_SCHEMA`, e um bump recusa todo projeto salvo). Só morde num SENSOR.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AreaFormDrag(pub f32);

impl AreaFormDrag {
    /// É neutro? Usado para DESTACAR, como os irmãos.
    #[must_use]
    pub fn is_neutral(self) -> bool {
        self.0 <= 0.0
    }
}

impl SimComponent for AreaFormDrag {}

/// **Torque de área** — o análogo ROTACIONAL do [`AreaEffector`] (W-AreaTorque).
///
/// Ausente (o caso comum) é uma área que não gira nada. Presente, seu `f32` é um torque
/// em N·m aplicado a cada sub-passo a todo corpo DINÂMICO que sobrepõe este collider: um
/// redemoinho, uma mesa giratória, uma esteira que gira. Enquanto o `AreaEffector` empurra
/// pelo centro de massa (não gira nada), este é a metade que FAZ girar.
///
/// ⚠️ **Um torque, resistido pelo MOMENTO DE INÉRCIA** — não uma aceleração angular. Um
/// tronco comprido gira mais devagar que uma bola de mesma área, exatamente como a folha
/// voa e o caixote não sob a `force`: a forma resiste ao giro como a massa resiste à
/// translação. Uma zona de *aceleração* seria independente da forma e uma segunda porta
/// para o que a inércia já responde.
///
/// ⚠️ **O sinal é o SENTIDO** (`> 0` anti-horário, `< 0` horário), então o neutro é
/// `== 0.0` — diferente dos irmãos de arrasto, cujo neutro é `<= 0.0` (arrasto negativo
/// adicionaria energia). Um torque negativo é uma direção, não um valor inválido.
///
/// Quinto componente da mesma área, pela quinta vez pela mesma razão: um blob de
/// componente é postcard POSICIONAL, então apendar campo no [`AreaEffector`] seria bump
/// de `PROJECT_SCHEMA` — e um bump **recusa todo projeto já salvo**. Mesmo acoplamento dos
/// irmãos: só morde num collider **SENSOR**, e a row da §11 é oferecida sob exatamente essa
/// condição. Config, nunca estado vivo.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AreaTorque(pub f32);

impl AreaTorque {
    /// É neutro — uma área que não gira nada? Um torque de exatamente zero. Usado para
    /// DESTACAR, mas com `== 0.0`: um torque negativo é um SENTIDO (horário), não um
    /// valor a descartar como o arrasto negativo dos irmãos.
    #[must_use]
    pub fn is_neutral(self) -> bool {
        self.0 == 0.0
    }
}

impl SimComponent for AreaTorque {}
