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
const SHAPE_LABELS: [&str; 3] = ["Ball", "Box", "Capsule"];

/// Collision-layer chip labels. Bare numbers because a layer has no meaning of
/// its own — what it MEANS is the row it occupies in the world matrix, and that
/// is where the naming belongs. Naming them here would be a second place to
/// keep names in sync with a matrix that does not know about them.
const LAYER_LABELS: [&str; 8] = ["0", "1", "2", "3", "4", "5", "6", "7"];

/// Sensor toggle labels, indexed by `is_sensor as u8`: `0` a solid collider,
/// `1` a sensor (trigger).
const SENSOR_LABELS: [&str; 2] = ["Solid", "Sensor"];

/// CCD toggle labels, indexed by `ccd as u8`: `0` discrete (rapier's default),
/// `1` continuous (sweeps the motion so a fast body does not tunnel). Unity's own
/// vocabulary for the same control.
const CCD_LABELS: [&str; 2] = ["Discrete", "Continuous"];

/// Lock-rotation toggle labels, indexed by `lock_rotation as u8`: `0` free
/// (rapier's default), `1` locked (the orientation is pinned — Freeze Rotation).
const LOCKROT_LABELS: [&str; 2] = ["Free", "Locked"];

/// Freeze-Position (X and Y) toggle labels, indexed by the flag: `0` free
/// (rapier's default), `1` locked (that translation axis is pinned).
const FREEZE_POS_LABELS: [&str; 2] = ["Free", "Locked"];

/// Bake channel selector labels, indexed by the tag: `0` the whole pose,
/// `1` position only, `2` rotation only.
const BAKE_CH_LABELS: [&str; 3] = ["All", "Position", "Rotation"];

/// The Bake button's label, carrying the range it would cover.
///
/// A function rather than an inline `format!` so the claim "the button shows
/// the range" is something a test can hold: the number is the only thing
/// telling the artist how much of the timeline they are about to write, and a
/// button that silently baked five seconds when the document said two would be
/// worse than one that asked.
pub fn bake_label(seconds: f32) -> String {
    format!("Bake {seconds:.1}s to Timeline")
}

/// Tag for the Dynamic body kind — the only kind with simulated motion to bake.
const KIND_DYNAMIC: u8 = 0;

/// Tag for the Ball shape — named because the painter branches on it and a
/// bare `0` at a branch is the kind of thing that survives a refactor pointing
/// at the wrong variant.
const SHAPE_BALL: u8 = 0;

/// Tag for the Capsule shape (`ColliderShape::Capsule`), named for the same
/// reason as [`SHAPE_BALL`].
const SHAPE_CAPSULE: u8 = 2;

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

    yy = paint_shape_dims(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        info.shape_tag,
    );

    // Collider offset from the sprite centre — a collider-geometry property (not
    // Dynamic-only), so it sits with the shape dimensions rather than the dynamics
    // block. The overlay draws the outline here so the offset is visible.
    for (label, id) in [
        ("Offset X (m)", ids::INSP_PHYS_OFFSET_X),
        ("Offset Y (m)", ids::INSP_PHYS_OFFSET_Y),
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

    // Mass source (W-Mass). A Dynamic body's mass is either density-derived (Auto,
    // mass = density × area) or an explicit override (Manual, kg) — the same quantity
    // by two roads, so the Auto|Manual toggle picks which single row is live (Density
    // vs Mass), never both. A Static/Kinematic body has infinite mass (rapier ignores
    // both), so it keeps the plain Density row without the toggle — a control that
    // cannot do anything is worse than a missing one, but this preserves today's UI
    // for those kinds rather than dropping a row.
    yy = super::physics_rows::paint_mass_source(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        info.kind_tag == KIND_DYNAMIC,
        info.mass_manual,
    );

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

    // Solid vs sensor. A sensor passes through and only reports overlaps, so it
    // is a property of the collider, not a body kind — a trigger can be static,
    // dynamic or kinematic. The overlay lights it up when a body is inside.
    yy = seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        "Trigger",
        ids::INSP_LIVE_PHYSICS_SENSOR,
        &ids::INSP_PHYS_SENSOR,
        &SENSOR_LABELS,
        u8::from(info.is_sensor),
    );

    paint_body_actions(scene, text_system, theme, hit_index, store, x, w, yy, info)
}

