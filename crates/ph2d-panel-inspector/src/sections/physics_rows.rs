//! §11 Physics Body — row-painting helpers split out of `physics.rs` for the
//! panel's 600-LOC file cap (W-Mass pushed it over).
//!
//! These are pure "which rows does this body show" helpers; the section painter in
//! `physics.rs` calls them. They take resolved booleans (`is_dynamic`, `mass_manual`)
//! rather than the whole `InspectorPhysicsInfo`, so this file shares no private
//! consts with `physics.rs` — the two only meet at the call site.

use super::rows::{num_row, seg_row};
use super::*;

/// Mass-source toggle labels, indexed by `mass_manual as u8`: `0` Auto (mass is
/// density×area, the Density row) · `1` Manual (an explicit mass in kg, the Mass row).
const MASS_MODE_LABELS: [&str; 2] = ["Auto", "Manual"];

/// Combine-rule labels, indexed by `CombineRule` tag: how two colliders'
/// friction/restitution merge on contact (Unity's `PhysicMaterial` combine).
/// `Max` makes a superball bounce off any floor; `Average` (tag 0) is the default.
const COMBINE_LABELS: [&str; 4] = ["Average", "Min", "Multiply", "Max"];

/// Damping-mode toggle labels, indexed by `DampMode` tag: `0` Combine (adds to the
/// world default drag) · `1` Replace (ignores it — Unity's absolute per-body drag).
const DAMP_MODE_LABELS: [&str; 2] = ["Combine", "Replace"];

/// The **Dynamic-only** damping rows: linear + angular drag, and the mode that says
/// how they meet the world default drag (Combine adds, Replace ignores) (W-Damping).
///
/// Damping decays a velocity the solver owns, so it is meaningless on a Static
/// (never moves) or Kinematic (pose-driven) body — the same Dynamic-only rule the
/// gravity/velocity rows follow. Split here so the caller stays under the panel's
/// 200-LOC fn cap; the mode selection reads straight off the snapshot, so only the
/// two number boxes are synced.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_damping_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    damp_mode_tag: u8,
) -> f32 {
    let mut yy = y;
    for (label, id) in [
        ("Linear Damping", ids::INSP_PHYS_LINEAR_DAMPING),
        ("Angular Damping", ids::INSP_PHYS_ANGULAR_DAMPING),
    ] {
        yy = num_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            label,
            id,
        );
    }
    seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        "Damp Mode",
        ids::INSP_LIVE_PHYSICS_DAMPMODE,
        &ids::INSP_PHYS_DAMPMODE,
        &DAMP_MODE_LABELS,
        damp_mode_tag,
    )
}

/// The collider MATERIAL rows: **Bounce** + **Friction** (the coefficients) and,
/// right under each, how it COMBINES with the other collider on contact — a Bounce
/// Combine and a Friction Combine segmented control (W-Material).
///
/// Offered for ANY body kind, not Dynamic-only: a static floor's combine rule
/// matters too, because rapier takes the higher-priority of the two colliders' rules
/// (so a `Max` superball bounces off any floor). The two combine selections read
/// straight off the snapshot, so there is nothing to sync. Split here so
/// `paint_physics_section` stays under the panel's 200-LOC fn cap.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_material_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    restitution_combine_tag: u8,
    friction_combine_tag: u8,
) -> f32 {
    let mut yy = y;
    for (label, id) in [
        ("Bounce", ids::INSP_PHYS_RESTITUTION),
        ("Friction", ids::INSP_PHYS_FRICTION),
    ] {
        yy = num_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            label,
            id,
        );
    }
    // How Bounce/Friction combine with the OTHER collider — one segmented control
    // each, sitting right under the value it governs.
    for (label, group, ids, tag) in [
        (
            "Bounce Combine",
            ids::INSP_LIVE_PHYSICS_REST_COMBINE,
            &ids::INSP_PHYS_REST_COMBINE,
            restitution_combine_tag,
        ),
        (
            "Friction Combine",
            ids::INSP_LIVE_PHYSICS_FRIC_COMBINE,
            &ids::INSP_PHYS_FRIC_COMBINE,
            friction_combine_tag,
        ),
    ] {
        yy = seg_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            label,
            group,
            ids,
            &COMBINE_LABELS,
            tag,
        );
    }
    yy
}

