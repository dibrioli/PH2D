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

use super::rows::num_row;
use super::*;
use ph2d_editor_core::screens::hero::InspectorPhysicsInfo;

/// Collider-shape labels, indexed by `ColliderShape` tag.
///
/// ⚠️ `pub(super)` porque a face de PEÇA (W-PartFace) pinta o MESMO seletor: a
/// forma de uma peça é a mesma pergunta que a forma de um corpo, e uma segunda
/// tabela de rótulos divergiria no dia em que a quarta forma chegasse.
pub(super) const SHAPE_LABELS: [&str; 3] = ["Ball", "Box", "Capsule"];

/// The Bake button's label, carrying the window it would cover.
///
/// A function rather than an inline `format!` so the claim "the button shows
/// the range" is something a test can hold: the numbers are the only thing
/// telling the artist how much of the timeline they are about to write, and a
/// button that silently baked five seconds when the document said two would be
/// worse than one that asked.
///
/// When `start` is `0` (the common case — no loop, or a loop from the top) the
/// label collapses to the plain `Bake Ns` form. A positive start (an armed loop
/// like `[2s, 5s]`, W-BakeRange) shows the full window `Bake 2.0-5.0s`, because
/// a partial-range bake writes keys at those absolute times and the artist has
/// to know that before clicking.
pub fn bake_label(start: f32, end: f32) -> String {
    if start > 0.0 {
        format!("Bake {start:.1}-{end:.1}s to Timeline")
    } else {
        format!("Bake {end:.1}s to Timeline")
    }
}

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
    let header = section_header(store, ids::INSP_LIVE_PHYSICS_SECTION, "Physics Body").color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle_rect) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect)
    {
        hit_index.register(color_id, circle_rect);
    }
    if collapsed {
        return y + header_h;
    }

    let yy = y + header_h;

    // **TRÊS faces, não duas** (W-PartFace). Um `Collider` sem `RigidBody` é uma
    // PEÇA — mais uma forma do corpo ancestral (W-Compound) — e ela é simulada;
    // mandá-la para a face vazia era o §11 dizendo *"Not simulated"* sobre algo
    // que o solver está de fato integrando, com a forma autorada invisível e a
    // porta que a criou re-oferecida.
    if info.has_collider && !info.has_body {
        return super::physics_part::paint_part_face(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            info,
            x,
            w,
            yy,
        );
    }
    if !info.has_body {
        return super::physics_doors::paint_empty_face(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            info,
            x,
            w,
            yy,
        );
    }

    super::physics_body::paint_body_face(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        info,
        x,
        w,
        yy,
    )
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
pub(super) fn paint_shape_dims(
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