/// The dimension rows of the SELECTED shape, and only those.
///
/// A radius field on a box is a control that cannot do anything — worse than a
/// missing one, because it looks like it should work. Its own function because
/// the section is at the panel crate's 200-LOC cap, and because "which numbers
/// does this shape have" is one question that deserves one answer in one place.
///
/// The capsule reuses **Radius** deliberately: a cap's radius is the same
/// quantity under the same name. Its second row is a DIFFERENT id from the box's
/// `HALF_Y` because "half height" means the half-extent on a box and the
/// straight segment on a capsule (the caps add `radius` on top) — one control
/// meaning two things is the bug this section keeps writing gates against.
#[allow(clippy::too_many_arguments)]
fn paint_shape_dims(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    shape_tag: u8,
) -> f32 {
    let rows: &[(&str, ph2d_editor_core::NodeId)] = match shape_tag {
        SHAPE_BALL => &[("Radius (m)", ids::INSP_PHYS_RADIUS)],
        SHAPE_CAPSULE => &[
            ("Radius (m)", ids::INSP_PHYS_RADIUS),
            ("Half Height (m)", ids::INSP_PHYS_CAP_HALF_H),
        ],
        _ => &[
            ("Half Width (m)", ids::INSP_PHYS_HALF_X),
            ("Half Height (m)", ids::INSP_PHYS_HALF_Y),
        ],
    };
    let mut yy = y;
    for (label, id) in rows {
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
            *id,
        );
    }
    yy
}

/// The **Dynamic-only** dynamics fields: the gravity multiplier and the authored
/// initial velocity (linear X/Y + angular).
///
/// All four are meaningless on a Static body (it never moves under forces) or a
/// Kinematic one (its motion is driven by curves), so the section offers them
/// only for a Dynamic body — a control that cannot do anything is worse than a
/// missing one, the rule this section keeps. Its own function so the caller stays
/// under the panel's 200-LOC cap, beside `paint_shape_dims` for the same reason.
#[allow(clippy::too_many_arguments)]
fn paint_dynamics_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let mut yy = y;
    for (label, id) in [
        ("Gravity Scale", ids::INSP_PHYS_GRAVITY_SCALE),
        ("Init Vel X (m/s)", ids::INSP_PHYS_LINVEL_X),
        ("Init Vel Y (m/s)", ids::INSP_PHYS_LINVEL_Y),
        ("Init Spin (deg/s)", ids::INSP_PHYS_ANGVEL),
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
    yy
}

