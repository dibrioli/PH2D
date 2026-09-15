//! **O que uma ÁREA faz a quem está dentro dela** — o bloco de ZONA da §11 (força, eixos, torque,
//! falloff e arrasto), irmão do `physics_rows.rs`.
//!
//! ⚠️ **Corte por RESPONSABILIDADE** (2026-09-13): o `physics_rows.rs` passou o tecto de 600 LOC
//! quando as tabelas de rótulos viraram `TextKey` (uma entrada por linha). O doc do
//! [`paint_area_rows`] já desenhava a fronteira — as rows de lá dizem *como este COLLIDER participa
//! de uma colisão*, as daqui *o que esta ÁREA faz* —, e uma **peça** tem as primeiras e nenhuma
//! destas.

use super::rows::{num_row, num_row_unit, seg_row};
use super::*;
use ph2d_i18n::TextKey;
use ph2d_i18n::tr;

/// Force-frame labels, indexed by `world_axes as u8`: `0` the zone's own frame (turn
/// the sensor and the wind turns with it), `1` pinned to world axes (the zone turns,
/// the blow does not).
const FORCE_AXES_LABELS: [TextKey; 2] = [
    TextKey::new("panel.inspector.physics.zone"),
    TextKey::new("panel.inspector.physics.world"),
];

/// **O que esta ÁREA faz a quem está dentro dela** — o bloco de zona, irmão do
/// [`paint_collision_rows`] (W-PartFace).
///
/// Só faz sentido num collider **sensor**, e o chamador é quem decide isso: um
/// corpo sólido não tem zona, e uma **peça** não tem zona alguma (a ponte não lê
/// efetor nenhum de uma peça — ver o doc do irmão).
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_area_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    force_world_axes: bool,
) -> f32 {
    let mut yy = y;
    {
        // A SENSOR: what force does this area apply to whatever is inside it? Wind,
        // an updraft, a conveyor. Newtons, so it is resisted by mass — the number an
        // artist tunes against a body's own weight.
        // Force is what the area PUSHES with; Drag is what it RESISTS with. Together
        // they are the difference between wind (push, no resistance) and water.
        for (label, id, unit) in [
            (
                tr("panel.inspector.physics.force_x_n"),
                ids::INSP_PHYS_FORCE_X,
                Some(ph2d_editor_core::widget::Unit::Newtons),
            ),
            (
                tr("panel.inspector.physics.force_y_n"),
                ids::INSP_PHYS_FORCE_Y,
                Some(ph2d_editor_core::widget::Unit::Newtons),
            ),
        ] {
            yy = num_row_unit(
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
                unit,
                None,
            );
        }
        // In WHOSE axes are those two numbers? Directly under them, and deliberately
        // ABOVE everything else in this branch: it governs the FORCE and nothing else.
        // That is geometry rather than a scope someone chose — a 2D torque is a scalar
        // about Z and an in-plane rotation is about Z, so there is nothing to turn; drag
        // is isotropic; buoyancy measures its surface from GRAVITY (water is level even
        // in a tilted pool); and shape drag pushes along each edge normal of the BODY.
        // Painted below the others, the row would read as qualifying all of them.
        yy = seg_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            tr("panel.inspector.physics.force_axes"),
            ids::INSP_LIVE_PHYSICS_FORCE_AXES,
            &ids::INSP_PHYS_FORCE_AXES,
            &FORCE_AXES_LABELS.map(TextKey::tr),
            u8::from(force_world_axes),
        );
        // Then the PUSH block closes with the two rows that qualify it: the spin the area
        // imprints, and how much of both survives the trip to the edge.
        //
        // ⚠️ Falloff sits directly under Torque, and ABOVE Drag, because that is exactly
        // the boundary of what it weighs: the force and the torque — the two PUSHES —
        // and nothing below. Drag, Fluid Density and Shape Drag describe a MEDIUM, and a
        // medium does not thin out near its own edge (the water at the side of the pool
        // is just as wet). Painted below them the row would read as governing all six.
        for (label, id) in [
            (
                tr("panel.inspector.physics.torque_n_m"),
                ids::INSP_PHYS_AREA_TORQUE,
            ),
            (
                tr("panel.inspector.physics.falloff"),
                ids::INSP_PHYS_AREA_FALLOFF,
            ),
            (tr("panel.inspector.physics.drag"), ids::INSP_PHYS_AREA_DRAG),
            (
                tr("panel.inspector.physics.fluid_density"),
                ids::INSP_PHYS_AREA_DENSITY,
            ),
            (
                tr("panel.inspector.physics.shape_drag"),
                ids::INSP_PHYS_AREA_FORM_DRAG,
            ),
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
    }
    yy
}
