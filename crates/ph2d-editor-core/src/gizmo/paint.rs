//! Gizmo paint: GizmoView struct + paint_sprite_gizmo + helpers.
//!
//! Extracted from monolithic `gizmo.rs` in Wave 6+7 Phase 1.B.

use super::drag::{GizmoDragKind, GizmoHit, GizmoTarget};
use super::hit::{gizmo_kind_for_id, ids};
use super::paint_geom::{corners_screen, midpoint, paint_oriented_bbox, rotate_rects};
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
            // MULTIPLY (not XOR) the bits by the golden constant. The old
            // `canonical ^ bits ^ GOLDEN` is LINEAR — GOLDEN cancels when two
            // ids are compared, so two handles collide whenever
            // `canonical_a ^ canonical_b == bits_a ^ bits_b`. With CONSECUTIVE
            // rotate-corner ids (960..963) and CONSECUTIVE entity bits, that
            // condition is met constantly (e.g. sprite589.TL, sprite588.TR,
            // sprite591.BL, sprite590.BR all hashed to the same id), so
            // `gizmo_hit_map.insert` overwrote and a click on one sprite's
            // handle resolved to whichever sprite painted last — the "2nd/3rd
            // gizmo pivots on the 4th sprite" bug (Enio 2026-06-08).
            // `bits * GOLDEN` is non-linear: consecutive bits map far apart, so
            // the difference can never equal the tiny `canonical ^ canonical`.
            let h = bits.wrapping_mul(0x_9E37_79B9_7F4A_7C15);
            NodeId(canonical_id.0 ^ h)
        }
        GizmoTarget::Global => NodeId(canonical_id.0 ^ 0x_91A0_5B12_4FE9_3A8D),
        // Flip W7.5: espaço de id próprio para os handles da POSE — nunca colide com
        // os canônicos do primário, com os dos extras (que multiplicam os bits da
        // entidade) nem com o global (scrambler distinto).
        GizmoTarget::FlipPose => NodeId(canonical_id.0 ^ 0x_C3A5_C85C_97CB_3127),
        // Flip §4.A: espaço de id próprio para os handles da SELEÇÃO — scrambler
        // distinto do da pose (os dois nunca coexistem, mas a distinção mantém o
        // gizmo_hit_map limpo se um frame publicar os dois por engano).
        GizmoTarget::FlipSelection => NodeId(canonical_id.0 ^ 0x_5F1E_C7A0_2B94_D6E3),
        // Motion Nodes (fields): espaço de id próprio para os handles do gizmo de um
        // field espacial — scrambler distinto dos demais (o gizmo de field nunca
        // coexiste com os de sprite/flip por modalidade da tool, mas a distinção
        // mantém o gizmo_hit_map limpo em qualquer frame).
        GizmoTarget::MotionField => NodeId(canonical_id.0 ^ 0x_2D4B_86F1_9E0A_7C53),
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
    /// The PARENT's affine (local → world, `[a, b, c, d, tx, ty]`) the bbox and the pivot live
    /// under — [`super::model::MODEL_IDENTITY`] for every world-space gizmo. Applied before the
    /// camera, so a non-uniformly scaled parent paints the exact parallelogram (`gizmo/model.rs`).
    pub model: [f32; 6],
}

/// Side length of the square handle hit + visual rects.
///
/// **Public** because it is the floor of "two handles must never overlap": the handles
/// along one side are the CORNER and the EDGE-MIDPOINT, and the distance between them is
/// the box's HALF-extent. Anyone framing a selection that can be degenerate on an axis
/// (Flip's `flip_selection_gizmo`: two collinear points) must pad its box by at least this
/// — so it derives the pad from here instead of re-guessing a number that silently rots
/// when this one changes.
pub const HANDLE_SIZE_PX: f32 = 12.0;
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

/// Project a point of the gizmo's space to screen pixels: `view.model` first (the parent's
/// affine), then the same Y-flip camera math the grid painter uses ([`crate::grid`]).
pub(super) fn world_to_screen(view: &GizmoView, world_pos: [f32; 2]) -> [f32; 2] {
    super::camera::world_to_screen_px(
        view.camera_center,
        view.camera_height_world,
        view.window_w,
        view.window_h,
        super::model::model_apply(&view.model, world_pos),
    )
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
    paint_sprite_gizmo_with(scene, view, theme, hit_index, true);
}

