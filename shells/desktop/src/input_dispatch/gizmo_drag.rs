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
    ///   center / corners / edge midpoints AND the content-bbox center.
    /// - **Scale / Rotate / Translate**: the pure
    ///   `compute_gizmo_transform` math, with the grid-snap closure on
    ///   the dragged corner for Scale, written back to the entity
    ///   `Transform`.
    ///
    /// Called from `on_cursor_moved` after the pointer is forwarded to
    /// the hero. The next frame's extract + paint mirror the change.
    pub(crate) fn advance_gizmo_drag(&mut self) {
        // Peek the open drag (immutable; released before the mutable
        // pass below). `GizmoDragState` is Copy.
        let open = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| h.gizmo.drag);
        let Some(drag) = open else {
            // No drag in flight → drop any cached content-bbox center so
            // the next MovePivot drag recomputes it for ITS sprite.
            self.pivot_content_center = None;
            return;
        };
        let ctrl = self.modifiers.control_key() || self.modifiers.super_key();
        // MovePivot + CTRL: compute the content-bbox center ONCE per drag
        // (lazy — first CTRL-held move triggers the readback) and cache
        // it on `self`. Done in its own borrow so the readback doesn't
        // alias the mutable pass below.
        if matches!(drag.kind, ph2d_editor::GizmoDragKind::MovePivot)
            && ctrl
            && self.pivot_content_center.is_none()
        {
            self.pivot_content_center = self.compute_pivot_content_center(&drag);
        }
        let content_center = self.pivot_content_center;

        if let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
            && let Some(mut drag) = hero.gizmo.drag
        {
            drag.cursor_screen = (self.last_pointer.0, self.last_pointer.1);
            hero.gizmo.drag = Some(drag);
            if matches!(drag.kind, ph2d_editor::GizmoDragKind::MovePivot) {
                // TOOL_PIVOT: relocate the pivot to the cursor while the
                // sprite's quad stays world-fixed (compensating anchor).
                // CTRL snaps to the quad center / corners / edge mids +
                // the content-bbox center (`content_center`).
                let window_size = gfx.surface.size();
                let entity = ph2d_ecs::Entity::from_bits(drag.entity_bits);
                let raw_world = gfx.camera.screen_to_world(drag.cursor_screen, window_size);
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
                    let consider = |c: [f32; 2], best: &mut [f32; 2], best_d2: &mut f32| {
                        let dx = c[0] - raw_world[0];
                        let dy = c[1] - raw_world[1];
                        let d2 = dx * dx + dy * dy;
                        if d2 <= *best_d2 {
                            *best_d2 = d2;
                            *best = c;
                        }
                    };
                    for c in cands {
                        consider(c, &mut best, &mut best_d2);
                    }
                    if let Some(cc) = content_center {
                        consider(cc, &mut best, &mut best_d2);
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
                // Onda 2 hotfix: for a Global gizmo drag, the axis math
                // inside `compute_gizmo_transform` projects the cursor
                // delta into the PRIMARY's LOCAL rotated frame —
                // correct for a single-sprite gizmo (whose handles
                // ARE in that rotated frame) but wrong for the global
                // gizmo, which is axis-aligned in world space. If the
                // primary happens to be rotated 90°, dragging the
                // global's right edge would scale the primary's local
                // Y axis (which IS world X) — the symptom Enio saw
                // as "scale em x muda em y e vice versa". Solution:
                // run `compute_gizmo_transform` against a drag whose
                // start_transform.rotation is zeroed so the axis
                // projection happens in WORLD coords, then restore
                // the primary's actual start rotation when applying
                // the new transform.
                let is_global_drag = matches!(drag.target, ph2d_editor::GizmoTarget::Global);
                let drag_for_math = if is_global_drag {
                    let mut d = drag;
                    d.start_transform.rotation = 0.0;
                    d
                } else {
                    drag
                };
                let new_t = if is_scale {
                    let snap_state = &mut hero.grid.snap_state;
                    let mut snap_closure = |w: [f32; 2]| -> [f32; 2] {
                        snap_state.snap_world(w, sprite_half_rendered)
                    };
                    ph2d_editor::compute_gizmo_transform(
                        &drag_for_math,
                        &cam,
                        mods,
                        snap,
                        Some(&mut snap_closure),
                    )
                } else {
                    ph2d_editor::compute_gizmo_transform(&drag_for_math, &cam, mods, snap, None)
                };
                // Restore the primary's actual rotation: in Global
                // drags `compute_gizmo_transform` returned a rotation
                // computed against the zeroed start, so we shift it
                // back by the primary's original start rotation. For
                // non-Global drags this is a no-op.
                let new_t = if is_global_drag {
                    ph2d_editor::TransformSnapshot {
                        rotation: drag.start_transform.rotation
                            + (new_t.rotation - drag_for_math.start_transform.rotation),
                        ..new_t
                    }
                } else {
                    new_t
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
                // Onda 2C.4 fix: in Local-pivot mode with multi-select
                // active, every sprite (INCLUDING the primary) must
                // scale / rotate around its own pivot — translation
                // stays put. The default `compute_gizmo_transform`
                // would shift the primary's translation for non-
                // center anchors. Restore start translation so the
                // primary matches the extras in Local mode.
                //
                // Global mode is the opposite — `compute_gizmo_transform`
                // returns translation computed via `opposite_anchor_translation`
                // using the PRIMARY's `sprite_half_intrinsic` and the GLOBAL
                // pivot, which is geometrically incorrect (it treats the
                // global center as if it were the primary's opposite
                // corner). That math sends the primary jumping to wild
                // positions on tiny drags (smoke: "algumas sprite saltam
                // para outra posição distante mesmo sem escalonar muito").
                // For Global Scale/Rotate, override the primary's
                // translation with the same group-pivot formula the extras
                // already use: `pivot + R(delta_rot) * factor * (start -
                // pivot)`. The primary then behaves consistently with
                // every other selected sprite — "as if the group is a
                // single rigid object around the global pivot".
                let start_scale = drag.start_transform.scale;
                let factor_x = if start_scale[0].abs() > f32::EPSILON {
                    new_t.scale[0] / start_scale[0]
                } else {
                    1.0
                };
                let factor_y = if start_scale[1].abs() > f32::EPSILON {
                    new_t.scale[1] / start_scale[1]
                } else {
                    1.0
                };
                let delta_rot_outer = new_t.rotation - drag.start_transform.rotation;
                let in_local_multi = !self.group_drag_starts.is_empty()
                    && !matches!(drag.target, ph2d_editor::GizmoTarget::Global)
                    && !matches!(
                        drag.kind,
                        ph2d_editor::GizmoDragKind::Translate
                            | ph2d_editor::GizmoDragKind::MovePivot
                    );
                let in_global_xform = matches!(drag.target, ph2d_editor::GizmoTarget::Global)
                    && !matches!(
                        drag.kind,
                        ph2d_editor::GizmoDragKind::Translate
                            | ph2d_editor::GizmoDragKind::MovePivot
                    );
                let primary_translation = if in_local_multi {
                    drag.start_transform.translation
                } else if in_global_xform {
                    let pivot = drag.pivot_world;
                    let st = drag.start_transform;
                    let rel_x = st.translation[0] - pivot[0];
                    let rel_y = st.translation[1] - pivot[1];
                    let scaled_x = rel_x * factor_x;
                    let scaled_y = rel_y * factor_y;
                    let (sin_d, cos_d) = delta_rot_outer.sin_cos();
                    let rotated_x = scaled_x * cos_d - scaled_y * sin_d;
                    let rotated_y = scaled_x * sin_d + scaled_y * cos_d;
                    [pivot[0] + rotated_x, pivot[1] + rotated_y]
                } else {
                    new_t.translation
                };
                if let Some(mut t) = gfx.sim.world_mut().get_mut::<Transform>(entity) {
                    t.translation =
                        ph2d_core::Vec2::new(primary_translation[0], primary_translation[1]);
                    t.rotation = new_t.rotation;
                    t.scale = ph2d_core::Vec2::new(new_t.scale[0], new_t.scale[1]);
                }
                // Onda 1 + 2C.4: propagate the drag to the rest of the
                // multi-selection. Three families:
                //
                // - Translate (any target): add the primary's world
                //   delta to every extra's start translation.
                // - Scale / Rotate with `PrimaryIndividual` /
                //   `ExtraIndividual` target (LOCAL pivot — each
                //   sprite transforms around its OWN anchor, position
                //   stays put): scale.x *= factor.x; rotation +=
                //   delta_angle; translation unchanged.
                // - Scale / Rotate with `Global` target (group pivot
                //   = global bbox center, stored on
                //   `drag.pivot_world`): each sprite scales / rotates
                //   AROUND that shared pivot, so its translation
                //   shifts too.
                //
                // MovePivot stays primary-only (drag.kind ==
                // MovePivot branch above writes Sprite.anchor; group
                // semantics aren't defined for pivot relocation).
                if !self.group_drag_starts.is_empty()
                    && !matches!(drag.kind, ph2d_editor::GizmoDragKind::MovePivot)
                {
                    let dx = new_t.translation[0] - drag.start_transform.translation[0];
                    let dy = new_t.translation[1] - drag.start_transform.translation[1];
                    let start_scale = drag.start_transform.scale;
                    let new_scale = new_t.scale;
                    let factor_x = if start_scale[0].abs() > f32::EPSILON {
                        new_scale[0] / start_scale[0]
                    } else {
                        1.0
                    };
                    let factor_y = if start_scale[1].abs() > f32::EPSILON {
                        new_scale[1] / start_scale[1]
                    } else {
                        1.0
                    };
                    let delta_rot = new_t.rotation - drag.start_transform.rotation;
                    let is_translate = matches!(drag.kind, ph2d_editor::GizmoDragKind::Translate);
                    let is_global = matches!(drag.target, ph2d_editor::GizmoTarget::Global);
                    let pivot = drag.pivot_world;
                    let (sin_d, cos_d) = delta_rot.sin_cos();
                    for snap in self.group_drag_starts.iter().copied() {
                        let extra_entity = ph2d_ecs::Entity::from_bits(snap.entity_bits);
                        let st = snap.start_transform;
                        let new_translation;
                        let new_rotation;
                        let new_scale_extra;
                        if is_translate {
                            // Translate: rigid body shift.
                            new_translation = [st.translation[0] + dx, st.translation[1] + dy];
                            new_rotation = st.rotation;
                            new_scale_extra = st.scale;
                        } else if is_global {
                            // Group scale/rotate around the shared
                            // global pivot. Compose: first scale by
                            // (factor_x, factor_y), then rotate by
                            // delta_rot, both around `pivot`.
                            let rel_x = st.translation[0] - pivot[0];
                            let rel_y = st.translation[1] - pivot[1];
                            let scaled_x = rel_x * factor_x;
                            let scaled_y = rel_y * factor_y;
                            let rotated_x = scaled_x * cos_d - scaled_y * sin_d;
                            let rotated_y = scaled_x * sin_d + scaled_y * cos_d;
                            new_translation = [pivot[0] + rotated_x, pivot[1] + rotated_y];
                            new_rotation = st.rotation + delta_rot;
                            new_scale_extra = [st.scale[0] * factor_x, st.scale[1] * factor_y];
                        } else {
                            // Local scale/rotate: each sprite
                            // transforms around its own anchor →
                            // translation stays put; only scale /
                            // rotation change.
                            new_translation = st.translation;
                            new_rotation = st.rotation + delta_rot;
                            new_scale_extra = [st.scale[0] * factor_x, st.scale[1] * factor_y];
                        }
                        if let Some(mut t) = gfx.sim.world_mut().get_mut::<Transform>(extra_entity)
                        {
                            t.translation =
                                ph2d_core::Vec2::new(new_translation[0], new_translation[1]);
                            t.rotation = new_rotation;
                            t.scale = ph2d_core::Vec2::new(new_scale_extra[0], new_scale_extra[1]);
                        }
                    }
                }
            }
        }
    }

    /// Compute the world-space center of the selected sprite's CONTENT
    /// bbox (the bounds of its non-transparent pixels) for the current
    /// MovePivot drag, or `None` if the source can't be read or the
    /// sprite is fully transparent.
    ///
    /// Reads the sprite source via the arch-gated `read_sprite_source`
    /// chokepoint (one GPU readback for an Individual texture — done once
    /// per drag, cached by the caller), scans the alpha channel for the
    /// opaque bounds, then maps the bbox center (texture px) through the
    /// quad's local frame to world: `quad_center + R·((u-0.5)·size·scale,
    /// −(v-0.5)·size·scale)`, where `quad_center = drag.pivot_world` and
    /// `R` is `start_transform.rotation`. The Y term is negated because
    /// image rows run top-down while world Y is up.
    fn compute_pivot_content_center(
        &mut self,
        drag: &ph2d_editor::GizmoDragState,
    ) -> Option<[f32; 2]> {
        let gfx = self.gfx.as_mut()?;
        let entity = ph2d_ecs::Entity::from_bits(drag.entity_bits);
        let size = gfx
            .sim
            .world()
            .get::<ph2d_render::Sprite>(entity)
            .map(|s| s.size)?;
        let src = crate::hero_intents::texture_edit::read_sprite_source(
            entity,
            &gfx.sim,
            &mut gfx.renderer,
            &gfx.asset_db,
            &gfx.atlas_asset_map,
        )?;
        let (w, h) = (src.image.width, src.image.height);
        if w == 0 || h == 0 {
            return None;
        }
        let px = &src.image.pixels;
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (w, h, 0u32, 0u32);
        let mut any = false;
        for y in 0..h {
            for x in 0..w {
                let a = px[((y * w + x) * 4 + 3) as usize];
                if a > 0 {
                    any = true;
                    min_x = min_x.min(x);
                    max_x = max_x.max(x);
                    min_y = min_y.min(y);
                    max_y = max_y.max(y);
                }
            }
        }
        if !any {
            return None;
        }
        // Pixel-center of the opaque bbox → normalized [0,1] → local quad
        // offset (world-scaled) → world (rotate about the quad center).
        let cx = (min_x as f32 + max_x as f32 + 1.0) * 0.5;
        let cy = (min_y as f32 + max_y as f32 + 1.0) * 0.5;
        let u = cx / w as f32 - 0.5;
        let v = cy / h as f32 - 0.5;
        let scale = drag.start_transform.scale;
        let local_x = u * size[0] * scale[0];
        let local_y = -v * size[1] * scale[1];
        let (sin_r, cos_r) = drag.start_transform.rotation.sin_cos();
        let qc = drag.pivot_world;
        Some([
            qc[0] + local_x * cos_r - local_y * sin_r,
            qc[1] + local_x * sin_r + local_y * cos_r,
        ])
    }
}
