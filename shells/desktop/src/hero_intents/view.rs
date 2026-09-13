//! Camera-framing drain (`EditorAction::SetViewFocus`).
//!
//! Wave 3.1 stage A — extracted from `hero_intents.rs` as part of
//! the HR-18 closeout split. Behavior-preserving lift.

use ph2d_ecs::PresentWorld;
use ph2d_editor_core::{Toast, ToastQueue, ViewFocusKind};
use ph2d_host::WindowSize;
use ph2d_i18n::tr;
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
                tr("shell.view.view_selected")
            } else {
                camera.center = [0.0, 0.0];
                tr("shell.view.view_selected_no")
            }
        }
        ViewFocusKind::Camera => {
            // No camera-object yet — frame the origin.
            camera.center = [0.0, 0.0];
            tr("shell.view.view_camera_origin")
        }
        ViewFocusKind::All => {
            // Fit the camera around the union of every sprite's DRAWN box — the picking law
            // (`ph2d_render::scene_sprites_bbox_world`: anchor, basis, and the mesh of a skinned
            // image), not the pivot-centred quad this branch used to rebuild by hand. 10% pad so
            // handles + the bbox stroke have room.
            if let Some(b) = ph2d_render::scene_sprites_bbox_world(present.world_mut()) {
                let (min_x, min_y, max_x, max_y) = (b.min[0], b.min[1], b.max[0], b.max[1]);
                let cx = (min_x + max_x) * 0.5;
                let cy = (min_y + max_y) * 0.5;
                let span_x = max_x - min_x;
                let span_y = max_y - min_y;
                let aspect = (window_size.width as f32) / (window_size.height.max(1) as f32);
                let need_h = span_y.max(span_x / aspect.max(1e-3));
                camera.center = [cx, cy];
                camera.height_world = (need_h * 1.1).max(0.5);
                tr("shell.view.view_all")
            } else {
                *camera = Camera2d::default();
                tr("shell.view.view_all_empty_scene")
            }
        }
    };
    toasts.push(Toast::info(label));
    true
}
