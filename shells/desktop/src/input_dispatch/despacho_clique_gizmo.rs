//! **O clique, o GIZMO** — ramos do `on_mouse_input` ([`super`]): o toque com modificador, o que o hit diz
//! (alça, traço aberto, arte Flip, Translate chaveado), o pivô, as âncoras de junta, a roldana, e a cadeia
//! que decide entre a alça e o pick de canvas. Os corpos MUDARAM-SE verbatim (`line/input-dispatch`,
//! 2026-09-13) e correm no sítio da chamada, pela mesma ordem.
//!
//! ⚠️ Cada ramo re-empresta `gfx`/`hero` com o MESMO guarda do braço de onde saiu (`if let … && let …`), e
//! chama o seguinte só depois do último uso deles (NLL) — é isso que deixa o texto correr em sequência.

use super::*;

impl crate::App {
    /// A alça (escala/rotação) de um alvo: a recusa do trancado, a pose de partida, o pivô e o grupo a semear.
    pub(super) fn ramo_gizmo_alca(
        &mut self,
        evt: PointerEvent,
        gkind: ph2d_editor_core::GizmoDragKind,
        entity_bits: u64,
        effective_target: ph2d_editor_core::GizmoTarget,
    ) -> bool {
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
        {
            let entity = ph2d_ecs::Entity::from_bits(entity_bits);
            // 2026-05-26 — bloqueia drag se entidade tem
            // `Locked` OU ancestral tem `GroupedChildren`.
            if ph2d_ecs::is_locked_for_edit(gfx.sim.world(), entity) {
                return true;
            }
            let window_size = gfx.surface.size();
            let start_world = gfx.camera.screen_to_world((evt.x, evt.y), window_size);
            // ⚠️ A semeadura do grupo saiu de DENTRO deste bloco (W-JG): ela
            // precisa de `&mut gfx.sim` (o `jointed_group` monta queries) e o
            // `t` abaixo empresta o mundo imutavelmente. Sem drag aberto não
            // há grupo a semear, daí o sinalizador.
            let mut opened_drag = false;
            if let Some(t) = gfx.sim.world().get::<Transform>(entity) {
                let snap = ph2d_editor_core::TransformSnapshot {
                    translation: [t.translation.x, t.translation.y],
                    rotation: t.rotation,
                    scale: [t.scale.x, t.scale.y],
                };
                // Enio 2026-05-26 fix: capture parent's world
                // transform so compute_gizmo_transform can
                // unrotate/unscale the delta before writing
                // back to the entity's LOCAL Transform.
                let pw = ph2d_ecs::parent_world_transform(gfx.sim.world(), entity);
                let parent_world = ph2d_editor_core::TransformSnapshot {
                    translation: [pw.translation.x, pw.translation.y],
                    rotation: pw.rotation,
                    scale: [pw.scale.x, pw.scale.y],
                };
                // ⚠️ **As DUAS metades.** O `.1` sozinho descartava o
                // `anchor` — *onde a caixa está em relação ao pivô* — e
                // `anchor_pivot_world` então fixava um ponto deslocado de
                // `anchor ⊙ scale`. Ficou invisível enquanto toda caixa
                // estava centrada no próprio pivô (o `settle_origins`
                // garante isso para um path comum, e uma Live Shape nasce
                // assim), e apareceu quando redimensionar uma moldura passou
                // a deixá-la fora do centro: a borda oposta caminhava
                // 150 → 162,5 → 175 em três arrastos.
                let (sprite_anchor_intrinsic, sprite_half_intrinsic) =
                    gizmo_anchor_half(&gfx.sim, &gfx.vec_scene, &gfx.flip, entity);
                // Onda 2C: pivot world depends on target.
                // PrimaryIndividual / ExtraIndividual use the
                // sprite's own anchor (transforms local to it).
                // Global overrides pivot to the global bbox
                // center so group transforms rotate/scale every
                // sprite around a single shared point.
                let pivot = if let ph2d_editor_core::GizmoTarget::Global = effective_target
                    && let Some(gv) = hero.gizmo.global_view.as_ref()
                {
                    [
                        (gv.bbox_min_world[0] + gv.bbox_max_world[0]) * 0.5,
                        (gv.bbox_min_world[1] + gv.bbox_max_world[1]) * 0.5,
                    ]
                } else {
                    // Composição parent×local pra que o
                    // pivot world seja correto mesmo com pai
                    // rotacionado/escalonado (Enio 2026-05-26
                    // fix: child de pai rotacionado tinha
                    // pivot calculado como root).
                    let world_snap = ph2d_editor_core::compose_snapshot(parent_world, snap);
                    // ⚠️ **O CANTO oposto, sempre — mesmo com Ctrl premido agora.**
                    //
                    // A âncora deste gizmo é VIVA (`ph2d_editor_core::live_anchor`, chamada
                    // a cada movimento): o Ctrl deste frame é que decide se o ponto
                    // fixo é o canto ou o centro. O centro é derivável da pose de
                    // partida a qualquer momento; o CANTO não é derivável de nada
                    // depois do pen-down, porque só aqui se sabe QUAL alça foi pega.
                    // Guardar o centro aqui tornaria soltar o Ctrl no meio do arrasto
                    // uma operação sem volta — e o modificador deixaria de ser vivo
                    // justamente na direção em que o artista o larga.
                    ph2d_editor_core::anchor_pivot_world(
                        gkind,
                        sprite_anchor_intrinsic,
                        sprite_half_intrinsic,
                        world_snap,
                        false,
                    )
                };
                // Onda 2 polish: capture the global view at
                // drag start so snapshots::publish can keep
                // the global gizmo's visual orientation /
                // scale in lockstep with the live group
                // transform (otherwise it would be the
                // axis-aligned union of rotated sprites,
                // which grows during rotation instead of
                // rotating).
                if matches!(effective_target, ph2d_editor_core::GizmoTarget::Global) {
                    hero.gizmo.global_view_start = hero.gizmo.global_view;
                } else {
                    hero.gizmo.global_view_start = None;
                }
                hero.gizmo.drag = Some(ph2d_editor_core::GizmoDragState {
                    kind: gkind,
                    entity_bits,
                    start_screen: (evt.x, evt.y),
                    cursor_screen: (evt.x, evt.y),
                    start_transform: snap,
                    pivot_world: pivot,
                    start_cursor_world: start_world,
                    sprite_half_intrinsic,
                    // O modificador VIVO o reescreve a cada movimento; nascer em
                    // `false` deixa a pose de partida intacta até o 1º `CursorMoved`.
                    anchor_is_center: false,
                    target: effective_target,
                    parent_world,
                    turns: 0,
                });
                opened_drag = true;
            }
            if opened_drag {
                // Onda 1 + 2C.4: snapshot every OTHER selected
                // sprite's full start_transform so
                // advance_gizmo_drag can apply translate /
                // local-scale / local-rotate / global-scale /
                // global-rotate to the whole group. Captured
                // for ANY drag kind that touches multi-select
                // (Translate / Scale / Rotate) so the math
                // branches can fire uniformly later.
                //
                // W-JG: e, num Translate em repouso **com ALT**, o
                // **rig articulado** do conjunto entra junto — a
                // MESMA porta que o pick de canvas usa
                // (`ph2d_app_physics::joint_rig_drag`), porque duas cópias da
                // regra é como arrastar pela alça passaria a
                // carregar a corrente e arrastar pelo corpo, não.
                let selected: Vec<u64> = hero.gizmo.iter_selected().collect();
                let carry_reach = if matches!(gkind, ph2d_editor_core::GizmoDragKind::Translate)
                    && !self.playhead.is_playing()
                {
                    self.physics
                        .interaction
                        .joint
                        .drag_reach(self.modifiers.alt_key())
                } else {
                    None
                };
                ph2d_app_physics::joint_rig_drag::seed_group_drag_starts(
                    &mut self.group_drag_starts,
                    &mut gfx.sim,
                    entity_bits,
                    &selected,
                    carry_reach,
                );
            }
        }
        false
    }
}
