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
    /// `0` Ball · `1` Box.
    pub shape_tag: u8,
    /// Ball radius, meters (meaningless when `shape_tag == 1`).
    pub radius: f32,
    /// Box HALF-extents, meters (meaningless when `shape_tag == 0`).
    pub half_x: f32,
    pub half_y: f32,
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
    /// `ColliderShape` tag: `0` Ball · `1` Box. Switching preserves the
    /// footprint (a box becomes the ball that fits it, and back).
    Shape(u8),
    Radius(f32),
    HalfX(f32),
    HalfY(f32),
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
