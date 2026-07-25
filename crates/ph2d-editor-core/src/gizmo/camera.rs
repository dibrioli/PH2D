//! Camera + modifier + snap configs piped from host to math.
//!
//! Extracted from monolithic `gizmo.rs` in Wave 6+7 Phase 1.B.

/// Camera + window snapshot the host pipes through to the math so
/// the gizmo module doesn't depend on the renderer crate. Mirrors
/// the fields `GizmoView` already carries; kept separate because the
/// view is also used for paint-only contexts (no drag in progress).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GizmoCamera {
    pub center: [f32; 2],
    pub height_world: f32,
    pub window_w: f32,
    pub window_h: f32,
}

/// M14.7 D: which modifier keys are held during a gizmo drag. The
/// host samples the live keyboard state on every Move and pipes the
/// result through to [`compute_gizmo_transform`].
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct GizmoModifiers {
    /// Shift held: lock aspect ratio on corner scales, snap to
    /// `snap_rotate_deg` on rotate.
    pub shift: bool,
    /// Ctrl/Cmd held: snap translate to `snap_move_meters` grid AND
    /// flip Scale gizmo to center-anchor (default = opposite-corner).
    /// The center-anchor swap happens at Down via
    /// [`anchor_pivot_world`]; the snap-translate path checks this
    /// flag in [`compute_gizmo_transform`].
    pub ctrl: bool,
    /// Alt held: reserved for cursor-hint changes (Figma "mirror
    /// anchor" hint). Not currently consumed by the math.
    pub alt: bool,
}

/// M14.7 F: snap step config sourced from `ProjectSettings`. Empty
/// values (`0.0`) disable the corresponding modifier path; the math
/// in [`compute_gizmo_transform`] checks this before quantizing.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GizmoSnap {
    pub move_meters: f32,
    pub rotate_deg: f32,
}

impl Default for GizmoSnap {
    fn default() -> Self {
        Self {
            move_meters: crate::project::DEFAULT_SNAP_MOVE_METERS,
            rotate_deg: crate::project::DEFAULT_SNAP_ROTATE_DEG,
        }
    }
}

impl GizmoCamera {
    /// Reverse-project a screen pixel to world coords. Mirrors
    /// `Camera2d::screen_to_world` from `ph2d-render` exactly —
    /// keeping it inlined here avoids a ph2d-editor → ph2d-render
    /// dep arrow (ph2d-render already depends on ph2d-editor in
    /// the shell's reverse direction).
    pub fn screen_to_world(&self, cursor_px: (f32, f32)) -> [f32; 2] {
        let aspect = self.window_w / self.window_h.max(1.0);
        let half_h = self.height_world * 0.5;
        let half_w = half_h * aspect;
        let nx = (cursor_px.0 / self.window_w) * 2.0 - 1.0;
        let ny = (cursor_px.1 / self.window_h) * 2.0 - 1.0;
        [self.center[0] + nx * half_w, self.center[1] - ny * half_h]
    }
}

/// Forward-project a world point to screen pixels, camera fields passed raw.
///
/// The gizmo painters carry the camera inside a `GizmoView`, but the point gizmo
/// carries no bbox — so the projection lives here, taking the fields directly,
/// and both the box painter (`paint::world_to_screen`) and the point painter
/// share the exact same math instead of each re-deriving it.
pub(crate) fn world_to_screen_px(
    camera_center: [f32; 2],
    camera_height_world: f32,
    window_w: f32,
    window_h: f32,
    world_pos: [f32; 2],
) -> [f32; 2] {
    let aspect = window_w / window_h;
    let half_h = camera_height_world * 0.5;
    let half_w = half_h * aspect;
    let cx = camera_center[0];
    let cy = camera_center[1];
    let tx = (world_pos[0] - (cx - half_w)) / (2.0 * half_w);
    let ty = (world_pos[1] - (cy - half_h)) / (2.0 * half_h);
    [tx * window_w, window_h - ty * window_h]
}
