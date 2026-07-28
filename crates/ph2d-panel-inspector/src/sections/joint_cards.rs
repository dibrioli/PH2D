//! **Os dois CARDS da §12** — o Motor e o Break.
//!
//! Irmão de `joint.rs`, separado dele quando os dois juntos passaram do cap de
//! 600 LOC do painel, e o corte é por ASSUNTO: aqui *o que um motor é* e *o que
//! romper é*, lá *quais linhas este tipo tem*. Cada card é um subject que vários
//! tipos compartilham, e é isso que os torna uma unidade.

use super::rows::{num_row, seg_row};
use super::*;
use ph2d_editor_core::screens::hero::InspectorJointInfo;

/// **The Motor card** — offered to every driven kind (`kind_has_motor`).
///
/// Three questions, in the order the artist asks them: *is it driven?*, *is it
/// chasing a rate or a place?*, and then the one number that mode needs plus the
/// force ceiling. The mode chips are not a second on/off switch: Velocity and
/// Position carry genuinely different instructions, and each shows its OWN row —
/// painting both Speed and Target at once would be two numbers where only one is
/// read, which is the same knob-that-does-nothing this section refuses elsewhere.
///
/// Its own fn for the 200-LOC panel-fn cap, and because "what a motor is" is one
/// subject that three kinds now share.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_motor_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorJointInfo,
) -> f32 {
    let mut yy = seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        "Motor",
        ids::INSP_JOINT_MOTOR_GROUP,
        &ids::INSP_JOINT_MOTOR,
        &SWITCH_LABELS,
        u8::from(info.motor_enabled),
    );
    if !info.motor_enabled {
        return yy;
    }
    yy = seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        "Mode",
        ids::INSP_JOINT_MOTOR_MODE_GROUP,
        &ids::INSP_JOINT_MOTOR_MODE,
        &MOTOR_MODE_LABELS,
        info.motor_mode_tag,
    );
    let (rate_unit, place_unit) = motor_units(info.kind_tag);
    let (label, id) = if info.motor_mode_tag == MOTOR_MODE_POSITION {
        (
            format!("Target ({place_unit})"),
            ids::INSP_JOINT_MOTOR_TARGET,
        )
    } else {
        (format!("Speed ({rate_unit})"), ids::INSP_JOINT_MOTOR_SPEED)
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
        &label,
        id,
    );
    num_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        "Max Force",
        ids::INSP_JOINT_MOTOR_FORCE,
    )
}

/// **The Breakable card** (W-J7) — the switch, then the thresholds it gates.
///
/// ⚠️ **A POLIA era a exceção, e deixou de ser** (W-Pulley W2): ela não vive no
/// `ImpulseJointSet`, então não havia reação publicada a comparar e a caixa
/// seria um limiar que nunca podia ser cruzado. Hoje o passe dela publica a
/// própria tensão (`λ/dt`, medida contra `m·g` com razão 0,9999), e a frase
/// abaixo voltou a valer para todos os tipos.
///
/// Offered for every OTHER kind, unlike the Motor card: any joint can be pulled apart,
/// and the reaction that decides it (the linear one) is reported exactly on all
/// five (measured — a hanging weight reads its own weight to 1.0000).
///
/// ⚠️ **The TORQUE row is a Pin's alone**, and the reason is a measurement, not a
/// preference: rapier publishes the reaction of a limited or motorised angular
/// axis and *nothing* for a locked one, so a Weld cantilever holding 4.905 N·m
/// reads `0.0000`. Painting the row there would be a threshold that can never be
/// crossed — a control in name only, which is the failure this section's
/// per-kind rule exists to prevent. The snapshot answers with
/// `breaks_on_torque`; a seam gate walks all five kinds and pins which one
/// offers it.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_break_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorJointInfo,
) -> f32 {
    let mut yy = seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        "Breakable",
        ids::INSP_JOINT_BREAK_GROUP,
        &ids::INSP_JOINT_BREAK,
        &SWITCH_LABELS,
        u8::from(info.break_enabled),
    );
    if !info.break_enabled {
        return yy;
    }
    yy = num_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        "Break Force (N)",
        ids::INSP_JOINT_BREAK_FORCE,
    );
    if info.breaks_on_torque {
        yy = num_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            "Break Torque (N.m)",
            ids::INSP_JOINT_BREAK_TORQUE,
        );
    }
    yy
}
