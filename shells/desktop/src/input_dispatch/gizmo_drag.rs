//! Gizmo drag advance — per-CursorMoved update of an open gizmo drag.
//!
//! Extracted from `input_dispatch.rs` (HR-18 LOC cap): the MovePivot
//! (TOOL_PIVOT) and the scale/rotate/translate advance paths are large
//! enough that keeping them inline tipped the window-event dispatch hub
//! past 600 LOC. The begin/end of a drag still live in the MouseInput
//! arm; only the per-move advance moved here.

use crate::{App, Transform};

impl App {
    /// Advance an in-progress gizmo drag against the latest cursor
    /// position. No-op when no drag is open. Two paths:
    ///
    /// - **MovePivot** (TOOL_PIVOT): relocate the pivot to the cursor
    ///   while the sprite's quad stays world-fixed (writes a
    ///   compensating `Sprite.anchor`); CTRL snaps the pivot to the quad
    ///   center / corners / edge midpoints.
    /// - **Scale / Rotate / Translate**: the pure
    ///   `compute_gizmo_transform` math, with the grid-snap closure on
    ///   the dragged corner for Scale, written back to the entity
    ///   `Transform`.
    ///
    /// Called from `on_cursor_moved` after the pointer is forwarded to
    /// the hero. The next frame's extract + paint mirror the change.
    pub(crate) fn advance_gizmo_drag(&mut self) {
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
            && let Some(mut drag) = hero.gizmo.drag
        {
            drag.cursor_screen = (self.last_pointer.0, self.last_pointer.1);
            hero.gizmo.drag = Some(drag);
            if matches!(drag.kind, ph2d_editor::GizmoDragKind::MovePivot) {
                // TOOL_PIVOT: relocate the pivot to the cursor while the
                // sprite's quad stays world-fixed (compensating anchor).
                // CTRL snaps to the quad center / corners / edge mids.
                let window_size = gfx.surface.size();
                let entity = ph2d_ecs::Entity::from_bits(drag.entity_bits);
                let raw_world = gfx.camera.screen_to_world(drag.cursor_screen, window_size);
                let ctrl = self.modifiers.control_key() || self.modifiers.super_key();
                let target = if ctrl {
                    let half_world = gfx
                        .sim
                        .world()
                        .get::<ph2d_render::Sprite>(entity)
                        .map(|s| {
                            [
                                s.size[0] * drag.start_transform.scale[0] * 0.5,
                                s.size[1] * drag.start_transform.scale[1] * 0.5,
                            ]
                        })
                        .unwrap_or([0.0, 0.0]);
                    let cands = ph2d_editor::pivot_snap_candidates(
                        drag.pivot_world,
                        drag.start_transform.rotation,
                        half_world,
                    );
                    // Snap when within ~14 px of a candidate, converted
                    // to world units at the current zoom.
                    let thresh = 14.0 * gfx.camera.height_world / window_size.height as f32;
                    let mut best = raw_world;
                    let mut best_d2 = thresh * thresh;
                    for c in cands {
                        let dx = c[0] - raw_world[0];
                        let dy = c[1] - raw_world[1];
                        let d2 = dx * dx + dy * dy;
                        if d2 <= best_d2 {
                            best_d2 = d2;
                            best = c;
                        }
                    }
                    best
                } else {
                    raw_world
                };
                let (new_translation, new_anchor) = ph2d_editor::move_pivot_transform(
                    drag.start_transform,
                    drag.pivot_world,
                    target,
                );
                if let Some(mut t) = gfx.sim.world_mut().get_mut::<Transform>(entity) {
                    t.translation = ph2d_core::Vec2::new(new_translation[0], new_translation[1]);
                }
                if let Some(mut s) = gfx.sim.world_mut().get_mut::<ph2d_render::Sprite>(entity) {
                    s.anchor = new_anchor;
                }
            } else {
                let window_size = gfx.surface.size();
                let cam = ph2d_editor::GizmoCamera {
                    center: gfx.camera.center,
                    height_world: gfx.camera.height_world,
                    window_w: window_size.width as f32,
                    window_h: window_size.height as f32,
                };
                // M14.7 D: sample winit's tracked modifier state (updated
                // on ModifiersChanged). Shift / Ctrl / Alt feed AR lock +
                // snap + mirror-anchor. On macOS we treat Cmd as Ctrl
                // (industry convention for snap-to-grid).
                let mods = ph2d_editor::GizmoModifiers {
                    shift: self.modifiers.shift_key(),
                    ctrl: self.modifiers.control_key() || self.modifiers.super_key(),
                    alt: self.modifiers.alt_key(),
                };
                let snap = ph2d_editor::GizmoSnap {
                    move_meters: hero.project.snap_move_meters,
                    rotate_deg: hero.project.snap_rotate_deg,
                };
                // Grid-snap apply (gizmo sites). The grid_snap subsystem's
                // `snap_world` is the canonical place to align world
                // positions to the active grid; it's a no-op when
                // `state.snap_enabled` is false or the active kind has no
                // snap target (Quadtree / Voronoi).
                let entity = ph2d_ecs::Entity::from_bits(drag.entity_bits);
                let sprite_half_rendered = gfx
                    .sim
                    .world()
                    .get::<ph2d_render::Sprite>(entity)
                    .map(|s| {
                        [
                            s.size[0] * drag.start_transform.scale[0] * 0.5,
                            s.size[1] * drag.start_transform.scale[1] * 0.5,
                        ]
                    })
                    .unwrap_or([0.0, 0.0]);
                let is_scale = matches!(
                    drag.kind,
                    ph2d_editor::GizmoDragKind::ScaleCorner { .. }
                        | ph2d_editor::GizmoDragKind::ScaleEdge { .. }
                );
                let new_t = if is_scale {
                    let snap_state = &mut hero.grid.snap_state;
                    let mut snap_closure = |w: [f32; 2]| -> [f32; 2] {
                        snap_state.snap_world(w, sprite_half_rendered)
                    };
                    ph2d_editor::compute_gizmo_transform(
                        &drag,
                        &cam,
                        mods,
                        snap,
                        Some(&mut snap_closure),
                    )
                } else {
                    ph2d_editor::compute_gizmo_transform(&drag, &cam, mods, snap, None)
                };
                let new_t = if is_scale {
                    new_t
                } else {
                    let mut new_t = new_t;
                    let sprite_half_new = gfx
                        .sim
                        .world()
                        .get::<ph2d_render::Sprite>(entity)
                        .map(|s| {
                            [
                                s.size[0] * new_t.scale[0] * 0.5,
                                s.size[1] * new_t.scale[1] * 0.5,
                            ]
                        })
                        .unwrap_or([0.0, 0.0]);
                    new_t.translation = hero
                        .grid
                        .snap_state
                        .snap_world(new_t.translation, sprite_half_new);
                    new_t
                };
                if let Some(mut t) = gfx.sim.world_mut().get_mut::<Transform>(entity) {
                    t.translation =
                        ph2d_core::Vec2::new(new_t.translation[0], new_t.translation[1]);
                    t.rotation = new_t.rotation;
                    t.scale = ph2d_core::Vec2::new(new_t.scale[0], new_t.scale[1]);
                }
            }
        }
    }
}
