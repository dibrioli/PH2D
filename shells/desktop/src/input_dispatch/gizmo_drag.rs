//! Gizmo drag advance — per-CursorMoved update of an open gizmo drag.
//!
//! Extracted from `input_dispatch.rs` (HR-18 LOC cap): the MovePivot
//! (TOOL_PIVOT) and the scale/rotate/translate advance paths are large
//! enough that keeping them inline tipped the window-event dispatch hub
//! past 600 LOC. The begin/end of a drag still live in the MouseInput
//! arm; only the per-move advance moved here.
// ph2d-loc-cap: 680 LOC — the keyed-handle-id multi-select rotate/scale/translate
// advance paths are inherently large; a finer per-path split is a desktop-gizmo follow-up.
// +12 (gold-standard joint anchor): a Translate on a joint marks it `anchored = false`
// so the bridge re-derives its body-local anchors from the dragged pivot.

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

        // Scale-snap vetorial: se a forma arrastada é vetorial e o gesto é scale, o
        // canto arrastado encaixa nas OUTRAS formas (bordas/centros/vértices), como no
        // translate. Recolhe os alvos + cfg agora, fora do borrow mutável de gfx.
        let vec_scale_ids = if matches!(
            drag.kind,
            ph2d_editor::GizmoDragKind::ScaleCorner { .. }
                | ph2d_editor::GizmoDragKind::ScaleEdge { .. }
        ) {
            self.dragged_vec_path_ids(drag.entity_bits)
        } else {
            Vec::new()
        };
        let vec_scale_snap = !vec_scale_ids.is_empty();
        let vec_cfg = self.vec_snap_cfg(self.vec_px_to_world());
        if vec_scale_snap {
            self.vec_rebuild_snap_targets(&vec_scale_ids, &[]);
        }

        if let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
            && let Some(mut drag) = hero.gizmo.drag
        {
            // Advance the cursor THROUGH the drag, not around it: on a Rotate the
            // step also counts any revolution the cursor just completed, which is
            // the only record of it (`atan2` cannot see past one turn). Skipping
            // this and assigning `cursor_screen` directly is exactly the bug —
            // rotation would silently jump 2π at the branch cut.
            let size = gfx.surface.size();
            let cam = ph2d_editor::GizmoCamera {
                center: gfx.camera.center,
                height_world: gfx.camera.height_world,
                window_w: size.width as f32,
                window_h: size.height as f32,
            };
            drag.advance_cursor((self.last_pointer.0, self.last_pointer.1), &cam);
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
                    drag.parent_world,
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
                let new_t = if is_scale && vec_scale_snap {
                    // Encaixa o CANTO arrastado (cursor) nas outras formas + grade e
                    // publica as guias — mesmo motor do translate, mas quem aplica é o
                    // gizmo (o cursor encaixado dirige a razão de escala, pivô fixo). O
                    // bloco interno solta os borrows do closure antes de gravar as guias.
                    let targets = &self.vec_snap_targets;
                    let mut guides: Vec<ph2d_vec_render::Guide> = Vec::new();
                    let snap_state = &mut hero.grid.snap_state;
                    let t = {
                        let mut snap_closure = |w: [f32; 2]| -> [f32; 2] {
                            let p = [f64::from(w[0]), f64::from(w[1])];
                            let mut grid = |q: [f64; 2]| crate::vec_snap::ask_grid(snap_state, q);
                            let r =
                                ph2d_vec_edit::snap::snap(&[p], targets, vec_cfg, Some(&mut grid));
                            guides = crate::vec_snap::guides_of(&r);
                            let s = r.apply(p);
                            [s[0] as f32, s[1] as f32]
                        };
                        ph2d_editor::compute_gizmo_transform(
                            &drag_for_math,
                            &cam,
                            mods,
                            snap,
                            Some(&mut snap_closure),
                        )
                    };
                    self.vec_snap_guides = guides;
                    t
                } else if is_scale {
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
                // ── Uniform-only group scale for ROTATED selections (Enio
                // 2026-06-03). A non-uniform WORLD-axis scale of a rotated child is
                // a SHEAR, which `Transform.scale` (local, per-child) cannot
                // represent → the reported X↔Y swap. Standard editor behaviour:
                // when the multi-selection contains a rotated object, scaling stays
                // PROPORTIONAL (both axes by the dominant drag factor). A single
                // (non-group) scale keeps its correct local-axis non-uniform
                // behaviour. Applied to `new_t` BEFORE the factor + primary write
                // below so the primary AND every extra scale uniformly.
                let new_t = if !self.group_drag_starts.is_empty()
                    && matches!(
                        drag.kind,
                        ph2d_editor::GizmoDragKind::ScaleCorner { .. }
                            | ph2d_editor::GizmoDragKind::ScaleEdge { .. }
                    )
                    && (drag.start_transform.rotation != 0.0
                        || self
                            .group_drag_starts
                            .iter()
                            .any(|s| s.start_transform.rotation != 0.0))
                {
                    let ss = drag.start_transform.scale;
                    let fx = if ss[0].abs() > f32::EPSILON {
                        new_t.scale[0] / ss[0]
                    } else {
                        1.0
                    };
                    let fy = if ss[1].abs() > f32::EPSILON {
                        new_t.scale[1] / ss[1]
                    } else {
                        1.0
                    };
                    // The axis the user is actually dragging drives both.
                    let uniform = if (fx - 1.0).abs() >= (fy - 1.0).abs() {
                        fx
                    } else {
                        fy
                    };
                    ph2d_editor::TransformSnapshot {
                        scale: [ss[0] * uniform, ss[1] * uniform],
                        ..new_t
                    }
                } else {
                    new_t
                };
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
                // Continuous rotation across the atan2 ±π seam (Enio
                // 2026-06-08). `compute_gizmo_transform` derives the angle from
                // a single `atan2(now) - atan2(start)`, which is confined to
                // (−2π, 2π] and JUMPS by 2π whenever the cursor crosses the −X
                // axis from the pivot — so full-turn rotation was impossible
                // and the sprite snapped backward at the seam. It bit WIDE
                // rectangles hardest: their corner handles start near ±π, so
                // the very first drag crossed the seam ("retângulos giram menos
                // e de forma inconsistente, sem dar voltas"); square handles
                // sit at ±45°/±135°, far from the seam, so they felt fine.
                // Unwrap the new rotation onto the 2π branch nearest the dragged
                // sprite's CURRENT rotation (last frame's written value) — the
                // per-frame cursor delta is small, so this accumulates smoothly
                // across unlimited turns. Applies to single- AND multi-select,
                // local AND global (delta_rot below flows from here).
                let new_t = if matches!(drag.kind, ph2d_editor::GizmoDragKind::Rotate) {
                    let current = gfx
                        .sim
                        .world()
                        .get::<Transform>(entity)
                        .map(|t| t.rotation)
                        .unwrap_or(new_t.rotation);
                    let mut r = new_t.rotation;
                    while r - current > std::f32::consts::PI {
                        r -= std::f32::consts::TAU;
                    }
                    while r - current < -std::f32::consts::PI {
                        r += std::f32::consts::TAU;
                    }
                    ph2d_editor::TransformSnapshot {
                        rotation: r,
                        ..new_t
                    }
                } else {
                    new_t
                };
                let delta_rot = new_t.rotation - drag.start_transform.rotation;
                let is_rot_or_scale = matches!(
                    drag.kind,
                    ph2d_editor::GizmoDragKind::Rotate
                        | ph2d_editor::GizmoDragKind::ScaleCorner { .. }
                        | ph2d_editor::GizmoDragKind::ScaleEdge { .. }
                );
                // ─── Multi-selection rotate / scale: ONE flat world-space
                // group transform for the dragged sprite AND every extra
                // (Onda 3, Enio 2026-06-08). Replaces the old primary-vs-
                // extras split, which had two defects:
                //
                //  (1) The PRIMARY's global orbit used its LOCAL translation
                //      (`drag.start_transform.translation`) against the WORLD
                //      pivot, while the extras used their WORLD position
                //      (`compose_snapshot`). A parented primary therefore
                //      orbited from the wrong point → "alguns gizmos ficam
                //      inconsistentes".
                //  (2) Every sprite's LOCAL rotation got `+= delta`, so a
                //      selected child of a selected parent ALSO inherited the
                //      parent's `+delta` → it rotated 2·delta ("a rotação dos
                //      filhos é incrementada pelo parentesco").
                //
                // Fix: compute each sprite's TARGET WORLD transform from its
                // OWN start world transform (rotate/scale by the group delta
                // around the global pivot, or in place for local-pivot mode),
                // then convert that target back to LOCAL against the parent's
                // CURRENT world transform. Writing ancestors before
                // descendants (depth-sorted) means a selected child reads its
                // selected parent's already-updated world this frame, so the
                // parent's rotation flows through inheritance exactly once —
                // the group transforms "como se não tivessem pais".
                if !self.group_drag_starts.is_empty() && is_rot_or_scale {
                    let is_global = matches!(drag.target, ph2d_editor::GizmoTarget::Global);
                    let pivot = drag.pivot_world;
                    // T1.3.5 cross-OS bit-identical.
                    let (sin_d, cos_d) = libm::sincosf(delta_rot);
                    // (bits, depth, start_local, start_parent_world) for the
                    // dragged primary + every extra. Depth = ChildOf chain
                    // length; sort ascending (tie-break on bits for HR-5
                    // determinism) so ancestors are written before descendants.
                    let depth_of = |bits: u64| -> u32 {
                        let mut d = 0u32;
                        let mut cur = gfx
                            .sim
                            .world()
                            .get::<ph2d_ecs::ChildOf>(ph2d_ecs::Entity::from_bits(bits))
                            .map(|c| c.parent());
                        while let Some(p) = cur {
                            d += 1;
                            cur = gfx
                                .sim
                                .world()
                                .get::<ph2d_ecs::ChildOf>(p)
                                .map(|c| c.parent());
                        }
                        d
                    };
                    let mut members: Vec<(
                        u64,
                        u32,
                        ph2d_editor::TransformSnapshot,
                        ph2d_editor::TransformSnapshot,
                    )> = Vec::with_capacity(self.group_drag_starts.len() + 1);
                    members.push((
                        drag.entity_bits,
                        depth_of(drag.entity_bits),
                        drag.start_transform,
                        drag.parent_world,
                    ));
                    for snap in self.group_drag_starts.iter() {
                        members.push((
                            snap.entity_bits,
                            depth_of(snap.entity_bits),
                            snap.start_transform,
                            snap.parent_world,
                        ));
                    }
                    members.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
                    for (bits, _depth, start_local, start_parent) in members {
                        let member = ph2d_ecs::Entity::from_bits(bits);
                        // START world = start_parent ∘ start_local.
                        let start_world = ph2d_editor::compose_snapshot(start_parent, start_local);
                        let target_rotation = start_world.rotation + delta_rot;
                        let target_scale = [
                            start_world.scale[0] * factor_x,
                            start_world.scale[1] * factor_y,
                        ];
                        let target_translation = if is_global {
                            // Orbit (+ scale) the world position around the
                            // shared global pivot.
                            let rel_x = start_world.translation[0] - pivot[0];
                            let rel_y = start_world.translation[1] - pivot[1];
                            let scaled_x = rel_x * factor_x;
                            let scaled_y = rel_y * factor_y;
                            [
                                pivot[0] + scaled_x * cos_d - scaled_y * sin_d,
                                pivot[1] + scaled_x * sin_d + scaled_y * cos_d,
                            ]
                        } else {
                            // Local pivot: each sprite turns about its own
                            // center → its world position is unchanged.
                            start_world.translation
                        };
                        // Convert the target WORLD transform back to LOCAL
                        // against the parent's CURRENT world (reflects any
                        // selected ancestor already written this frame).
                        let live_parent = ph2d_ecs::parent_world_transform(gfx.sim.world(), member);
                        let live_parent = ph2d_editor::TransformSnapshot {
                            translation: [live_parent.translation.x, live_parent.translation.y],
                            rotation: live_parent.rotation,
                            scale: [live_parent.scale.x, live_parent.scale.y],
                        };
                        let new_translation = ph2d_editor::world_translation_to_local(
                            live_parent,
                            target_translation,
                        );
                        let new_rotation = target_rotation - live_parent.rotation;
                        let psx = if live_parent.scale[0].abs() > 1e-6 {
                            live_parent.scale[0]
                        } else {
                            1.0
                        };
                        let psy = if live_parent.scale[1].abs() > 1e-6 {
                            live_parent.scale[1]
                        } else {
                            1.0
                        };
                        let new_scale = [target_scale[0] / psx, target_scale[1] / psy];
                        if let Some(mut t) = gfx.sim.world_mut().get_mut::<Transform>(member) {
                            t.translation =
                                ph2d_core::Vec2::new(new_translation[0], new_translation[1]);
                            t.rotation = new_rotation;
                            t.scale = ph2d_core::Vec2::new(new_scale[0], new_scale[1]);
                        }
                    }
                } else {
                    // Single-selection (any kind) + multi-selection TRANSLATE.
                    // Multi rotate/scale goes through the unified branch above,
                    // so the primary's translation here is always
                    // `new_t.translation` (the old in_local_multi /
                    // in_global_xform cases only ever fired for multi
                    // rotate/scale, now handled above).
                    if let Some(mut t) = gfx.sim.world_mut().get_mut::<Transform>(entity) {
                        t.translation =
                            ph2d_core::Vec2::new(new_t.translation[0], new_t.translation[1]);
                        t.rotation = new_t.rotation;
                        t.scale = ph2d_core::Vec2::new(new_t.scale[0], new_t.scale[1]);
                    }
                    // Multi-selection TRANSLATE: rigid-body shift — add the
                    // dragged primary's world delta to every extra's start
                    // translation, converted into each extra's LOCAL frame via
                    // inverse-parent (Enio 2026-05-26: child of a rotated
                    // parent in the group moved along the local axis, not
                    // world). Rotate/scale never reach here (handled by the
                    // unified branch above); MovePivot stays primary-only (its
                    // own branch writes Sprite.anchor).
                    if !self.group_drag_starts.is_empty()
                        && matches!(drag.kind, ph2d_editor::GizmoDragKind::Translate)
                    {
                        let dx = new_t.translation[0] - drag.start_transform.translation[0];
                        let dy = new_t.translation[1] - drag.start_transform.translation[1];
                        for snap in self.group_drag_starts.iter().copied() {
                            let extra_entity = ph2d_ecs::Entity::from_bits(snap.entity_bits);
                            let st = snap.start_transform;
                            let [dx_l, dy_l] =
                                ph2d_editor::world_delta_to_local(snap.parent_world, dx, dy);
                            if let Some(mut t) =
                                gfx.sim.world_mut().get_mut::<Transform>(extra_entity)
                            {
                                t.translation = ph2d_core::Vec2::new(
                                    st.translation[0] + dx_l,
                                    st.translation[1] + dy_l,
                                );
                                t.rotation = st.rotation;
                                t.scale = ph2d_core::Vec2::new(st.scale[0], st.scale[1]);
                            }
                        }
                    }
                }
            }
        }
        // Anchor-dot drag: a physics joint's `Transform` is its authored world
        // pivot, so a Translate on a joint entity REPOSITIONS the anchor. Mark it
        // un-anchored each Move so the next reconcile re-derives the body-local
        // anchors from the dragged pivot — the pin tracks the drag. Only a joint
        // carries a `PhysicsJoint`; a body or sprite Translate is untouched.
        if matches!(drag.kind, ph2d_editor::GizmoDragKind::Translate)
            && let Some(gfx) = self.gfx.as_mut()
            && let Some(mut j) = gfx
                .sim
                .world_mut()
                .get_mut::<ph2d_physics_ecs::PhysicsJoint>(ph2d_ecs::Entity::from_bits(
                    drag.entity_bits,
                ))
        {
            j.anchored = false;
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
        // T1.3.5 cross-OS bit-identical.
        let (sin_r, cos_r) = libm::sincosf(drag.start_transform.rotation);
        let qc = drag.pivot_world;
        Some([
            qc[0] + local_x * cos_r - local_y * sin_r,
            qc[1] + local_x * sin_r + local_y * cos_r,
        ])
    }
}
