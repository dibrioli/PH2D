//! **O arrasto do gizmo, a ESCRITA** — ramos do `advance_gizmo_drag` ([`super`]): o grupo que roda e
//! escala como um corpo só no mundo, a moldura que redimensiona, o fluxo que reordena, e a peça simples com os
//! extras que a seguem. Os corpos MUDARAM-SE verbatim (`line/input-dispatch`, 2026-09-13) e correm no sítio da
//! chamada, pela mesma ordem.
//!
//! ⚠️ A cadeia `grupo / moldura / fluxo / simples` fica CONTÍGUA num ramo só: dois gates leem as janelas entre
//! os `} else if` dela, e partir a cadeia mudaria o que eles medem.

use crate::Transform;

impl crate::App {
    /// Multi-selecção a rodar ou escalar: cada membro vai para o seu alvo de MUNDO a partir do próprio início, e volta
    /// a LOCAL contra o pai de agora — ancestrais escritos antes dos descendentes.
    pub(super) fn ramo_gizmo_escrita_grupo(
        &mut self,
        drag: ph2d_editor_core::GizmoDragState,
        delta_rot: f32,
        factor_x: f32,
        factor_y: f32,
    ) {
        if let Some(gfx) = self.gfx.as_mut() {
            let is_global = matches!(drag.target, ph2d_editor_core::GizmoTarget::Global);
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
                ph2d_editor_core::TransformSnapshot,
                ph2d_editor_core::TransformSnapshot,
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
                let start_world = ph2d_editor_core::compose_snapshot(start_parent, start_local);
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
                let live_parent = ph2d_editor_core::TransformSnapshot {
                    translation: [live_parent.translation.x, live_parent.translation.y],
                    rotation: live_parent.rotation,
                    scale: [live_parent.scale.x, live_parent.scale.y],
                };
                let new_translation =
                    ph2d_editor_core::world_translation_to_local(live_parent, target_translation);
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
                    t.translation = ph2d_core::Vec2::new(new_translation[0], new_translation[1]);
                    t.rotation = new_rotation;
                    t.scale = ph2d_core::Vec2::new(new_scale[0], new_scale[1]);
                }
            }
        }
    }
}
