//! Plain-data descriptors for [`crate::world::PhysicsWorld`] — the body the
//! ECS bridge asks for, and the snapshot it reads back.
//!
//! Split out of `world.rs` so it stays under the LOC cap. `BodyDesc` names one
//! rapier type (`RigidBodyType`), so — unlike [`super::shape::ShapeDesc`] — it
//! is not rapier-free; that is why they live in separate siblings.

use rapier2d::dynamics::{CoefficientCombineRule, RigidBodyType};

use super::shape::ShapeDesc;

/// How the two colliders' friction and restitution coefficients are combined
/// on contact (rapier's `CoefficientCombineRule`, Unity's `PhysicMaterial`
/// combine). Bundled as one struct because rapier itself groups them in
/// `ColliderMaterial`, and because appending ONE field to [`BodyDesc`] costs
/// every fixture a single `material: Default::default()` line instead of two.
///
/// **The higher-priority rule of the two colliders wins** (rapier combines with
/// `rule1.max(rule2)`, and the enum orders `Average < Min < Multiply < Max`), so
/// a superball set to `Max` bounces off ANY floor regardless of the floor's rule
/// — which is the whole point of exposing it: with the default `Average`, a
/// bouncy ball on a dead floor only returns to half its drop height.
///
/// `Average` on both is rapier's own default, so a body authored before this
/// existed simulates byte-identically. It rides the [`BodyDesc`] the world
/// rebuilds from, so a rewind re-arms it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CombineRules {
    /// How the two colliders' restitution (bounciness) is combined.
    pub restitution: CoefficientCombineRule,
    /// How the two colliders' friction is combined.
    pub friction: CoefficientCombineRule,
}

impl Default for CombineRules {
    /// rapier's own default: both `Average`. Byte-neutral, so an unset body is
    /// identical to before this existed.
    fn default() -> Self {
        Self {
            restitution: CoefficientCombineRule::Average,
            friction: CoefficientCombineRule::Average,
        }
    }
}

/// A per-body **damping override** (Unity's `Rigidbody2D` linear/angular drag,
/// Godot's `linear_damp`/`angular_damp` + `damp_mode`). `linear`/`angular` are drag
/// coefficients that decay the body's velocities each step.
///
/// `replace` chooses how they meet the world's [`super::BodyDefaults`] damping:
/// `false` = **Combine** (ADD to the global default — Godot's default), `true` =
/// **Replace** (IGNORE the global, use these outright — Unity's absolute per-body
/// drag). With the default global drag of `0.0` the two modes coincide, so the mode
/// only diverges once the artist authors a world drag.
///
/// It only bites a Dynamic body (damping decays a velocity the solver owns), and it
/// rides the [`BodyDesc`] the world rebuilds from, so a rewind re-arms it. It is
/// resolved against the global through the single door
/// [`super::PhysicsWorld::apply_damping_override`], used by both spawn and the
/// bridge's re-stamp pass so the two cannot resolve one override differently.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DampingDesc {
    pub linear: f32,
    pub angular: f32,
    /// `true` = Replace (ignore the world default); `false` = Combine (add to it).
    pub replace: bool,
}

