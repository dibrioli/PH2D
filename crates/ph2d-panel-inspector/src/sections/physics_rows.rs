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
