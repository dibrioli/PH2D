//! Showcase `paint_vector_section` painter.
//!
//! Extracted from monolithic `showcase.rs` in Wave 6+7 Phase 1.C.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_vector_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let (mut y, open) = paint_collapsible_header(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ids::INSP_SECTION_VECTOR,
        ids::INSP_SECTION_VECTOR_COLOR,
        "Vector",
        3,
    );
    if !open {
        return y;
    }
    let r = Rect::new(x, y, w, FIELD_H);
    // Pull each axis' live state from the store so focus/typing
    // actually drives the chip. Previously this rendered via the
    // static `paint_vector3_editor` which never reads the in-progress
    // edit buffer — clicking a chip "focused" it visually (border
    // accent) but every keystroke vanished because the painter kept
    // showing `input.value` regardless. See `docs/UI_Bugs/README.md`
    // §9.14.
    let (sx, vx, bx, cx, ax) = read_number_input(store, ids::INSP_SAMPLE_V3_X);
    let (sy, vy, by, cy, ay) = read_number_input(store, ids::INSP_SAMPLE_V3_Y);
    let (sz, vz, bz, cz, az) = read_number_input(store, ids::INSP_SAMPLE_V3_Z);
    let nx = NumberInput::new(ids::INSP_SAMPLE_V3_X, "X", vx).state(sx);
    let ny = NumberInput::new(ids::INSP_SAMPLE_V3_Y, "Y", vy).state(sy);
    let nz = NumberInput::new(ids::INSP_SAMPLE_V3_Z, "Z", vz).state(sz);
    let v3 = Vector3Editor::new(NodeId(0), "Position", nx, ny, nz);
    crate::widget::paint_vector3_editor_with_state(
        &v3,
        [Some(bx), Some(by), Some(bz)],
        [cx, cy, cz],
        [ax, ay, az],
        r,
        scene,
        text_system,
        theme,
    );
    let rects = v3.field_rects(r);
    for (id, fr) in [
        (ids::INSP_SAMPLE_V3_X, rects[0]),
        (ids::INSP_SAMPLE_V3_Y, rects[1]),
        (ids::INSP_SAMPLE_V3_Z, rects[2]),
    ] {
        hit_index.register(id, fr);
    }
    y += FIELD_H;
    y
}
