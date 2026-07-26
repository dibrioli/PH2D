//! Physics Joint — Inspector §12 section painter (W3).
//!
//! A joint is an **entity**, so this section describes the selected joint
//! object rather than a property of a body. It appears when — and only when —
//! the selection carries a `PhysicsJoint`.
//!
//! **Only the chosen kind's parameters are painted.** A stiffness field on a
//! rope is a control that cannot do anything, which is worse than a missing
//! one because it looks like it should work — the same rule §11 already
//! follows for a radius on a box. The question *"does this kind have a
//! motor?"* is answered by `JointKind::is_hinge` in `ph2d-physics-ecs`, and
//! the bridge asks the SAME function before handing a motor to the solver, so
//! a knob cannot be painted for a kind that ignores it.

use super::rows::{num_row, seg_row};
use super::*;
use ph2d_editor_core::screens::hero::InspectorJointInfo;

/// Joint-kind labels, indexed by the tag the snapshot carries. Hardcoded here
/// (not read from `ph2d-physics-ecs`) so the panel stays loose-coupled, like
/// every sibling section. English per HR-15.
const KIND_LABELS: [&str; 5] = ["Pin", "Spring", "Rope", "Weld", "Slider"];

/// The two Pin-only switches. A two-option segmented IS a switch, and it is
/// the widget this section already speaks.
///
/// `pub(super)` because the pair cluster next door speaks the same two words —
/// one list, so an "On"/"Enabled" drift between two halves of one section is not
/// a thing that can happen.
pub(super) const SWITCH_LABELS: [&str; 2] = ["Off", "On"];

/// Tag of the Pin kind — named because the painter branches on it and a bare
/// `0` at a branch survives a refactor pointing at the wrong variant.
const KIND_PIN: u8 = 0;
/// Tag of the Spring kind.
const KIND_SPRING: u8 = 1;
/// Tag of the Rope kind. Named so the Rope branch is explicit and a Weld
/// (tag 3) — which has no parameter rows at all — falls through to nothing
/// instead of inheriting the Rope's "Max Length" from a bare `else`.
const KIND_ROPE: u8 = 2;
/// Tag of the Slider kind. It shares the **Limits** switch with the Pin (both
/// have a range) and, since W-J6, a motor as well — so the painter asks the two
/// questions separately instead of branching on "is it a Pin?", which is the
/// same split `JointKind::has_limits` and `has_motor` made engine-side.
const KIND_SLIDER: u8 = 4;

/// Can this kind be DRIVEN? A Pin's hinge, a Slider's rail, a Rope's distance
/// (the winch). A Spring and a Weld cannot.
///
/// ⚠️ Second STATEMENT of `JointKind::has_motor`, not a second source of truth —
/// the panel is loose-coupled and never sees `ph2d-physics-ecs` (the convention
/// of every sibling section). The bridge asks the engine-side door before handing
/// a motor to the solver, so a kind that gained a card here without gaining one
/// there would paint a knob the solver drops; a seam gate walks all five kinds
/// and pins which ones offer the card.
const fn kind_has_motor(kind_tag: u8) -> bool {
    kind_tag == KIND_PIN || kind_tag == KIND_SLIDER || kind_tag == KIND_ROPE
}

/// The unit pair the motor rows are labelled with, **for this kind**:
/// `(rate, place)`. Degrees for a hinge, metres for a rail or a winch.
///
/// ⚠️ **Deliberately not `limit_unit`, and the Rope is why:** a Rope has no limit
/// range at all and still has a linear motor, so one function answering both
/// questions would label a winch's target in degrees. Engine-side the same two
/// doors are `limits_in_metres` and `motor_in_metres`.
const fn motor_units(kind_tag: u8) -> (&'static str, &'static str) {
    if kind_tag == KIND_PIN {
        ("\u{00b0}/s", "\u{00b0}")
    } else {
        ("m/s", "m")
    }
}

/// Velocity · Position — the two things a motor can be told.
const MOTOR_MODE_LABELS: [&str; 2] = ["Velocity", "Position"];
/// Tag of the Position (servo) mode, named because the painter branches on it.
const MOTOR_MODE_POSITION: u8 = 1;

