//! Physics Body — Inspector §11 section painter (ADR-0131 D8).
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

use super::rows::{num_row, seg_row};
use super::*;
use ph2d_editor_core::screens::hero::InspectorPhysicsInfo;

/// Body-kind labels, indexed by `BodyKind` tag. Hardcoded (not read from
/// `ph2d-physics-ecs`) so the panel stays loose-coupled — the snapshot
/// carries only the tag. English per HR-15, like every sibling section.
const KIND_LABELS: [&str; 3] = ["Dynamic", "Static", "Kinematic"];

/// Collider-shape labels, indexed by `ColliderShape` tag.
const SHAPE_LABELS: [&str; 2] = ["Ball", "Box"];

/// Collision-layer chip labels. Bare numbers because a layer has no meaning of
/// its own — what it MEANS is the row it occupies in the world matrix, and that
/// is where the naming belongs. Naming them here would be a second place to
/// keep names in sync with a matrix that does not know about them.
const LAYER_LABELS: [&str; 8] = ["0", "1", "2", "3", "4", "5", "6", "7"];

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

    // The per-body half of collision layers. The other half — WHICH layers
    // collide — is a world rule and lives in the Physics panel; a body only
    // says where it belongs.
    yy = seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        "Layer",
        ids::INSP_LIVE_PHYSICS_LAYER,
        &ids::INSP_PHYS_LAYER,
        &LAYER_LABELS,
        info.layer,
    );

    paint_body_actions(scene, text_system, theme, hit_index, store, x, w, yy, info)
}

/// The three things you can DO to a body, under its fields: join it to another,
/// bake its motion into curves, or take the body away.
///
/// Its own function because the section is at the panel crate's 200-LOC cap and
/// these are the part that is a list rather than a form — each is one button,
/// offered or not, and none of them reads a field above.
#[allow(clippy::too_many_arguments)]
fn paint_body_actions(
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
    let h = ROW_H_PX;
    let mut yy = y;
    let mut action = |id, label: &str, yy: &mut f32| {
        let rect = Rect::new(x, *yy, w, h);
        let btn = Button::new(id, label)
            .kind(ButtonKind::Default)
            .state(store.button_state(id).unwrap_or(ButtonState::Normal));
        paint_button(&btn, rect, scene, text_system, theme);
        hit_index.register(id, rect);
        *yy += h + Spacing::Sm.px();
    };

    // The creation gesture for a joint (W3). It lives HERE, in the body
    // section, because a joint does not exist yet when you want to make one —
    // the button has to be somewhere you already are, looking at the two
    // bodies you have selected. Offered only when the selection is exactly two
    // bodies, which is a fact only the shell can know.
    if info.can_join {
        action(ids::INSP_PHYS_JOIN, "Join Selected Bodies", &mut yy);
    }

    // Bake (W4): the simulated motion becomes timeline curves, and the body is
    // handed over to the scene (`BodyKind::Kinematic`). The label carries the
    // resolved range because the range is otherwise invisible — the artist
    // would have to press it to find out how much they were baking.
    action(
        ids::INSP_PHYS_BAKE,
        &format!("Bake {:.1}s to Timeline", info.bake_seconds),
        &mut yy,
    );

    action(ids::INSP_PHYS_REMOVE, "Remove Physics Body", &mut yy);
    yy - Spacing::Sm.px() + SECTION_BOTTOM_PAD_PX
}
