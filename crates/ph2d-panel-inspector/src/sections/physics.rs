//! Physics Body — Inspector §11 section painter (ADR-0130 D8).
//!
//! **This section has two faces, and the empty one is the important one.**
//! Every other section describes something the entity already has. Before
//! this existed, a `RigidBody` could only come from a smoke scene — there was
//! no gesture anywhere in the editor that made a sprite physical. So on an
//! entity with no body the section is a single **Add Physics Body** button;
//! that button is the entire door into the physics engine.
//!
//! Snapshot-driven like §10: the panel crate never sees `ph2d-physics-ecs`,
//! only resolved tags and floats.

use super::*;
use ph2d_editor_core::screens::hero::InspectorPhysicsInfo;
use ph2d_editor_core::widget::{SegmentedAdaptive, SegmentedOption, paint_segmented_adaptive};

/// Body-kind labels, indexed by `BodyKind` tag. Hardcoded (not read from
/// `ph2d-physics-ecs`) so the panel stays loose-coupled — the snapshot
/// carries only the tag. English per HR-15, like every sibling section.
const KIND_LABELS: [&str; 2] = ["Dynamic", "Static"];

/// Collider-shape labels, indexed by `ColliderShape` tag.
const SHAPE_LABELS: [&str; 2] = ["Ball", "Box"];

/// Tag for the Ball shape — named because the painter branches on it twice
/// and a bare `0` at a branch is the kind of thing that survives a refactor
/// pointing at the wrong variant.
const SHAPE_BALL: u8 = 0;

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_physics_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorPhysicsInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    let collapsed = store.is_collapsed(ids::INSP_LIVE_PHYSICS_SECTION);
    let color_id = ids::INSP_LIVE_PHYSICS_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: neutral default section accent
    let header = SectionHeader::new(ids::INSP_LIVE_PHYSICS_SECTION, "Physics Body")
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

    if !info.has_body {
        // The door. One line of explanation, because "Add Physics Body" on a
        // sprite that is about to start falling deserves to say so.
        paint_text(
            text_system,
            scene,
            "Not simulated \u{00b7} add a body to make it fall and collide",
            x,
            yy + (h - label_font) * 0.5,
            label_font,
            w,
            resolve(ColorToken::Text3, theme),
        );
        yy += h;
        let btn_rect = Rect::new(x, yy, w, h);
        let btn = Button::new(ids::INSP_PHYS_ADD, "Add Physics Body")
            .kind(ButtonKind::Default)
            .state(
                store
                    .button_state(ids::INSP_PHYS_ADD)
                    .unwrap_or(ButtonState::Normal),
            );
        paint_button(&btn, btn_rect, scene, text_system, theme);
        hit_index.register(ids::INSP_PHYS_ADD, btn_rect);
        return yy + h + SECTION_BOTTOM_PAD_PX;
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
        "Body",
        ids::INSP_LIVE_PHYSICS_SECTION,
        &ids::INSP_PHYS_KIND,
        &KIND_LABELS,
        info.kind_tag,
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
        "Collider",
        ids::INSP_LIVE_PHYSICS_COLOR,
        &ids::INSP_PHYS_SHAPE,
        &SHAPE_LABELS,
        info.shape_tag,
    );

    // Only the selected shape's dimensions are offered. A radius field on a
    // box is a control that cannot do anything — worse than a missing one,
    // because it looks like it should work.
    if info.shape_tag == SHAPE_BALL {
        yy = num_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            "Radius (m)",
            ids::INSP_PHYS_RADIUS,
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
            "Half Width (m)",
            ids::INSP_PHYS_HALF_X,
        );
        yy = num_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            "Half Height (m)",
            ids::INSP_PHYS_HALF_Y,
        );
    }

    for (label, id) in [
        ("Density", ids::INSP_PHYS_DENSITY),
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

    let btn_rect = Rect::new(x, yy, w, h);
    let btn = Button::new(ids::INSP_PHYS_REMOVE, "Remove Physics Body")
        .kind(ButtonKind::Default)
        .state(
            store
                .button_state(ids::INSP_PHYS_REMOVE)
                .unwrap_or(ButtonState::Normal),
        );
    paint_button(&btn, btn_rect, scene, text_system, theme);
    hit_index.register(ids::INSP_PHYS_REMOVE, btn_rect);
    yy + h + SECTION_BOTTOM_PAD_PX
}

/// A labelled segmented control. Returns the next `y`.
#[allow(clippy::too_many_arguments)]
fn seg_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    group_id: NodeId,
    option_ids: &[NodeId],
    labels: &[&str],
    selected: u8,
) -> f32 {
    let label_font = TypeToken::Sm.px();
    let label_h = label_font + Spacing::Xs.px();
    paint_text(
        text_system,
        scene,
        label,
        x,
        y + (label_h - label_font) * 0.5,
        label_font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    let seg = SegmentedAdaptive::new(
        group_id,
        label,
        option_ids
            .iter()
            .zip(labels)
            .map(|(&id, &l)| SegmentedOption::new(id, l))
            .collect(),
    )
    .selected((selected as usize).min(labels.len() - 1));
    let seg_h = paint_segmented_adaptive(
        &seg,
        Rect::new(x, y + label_h, w, ROW_H_PX),
        scene,
        text_system,
        theme,
        store,
        hit_index,
    );
    y + label_h + seg_h + Spacing::Sm.px()
}

/// A labelled number box, seeded from the store (which `sync` mirrors from
/// the snapshot). Returns the next `y`.
#[allow(clippy::too_many_arguments)]
fn num_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    id: NodeId,
) -> f32 {
    let label_font = TypeToken::Sm.px();
    let label_h = label_font + Spacing::Xs.px();
    paint_text(
        text_system,
        scene,
        label,
        x,
        y + (label_h - label_font) * 0.5,
        label_font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    let row_y = y + label_h;
    let rect = Rect::new(x, row_y, w, ROW_H_PX);
    hit_index.register(id, rect);
    let (state, value, buffer, caret, anchor) = read_number_input(store, id);
    let input = NumberInput::new(id, "", value).state(state);
    paint_number_input_with_buffer(
        &input,
        Some(buffer),
        caret,
        anchor,
        rect,
        scene,
        text_system,
        theme,
    );
    row_y + ROW_H_PX + Spacing::Sm.px()
}