/// What a **force zone** does to the bodies inside it (W-Area, W-AreaDrag).
///
/// Bundled for the same reason [`CombineRules`] is: they are two statements about ONE
/// area, they reach the world through one field, and bundling costs each fixture a
/// single `effector: None` instead of two. (On the ECS side they stay two separate
/// components — see `AreaDrag` — because THAT side is serialized, and a second
/// component is additive where a second field is a `PROJECT_SCHEMA` bump.)
///
/// The two are independent and each is meaningful alone: a wind that does not slow
/// you (`drag: 0`), a pool of syrup that does not push (`force: [0, 0]`).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AreaEffect {
    /// Constant force in newtons, world axes. Resisted by the body's mass.
    pub force: [f32; 2],
    /// **Drag coefficient inside the area** — a medium, not a push.
    ///
    /// It is the SAME law as rapier's `linear_damping` and the world's
    /// [`super::BodyDefaults`] drag (`v /= 1 + d·dt` — rapier's own `apply_damping`),
    /// so the word "drag" means one thing everywhere in this app: uniform decay,
    /// mass-independent. ⚠️ Same LAW, not the same arithmetic: rapier applies its
    /// damping deep inside the velocity solver while a zone applies it at the top of
    /// the substep, and the zone can only reach bodies the previous substep reported
    /// as overlapping — so over a run it performs exactly **one fewer** decay, a ratio
    /// of `1 + d·dt_substep` (measured 1.0125 at d = 3, and pinned in a gate so a
    /// change of application point shows up as a named quantity). It damps
    /// **both** the linear and the angular velocity, because a medium resists a spin
    /// too — Godot exposes the two separately on an `Area2D`, but a region is a
    /// substance rather than a tuning surface, and the per-BODY `DampingOverride`
    /// already covers the asymmetric case.
    pub drag: f32,
    /// **Densidade do fluido** (kg/m² em 2D) — o empuxo de Arquimedes.
    ///
    /// `0` (o default) é uma área sem empuxo, byte-idêntica a antes deste campo. Maior
    /// que zero, a área empurra cada corpo submerso para cima com `ρ·|g|·A_submersa`,
    /// aplicada no centroide da parte submersa (ver [`super::buoyancy`]).
    ///
    /// ⚠️ **Não é a mesma coisa que uma `force` para cima, e a diferença é o que faz
    /// valer a pena:** uma força constante não sabe onde a superfície está (o corpo leve
    /// é arremessado para fora em vez de parar na linha d'água), é por MASSA e não por
    /// densidade (o número certo muda para cada objeto, quando a intuição é *madeira
    /// boia, pedra afunda* — propriedade do MATERIAL), e não endireita nada. Arquimedes
    /// resolve os três com um número só, e o número é comparável ao `density` do
    /// collider: menor que ele afunda, maior boia.
    pub density: f32,
}

/// Snapshot of one rigid body for hashing / inspection. Sorted by
/// handle index in [`crate::world::PhysicsWorld::body_snapshots`] so cross-OS
/// hashing is stable.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BodySnapshot {
    pub handle_index: u32,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub linvel_x: f32,
    pub linvel_y: f32,
    pub angvel: f32,
}

