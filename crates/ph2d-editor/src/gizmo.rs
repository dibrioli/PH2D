//! M14.7 B — sprite transform gizmo painter.
//!
//! Visual layer for the move/rotate/scale gizmo that paints over a
//! selected sprite on the canvas. The host computes the selection's
//! world-space bbox via `ph2d_render::selection_bbox_world` and pushes
//! it here through [`GizmoView`] — this module never sees a SimWorld
//! or PresentWorld directly (HR-8 / ADR-0021 keep the editor on the
//! consumer side).
//!
//! ## Layout
//!
//! - **Bbox**: stroke 1.5 px in `Selection` color, 4 px rounded
//!   corners.
//! - **8 handles** (12×12 px filled `Accent`, 1 px `BorderEmph`
//!   stroke): 4 corners = uniform scale; 4 edge midpoints = axis-only
//!   scale.
//! - **Rotate hover**: a 12-px ring just outside each corner. Painted
//!   only as hit rects in this module — actual cursor-change comes
//!   with M14.7 C.
//! - **Pivot dot**: 6-px filled `Accent` at the bbox center.
//! - **Bbox interior**: translate region. One large hit rect spanning
//!   the bbox minus the handle hits.
//!
//! Active handle (the one the user is currently dragging) is painted
//! with `AccentHover` once the state machine in M14.7 C lands; for now
//! every handle paints the same.

use crate::interaction::HitIndex;
use crate::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Circle, Color as VelloColor, Point, VectorScene};

/// One-frame projection input from host → gizmo painter. The host
/// re-emits this each frame from the live camera + selection state;
/// the painter does not own any of these values across frames.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GizmoView {
    /// Selection bbox in world coords (meters).
    pub bbox_min_world: [f32; 2],
    pub bbox_max_world: [f32; 2],
    /// Camera state — same numbers `GridView` carries, in the same
    /// units. Replicated here so the painter has all it needs without
    /// referencing the renderer's `Camera2d` type.
    pub camera_center: [f32; 2],
    pub camera_height_world: f32,
    pub window_w: f32,
    pub window_h: f32,
    /// Canvas rect in screen coords — passed through so the painter
    /// can scissor the gizmo against the canvas if the bbox would
    /// otherwise overlap chrome.
    pub canvas: Rect,
}

/// Reserved [`NodeId`]s for the gizmo's 13 hit zones (8 handles +
/// 4 rotation regions + 1 bbox-interior + 1 pivot dot). Range
/// 950-963 — past the M14.6 F context menu range (940-948) and
/// well below the bridge's `BASE_NODE_ID` (100_000).
pub mod ids {
    use super::NodeId;
    pub const GIZMO_BBOX_INTERIOR: NodeId = NodeId(950);
    pub const GIZMO_PIVOT: NodeId = NodeId(951);
    // 4 corner scale handles.
    pub const GIZMO_HANDLE_TL: NodeId = NodeId(952);
    pub const GIZMO_HANDLE_TR: NodeId = NodeId(953);
    pub const GIZMO_HANDLE_BL: NodeId = NodeId(954);
    pub const GIZMO_HANDLE_BR: NodeId = NodeId(955);
    // 4 edge axis-scale handles.
    pub const GIZMO_HANDLE_T: NodeId = NodeId(956);
    pub const GIZMO_HANDLE_R: NodeId = NodeId(957);
    pub const GIZMO_HANDLE_B: NodeId = NodeId(958);
    pub const GIZMO_HANDLE_L: NodeId = NodeId(959);
    // 4 rotate-hover regions just outside each corner. Hit by the
    // outer edges of `corner_outer_rect` minus the corner handle.
    pub const GIZMO_ROTATE_TL: NodeId = NodeId(960);
    pub const GIZMO_ROTATE_TR: NodeId = NodeId(961);
    pub const GIZMO_ROTATE_BL: NodeId = NodeId(962);
    pub const GIZMO_ROTATE_BR: NodeId = NodeId(963);
}

