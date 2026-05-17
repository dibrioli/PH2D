//! Camera-framing drain (`EditorAction::SetViewFocus`).
//!
//! Wave 3.1 stage A — extracted from `hero_intents.rs` as part of
//! the HR-18 closeout split. Behavior-preserving lift.

use ph2d_ecs::PresentWorld;
use ph2d_editor::{Toast, ToastQueue, ViewFocusKind};
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;

/// Drain `hero.pending_view_focus`. Per [`ViewFocusKind`]:
///  - `Selected`: pan to gizmo_selection or (0,0).
///  - `Camera`: pan to (0,0) until camera-object exists.
///  - `All`: pan + zoom to fit every sprite (10% pad).
///
/// Returns `true` if a toast was pushed (caller marks title dirty).
pub(crate) fn drain_view_focus(
    kind: ViewFocusKind,
    gizmo_selection: Option<u64>,
    present: &mut PresentWorld,
    camera: &mut Camera2d,
    window_size: WindowSize,
    toasts: &mut ToastQueue,
) -> bool {
    let label = match kind {
        ViewFocusKind::Selected => {
            let target = gizmo_selection
                .and_then(|bits| ph2d_render::selection_bbox_world(present.world_mut(), bits));
            if let Some(bbox) = target {
                let ([cx, cy], _) = bbox.center_half();
                camera.center = [cx, cy];
                "View → Selected"
            } else {
                camera.center = [0.0, 0.0];
                "View → Selected (no selection → origin)"
            }
        }
        ViewFocusKind::Camera => {
            // No camera-object yet — frame the origin.
            camera.center = [0.0, 0.0];
            "View → Camera (origin)"
        }
        ViewFocusKind::All => {
            // Walk PresentWorld for every sprite's bbox and fit
            // camera around the union. 10% pad so handles + the bbox
            // stroke have room.
            let mut q = present
                .world_mut()
                .query::<(&ph2d_ecs::GlobalTransform, &ph2d_render::RenderInstance)>();
            let mut min_x = f32::INFINITY;
            let mut min_y = f32::INFINITY;
            let mut max_x = f32::NEG_INFINITY;
            let mut max_y = f32::NEG_INFINITY;
            let mut count = 0u32;
            for (gt, ri) in q.iter(present.world()) {
                let p = gt.translation();
                let hw = ri.size[0] * 0.5;
                let hh = ri.size[1] * 0.5;
                min_x = min_x.min(p.x - hw);
                min_y = min_y.min(p.y - hh);
                max_x = max_x.max(p.x + hw);
                max_y = max_y.max(p.y + hh);
                count += 1;
            }
            if count > 0 {
                let cx = (min_x + max_x) * 0.5;
                let cy = (min_y + max_y) * 0.5;
                let span_x = max_x - min_x;
                let span_y = max_y - min_y;
                let aspect = (window_size.width as f32) / (window_size.height.max(1) as f32);
                let need_h = span_y.max(span_x / aspect.max(1e-3));
                camera.center = [cx, cy];
                camera.height_world = (need_h * 1.1).max(0.5);
                "View → All"
            } else {
                *camera = Camera2d::default();
                "View → All (empty scene → reset)"
            }
        }
    };
    toasts.push(Toast::info(label));
    true
}
