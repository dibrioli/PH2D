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
