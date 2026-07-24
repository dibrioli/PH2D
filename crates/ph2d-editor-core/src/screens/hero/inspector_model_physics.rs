//! §11 Physics Body + §12 Physics Joint — the Inspector snapshot structs and
//! edit enums (ADR-0131 D8/W3/W8).
//!
//! Extracted from the sibling `inspector_model` (which was at the 700-LOC file
//! cap holding seven inspector domains) into its own module. The split is not
//! only a size fix: the physics/joint domain is the one this line churns, so
//! moving it off the shared file means future physics edits stop colliding with
//! sprite/ordering/blend edits from other lines. Re-exported by `screens::hero`
//! (`pub use inspector_model_physics::*`) exactly like `inspector_model`, so the
//! crate-wide import path of every consumer is unchanged.
//!
//! Pure primitives (tags + floats + `String`) — editor-core stays loose-coupled
//! from `ph2d-ecs` and the physics crate, and the shell maps tags ↔ enums at the
//! boundary, the same discipline as §10's `blend_tag`.

/// §11 Physics Body-section snapshot (ADR-0131 D8). Mirrors the optional
/// `RigidBody` + `Collider` pair from `ph2d-physics-ecs`.
///
/// **`has_body` is the whole reason this is `Some` for a plain sprite.** The
/// other sections describe something that exists; this one also has to offer
/// the thing that does not yet, or a body could never be authored at all —
/// physics would be reachable only from a smoke scene.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InspectorPhysicsInfo {
    pub entity_bits: u64,
    /// Does this entity carry `RigidBody` + `Collider` right now?
    pub has_body: bool,
    /// `0` Dynamic · `1` Static.
    pub kind_tag: u8,
    /// `0` Ball · `1` Box · `2` Capsule.
    pub shape_tag: u8,
    /// Ball radius, meters — and the capsule's CAP radius, which is the same
    /// quantity under the same name (`shape_tag` `0` or `2`).
    pub radius: f32,
    /// Box HALF-extents, meters (meaningless unless `shape_tag == 1`).
    pub half_x: f32,
    pub half_y: f32,
    /// Capsule STRAIGHT-segment half-length, meters (`shape_tag == 2`). The
    /// capsule's total half-extent along Y is `cap_half_height + radius`, the
    /// rapier decomposition — its own field rather than reusing [`half_y`],
    /// because "half height" means a different quantity on a box than on a
    /// capsule, and one control that means two things is the bug this section
    /// keeps writing gates against.
    ///
    /// [`half_y`]: InspectorPhysicsInfo::half_y
    pub cap_half_height: f32,
    pub density: f32,
    pub restitution: f32,
    pub friction: f32,
    /// Which collision layer this body is on (`0..MAX_LAYERS`).
    ///
    /// The per-body half of collision layers; the other half — *which layers
    /// collide with which* — is a WORLD rule and lives in the Physics panel.
    /// Splitting it this way is the whole point: the rule is authored once,
    /// and a body only says where it belongs.
    pub layer: u8,
    /// Collider offset (meters, local axes) — the collider's centre relative to
    /// the sprite. `[0, 0]` is centred; a non-zero value places the collider off
    /// the sprite centre (a character's feet, an off-centre hitbox). Mirrors
    /// `Collider.offset`. Not Dynamic-only — any collider can be offset.
    pub offset: [f32; 2],
    /// How many seconds a Bake would cover, resolved by the shell (W4): the
    /// armed loop if there is one, else the document's extent, else the
    /// measured default.
    ///
    /// Shown ON the button, because a button whose effect depends on an
    /// invisible number is a button you have to experiment with. The shell
    /// resolves it once and both halves read the same answer — the painter to
    /// label it, the bake to honour it.
    pub bake_seconds: f32,
    /// Is the current selection exactly **two** bodies? Then §11 offers the
    /// Join button (W3).
    ///
    /// The precondition is answered by the shell, which is the half that owns
    /// the selection — the panel sees one entity at a time and could not work
    /// it out. It is a snapshot field rather than a check inside the painter
    /// for the usual reason: the painter decides whether to OFFER the button
    /// and the event handler decides whether to HONOUR the click, and both
    /// have to read the same fact.
    pub can_join: bool,
    /// Is this collider a **sensor** (trigger, W7)? Passes through, reports overlaps; the overlay lights it up. `false` is solid.
    pub is_sensor: bool,
    /// Which pose channels the Bake writes: `0` All · `1` Position · `2` Rotation (a global bake option the shell owns).
    pub bake_channels_tag: u8,
    /// Per-body gravity multiplier (W8): `1.0` full gravity, `0.0` weightless,
    /// `< 0` floats up, `> 1` heavier. Mirrors the optional `GravityScale`
    /// component — absent means `1.0`. Offered only for a Dynamic body, the
    /// only kind rapier applies gravity to.
    pub gravity_scale: f32,
    /// Authored INITIAL linear velocity (m/s, world axes), applied at spawn (W9).
    /// Mirrors the optional `InitialVelocity` component — absent means `[0, 0]`.
    /// Dynamic-only, like [`gravity_scale`](InspectorPhysicsInfo::gravity_scale).
    pub linvel: [f32; 2],
    /// Authored INITIAL angular velocity, in **radians/second** (component-native,
    /// like `rotation_rad`). The panel renders it as deg/s and converts at its
    /// boundary — the shell's edit and the component both stay in radians.
    pub angvel: f32,
    /// Continuous collision detection (W-CCD): `false` discrete (rapier's default),
    /// `true` sweeps the body's motion so a fast one does not tunnel through thin
    /// geometry. Mirrors the presence of the optional `Ccd` marker component.
    /// Dynamic-only, like [`gravity_scale`](InspectorPhysicsInfo::gravity_scale) —
    /// the only kind the solver moves fast.
    pub ccd: bool,
    /// Lock rotation (Freeze Rotation): `false` the orientation is free (rapier's
    /// default), `true` pins it so the body translates but never rotates — a
    /// character stays upright, a crate does not roll. Mirrors the presence of the
    /// optional `LockRotation` marker. Dynamic-only, like the fields above.
    pub lock_rotation: bool,
    /// Freeze Position X: `false` the horizontal DOF is free (rapier's default),
    /// `true` pins the body's X so the solver can never move it sideways — an
    /// elevator on a rail, a lane-locked actor. Mirrors the presence of the optional
    /// `LockPositionX` marker. Dynamic-only, like the fields above.
    pub lock_x: bool,
    /// Freeze Position Y: `true` pins the body's Y so gravity cannot pull it down —
    /// a floating platform. Mirrors the optional `LockPositionY` marker. The two
    /// axes are independent. Dynamic-only.
    pub lock_y: bool,
    /// Mass source (Unity's `useAutoMass`): `false` = Auto (the mass is `density ×
    /// area`, and the section shows the Density row); `true` = Manual (the mass is
    /// [`mass`](InspectorPhysicsInfo::mass) kg, and the section shows the Mass row).
    /// Mirrors the PRESENCE of the optional `MassOverride` component. Density and
    /// mass are the same quantity by two roads, so exactly one is ever live.
    /// Dynamic-only — a Static/Kinematic body has infinite mass.
    pub mass_manual: bool,
    /// The explicit mass in kg, shown in the Mass row when
    /// [`mass_manual`](InspectorPhysicsInfo::mass_manual) is `true` (the
    /// `MassOverride` component's value). Meaningless in Auto mode, where Density is
    /// the live control instead.
    pub mass: f32,
    /// Dominance group — a collision priority (`0` neutral). A higher value bulldozes
    /// lower ones (infinite relative mass to them) while still falling and colliding
    /// normally with peers. Mirrors the optional `Dominance` component — absent means
    /// `0`. Dynamic-only, like gravity/velocity (a non-dynamic body is already at the
    /// max).
    pub dominance: i8,
    /// How this collider's RESTITUTION combines with another's on contact
    /// (W-Material): `0` Average · `1` Min · `2` Multiply · `3` Max. Mirrors the
    /// optional `MaterialCombine` component's `restitution` rule — absent means
    /// `Average`. NOT Dynamic-only: a static floor's combine rule matters too.
    pub restitution_combine_tag: u8,
    /// How this collider's FRICTION combines with another's — the sibling of
    /// [`restitution_combine_tag`](InspectorPhysicsInfo::restitution_combine_tag),
    /// from the same `MaterialCombine` component.
    pub friction_combine_tag: u8,
    /// Per-body linear damping (drag) — decays translation (W-Damping). Mirrors the
    /// optional `DampingOverride` component's `linear`; absent means the world default.
    /// Dynamic-only (damping decays a velocity the solver owns).
    pub linear_damping: f32,
    /// Per-body angular damping (drag) — decays rotation. Sibling of
    /// [`linear_damping`](InspectorPhysicsInfo::linear_damping).
    pub angular_damping: f32,
    /// How the damping combines with the world default drag: `0` Combine (adds to it)
    /// · `1` Replace (ignores it). Mirrors the `DampingOverride` component's `mode`.
    pub damp_mode_tag: u8,
    /// **One-way (jump-through) platform** (W-OneWay): the collider is solid only from
    /// its local +Y side, so a body arriving from below passes through and lands on it
    /// coming back down. Mirrors the presence of the optional `OneWayPlatform` marker.
    /// NOT Dynamic-only — a platform is usually Static, which is the whole point.
    pub one_way: bool,
    /// **Force zone** (W-Area): the force in newtons, world axes, this area applies to
    /// every dynamic body overlapping it — wind, an updraft, a conveyor. Mirrors the
    /// optional `AreaEffector` component's `force`; absent means a body that pushes
    /// nothing.
    ///
    /// ⚠️ Offered only when the collider is a **Sensor**, not by body kind: the narrow
    /// phase records an overlap only for a sensor, and a solid collider pushes bodies
    /// out rather than letting them in. It is the first §11 control gated on another
    /// CONTROL rather than on `kind_tag`.
    pub force: [f32; 2],
    /// **In whose axes is [`force`](InspectorPhysicsInfo::force)?** (W-AreaFrame)
    ///
    /// `false` (the default) is the ZONE's own frame — rotating the sensor rotates the
    /// wind, so a diagonal conveyor is a conveyor you turned. `true` pins the direction
    /// to world axes: the zone turns, the blow does not (Unity's `useGlobalAngle`).
    /// Mirrors the presence of the optional `AreaForceWorldAxes` marker.
    ///
    /// ⚠️ It qualifies the FORCE alone, which is geometry rather than a chosen scope: a
    /// 2D torque is a scalar about Z and an in-plane rotation is about Z, drag is
    /// isotropic, buoyancy takes its surface from gravity, and shape drag pushes along
    /// the BODY's edge normals. Same Sensor condition as the rows it sits under.
    pub force_world_axes: bool,
    /// **Area drag** (W-AreaDrag): the resistance the medium inside this sensor offers
    /// — the difference between wind and water. Mirrors the optional `AreaDrag`
    /// component; absent means an area that resists nothing. Offered under the same
    /// Sensor condition as [`force`](InspectorPhysicsInfo::force).
    pub area_drag: f32,
    /// **Densidade do fluido** (W-Buoyancy): o empuxo desta área. Espelha o opcional
    /// `AreaBuoyancy`; ausente significa uma área sem empuxo. Oferecida sob a mesma
    /// condição de Sensor que [`force`](InspectorPhysicsInfo::force).
    pub area_density: f32,
    /// **Arrasto de forma** (W-FormDrag): a resistência que sabe para onde o corpo
    /// aponta. Espelha o opcional `AreaFormDrag`. Mesma condição de Sensor que as irmãs.
    pub area_form_drag: f32,
    /// **Torque de área** (W-AreaTorque): o giro (N·m) que esta área imprime a cada corpo
    /// dentro dela — um redemoinho, uma mesa giratória. Espelha o opcional `AreaTorque`;
    /// o SINAL é o sentido (`> 0` anti-horário), então ausente/zero é uma área que não
    /// gira nada. Mesma condição de Sensor que as irmãs.
    pub area_torque: f32,
}