/// One body + its single collider, described in plain data for the ECS
/// bridge. All lengths are world units (meters); `rotation` is radians
/// CCW; `density` feeds rapier's mass computation (ignored for
/// non-dynamic bodies). **Append-only:** new fields go at the END.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BodyDesc {
    pub body_type: RigidBodyType,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub density: f32,
    pub shape: ShapeDesc,
    /// Bounciness `0..=1`. rapier's own default is `0.0`.
    pub restitution: f32,
    /// Coulomb friction. rapier's own default is `0.5`.
    pub friction: f32,
    /// Which collision layer this body belongs to (`0..MAX_LAYERS`).
    ///
    /// Only the LAYER travels here — never the resulting `InteractionGroups`.
    /// The filter is a function of `(layer, world matrix)`, and computing it in
    /// two places is how the two would come to disagree; [`crate::world::PhysicsWorld`]
    /// owns the matrix and is the single door (see `layers`).
    pub layer: u8,
    /// A **sensor** (trigger): the collider passes through — no contact forces,
    /// nothing is pushed — but the solver still reports which colliders overlap
    /// it. Default `false` is a solid collider, byte-identical to before sensors
    /// existed. The overlaps are read back through
    /// [`crate::world::PhysicsWorld::intersecting_body_pairs`]; a sensor with
    /// nothing reading them is inert, which is why this landed with the overlay
    /// + Inspector that make the detection visible (ADR-0131 W7).
    pub is_sensor: bool,
    /// Per-body multiplier on the world gravity. `1.0` = full gravity (rapier's
    /// own default, and what every body did before this field existed); `0.0` =
    /// weightless (a bullet, a top-down actor); `< 0` = floats up (a balloon);
    /// `> 1` = heavier. rapier ignores it for non-dynamic bodies, so it only
    /// bites a `Dynamic` one.
    ///
    /// It lives in `BodyDesc` — the spawn recipe the world stores per body —
    /// because `rewind_to` rebuilds the world FROM these descriptors, so a value
    /// that were not here would be silently dropped on the first scrub, and a
    /// gravity-scaled body would fall at full gravity after a rewind. The
    /// authored source is the optional `GravityScale` component in
    /// `ph2d-physics-ecs`; the bridge folds it in here (ADR-0131 W8).
    pub gravity_scale: f32,
    /// **Initial linear velocity** (m/s, world axes), applied at SPAWN. `[0, 0]`
    /// = at rest, what every body did before this existed. A launched projectile,
    /// a ball kicked at t=0 — until this, a body could only start still, which is
    /// why the smoke scenes had to tilt gravity to fake a push.
    ///
    /// It is applied when the body is BUILT and then owned by the solver, never
    /// re-applied per tick (that would re-launch it every frame). But it lives in
    /// `BodyDesc` — the spawn recipe `rewind_to` rebuilds from — precisely so a
    /// scrub back to t=0 re-arms the same launch, exactly like `gravity_scale`.
    /// The authored source is the optional `InitialVelocity` component (W9).
    pub linvel: [f32; 2],
    /// **Initial angular velocity** (rad/s, CCW), applied at spawn — a spinning
    /// wheel, a thrown object tumbling. Same lifetime as [`linvel`](BodyDesc::linvel):
    /// set at build, owned by the solver after, re-armed on rewind.
    pub angvel: f32,
    /// **Continuous collision detection.** `false` (rapier's own default, and
    /// what every body did before this field existed) is *discrete*: the body
    /// is only tested at each tick's end pose, so a small fast body can pass
    /// clean THROUGH thin geometry between two ticks — at 30 m/s it moves half a
    /// metre in a 60 Hz tick and a 0.1 m wall lands entirely inside that gap.
    /// `true` sweeps the body's motion and stops it at the first impact instead.
    ///
    /// It is a spawn-time flag on the rigid body, so — like [`gravity_scale`]
    /// and [`linvel`] — it rides the `BodyDesc` the world rebuilds from, and a
    /// rewind to t=0 re-arms it. Distinct from the sub-stepping that fixes deep
    /// LANDING overlap (`PhysicsWorld::DEFAULT_SUBSTEPS`): that reduces how far
    /// inside a body is when it first touches, which no solver undoes after the
    /// fact; this catches the collision that would otherwise be MISSED entirely.
    ///
    /// [`gravity_scale`]: BodyDesc::gravity_scale
    /// [`linvel`]: BodyDesc::linvel
    pub ccd: bool,
    /// **Lock rotation** (Unity's "Freeze Rotation", Godot's `lock_rotation`).
    /// `false` (rapier's default, and what every body did before this existed)
    /// leaves the angular DOF free — a box tips and tumbles as it slides. `true`
    /// pins the orientation: the body still translates and collides but never
    /// rotates, so a character stays upright and a crate does not roll.
    ///
    /// It maps to rapier's `LockedAxes::ROTATION_LOCKED`, applied at build, and —
    /// like [`ccd`] and [`gravity_scale`] — it rides the `BodyDesc` the world
    /// rebuilds from, so a rewind to t=0 re-arms it. A locked body ignores any
    /// authored [`angvel`]: with no angular DOF there is nothing to spin.
    ///
    /// [`ccd`]: BodyDesc::ccd
    /// [`gravity_scale`]: BodyDesc::gravity_scale
    /// [`angvel`]: BodyDesc::angvel
    pub lock_rotation: bool,
    /// **Collider offset** — the collider's position relative to the body, in
    /// world units, along the body's LOCAL axes. `[0, 0]` (the default, and what
    /// every collider did before this field existed) centres the collider on the
    /// body; a non-zero offset places it elsewhere — the feet of a character below
    /// its sprite centre, the tip of a projectile ahead of it.
    ///
    /// It becomes the collider's translation relative to its rigid body, so rapier
    /// rotates it WITH the body (a rotated character's foot-box rotates too). The
    /// caller is expected to have already folded the body's scale into it (a scaled
    /// sprite's offset scales with it, and a flipped one mirrors); the world does
    /// not see the `Transform`. It rides the `BodyDesc` the world rebuilds from, so
    /// a rewind preserves it.
    pub offset: [f32; 2],
    /// **Lock translation on X** (Unity's "Freeze Position X", Godot's
    /// `axis_lock_linear_x`). `false` (rapier's default, and what every body did
    /// before this field existed) leaves the horizontal DOF free. `true` pins the
    /// body's X position: it still falls, rotates and collides, but the solver can
    /// never move it sideways — a rail-constrained elevator, a side-scroller actor
    /// held to a lane.
    ///
    /// It ORs into the same rapier `LockedAxes` bitmask as [`lock_rotation`], applied
    /// at build, so — like the constraints beside it — it rides the `BodyDesc` the
    /// world rebuilds from and a rewind to t=0 re-arms it. A body with any initial
    /// [`linvel`] on X still has that component dropped by the lock (there is no X
    /// DOF to carry it).
    ///
    /// [`lock_rotation`]: BodyDesc::lock_rotation
    /// [`linvel`]: BodyDesc::linvel
    pub lock_x: bool,
    /// **Lock translation on Y** (Unity's "Freeze Position Y"). `false` leaves the
    /// vertical DOF free (rapier's default). `true` pins the body's Y position, so
    /// gravity cannot pull it down — a floating platform, a hovering pickup that
    /// still slides horizontally. Same lifetime and `LockedAxes` fold as [`lock_x`].
    ///
    /// [`lock_x`]: BodyDesc::lock_x
    pub lock_y: bool,
    /// **Explicit mass override** (Unity's manual `Rigidbody2D.mass`, the opposite
    /// of `useAutoMass`). `None` (the default, and what every body did before this
    /// existed) is *auto mass*: the mass is `density × area`, computed by rapier
    /// from the collider — `.density(desc.density)`. `Some(m)` sets the mass to `m`
    /// kg directly with `ColliderBuilder::mass(m)`, ignoring density; the angular
    /// inertia is still derived from the shape (so a heavy box rotates like a box).
    ///
    /// It is a spawn-time collider mass-property, so — like the flags around it — it
    /// rides the `BodyDesc` the world rebuilds from and a rewind re-arms it. It only
    /// bites a Dynamic body: a Static/Kinematic body has infinite mass and rapier
    /// ignores both density and mass, which is why the §11 control is Dynamic-only.
    ///
    /// The authored source is the optional `MassOverride` component in
    /// `ph2d-physics-ecs`; the bridge folds it in here. Density and mass are the
    /// SAME quantity by two roads (mass = density × area), so exactly one is ever
    /// live — `None` means density is the source, `Some` means it is not (the
    /// Auto/Manual mode the Inspector exposes, never both at once).
    pub mass_override: Option<f32>,
    /// **Dominance group** — a collision PRIORITY (rapier's `dominance_group`,
    /// Box2D's dominance). `0` (the default, and what every body did before this
    /// existed) is the neutral group. When two bodies collide, the one with the
    /// STRICTLY higher dominance is treated as having infinite mass by the lower one:
    /// it bulldozes through and is never pushed back, while still falling under
    /// gravity and colliding normally with equal-or-higher peers. Equal dominance is
    /// an ordinary collision.
    ///
    /// It is orthogonal to mass: a LIGHT body with high dominance shoves a HEAVY one
    /// with low dominance — the unstoppable mover, the boss, the player that pushes
    /// debris but is never shoved by it. Static/Kinematic bodies sit at the maximum
    /// (`i8::MAX + 1`), so a dynamic body never pushes them — consistent with their
    /// infinite mass.
    ///
    /// It maps to `RigidBodyBuilder::dominance_group`, set at build, so — like the
    /// flags around it — it rides the `BodyDesc` the world rebuilds from and a rewind
    /// re-arms it. It bites only a dynamic body (a non-dynamic one is already at the
    /// max), which is why the §11 row is Dynamic-only. The authored source is the
    /// optional `Dominance` component (attached only when non-zero).
    pub dominance: i8,
    /// **How this collider's friction and restitution combine with another's**
    /// on contact (see [`CombineRules`]). `Default` = both `Average`, rapier's
    /// own default and byte-identical to before this field existed.
    ///
    /// Unlike the flags around it this is a COLLIDER material property, not a
    /// rigid-body one, so — like `restitution`/`friction` — it applies to every
    /// body kind, not just Dynamic (a static floor's combine rule matters too).
    /// It rides the `BodyDesc` the world rebuilds from, so a rewind re-arms it.
    /// The authored source is the optional `MaterialCombine` component in
    /// `ph2d-physics-ecs` (attached only when either rule leaves `Average`).
    pub material: CombineRules,
    /// **Per-body damping override** (see [`DampingDesc`]). `None` (the default, and
    /// what every body did before this existed) leaves the body on the world's
    /// [`super::BodyDefaults`] damping — byte-identical. `Some` overrides it, resolved
    /// against the global through [`super::PhysicsWorld::apply_damping_override`].
    ///
    /// Like the flags around it it rides the `BodyDesc` the world rebuilds from, so a
    /// rewind re-arms it; unlike them it is re-stamped by the bridge each dispatch,
    /// so a change to the GLOBAL drag mid-play cannot leave an override body wearing
    /// the world value (the `apply_to_all` clobber). The authored source is the
    /// optional `DampingOverride` component in `ph2d-physics-ecs`.
    pub damping: Option<DampingDesc>,
    /// **One-way (jump-through) platform** — the iconic 2D platformer collider.
    /// `false` (the default, and what every collider did before this existed) is an
    /// ordinary solid collider, byte-identical. `true` makes the collider solid only
    /// from its **local +Y side**: a body arriving from above lands on it, a body
    /// arriving from below passes clean through (Godot's `one_way_collision`).
    ///
    /// The direction is the collider's own local up, so a ROTATED platform is one-way
    /// along its own axis — no separate direction field to disagree with the pose.
    ///
    /// It is realised by rapier's `ContactModificationContext::update_as_oneway_platform`
    /// through [`super::oneway::OneWayHooks`], which also owns the allowed/forbidden
    /// hysteresis that keeps a body from popping. The flag reaches the hook as a bit in
    /// the collider's `user_data` plus `ActiveHooks::MODIFY_SOLVER_CONTACTS`. It rides
    /// the `BodyDesc` the world rebuilds from, so a rewind re-arms it. The authored
    /// source is the optional `OneWayPlatform` marker in `ph2d-physics-ecs`.
    pub one_way: bool,
    /// **Force zone** — what this area does to the dynamic bodies overlapping it: a
    /// constant force in newtons, a drag that resists them, or both (see
    /// [`AreaEffect`]). Unity's `AreaEffector2D`, Godot's `Area2D` overrides. `None`
    /// (the default, and what every body did before this existed) is an ordinary body
    /// that neither pushes nor resists.
    ///
    /// It is a **force, not an acceleration**: the impulse `F·dt` is resisted by the
    /// body's mass, so a leaf is carried by a wind a crate barely feels. That is the
    /// half an artist cannot author per-body today — a per-body *acceleration* would
    /// duplicate `gravity_scale`, which already exists.
    ///
    /// ⚠️ **It only bites when the collider is a SENSOR** ([`is_sensor`]): the narrow
    /// phase records an overlap only when one side is a sensor, and a solid collider
    /// pushes bodies OUT rather than letting them in — an area you cannot enter is not
    /// an area. [`super::effector::zone_force`] is the single door that answers "is this
    /// body a force zone", so the world and its gates cannot disagree.
    ///
    /// Like the flags around it, it rides the `BodyDesc` the world rebuilds from, so a
    /// rewind re-arms it. The authored source is the optional `AreaEffector` component
    /// in `ph2d-physics-ecs`.
    ///
    /// [`is_sensor`]: BodyDesc::is_sensor
    pub effector: Option<AreaEffect>,
}
