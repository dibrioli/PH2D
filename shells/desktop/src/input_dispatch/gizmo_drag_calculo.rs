//! **O arrasto do gizmo, o CÁLCULO** — ramos do `advance_gizmo_drag` ([`super`]): o pivô que se muda
//! (`MovePivot`, com o snap do Ctrl), a câmera, os modificadores e o snap que dão o `new_t`, e o que se deriva
//! dele — a escala uniforme de uma selecção rodada, os factores e a rotação contínua pela costura do `atan2`. Os
//! corpos MUDARAM-SE verbatim (`line/input-dispatch`, 2026-09-13) e correm no sítio da chamada, pela mesma ordem.

use crate::Transform;

impl crate::App {
    /// Do `new_t` aos valores da escrita: a escala uniforme de uma selecção rodada, os factores, a rotação que atravessa
    /// a costura do `atan2` sem saltar, o `delta_rot` — e a escrita.
    pub(super) fn ramo_gizmo_factores(
        &mut self,
        drag: ph2d_editor_core::GizmoDragState,
        entity: ph2d_ecs::Entity,
        new_t: ph2d_editor_core::TransformSnapshot,
        is_scale: bool,
    ) {
        if let Some(gfx) = self.gfx.as_mut() {
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
                    ph2d_editor_core::GizmoDragKind::ScaleCorner { .. }
                        | ph2d_editor_core::GizmoDragKind::ScaleEdge { .. }
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
                ph2d_editor_core::TransformSnapshot {
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
            let new_t = if matches!(drag.kind, ph2d_editor_core::GizmoDragKind::Rotate) {
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
                ph2d_editor_core::TransformSnapshot {
                    rotation: r,
                    ..new_t
                }
            } else {
                new_t
            };
            let delta_rot = new_t.rotation - drag.start_transform.rotation;
            let is_rot_or_scale = matches!(
                drag.kind,
                ph2d_editor_core::GizmoDragKind::Rotate
                    | ph2d_editor_core::GizmoDragKind::ScaleCorner { .. }
                    | ph2d_editor_core::GizmoDragKind::ScaleEdge { .. }
            );
            self.ramo_gizmo_escrita(crate::input_dispatch::gizmo_drag::escrita::EscritaDoGizmo {
                drag,
                entity,
                new_t,
                is_scale,
                factor_x,
                factor_y,
                delta_rot,
                is_rot_or_scale,
            });
        }
    }
}