/// A single editable §11 physics field, dispatched as
/// [`EditorAction::InspectorPhysicsEdit`].
///
/// [`PhysicsFieldEdit::Add`] carries no geometry on purpose: the default
/// collider is derived from the sprite's own bounds **by the shell**, which
/// is the half that knows how big the art is. A collider that starts as the
/// sprite's box is the one shape that can never disagree with what is drawn.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PhysicsFieldEdit {
    /// Attach `RigidBody{Dynamic}` + a `Collider` boxed to the sprite.
    Add,
    /// Detach both components — the entity goes back to being plain art.
    Remove,
    /// `BodyKind` tag: `0` Dynamic · `1` Static · `2` Kinematic.
    Kind(u8),
    /// `ColliderShape` tag: `0` Ball · `1` Box · `2` Capsule. Switching
    /// preserves the footprint (a box becomes the ball that fits it, and back;
    /// a capsule keeps the box's width as its radius).
    Shape(u8),
    Radius(f32),
    HalfX(f32),
    HalfY(f32),
    /// Capsule straight-segment half-length (see
    /// [`InspectorPhysicsInfo::cap_half_height`]).
    CapHalfHeight(f32),
    /// Collider offset from the sprite centre, meters (local axes). Read-modify-
    /// write on `Collider.offset`; not Dynamic-only.
    OffsetX(f32),
    OffsetY(f32),
    Density(f32),
    Restitution(f32),
    Friction(f32),
    /// Move this body to a collision layer (`0..MAX_LAYERS`).
    Layer(u8),
    /// The "Solid | Sensor" toggle (W7): make this collider a sensor or solid.
    Sensor(bool),
    /// Per-body gravity multiplier (W8). Attaches/updates the optional
    /// `GravityScale` component, or detaches it at the neutral `1.0` so an
    /// unscaled body carries no component (the presence-override idiom the
    /// ordering fields use).
    GravityScale(f32),
    /// Authored initial linear velocity, X axis, m/s (W9). Read-modify-write on
    /// the optional `InitialVelocity` component; detached when it returns to rest.
    LinvelX(f32),
    /// Authored initial linear velocity, Y axis, m/s (W9).
    LinvelY(f32),
    /// Authored initial angular velocity, in **radians/second** — the panel
    /// converts the deg/s the artist sees before emitting this (W9).
    Angvel(f32),
    /// Continuous collision detection toggle (W-CCD). Attaches/detaches the
    /// optional `Ccd` marker component (present = continuous, absent = discrete),
    /// the presence-override idiom gravity and initial velocity use.
    Ccd(bool),
    /// Lock-rotation toggle (Freeze Rotation). Attaches/detaches the optional
    /// `LockRotation` marker (present = locked, absent = free), same idiom as CCD.
    LockRotation(bool),
    /// Freeze-Position-X toggle. Attaches/detaches the optional `LockPositionX`
    /// marker (present = locked, absent = free), same idiom as Freeze Rotation.
    LockPositionX(bool),
    /// Freeze-Position-Y toggle. Attaches/detaches the optional `LockPositionY`
    /// marker.
    LockPositionY(bool),
    /// Mass-source toggle (Auto | Manual). `true` (Manual) attaches a `MassOverride`
    /// seeded from the current auto mass; `false` (Auto) detaches it so the body
    /// weighs `density × area` again — the presence-override idiom.
    MassMode(bool),
    /// The explicit mass, kg (only meaningful in Manual mode). Updates the
    /// `MassOverride` component's value.
    Mass(f32),
    /// Dominance group (collision priority). Attaches/updates the optional
    /// `Dominance` component, or detaches it at the neutral `0` — the presence-
    /// override idiom. The panel rounds the widget's float to this `i8`.
    Dominance(i8),
    /// Restitution combine rule (W-Material): `0` Average · `1` Min · `2` Multiply
    /// · `3` Max. Read-modify-write on the optional `MaterialCombine` component,
    /// detached when both rules return to `Average` — the presence-override idiom.
    RestitutionCombine(u8),
    /// Friction combine rule — the sibling of [`RestitutionCombine`], on the same
    /// `MaterialCombine` component.
    ///
    /// [`RestitutionCombine`]: PhysicsFieldEdit::RestitutionCombine
    FrictionCombine(u8),
    /// Per-body linear damping (drag), Dynamic-only (W-Damping). Read-modify-write on
    /// the optional `DampingOverride` component, detached at neutral (zero drag +
    /// Combine mode).
    LinearDamping(f32),
    /// Per-body angular damping (drag), Dynamic-only. Sibling of [`LinearDamping`], on
    /// the same `DampingOverride` component.
    ///
    /// [`LinearDamping`]: PhysicsFieldEdit::LinearDamping
    AngularDamping(f32),
    /// Damping combine mode: `0` Combine (adds to the world default) · `1` Replace
    /// (ignores it). On the same `DampingOverride` component.
    DampMode(u8),
    /// One-way (jump-through) platform toggle (W-OneWay). Attaches/detaches the optional
    /// `OneWayPlatform` marker — the presence-override idiom. NOT Dynamic-only.
    OneWay(bool),
    /// The FRAME of the zone's force (W-AreaFrame): `false` the zone's own axes (turning
    /// the sensor turns the wind), `true` pinned to world axes. Attaches/detaches the
    /// optional `AreaForceWorldAxes` marker — the presence-override idiom, so the default
    /// costs no component. SENSOR-only, the condition the painter offers it under.
    ForceWorldAxes(bool),
    /// Force-zone push, X axis, in newtons (W-Area). Read-modify-write on the optional
    /// `AreaEffector` component, detached at neutral (zero on both axes). Honoured only
    /// for a SENSOR collider — the same condition the painter offers it under.
    ForceX(f32),
    /// Force-zone push, Y axis. Sibling of [`ForceX`], on the same `AreaEffector`.
    ///
    /// [`ForceX`]: PhysicsFieldEdit::ForceX
    ForceY(f32),
    /// Area drag (W-AreaDrag) — the medium's resistance inside a force zone. Attaches
    /// the optional `AreaDrag` component, detached at zero. SENSOR-only, like Force.
    AreaDrag(f32),
    /// Densidade do fluido (W-Buoyancy) — o empuxo de Arquimedes. Anexa o opcional
    /// `AreaBuoyancy`, destacado em zero. SENSOR-only, como Force e Drag.
    AreaDensity(f32),
    /// Arrasto de FORMA (W-FormDrag) — resistência por secção + freio de rotação pela
    /// forma. Anexa o opcional `AreaFormDrag`, destacado em zero. SENSOR-only.
    AreaFormDrag(f32),
    /// Torque de área (W-AreaTorque) — o giro que a zona imprime. Anexa o opcional
    /// `AreaTorque`, destacado em zero (o SINAL é o sentido, não um valor a descartar).
    /// SENSOR-only, como as irmãs.
    AreaTorque(f32),
    /// Which pose channels the Bake writes: `0` All · `1` Position · `2` Rotation.
    BakeChannels(u8),
    /// Create a joint between the two selected bodies (W3). Carries no
    /// operands: the shell owns the selection, and a second copy of "which
    /// two" would be a second answer to a question only one half can answer.
    Join,
    /// Bake the selection's simulated motion into timeline curves (W4).
    ///
    /// Carries no range for the same reason `Join` carries no bodies: the
    /// shell owns the clock, and the panel only shows the number it is told
    /// ([`InspectorPhysicsInfo::bake_seconds`]). And like `Join`, it must NOT
    /// fan out over a multi-selection — one bake covers every selected body in
    /// one run of the simulation, where a fan-out would re-simulate the whole
    /// scene once per body and file a separate undo step for each.
    Bake,
}

