//! Gizmo paint: GizmoView struct + paint_sprite_gizmo + helpers.
//!
//! Extracted from monolithic `gizmo.rs` in Wave 6+7 Phase 1.B.

use super::drag::{GizmoDragKind, GizmoHit, GizmoTarget};
use super::hit::{gizmo_kind_for_id, ids};
use crate::interaction::HitIndex;
use crate::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Circle, Color as VelloColor, Point, VectorScene};
use std::collections::BTreeMap;

/// Onda 2C: derive a hit-id unique per (target, canonical handle id).
/// Primary keeps canonical IDs (preserves the legacy
/// `gizmo_kind_for_id` lookup for primary handles). Extra and Global
/// XOR with a target-specific scrambler to drop into distinct id
/// spaces — collision-free with the canonical primary set + with each
/// other (every extra carries the sprite's `entity_bits` which is
/// globally unique by construction).
pub(crate) fn keyed_handle_id(target: GizmoTarget, canonical_id: NodeId) -> NodeId {
    match target {
        GizmoTarget::PrimaryIndividual => canonical_id,
        GizmoTarget::ExtraIndividual(bits) => {
            // Golden-ratio scrambler so a sprite with `bits` close to
            // a canonical-id integer can't collide by accident.
            let h = bits ^ 0x_9E37_79B9_7F4A_7C15;
            NodeId(canonical_id.0 ^ h)
        }
        GizmoTarget::Global => NodeId(canonical_id.0 ^ 0x_91A0_5B12_4FE9_3A8D),
    }
}