/// The unit the limit rows are in, **for this kind**. Degrees for a hinge's
/// angular range, metres for a slider's stroke.
///
/// ⚠️ The panel is loose-coupled and hardcodes its own labels (the convention of
/// every sibling section), so this is a second STATEMENT of
/// `JointKind::limits_in_metres` rather than a second source of truth: the
/// shell converts the value, this only names it. A seam gate pins that a slider's
/// rows say metres, so the two cannot drift apart in silence.
const fn limit_unit(kind_tag: u8) -> &'static str {
    if kind_tag == KIND_SLIDER {
        "m"
    } else {
        "\u{00b0}"
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_joint_section(
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
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    let collapsed = store.is_collapsed(ids::INSP_LIVE_JOINT_SECTION);
    let color_id = ids::INSP_LIVE_JOINT_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: neutral default section accent
    let header = SectionHeader::new(ids::INSP_LIVE_JOINT_SECTION, "Physics Joint")
        .collapsible(!collapsed)
        .color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle_rect) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect)
    {
        hit_index.register(color_id, circle_rect);
    }
    if collapsed {
        return y + header_h;
    }

    let mut yy = y + header_h;
    let h = ROW_H_PX;

    // **Active comes FIRST** (W-J8) — it qualifies everything below it. The rows
    // stay painted and stay editable while it is off: an inactive joint is one
    // you are still authoring, which is the whole difference from a deleted one.
    yy = seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        "Active",
        ids::INSP_JOINT_ACTIVE_GROUP,
        &ids::INSP_JOINT_ACTIVE,
        &SWITCH_LABELS,
        u8::from(info.active),
    );

    // The PAIR cluster: who the two ends are, the gesture that exchanges them,
    // and the one fact that is about the pair rather than about the constraint.
    // Its own module (`joint_pair_rows`) — the same cut the section draws on
    // screen, and the one the 600-LOC panel cap asked for.
    yy = super::joint_pair_rows::paint_pair_rows(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        info,
    );

    yy = seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        "Kind",
        ids::INSP_JOINT_KIND_GROUP,
        &ids::INSP_JOINT_KIND,
        &KIND_LABELS,
        info.kind_tag,
    );

    if info.kind_tag == KIND_PIN || info.kind_tag == KIND_SLIDER {
        yy = seg_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            "Limits",
            ids::INSP_JOINT_LIMITS_GROUP,
            &ids::INSP_JOINT_LIMITS,
            &SWITCH_LABELS,
            u8::from(info.limits_enabled),
        );
        if info.limits_enabled {
            let unit = limit_unit(info.kind_tag);
            for (label, id) in [
                (format!("Min ({unit})"), ids::INSP_JOINT_LIMIT_MIN),
                (format!("Max ({unit})"), ids::INSP_JOINT_LIMIT_MAX),
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
                    &label,
                    id,
                );
            }
        }
    } else if info.kind_tag == KIND_SPRING {
        for (label, id) in [
            ("Rest Length (m)", ids::INSP_JOINT_REST_LENGTH),
            ("Stiffness", ids::INSP_JOINT_STIFFNESS),
            ("Damping", ids::INSP_JOINT_DAMPING),
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
    } else if info.kind_tag == KIND_ROPE {
        yy = num_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            "Max Length (m)",
            ids::INSP_JOINT_MAX_LENGTH,
        );
    }

    // The motor comes LAST and is asked of every driven kind, rather than living
    // inside the Pin's branch as it did until W-J6: a rail and a winch are driven
    // too, and burying the card in one kind's arm is how the other two would have
    // been given a knob the solver ignores.
    if kind_has_motor(info.kind_tag) {
        yy = paint_motor_rows(scene, text_system, theme, hit_index, store, x, w, yy, info);
    }

    // Breaking comes after the parameters and before Delete: it is a property of
    // the joint as a whole (every kind can be pulled apart), not of one kind's
    // degree of freedom, so it is asked of ALL five.
    yy = paint_break_rows(scene, text_system, theme, hit_index, store, x, w, yy, info);

    let btn_rect = Rect::new(x, yy, w, h);
    let btn = Button::new(ids::INSP_JOINT_REMOVE, "Delete Joint")
        .kind(ButtonKind::Default)
        .state(
            store
                .button_state(ids::INSP_JOINT_REMOVE)
                .unwrap_or(ButtonState::Normal),
        );
    paint_button(&btn, btn_rect, scene, text_system, theme);
    hit_index.register(ids::INSP_JOINT_REMOVE, btn_rect);
    yy + h + SECTION_BOTTOM_PAD_PX
}

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
fn paint_motor_rows(
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
/// Offered for EVERY kind, unlike the Motor card: any joint can be pulled apart,
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
fn paint_break_rows(
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

#[cfg(test)]
mod kind_chip_tests {
    use super::{KIND_LABELS, KIND_SLIDER, limit_unit};
    use ph2d_editor_core::ids;

    /// **Um rótulo por id, e a razão é um `zip` que TRUNCA.**
    ///
    /// O `seg_row` casa `option_ids.zip(labels)`, então um rótulo sem id **não é
    /// pintado** — sem erro, sem warning, e o chip nasce inalcançável. Foi
    /// exatamente o que aconteceu quando o Slider chegou: cinco rótulos, quatro
    /// ids, e o gate de seam dos chips ficou verde porque ele iterava a lista
    /// CURTA (os ids). Comparar os dois comprimentos é a asserção que nenhuma
    /// das duas listas pode satisfazer sozinha.
    ///
    /// ⚠️ **E o par existe DUAS vezes:** este (§12, o tipo que a joint É) e o
    /// *Join As* do §11 (o tipo que o próximo gesto CRIA), com gate irmão em
    /// `sections::physics_rows`. Foi escrever só ESTE que deixou o chip do Slider
    /// faltar no seletor de criação — o artista via o tipo na simulação e não
    /// conseguia escolhê-lo.
    #[test]
    fn every_kind_label_has_an_id_to_be_clicked_by() {
        assert_eq!(
            KIND_LABELS.len(),
            ids::INSP_JOINT_KIND.len(),
            "um rotulo sem id e um chip que o seg_row DESCARTA no zip"
        );
    }

    /// O rótulo do curso diz **metros** para um trilho e **graus** para o resto —
    /// a segunda metade da porta `JointKind::limits_in_metres` (a primeira
    /// converte o número; esta o nomeia).
    #[test]
    fn a_rails_range_is_named_in_metres() {
        assert_eq!(limit_unit(KIND_SLIDER), "m");
        for other in [0u8, 1, 2, 3] {
            assert_eq!(limit_unit(other), "\u{00b0}");
        }
    }
}