/// §12 Physics Joint snapshot (W3) — the selected **joint object**.
///
/// A joint is an entity, so this is the section that describes it: what kind
/// of constraint it is, which two bodies it names, and the parameters that
/// kind actually uses. Not `Copy`, unlike its siblings, because it carries the
/// two bodies' NAMES — the joint stores name hashes, and a hash is not
/// something to show a person.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorJointInfo {
    pub entity_bits: u64,
    /// `0` Pin · `1` Spring · `2` Rope · `3` Weld.
    pub kind_tag: u8,
    /// The bodies, resolved for display. Empty means the name no longer
    /// matches any body in the scene — deleted or renamed.
    pub body_a_name: String,
    pub body_b_name: String,
    /// Are BOTH bodies present right now? The section says so out loud: a
    /// joint whose body was renamed is dormant, not broken, and silently
    /// showing its parameters as if it were live would be a lie.
    pub bound: bool,
    pub limits_enabled: bool,
    /// **Degrees** at this boundary; the component stores radians, exactly as
    /// `rotation_rad` does.
    pub limit_min_deg: f32,
    pub limit_max_deg: f32,
    pub motor_enabled: bool,
    /// Degrees per second.
    pub motor_speed_deg: f32,
    pub motor_max_force: f32,
    pub rest_length: f32,
    pub stiffness: f32,
    pub damping: f32,
    pub max_length: f32,
}

/// A single editable §12 joint field, dispatched as
/// [`EditorAction::InspectorJointEdit`](crate::action_bus::EditorAction).
///
/// Angles arrive in **degrees** and the shell converts, so the panel never
/// holds a radian and the component never holds a degree.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum JointFieldEdit {
    /// `JointKind` tag: `0` Pin · `1` Spring · `2` Rope · `3` Weld.
    Kind(u8),
    LimitsEnabled(bool),
    LimitMinDeg(f32),
    LimitMaxDeg(f32),
    MotorEnabled(bool),
    MotorSpeedDeg(f32),
    MotorMaxForce(f32),
    RestLength(f32),
    Stiffness(f32),
    Damping(f32),
    MaxLength(f32),
    /// Delete the joint object.
    Remove,
}