/// Register a handle in both the hit index and the shell's hit map
/// using the canonical-id → keyed-id mapping above. Kind comes from
/// `gizmo_kind_for_id` for handles the legacy dispatcher knows about;
/// callers pass an explicit kind for pivot / interior hits whose
/// canonical ids aren't in that table.
pub(crate) fn register_keyed_handle(
    hit_index: &mut HitIndex,
    hit_map: &mut BTreeMap<NodeId, GizmoHit>,
    target: GizmoTarget,
    canonical_id: NodeId,
    kind: GizmoDragKind,
    rect: Rect,
) {
    let id = keyed_handle_id(target, canonical_id);
    hit_index.register(id, rect);
    hit_map.insert(id, GizmoHit { target, kind });
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GizmoView {
    /// Selection bbox in world coords (meters). These are the
    /// UNROTATED local rect — the painter rotates the 4 corners by
    /// `rotation` around the center before projecting. Center =
    /// midpoint of min/max.
    pub bbox_min_world: [f32; 2],
    pub bbox_max_world: [f32; 2],
    /// World-space rotation in radians (the entity's
    /// `Transform.rotation`). Painter rotates the bbox + handles
    /// around the pivot (= bbox center) so the gizmo tracks the
    /// sprite's actual orientation.
    pub rotation: f32,
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
    /// Current cursor position in screen pixels, or `None` when no
    /// cursor data is available (e.g. paint preview tests). Painter
    /// reads this to switch corner-handle visuals from squares to
    /// circles when the cursor is inside one of the 4 rotate-hover
    /// rings — gives the user a "you're about to rotate" cue.
    pub cursor_screen: Option<(f32, f32)>,
    /// World position of the actual PIVOT — the entity's
    /// `Transform.translation`. Distinct from the bbox center once the
    /// sprite carries a non-zero `Sprite.anchor` (TOOL_PIVOT / Padding
    /// Keep): the bbox tracks the visible quad while the pivot dot is
    /// drawn here, at the true rotation/scale origin. Equals the bbox
    /// center for every centered (anchor == 0) sprite.
    pub pivot_world: [f32; 2],
    /// `true` while the Pivot transform tool is the active radio
    /// selection — the painter emphasizes the pivot dot (filled, slightly
    /// larger) so the user sees the grabbable handle.
    pub pivot_tool_active: bool,
}

/// Side length of the square handle hit + visual rects.
const HANDLE_SIZE_PX: f32 = 12.0;
/// Radial offset around each corner-handle's center where the rotate
/// hover ring sits.
// Onda 2 hotfix: bumped from 12 → 20 (Enio: "aumente a área sensível
// de detecção para rot em todos os gzimo"). The hit rect is also
// `ROTATE_HANDLE_OFFSET × ROTATE_HANDLE_OFFSET`, so this widens both
// the radial offset past each corner AND the rect side length —
// rotate zones are ~2.7× the prior area.
pub(super) const ROTATE_HANDLE_OFFSET: f32 = 20.0;
/// Side length of the pivot dot.
/// Diameter of the pivot ring drawn at the bbox center. 12 px per
/// user feedback — visible enough to grab with the cursor without
/// overlapping the interior bbox-translate hit. Rendered as a
/// hollow ring (stroke, not fill) in a fixed red color across all
/// themes for unambiguous rotation-anchor identity.
const PIVOT_DOT_SIZE: f32 = 12.0;

/// Project a world-space point to screen pixels using the same Y-flip
/// math the grid painter uses ([`crate::grid`]). Inlined here so the
/// gizmo module has no cross-imports against the grid painter.
pub(super) fn world_to_screen(view: &GizmoView, world_pos: [f32; 2]) -> [f32; 2] {
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
/// camera/window pair the painter projects through. Honors
/// `view.rotation` by rotating the 4 corner points around the bbox
/// center before projection — gizmo tracks the sprite's actual
/// orientation under any Transform.rotation.
pub fn paint_sprite_gizmo(
    scene: &mut VectorScene,
    view: &GizmoView,
    theme: Theme,
    hit_index: &mut HitIndex,
) {
    // World-space center + half-extents of the unrotated rect.
    let cx_w = (view.bbox_min_world[0] + view.bbox_max_world[0]) * 0.5;
    let cy_w = (view.bbox_min_world[1] + view.bbox_max_world[1]) * 0.5;
    let hx_w = (view.bbox_max_world[0] - view.bbox_min_world[0]) * 0.5;
    let hy_w = (view.bbox_max_world[1] - view.bbox_min_world[1]) * 0.5;

    // Rotate the 4 corners around the world-space center by
    // `view.rotation`, then project each to screen pixels. Local
    // offsets in (x, y) world coords: TL = (-hx, +hy), TR = (+hx,
    // +hy), BL = (-hx, -hy), BR = (+hx, -hy). The y axis here is
    // world-Y up; `world_to_screen` does the Y flip.
    let cos_r = view.rotation.cos();
    let sin_r = view.rotation.sin();
    let rotate_world = |lx: f32, ly: f32| -> [f32; 2] {
        [
            cx_w + lx * cos_r - ly * sin_r,
            cy_w + lx * sin_r + ly * cos_r,
        ]
    };
    let tl_w = rotate_world(-hx_w, hy_w);
    let tr_w = rotate_world(hx_w, hy_w);
    let bl_w = rotate_world(-hx_w, -hy_w);
    let br_w = rotate_world(hx_w, -hy_w);

    let tl = world_to_screen(view, tl_w);
    let tr = world_to_screen(view, tr_w);
    let bl = world_to_screen(view, bl_w);
    let br = world_to_screen(view, br_w);

    // Interior hit: axis-aligned bounding box of the 4 rotated
    // corners. Picking precision suffers slightly at high rotation
    // angles (a 45° square's AABB has 2× the area of the OBB), but
    // the 4 corner handles register their own tighter hits which
    // outrank this on overlap.
    let sx_min = tl[0].min(tr[0]).min(bl[0]).min(br[0]);
    let sx_max = tl[0].max(tr[0]).max(bl[0]).max(br[0]);
    let sy_min = tl[1].min(tr[1]).min(bl[1]).min(br[1]);
    let sy_max = tl[1].max(tr[1]).max(bl[1]).max(br[1]);
    let aabb = Rect::new(sx_min, sy_min, sx_max - sx_min, sy_max - sy_min);
    hit_index.register(ids::GIZMO_BBOX_INTERIOR, aabb);

    // Stroke the oriented bbox as a 4-segment closed path so the
    // visible outline follows the sprite's rotation.
    paint_oriented_bbox(
        scene,
        [tl, tr, br, bl],
        1.5,
        resolve(ColorToken::Selection, theme),
    );

    // 4 rotate-hover hit rects, registered BEFORE the corner handle
    // so the corner handle's hit takes priority on overlap. The
    // outer-rect helper takes a SCREEN-space corner pos + the local
    // sign (+/-1 along each axis). For a rotated bbox we want the
    // outer offset to be along the bbox's LOCAL axes; we approximate
    // by using the unit corner direction (cos/sin of rotation) so
    // the hover region sits "past" the corner on the rotated rect.
    let rotate_rects = [
        (
            ids::GIZMO_ROTATE_TL,
            corner_outer_rect_oriented(tl, view.rotation, -1.0, 1.0),
        ),
        (
            ids::GIZMO_ROTATE_TR,
            corner_outer_rect_oriented(tr, view.rotation, 1.0, 1.0),
        ),
        (
            ids::GIZMO_ROTATE_BL,
            corner_outer_rect_oriented(bl, view.rotation, -1.0, -1.0),
        ),
        (
            ids::GIZMO_ROTATE_BR,
            corner_outer_rect_oriented(br, view.rotation, 1.0, -1.0),
        ),
    ];
    for (id, r) in rotate_rects {
        hit_index.register(id, r);
    }

    // Detect whether the cursor is inside any rotate hover rect.
    // When yes, the corresponding corner handle paints as a circle
    // instead of a square — user-asked visual cue for "rotate mode".
    let cursor_over_rotate = view
        .cursor_screen
        .map(|(cx, cy)| rotate_rects.iter().any(|(_, r)| r.contains(cx, cy)))
        .unwrap_or(false);

    // 4 corner handles + 4 edge midpoints. Registered LAST so they
    // outrank the rotate-hover + interior hits.
    let mid_top = midpoint(tl, tr);
    let mid_right = midpoint(tr, br);
    let mid_bottom = midpoint(br, bl);
    let mid_left = midpoint(bl, tl);
    let corners = [
        (ids::GIZMO_HANDLE_TL, tl[0], tl[1]),
        (ids::GIZMO_HANDLE_TR, tr[0], tr[1]),
        (ids::GIZMO_HANDLE_BL, bl[0], bl[1]),
        (ids::GIZMO_HANDLE_BR, br[0], br[1]),
    ];
    let edges = [
        (ids::GIZMO_HANDLE_T, mid_top[0], mid_top[1]),
        (ids::GIZMO_HANDLE_R, mid_right[0], mid_right[1]),
        (ids::GIZMO_HANDLE_B, mid_bottom[0], mid_bottom[1]),
        (ids::GIZMO_HANDLE_L, mid_left[0], mid_left[1]),
    ];
    let handle_fill = resolve(ColorToken::Accent, theme);
    let handle_stroke = resolve(ColorToken::BorderEmph, theme);
    let half = HANDLE_SIZE_PX * 0.5;
    // Corners (may render as circles in rotate-hover mode).
    for (id, cx, cy) in corners.iter() {
        let r = Rect::new(cx - half, cy - half, HANDLE_SIZE_PX, HANDLE_SIZE_PX);
        hit_index.register(*id, r);
        if cursor_over_rotate {
            // Circle handle = rotate cue.
            let circle = Circle::new(
                Point::new(*cx as f64, *cy as f64),
                (HANDLE_SIZE_PX * 0.5) as f64,
            );
            scene.inner_mut().fill(
                ph2d_vector::Fill::NonZero,
                ph2d_vector::Affine::IDENTITY,
                VelloColor::new([
                    handle_fill.components[0],
                    handle_fill.components[1],
                    handle_fill.components[2],
                    handle_fill.components[3],
                ]),
                None,
                &circle,
            );
            scene.inner_mut().stroke(
                &ph2d_vector::Stroke::new(1.0),
                ph2d_vector::Affine::IDENTITY,
                VelloColor::new([
                    handle_stroke.components[0],
                    handle_stroke.components[1],
                    handle_stroke.components[2],
                    handle_stroke.components[3],
                ]),
                None,
                &circle,
            );
        } else {
            fill_rounded_rect(scene, r, 2.0, handle_fill);
            stroke_rounded_rect(scene, r, 2.0, 1.0, handle_stroke);
        }
    }
    // Edge midpoints stay as squares — scale-only, never rotate.
    for (id, cx, cy) in edges.iter() {
        let r = Rect::new(cx - half, cy - half, HANDLE_SIZE_PX, HANDLE_SIZE_PX);
        hit_index.register(*id, r);
        fill_rounded_rect(scene, r, 2.0, handle_fill);
        stroke_rounded_rect(scene, r, 2.0, 1.0, handle_stroke);
    }
    // Pivot dot at the TRUE pivot (`Transform.translation`), projected
    // to screen. For a centered sprite this coincides with the bbox
    // center; once the sprite carries an anchor (TOOL_PIVOT / Padding
    // Keep) the pivot sits off-center and the dot tracks it there.
    let pivot_screen = world_to_screen(view, view.pivot_world);
    let pivot_cx = pivot_screen[0];
    let pivot_cy = pivot_screen[1];
    let _ = (sx_min, sx_max, sy_min, sy_max);
    // Slightly larger hit/visual when the Pivot tool is active so the
    // grabbable handle is obvious.
    let dot_size = if view.pivot_tool_active {
        PIVOT_DOT_SIZE * 1.5
    } else {
        PIVOT_DOT_SIZE
    };
    let pivot_rect = Rect::new(
        pivot_cx - dot_size * 0.5,
        pivot_cy - dot_size * 0.5,
        dot_size,
        dot_size,
    );
    hit_index.register(ids::GIZMO_PIVOT, pivot_rect);
    // Pivot ring: hollow red circle, fixed color across themes.
    // Theme-independent so the rotation anchor reads the same on
    // Forge / Workshop / Sunstone / Blueprint without changing
    // semantic meaning.
    let _ = theme;
    let pivot_red = VelloColor::from_rgba8(0xE0, 0x40, 0x40, 0xFF);
    let pivot_circle = Circle::new(
        Point::new(pivot_cx as f64, pivot_cy as f64),
        (dot_size * 0.5) as f64,
    );
    if view.pivot_tool_active {
        // Active tool: fill the dot so it reads as a grabbable handle.
        scene.inner_mut().fill(
            ph2d_vector::Fill::NonZero,
            ph2d_vector::Affine::IDENTITY,
            pivot_red,
            None,
            &pivot_circle,
        );
    } else {
        // Idle: hollow ring (stroke) with the bbox interior visible
        // through the hole — user explicitly asked for "vazado no meio".
        scene.inner_mut().stroke(
            &ph2d_vector::Stroke::new(1.5),
            ph2d_vector::Affine::IDENTITY,
            pivot_red,
            None,
            &pivot_circle,
        );
    }
}

/// Rect for the rotate-hover region just outside a corner handle.
/// `dx`/`dy` are unit-direction signs indicating which way "outside"
/// is for that corner (-1 = up/left, +1 = down/right). Kept as a
/// legacy helper used by the gizmo's tests + axis-aligned smoke
/// path before M14.7 polish added the oriented variant.
#[allow(dead_code)]
pub(super) fn corner_outer_rect(cx: f32, cy: f32, dx: f32, dy: f32) -> Rect {
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

/// Oriented version of [`corner_outer_rect`]: places the rotate-hover
/// rect "past" a corner along the BBOX's local axes (which themselves
/// rotated by `rotation`). For a heavy rotation the local +X axis no
/// longer matches screen +X, so a fixed-axis offset would put the
/// hover region in the wrong place.
///
/// The shape stays an AABB in screen coords (no rotated hit rect —
/// HitIndex tests against axis-aligned `Rect::contains`), but we
/// center it on a point that's been rotated into the local-frame
/// "outside" direction. Trade-off: at 45° the AABB hover region
/// overlaps slightly with the adjacent corner's rotate region; for
/// the user it just means rotation gestures stay live across a small
/// dead zone between corners.
fn corner_outer_rect_oriented(
    corner_screen: [f32; 2],
    rotation: f32,
    lx_sign: f32,
    ly_sign: f32,
) -> Rect {
    let half_handle = HANDLE_SIZE_PX * 0.5;
    let offset = half_handle + ROTATE_HANDLE_OFFSET * 0.5;
    // Local axes in SCREEN space. `world_to_screen` flips Y (world
    // +Y → screen -Y), so each world-Y component picks up a sign
    // flip when projected.
    //   World local +X = (cos r, sin r)  → screen (cos r, -sin r)
    //   World local +Y = (-sin r, cos r) → screen (-sin r, -cos r)
    // The earlier version dropped the negation on local_x; at any
    // non-zero rotation the hover rect drifted ROTATE_HANDLE_OFFSET
    // px off the visible corner — user reported "área que detecta a
    // rot não ficou bem posicionada na quina".
    let cos_r = rotation.cos();
    let sin_r = rotation.sin();
    let local_x_screen = (cos_r, -sin_r);
    let local_y_screen = (-sin_r, -cos_r);
    let off_x = lx_sign * offset;
    let off_y = ly_sign * offset;
    let center_x = corner_screen[0] + local_x_screen.0 * off_x + local_y_screen.0 * off_y;
    let center_y = corner_screen[1] + local_x_screen.1 * off_x + local_y_screen.1 * off_y;
    Rect::new(
        center_x - ROTATE_HANDLE_OFFSET * 0.5,
        center_y - ROTATE_HANDLE_OFFSET * 0.5,
        ROTATE_HANDLE_OFFSET,
        ROTATE_HANDLE_OFFSET,
    )
}

fn midpoint(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5]
}

/// Stroke a 4-corner closed polygon as the gizmo's oriented bbox
/// outline. Replaces the rounded-rect draw the axis-aligned painter
/// used; with rotation, the rect's edges no longer align with screen
/// axes so we walk the corner sequence directly via `BezPath`.
/// Onda 2: paint a non-interactive gizmo outline — just the rotated
/// bbox rectangle, no handles, no hit registration. Used by the shell
/// to mark every EXTRA member of a multi-selection (the primary still
/// gets the full interactive gizmo via `paint_sprite_gizmo`) and to
/// hint the GLOBAL bbox covering every selected sprite. `stroke_w`
/// lets the caller distinguish extras (1.0 px) from global (2.0 px)
/// from the primary (1.5 px) without three separate functions.
pub fn paint_gizmo_outline(scene: &mut VectorScene, view: &GizmoView, theme: Theme, stroke_w: f32) {
    let cx_w = (view.bbox_min_world[0] + view.bbox_max_world[0]) * 0.5;
    let cy_w = (view.bbox_min_world[1] + view.bbox_max_world[1]) * 0.5;
    let hx_w = (view.bbox_max_world[0] - view.bbox_min_world[0]) * 0.5;
    let hy_w = (view.bbox_max_world[1] - view.bbox_min_world[1]) * 0.5;
    let cos_r = view.rotation.cos();
    let sin_r = view.rotation.sin();
    let rotate_world = |lx: f32, ly: f32| -> [f32; 2] {
        [
            cx_w + lx * cos_r - ly * sin_r,
            cy_w + lx * sin_r + ly * cos_r,
        ]
    };
    let tl = world_to_screen(view, rotate_world(-hx_w, hy_w));
    let tr = world_to_screen(view, rotate_world(hx_w, hy_w));
    let bl = world_to_screen(view, rotate_world(-hx_w, -hy_w));
    let br = world_to_screen(view, rotate_world(hx_w, -hy_w));
    paint_oriented_bbox(
        scene,
        [tl, tr, br, bl],
        stroke_w,
        resolve(ColorToken::Selection, theme),
    );
}

/// Onda 2C: interactive gizmo painter for extras + global. Same
/// rotated bbox outline + scale-corner / scale-edge / rotate-hover
/// handles as `paint_sprite_gizmo`, but every handle's hit id goes
/// through `keyed_handle_id(target, …)` so the dispatcher can tell
/// WHICH gizmo (which sprite, or the global one) was clicked. Skips
/// the pivot dot — that's primary-only chrome. Skips the cursor-
/// over-rotate circular-handle visual cue too — keeps the painter
/// simple while the primary still gets the full UX.
pub fn paint_sprite_gizmo_keyed(
    scene: &mut VectorScene,
    view: &GizmoView,
    theme: Theme,
    hit_index: &mut HitIndex,
    hit_map: &mut BTreeMap<NodeId, GizmoHit>,
    target: GizmoTarget,
    outline_stroke_w: f32,
) {
    let cx_w = (view.bbox_min_world[0] + view.bbox_max_world[0]) * 0.5;
    let cy_w = (view.bbox_min_world[1] + view.bbox_max_world[1]) * 0.5;
    let hx_w = (view.bbox_max_world[0] - view.bbox_min_world[0]) * 0.5;
    let hy_w = (view.bbox_max_world[1] - view.bbox_min_world[1]) * 0.5;
    let cos_r = view.rotation.cos();
    let sin_r = view.rotation.sin();
    let rotate_world = |lx: f32, ly: f32| -> [f32; 2] {
        [
            cx_w + lx * cos_r - ly * sin_r,
            cy_w + lx * sin_r + ly * cos_r,
        ]
    };
    let tl = world_to_screen(view, rotate_world(-hx_w, hy_w));
    let tr = world_to_screen(view, rotate_world(hx_w, hy_w));
    let bl = world_to_screen(view, rotate_world(-hx_w, -hy_w));
    let br = world_to_screen(view, rotate_world(hx_w, -hy_w));

    // Onda 2C hotfix: do NOT register a BBOX_INTERIOR hit for extras
    // or the global. Their bbox AABBs are big (and the global covers
    // EVERY sprite + 32 px offset), so registering them after the
    // primary's handles would shadow every individual scale / rotate
    // handle on `hit_index`'s back-to-front walk. Translate dispatch
    // for these gizmos goes through the shell's canvas-pick branch
    // instead: a click in the empty area of an extra resolves to the
    // sprite via `pick_sprites_at_world`, and the smart-click
    // preservation logic keeps the multi-selection alive while a
    // group-translate drag opens against that sprite.

    // Outline stroke.
    paint_oriented_bbox(
        scene,
        [tl, tr, br, bl],
        outline_stroke_w,
        resolve(ColorToken::Selection, theme),
    );

    // 4 rotate-hover rects (registered BEFORE corner handles so the
    // tighter corner hits outrank them on overlap).
    let rotate_rects = [
        (
            ids::GIZMO_ROTATE_TL,
            corner_outer_rect_oriented(tl, view.rotation, -1.0, 1.0),
        ),
        (
            ids::GIZMO_ROTATE_TR,
            corner_outer_rect_oriented(tr, view.rotation, 1.0, 1.0),
        ),
        (
            ids::GIZMO_ROTATE_BL,
            corner_outer_rect_oriented(bl, view.rotation, -1.0, -1.0),
        ),
        (
            ids::GIZMO_ROTATE_BR,
            corner_outer_rect_oriented(br, view.rotation, 1.0, -1.0),
        ),
    ];
    for (id, r) in rotate_rects {
        register_keyed_handle(hit_index, hit_map, target, id, GizmoDragKind::Rotate, r);
    }
    // Onda 2 hotfix: mirror the primary gizmo's cursor-over-rotate
    // hover swap — when the cursor enters any of the rotate hit rects,
    // the four scale-corner handles draw as CIRCLES instead of
    // squares. That's the visual rotate cue Enio asked for (no extra
    // ring drawn on the canvas; the corners themselves change shape).
    let cursor_over_rotate = view
        .cursor_screen
        .map(|(cx, cy)| rotate_rects.iter().any(|(_, r)| r.contains(cx, cy)))
        .unwrap_or(false);

    // Corner scale handles + edge scale handles (squares).
    let mid_top = midpoint(tl, tr);
    let mid_right = midpoint(tr, br);
    let mid_bottom = midpoint(br, bl);
    let mid_left = midpoint(bl, tl);
    let handle_fill = resolve(ColorToken::Accent, theme);
    let handle_stroke = resolve(ColorToken::BorderEmph, theme);
    let half = HANDLE_SIZE_PX * 0.5;
    let corners: [(NodeId, f32, f32, GizmoDragKind); 4] = [
        (
            ids::GIZMO_HANDLE_TL,
            tl[0],
            tl[1],
            gizmo_kind_for_id(ids::GIZMO_HANDLE_TL).unwrap_or(GizmoDragKind::Translate),
        ),
        (
            ids::GIZMO_HANDLE_TR,
            tr[0],
            tr[1],
            gizmo_kind_for_id(ids::GIZMO_HANDLE_TR).unwrap_or(GizmoDragKind::Translate),
        ),
        (
            ids::GIZMO_HANDLE_BL,
            bl[0],
            bl[1],
            gizmo_kind_for_id(ids::GIZMO_HANDLE_BL).unwrap_or(GizmoDragKind::Translate),
        ),
        (
            ids::GIZMO_HANDLE_BR,
            br[0],
            br[1],
            gizmo_kind_for_id(ids::GIZMO_HANDLE_BR).unwrap_or(GizmoDragKind::Translate),
        ),
    ];
    let edges: [(NodeId, f32, f32, GizmoDragKind); 4] = [
        (
            ids::GIZMO_HANDLE_T,
            mid_top[0],
            mid_top[1],
            gizmo_kind_for_id(ids::GIZMO_HANDLE_T).unwrap_or(GizmoDragKind::Translate),
        ),
        (
            ids::GIZMO_HANDLE_R,
            mid_right[0],
            mid_right[1],
            gizmo_kind_for_id(ids::GIZMO_HANDLE_R).unwrap_or(GizmoDragKind::Translate),
        ),
        (
            ids::GIZMO_HANDLE_B,
            mid_bottom[0],
            mid_bottom[1],
            gizmo_kind_for_id(ids::GIZMO_HANDLE_B).unwrap_or(GizmoDragKind::Translate),
        ),
        (
            ids::GIZMO_HANDLE_L,
            mid_left[0],
            mid_left[1],
            gizmo_kind_for_id(ids::GIZMO_HANDLE_L).unwrap_or(GizmoDragKind::Translate),
        ),
    ];
    // Corners: render as circle when cursor hovers a rotate zone
    // (Enio: "mudar os rect do gizmo para circulos"). Edges always
    // stay square — edges are scale-only, never rotate.
    for (id, cx, cy, kind) in corners.iter() {
        let r = Rect::new(cx - half, cy - half, HANDLE_SIZE_PX, HANDLE_SIZE_PX);
        register_keyed_handle(hit_index, hit_map, target, *id, *kind, r);
        if cursor_over_rotate {
            let circle = Circle::new(
                Point::new(*cx as f64, *cy as f64),
                (HANDLE_SIZE_PX * 0.5) as f64,
            );
            scene.inner_mut().fill(
                ph2d_vector::Fill::NonZero,
                ph2d_vector::Affine::IDENTITY,
                handle_fill,
                None,
                &circle,
            );
            scene.inner_mut().stroke(
                &ph2d_vector::Stroke::new(1.0),
                ph2d_vector::Affine::IDENTITY,
                handle_stroke,
                None,
                &circle,
            );
        } else {
            fill_rounded_rect(scene, r, 2.0, handle_fill);
            stroke_rounded_rect(scene, r, 2.0, 1.0, handle_stroke);
        }
    }
    for (id, cx, cy, kind) in edges.iter() {
        let r = Rect::new(cx - half, cy - half, HANDLE_SIZE_PX, HANDLE_SIZE_PX);
        register_keyed_handle(hit_index, hit_map, target, *id, *kind, r);
        fill_rounded_rect(scene, r, 2.0, handle_fill);
        stroke_rounded_rect(scene, r, 2.0, 1.0, handle_stroke);
    }
}

fn paint_oriented_bbox(
    scene: &mut VectorScene,
    corners: [[f32; 2]; 4],
    width: f32,
    color: ph2d_vector::Color,
) {
    let mut path = ph2d_vector::BezPath::new();
    path.move_to((corners[0][0] as f64, corners[0][1] as f64));
    for c in corners.iter().skip(1) {
        path.line_to((c[0] as f64, c[1] as f64));
    }
    path.close_path();
    scene.inner_mut().stroke(
        &ph2d_vector::Stroke::new(width as f64),
        ph2d_vector::Affine::IDENTITY,
        color,
        None,
        &path,
    );
}