/// [`paint_sprite_gizmo`] com o CORPO opcional: `body = false` (F10, o modo Forma da ferramenta
/// vetorial) não pinta nem regista o miolo (`GIZMO_BBOX_INTERIOR`) nem o pivô — só o contorno, as
/// alças de escala e as zonas de rotação. O miolo continua canvas: o press nele desenha.
pub fn paint_sprite_gizmo_with(
    scene: &mut VectorScene,
    view: &GizmoView,
    theme: Theme,
    hit_index: &mut HitIndex,
    body: bool,
) {
    // The 4 corners rotated around the bbox center and projected (`paint_geom`).
    let corners_s = corners_screen(view);
    let [tl, tr, bl, br] = corners_s;

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
    if body {
        hit_index.register(ids::GIZMO_BBOX_INTERIOR, aabb);
    }

    // Stroke the oriented bbox as a 4-segment closed path so the
    // visible outline follows the sprite's rotation.
    paint_oriented_bbox(
        scene,
        [tl, tr, br, bl],
        1.5,
        resolve(ColorToken::Selection, theme),
    );

    // 4 rotate-hover hit rects, registered BEFORE the corner handle
    // so the corner handle's hit takes priority on overlap — each sits
    // "past" its corner along the bbox's LOCAL axes on screen.
    let rotate_rects = rotate_rects(view, corners_s);
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
            // FRAME-RAW-OK: as ALCAS de um gizmo sobre o CANVAS: o contorno e' o que as separa da imagem por baixo
            stroke_rounded_rect(scene, r, 2.0, 1.0, handle_stroke);
        }
    }
    // Edge midpoints stay as squares — scale-only, never rotate.
    for (id, cx, cy) in edges.iter() {
        let r = Rect::new(cx - half, cy - half, HANDLE_SIZE_PX, HANDLE_SIZE_PX);
        hit_index.register(*id, r);
        fill_rounded_rect(scene, r, 2.0, handle_fill);
        // FRAME-RAW-OK: as ALCAS de um gizmo sobre o CANVAS: o contorno e' o que as separa da imagem por baixo
        stroke_rounded_rect(scene, r, 2.0, 1.0, handle_stroke);
    }
    if !body {
        return;
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

/// Onda 2: paint a non-interactive gizmo outline — just the rotated
/// bbox rectangle, no handles, no hit registration. Used by the shell
/// to mark every EXTRA member of a multi-selection (the primary still
/// gets the full interactive gizmo via `paint_sprite_gizmo`) and to
/// hint the GLOBAL bbox covering every selected sprite. `stroke_w`
/// lets the caller distinguish extras (1.0 px) from global (2.0 px)
/// from the primary (1.5 px) without three separate functions.
pub fn paint_gizmo_outline(scene: &mut VectorScene, view: &GizmoView, theme: Theme, stroke_w: f32) {
    let [tl, tr, bl, br] = corners_screen(view);
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
    let corners_s = corners_screen(view);
    let [tl, tr, bl, br] = corners_s;

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
    let rotate_rects = rotate_rects(view, corners_s);
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
            // FRAME-RAW-OK: as ALCAS de um gizmo sobre o CANVAS: o contorno e' o que as separa da imagem por baixo
            stroke_rounded_rect(scene, r, 2.0, 1.0, handle_stroke);
        }
    }
    for (id, cx, cy, kind) in edges.iter() {
        let r = Rect::new(cx - half, cy - half, HANDLE_SIZE_PX, HANDLE_SIZE_PX);
        register_keyed_handle(hit_index, hit_map, target, *id, *kind, r);
        fill_rounded_rect(scene, r, 2.0, handle_fill);
        // FRAME-RAW-OK: as ALCAS de um gizmo sobre o CANVAS: o contorno e' o que as separa da imagem por baixo
        stroke_rounded_rect(scene, r, 2.0, 1.0, handle_stroke);
    }
}