/// The mass-source rows: for a Dynamic body, the **Auto | Manual** toggle plus the
/// single live quantity row (Density in Auto, Mass in Manual); for any other kind, a
/// plain Density row.
///
/// Density and mass are the same quantity by two roads (`mass = density × area`), so
/// exactly one is ever live — showing both would be the "two doors to one quantity"
/// bug. The toggle is Dynamic-only because a Static/Kinematic body has infinite mass
/// (rapier ignores both); those keep the plain Density row, unchanged from before
/// this existed. `is_dynamic`/`mass_manual` are resolved by the caller so this file
/// needs none of `physics.rs`'s private tag consts.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_mass_source(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    is_dynamic: bool,
    mass_manual: bool,
) -> f32 {
    let mut yy = y;
    if is_dynamic {
        yy = seg_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            "Mass",
            ids::INSP_LIVE_PHYSICS_MASSMODE,
            &ids::INSP_PHYS_MASSMODE,
            &MASS_MODE_LABELS,
            u8::from(mass_manual),
        );
        let (label, id) = if mass_manual {
            ("Mass (kg)", ids::INSP_PHYS_MASS)
        } else {
            ("Density", ids::INSP_PHYS_DENSITY)
        };
        yy = num_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            label,
            id,
        );
    } else {
        yy = num_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            "Density",
            ids::INSP_PHYS_DENSITY,
        );
    }
    yy
}

/// Collision-layer chip labels. Bare numbers because a layer has no meaning of
/// its own — what it MEANS is the row it occupies in the world matrix, and that
/// is where the naming belongs. Naming them here would be a second place to
/// keep names in sync with a matrix that does not know about them.
const LAYER_LABELS: [&str; 8] = ["0", "1", "2", "3", "4", "5", "6", "7"];

/// Sensor toggle labels, indexed by `is_sensor as u8`: `0` a solid collider,
/// `1` a sensor (trigger).
const SENSOR_LABELS: [&str; 2] = ["Solid", "Sensor"];

/// One-way toggle labels, indexed by `one_way as u8`: `0` an ordinary solid collider,
/// `1` a jump-through platform (solid only from its local +Y side).
const ONEWAY_LABELS: [&str; 2] = ["Off", "On"];

/// The per-collider COLLISION rules: which layer it is on, whether it is solid or a
/// trigger, and then the one question that follows from THAT answer — a solid collider
/// asks *from which side* (one-way), a sensor asks *with what force* (the force zone).
///
/// ⚠️ **Those last two are mutually exclusive, and that is physics, not layout.** A
/// one-way platform is realised by modifying solver CONTACTS, and a sensor generates
/// none; a force zone is realised from the narrow phase's INTERSECTION graph, which
/// only records a pair when one side is a sensor. Each control is dead in the other
/// mode, so each is offered only in its own — the first §11 controls gated on another
/// CONTROL rather than on `kind_tag`.
///
/// **None is Dynamic-only:** the layer is a filter, a trigger is commonly Static
/// scenery, a jump-through platform is almost always Static and so is a wind column —
/// gating any of them on Dynamic would delete the control from its own use case. Split
/// here so `paint_physics_section` stays under the panel's 200-LOC fn cap; the
/// selections read straight off the snapshot, so only the force numbers are synced.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_collision_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    layer: u8,
    is_sensor: bool,
    one_way: bool,
) -> f32 {
    let mut yy = y;
    // The per-body half of collision layers. The other half — WHICH layers collide —
    // is a world rule and lives in the Physics panel; a body only says where it belongs.
    for (label, group, opts, labels, sel) in [
        (
            "Layer",
            ids::INSP_LIVE_PHYSICS_LAYER,
            &ids::INSP_PHYS_LAYER[..],
            &LAYER_LABELS[..],
            layer,
        ),
        (
            "Trigger",
            ids::INSP_LIVE_PHYSICS_SENSOR,
            &ids::INSP_PHYS_SENSOR[..],
            &SENSOR_LABELS[..],
            u8::from(is_sensor),
        ),
    ] {
        yy = seg_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            label,
            group,
            opts,
            labels,
            sel,
        );
    }
    if is_sensor {
        // A SENSOR: what force does this area apply to whatever is inside it? Wind,
        // an updraft, a conveyor. Newtons, so it is resisted by mass — the number an
        // artist tunes against a body's own weight.
        // Force is what the area PUSHES with; Drag is what it RESISTS with. Together
        // they are the difference between wind (push, no resistance) and water.
        for (label, id) in [
            ("Force X (N)", ids::INSP_PHYS_FORCE_X),
            ("Force Y (N)", ids::INSP_PHYS_FORCE_Y),
            ("Torque (N·m)", ids::INSP_PHYS_AREA_TORQUE),
            ("Drag", ids::INSP_PHYS_AREA_DRAG),
            ("Fluid Density", ids::INSP_PHYS_AREA_DENSITY),
            ("Shape Drag", ids::INSP_PHYS_AREA_FORM_DRAG),
        ] {
            yy = num_row(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                x,
                w,
                yy,
                label,
                id,
            );
        }
    } else {
        // A SOLID collider: which side is it solid from?
        yy = seg_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            "One-Way",
            ids::INSP_LIVE_PHYSICS_ONEWAY,
            &ids::INSP_PHYS_ONEWAY,
            &ONEWAY_LABELS,
            u8::from(one_way),
        );
    }
    yy
}
