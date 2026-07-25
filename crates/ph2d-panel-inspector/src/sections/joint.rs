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
const KIND_LABELS: [&str; 4] = ["Pin", "Spring", "Rope", "Weld"];

/// The two Pin-only switches. A two-option segmented IS a switch, and it is
/// the widget this section already speaks.
const SWITCH_LABELS: [&str; 2] = ["Off", "On"];

/// Tag of the Pin kind — named because the painter branches on it and a bare
/// `0` at a branch survives a refactor pointing at the wrong variant.
const KIND_PIN: u8 = 0;
/// Tag of the Spring kind.
const KIND_SPRING: u8 = 1;
/// Tag of the Rope kind. Named so the Rope branch is explicit and a Weld
/// (tag 3) — which has no parameter rows at all — falls through to nothing
/// instead of inheriting the Rope's "Max Length" from a bare `else`.
const KIND_ROPE: u8 = 2;

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
    let label_font = TypeToken::Sm.px();

    // Which two bodies, by name — and whether they are actually there. A joint
    // whose body was renamed is dormant, and saying so is the difference
    // between "this is off" and "this is broken".
    let line = if info.bound {
        format!("{} \u{2194} {}", info.body_a_name, info.body_b_name)
    } else {
        format!(
            "{} \u{2194} {} \u{00b7} not connected",
            display_name(&info.body_a_name),
            display_name(&info.body_b_name)
        )
    };
    paint_text(
        text_system,
        scene,
        &line,
        x,
        yy + (h - label_font) * 0.5,
        label_font,
        w,
        resolve(
            if info.bound {
                ColorToken::Text2
            } else {
                ColorToken::Text3
            },
            theme,
        ),
    );
    yy += h;

    // Re-pick which body each slot binds. Dimmed until a single OTHER body is
    // selected alongside the joint — the shell owns that fact
    // (`rebind_target_name`), the same shell-owned-selection idiom the Join
    // button uses. When live the button shows the target's name, so the artist
    // sees what the click will bind before pressing it. Fixes a mis-joined pair
    // without deleting and re-creating the joint. A Disabled button registers no
    // hit — a dimmed control that dispatches lies.
    for (id, slot) in [(ids::INSP_JOINT_SET_A, 'A'), (ids::INSP_JOINT_SET_B, 'B')] {
        let brect = Rect::new(x, yy, w, h);
        let label = match &info.rebind_target_name {
            Some(name) => format!("Set Body {slot}: {name}"),
            None => format!("Set Body {slot}"),
        };
        let state = if info.rebind_target_name.is_some() {
            store.button_state(id).unwrap_or(ButtonState::Normal)
        } else {
            ButtonState::Disabled
        };
        let btn = Button::new(id, &label)
            .kind(ButtonKind::Default)
            .state(state);
        paint_button(&btn, brect, scene, text_system, theme);
        if info.rebind_target_name.is_some() {
            hit_index.register(id, brect);
        }
        yy += h;
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
        "Kind",
        ids::INSP_JOINT_KIND_GROUP,
        &ids::INSP_JOINT_KIND,
        &KIND_LABELS,
        info.kind_tag,
    );

    if info.kind_tag == KIND_PIN {
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
            for (label, id) in [
                ("Min (\u{00b0})", ids::INSP_JOINT_LIMIT_MIN),
                ("Max (\u{00b0})", ids::INSP_JOINT_LIMIT_MAX),
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
        yy = seg_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            "Motor",
            ids::INSP_JOINT_MOTOR_GROUP,
            &ids::INSP_JOINT_MOTOR,
            &SWITCH_LABELS,
            u8::from(info.motor_enabled),
        );
        if info.motor_enabled {
            for (label, id) in [
                ("Speed (\u{00b0}/s)", ids::INSP_JOINT_MOTOR_SPEED),
                ("Max Force", ids::INSP_JOINT_MOTOR_FORCE),
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

/// A body name for display, with a stand-in when it could not be resolved.
/// The joint stores a hash, and a hash is not something to show a person —
/// but neither is an empty gap where a name should be.
fn display_name(name: &str) -> &str {
    if name.is_empty() { "(missing)" } else { name }
}