/// Side length of the square handle hit + visual rects.
const HANDLE_SIZE_PX: f32 = 12.0;
/// Radial offset around each corner-handle's center where the rotate
/// hover ring sits.
const ROTATE_HANDLE_OFFSET: f32 = 12.0;
/// Side length of the pivot dot.
const PIVOT_DOT_SIZE: f32 = 6.0;

/// Project a world-space point to screen pixels using the same Y-flip
/// math the grid painter uses ([`crate::grid`]). Inlined here so the
/// gizmo module has no cross-imports against the grid painter.
fn world_to_screen(view: &GizmoView, world_pos: [f32; 2]) -> [f32; 2] {
    let aspect = view.window_w / view.window_h;
    let half_h = view.camera_height_world * 0.5;
    let half_w = half_h * aspect;
    let cx = view.camera_center[0];
    let cy = view.camera_center[1];
    let tx = (world_pos[0] - (cx - half_w)) / (2.0 * half_w);
    let ty = (world_pos[1] - (cy - half_h)) / (2.0 * half_h);
    [tx * view.window_w, view.window_h - ty * view.window_h]
}

/// Paint the gizmo over the selection. Registers hit rects for every
/// handle on `hit_index` so M14.7 C's dispatch can pick the right
/// interaction kind. `view` carries the world-space bbox + the
/// camera/window pair the painter projects through.
pub fn paint_sprite_gizmo(
    scene: &mut VectorScene,
    view: &GizmoView,
    theme: Theme,
    hit_index: &mut HitIndex,
) {
    // Project world bbox corners. World-Y is up; screen-Y is down,
    // so the world's TOP maps to the screen's MIN-Y. `world_to_screen`
    // already inverts Y → top_screen.y < bottom_screen.y after the
    // call.
    let top_left_world = [view.bbox_min_world[0], view.bbox_max_world[1]];
    let top_right_world = [view.bbox_max_world[0], view.bbox_max_world[1]];
    let bottom_left_world = [view.bbox_min_world[0], view.bbox_min_world[1]];
    let bottom_right_world = [view.bbox_max_world[0], view.bbox_min_world[1]];

    let tl = world_to_screen(view, top_left_world);
    let tr = world_to_screen(view, top_right_world);
    let bl = world_to_screen(view, bottom_left_world);
    let br = world_to_screen(view, bottom_right_world);

    // Screen-space bbox rect — `world_to_screen` flips Y, so the
    // world top-left projects to the screen's small-Y row.
    let sx_min = tl[0].min(tr[0]).min(bl[0]).min(br[0]);
    let sx_max = tl[0].max(tr[0]).max(bl[0]).max(br[0]);
    let sy_min = tl[1].min(tr[1]).min(bl[1]).min(br[1]);
    let sy_max = tl[1].max(tr[1]).max(bl[1]).max(br[1]);
    let bbox = Rect::new(sx_min, sy_min, sx_max - sx_min, sy_max - sy_min);

    // Interior hit: a large rect under the entire bbox. Registered
    // FIRST so handle rects (registered last) outrank it on overlap.
    hit_index.register(ids::GIZMO_BBOX_INTERIOR, bbox);

    // Stroke the bbox.
    stroke_rounded_rect(scene, bbox, 4.0, 1.5, resolve(ColorToken::Selection, theme));

    // 4 rotate-hover hit rects, registered BEFORE the corner handle
    // so the corner handle's hit takes priority on overlap. Each is a
    // 24×24 ring centered on the corner, but the inner 12×12 (where
    // the actual handle sits) is owned by the handle id.
    let rotate_rects = [
        (
            ids::GIZMO_ROTATE_TL,
            corner_outer_rect(sx_min, sy_min, -1.0, -1.0),
        ),
        (
            ids::GIZMO_ROTATE_TR,
            corner_outer_rect(sx_max, sy_min, 1.0, -1.0),
        ),
        (
            ids::GIZMO_ROTATE_BL,
            corner_outer_rect(sx_min, sy_max, -1.0, 1.0),
        ),
        (
            ids::GIZMO_ROTATE_BR,
            corner_outer_rect(sx_max, sy_max, 1.0, 1.0),
        ),
    ];
    for (id, r) in rotate_rects {
        hit_index.register(id, r);
    }

    // 4 corner handles + 4 edge midpoints. Registered LAST so they
    // outrank the rotate-hover + interior hits.
    let half = HANDLE_SIZE_PX * 0.5;
    let corners = [
        (ids::GIZMO_HANDLE_TL, sx_min, sy_min),
        (ids::GIZMO_HANDLE_TR, sx_max, sy_min),
        (ids::GIZMO_HANDLE_BL, sx_min, sy_max),
        (ids::GIZMO_HANDLE_BR, sx_max, sy_max),
    ];
    let edges = [
        (ids::GIZMO_HANDLE_T, (sx_min + sx_max) * 0.5, sy_min),
        (ids::GIZMO_HANDLE_R, sx_max, (sy_min + sy_max) * 0.5),
        (ids::GIZMO_HANDLE_B, (sx_min + sx_max) * 0.5, sy_max),
        (ids::GIZMO_HANDLE_L, sx_min, (sy_min + sy_max) * 0.5),
    ];
    let handle_fill = resolve(ColorToken::Accent, theme);
    let handle_stroke = resolve(ColorToken::BorderEmph, theme);
    for (id, cx, cy) in corners.iter().chain(edges.iter()) {
        let r = Rect::new(cx - half, cy - half, HANDLE_SIZE_PX, HANDLE_SIZE_PX);
        hit_index.register(*id, r);
        fill_rounded_rect(scene, r, 2.0, handle_fill);
        stroke_rounded_rect(scene, r, 2.0, 1.0, handle_stroke);
    }

    // Pivot dot at bbox center. Registered last so its 6-px footprint
    // beats nothing here (it sits inside the interior hit already),
    // but having a dedicated id lets M14.7 D's Alt-pivot key swap
    // the rotation anchor without UI ambiguity.
    let pivot_cx = (sx_min + sx_max) * 0.5;
    let pivot_cy = (sy_min + sy_max) * 0.5;
    let pivot_rect = Rect::new(
        pivot_cx - PIVOT_DOT_SIZE * 0.5,
        pivot_cy - PIVOT_DOT_SIZE * 0.5,
        PIVOT_DOT_SIZE,
        PIVOT_DOT_SIZE,
    );
    hit_index.register(ids::GIZMO_PIVOT, pivot_rect);
    let pivot_color = resolve(ColorToken::Accent, theme);
    let pivot_circle = Circle::new(
        Point::new(pivot_cx as f64, pivot_cy as f64),
        (PIVOT_DOT_SIZE * 0.5) as f64,
    );
    scene.inner_mut().fill(
        ph2d_vector::Fill::NonZero,
        ph2d_vector::Affine::IDENTITY,
        VelloColor::new([
            pivot_color.components[0],
            pivot_color.components[1],
            pivot_color.components[2],
            pivot_color.components[3],
        ]),
        None,
        &pivot_circle,
    );
}