/// The tail of the body section: the two **Dynamic-only** field rows (Gravity
/// Scale + the Bake channel selector) and then the three things you can DO to a
/// body — join it to another, bake its motion into curves, or take the body away.
///
/// Its own function because the section is at the panel crate's 200-LOC cap. The
/// Dynamic-only rows live here (rather than up in the form) both for that reason
/// and because they share the "offered only for a Dynamic body" gate with Bake;
/// each button below is offered or not, and none reads a field above.
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

    // Gravity Scale + the Bake channel selector — both Dynamic-only (rapier
    // applies gravity to Dynamic bodies only, and only a Dynamic body has
    // simulated motion to bake). Painted here, BEFORE the `paint_at` closure
    // below borrows `scene`/`text_system`, so they do not fight that borrow. A
    // gravity-scale field on a Static/Kinematic body would be a control that
    // cannot do anything — worse than a missing one (the section's own rule for
    // the radius-on-a-box case).
    if info.kind_tag == KIND_DYNAMIC {
        yy = paint_dynamics_rows(scene, text_system, theme, hit_index, store, x, w, yy);
        // Collision mode: a fast Dynamic body can tunnel through thin geometry
        // between two ticks; Continuous sweeps its motion instead. Dynamic-only,
        // like the rows above — a Static body never moves and a Kinematic one is
        // pose-driven, so neither tunnels under its own forces.
        yy = seg_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            "Collision",
            ids::INSP_LIVE_PHYSICS_CCD,
            &ids::INSP_PHYS_CCD,
            &CCD_LABELS,
            u8::from(info.ccd),
        );
        // Freeze Rotation: pin the orientation so the body slides/falls but never
        // tumbles. Dynamic-only, like the rows above — only a body the solver
        // rotates under forces has a rotation to freeze.
        yy = seg_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            "Rotation",
            ids::INSP_LIVE_PHYSICS_LOCKROT,
            &ids::INSP_PHYS_LOCKROT,
            &LOCKROT_LABELS,
            u8::from(info.lock_rotation),
        );
        // Freeze Position X / Y: pin a translation axis so the body is held to a
        // rail (X) or floats against gravity (Y). Dynamic-only, the same rule as the
        // constraints above — only a body the solver MOVES has a position to freeze.
        yy = seg_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            "Freeze X",
            ids::INSP_LIVE_PHYSICS_LOCKX,
            &ids::INSP_PHYS_LOCKX,
            &FREEZE_POS_LABELS,
            u8::from(info.lock_x),
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
            "Freeze Y",
            ids::INSP_LIVE_PHYSICS_LOCKY,
            &ids::INSP_PHYS_LOCKY,
            &FREEZE_POS_LABELS,
            u8::from(info.lock_y),
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
            "Bake",
            ids::INSP_PHYS_BAKE_CH_GROUP,
            &ids::INSP_PHYS_BAKE_CH,
            &BAKE_CH_LABELS,
            info.bake_channels_tag,
        );
    }

    let mut paint_at = |id, label: &str, yy: &mut f32| -> Rect {
        let rect = Rect::new(x, *yy, w, h);
        let btn = Button::new(id, label)
            .kind(ButtonKind::Default)
            .state(store.button_state(id).unwrap_or(ButtonState::Normal));
        paint_button(&btn, rect, scene, text_system, theme);
        *yy += h + Spacing::Sm.px();
        rect
    };

    // ⚠️ The `hit_index.register` calls below are spelled out with LITERAL ids,
    // one per button, and that is not verbosity — it is the only form
    // `architecture_panel_wiring_parity` can see. It collects
    // `.register(ids::<LITERAL>` and deliberately skips a variable first
    // argument, so folding these into the closure above (which was the first
    // shape of this refactor) silently deleted the parity coverage of all three
    // buttons: deleting Bake and Remove from `populate` then left the whole
    // panel suite AND the parity gate green, with two buttons painted,
    // hit-registered and dead under the mouse.

    // The creation gesture for a joint (W3). It lives HERE, in the body
    // section, because a joint does not exist yet when you want to make one —
    // the button has to be somewhere you already are, looking at the two
    // bodies you have selected. Offered only when the selection is exactly two
    // bodies, which is a fact only the shell can know.
    if info.can_join {
        let r = paint_at(ids::INSP_PHYS_JOIN, "Join Selected Bodies", &mut yy);
        hit_index.register(ids::INSP_PHYS_JOIN, r);
    }

    // Bake (W4): the simulated motion becomes timeline curves, and the body is
    // handed over to the scene (`BodyKind::Kinematic`). The label carries the
    // resolved range because the range is otherwise invisible — the artist
    // would have to press it to find out how much they were baking.
    //
    // Offered only for a body the SOLVER moves. A `Static` body never moves and
    // a `Kinematic` one is already driven by the scene, so for both the bake can
    // only ever report "nothing moved" — a button promising 5 seconds of work
    // that is impossible for the thing it is pointing at. The shell honours the
    // same rule (`event_physics.rs`), because a refusal in the paint loop is
    // not a refusal.
    if info.kind_tag == KIND_DYNAMIC {
        let r = paint_at(ids::INSP_PHYS_BAKE, &bake_label(info.bake_seconds), &mut yy);
        hit_index.register(ids::INSP_PHYS_BAKE, r);
    }

    let r = paint_at(ids::INSP_PHYS_REMOVE, "Remove Physics Body", &mut yy);
    hit_index.register(ids::INSP_PHYS_REMOVE, r);
    yy - Spacing::Sm.px() + SECTION_BOTTOM_PAD_PX
}