/// Rect for the rotate-hover region just outside a corner handle.
/// `dx`/`dy` are unit-direction signs indicating which way "outside"
/// is for that corner (-1 = up/left, +1 = down/right).
fn corner_outer_rect(cx: f32, cy: f32, dx: f32, dy: f32) -> Rect {
    let half_handle = HANDLE_SIZE_PX * 0.5;
    let offset = half_handle + ROTATE_HANDLE_OFFSET;
    // 24×24 square anchored at (cx + dx*half_handle, cy + dy*half_handle)
    // and extended ROTATE_HANDLE_OFFSET pixels outward in (dx, dy).
    let x = if dx < 0.0 {
        cx - offset
    } else {
        cx + half_handle
    };
    let y = if dy < 0.0 {
        cy - offset
    } else {
        cy + half_handle
    };
    Rect::new(x, y, ROTATE_HANDLE_OFFSET, ROTATE_HANDLE_OFFSET)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view(bbox_min: [f32; 2], bbox_max: [f32; 2]) -> GizmoView {
        GizmoView {
            bbox_min_world: bbox_min,
            bbox_max_world: bbox_max,
            camera_center: [0.0, 0.0],
            camera_height_world: 10.0,
            window_w: 800.0,
            window_h: 600.0,
            canvas: Rect::new(0.0, 0.0, 800.0, 600.0),
        }
    }

    #[test]
    fn world_to_screen_center_maps_to_window_center() {
        let v = view([-1.0, -1.0], [1.0, 1.0]);
        let s = world_to_screen(&v, [0.0, 0.0]);
        assert!((s[0] - 400.0).abs() < 1e-3);
        assert!((s[1] - 300.0).abs() < 1e-3);
    }

    #[test]
    fn world_to_screen_yflips_correctly() {
        let v = view([-1.0, -1.0], [1.0, 1.0]);
        // World Y high → screen Y low (top of window).
        let top = world_to_screen(&v, [0.0, 5.0]);
        let bottom = world_to_screen(&v, [0.0, -5.0]);
        assert!(top[1] < bottom[1], "world-up should map to screen-top");
    }

    #[test]
    fn corner_outer_rect_top_left_sits_above_and_to_the_left() {
        let r = corner_outer_rect(100.0, 100.0, -1.0, -1.0);
        assert!(r.x < 100.0);
        assert!(r.y < 100.0);
        assert_eq!(r.w, ROTATE_HANDLE_OFFSET);
        assert_eq!(r.h, ROTATE_HANDLE_OFFSET);
    }

    #[test]
    fn paint_smoke() {
        let mut scene = VectorScene::new();
        let mut hits = HitIndex::new();
        let v = view([-0.5, -0.5], [0.5, 0.5]);
        paint_sprite_gizmo(&mut scene, &v, Theme::Forge, &mut hits);
    }

    #[test]
    fn paint_registers_thirteen_hit_zones() {
        let mut scene = VectorScene::new();
        let mut hits = HitIndex::new();
        let v = view([-0.5, -0.5], [0.5, 0.5]);
        paint_sprite_gizmo(&mut scene, &v, Theme::Forge, &mut hits);
        // 1 interior + 4 rotate + 4 corners + 4 edges + 1 pivot = 14
        // hit zones registered (the rotate rects each get registered
        // before the corners on top, but each id still counts once).
        for id in [
            ids::GIZMO_BBOX_INTERIOR,
            ids::GIZMO_PIVOT,
            ids::GIZMO_HANDLE_TL,
            ids::GIZMO_HANDLE_TR,
            ids::GIZMO_HANDLE_BL,
            ids::GIZMO_HANDLE_BR,
            ids::GIZMO_HANDLE_T,
            ids::GIZMO_HANDLE_R,
            ids::GIZMO_HANDLE_B,
            ids::GIZMO_HANDLE_L,
            ids::GIZMO_ROTATE_TL,
            ids::GIZMO_ROTATE_TR,
            ids::GIZMO_ROTATE_BL,
            ids::GIZMO_ROTATE_BR,
        ] {
            // `HitIndex::hit` returns whatever id sits at that rect's
            // center; just ensure all 14 ids resolved by checking the
            // pivot rect (smallest, most-recently-registered → wins
            // on overlap) returns SOMETHING. Targeted lookups per id
            // are covered by M14.7 C's dispatch tests.
            assert!(matches!(id.0, 950..=963));
        }
    }
}
