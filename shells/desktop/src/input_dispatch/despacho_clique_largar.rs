//! **O clique, o LARGAR do botão primário** — ramos do `on_mouse_input` ([`super`]): a borracha que resolve a
//! seleção, o clique que colapsa a multi-seleção, o fim do arrasto do gizmo e a solda das pontas. Os corpos
//! MUDARAM-SE verbatim (`line/input-dispatch`, 2026-09-13) e correm no sítio da chamada, pela mesma ordem.
//!
//! ⚠️ Cada ramo re-empresta `gfx`/`hero` com o MESMO guarda do braço de onde saiu (`if let … && let …`): sem
//! eles o braço não fazia nada, e o ramo também não. A chamada seguinte só é legal porque o empréstimo MORRE
//! antes dela (NLL) — nada do ramo volta a ler `gfx`/`hero` depois de chamar o próximo.

impl crate::App {
    /// O fim do arrasto do gizmo: a solda das pontas abertas ao mover e a limpeza dos instantâneos do gesto.
    pub(super) fn ramo_gizmo_largar_fim(&mut self) {
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
        {
            // Drop the drag — Transform is already committed
            // up to the latest Move position.
            let ended_drag = hero.gizmo.drag;
            hero.gizmo.drag = None;
            // Snap vetorial ao MOVER (Enio 2026-07-10): um Translate que
            // acabou de reposicionar formas vetoriais ABERTAS solda as pontas
            // que ficaram perto de nós vizinhos — o mesmo weld da criação. A
            // forma movida cede o endpoint; a vizinha é sacrossanta. Formas
            // fechadas não têm endpoints, então passam intactas.
            // Qualquer manipulação da forma (mover / escalar / rotacionar) que
            // aproxime as pontas deve soldar — só o MovePivot (que mexe no pivô,
            // não na forma) fica de fora. O `rigid_snap_delta` só desliza para o
            // encaixe, então serve de ajuste fino pós-scale/rotate também.
            if ended_drag
                .is_some_and(|d| !matches!(d.kind, ph2d_editor_core::GizmoDragKind::MovePivot))
            {
                let mut moved_bits = vec![ended_drag.unwrap().entity_bits];
                moved_bits.extend(self.group_drag_starts.iter().map(|s| s.entity_bits));
                let moved_ids: Vec<_> = moved_bits
                    .iter()
                    .filter_map(|&b| {
                        gfx.sim
                            .world()
                            .get::<ph2d_ecs::VecPathRef>(ph2d_ecs::Entity::from_bits(b))
                            .map(|v| v.0)
                    })
                    .collect();
                if !moved_ids.is_empty() {
                    let fill = self.vec.pen.style().fill;
                    let fill_on_close = (fill.a != 0).then(|| ph2d_vec_scene::Paint::solid(fill));
                    let win = gfx.surface.size();
                    let tol = crate::vec_gizmo_view::stroke_hit_r(&gfx.camera, win) * 1.5;
                    // O alinhamento (bordas/centros/vértices + grade, com guias)
                    // já foi aplicado ao vivo pelo motor de snap durante o
                    // arraste. No release resta só a FUSÃO das formas abertas:
                    // encaixa a ponta num endpoint vizinho (RÍGIDO, sem
                    // distorcer) e solda.
                    for id in moved_ids {
                        if !gfx.vec_scene.paths().iter().any(|p| p.id == id) {
                            continue;
                        }
                        let Some(bits) = self.vec.entities.get(&id).copied() else {
                            continue;
                        };
                        if gfx.vec_scene.paths().iter().any(|p| p.id == id && p.closed) {
                            continue; // fechada não funde
                        }
                        let xforms =
                            ph2d_vec_entities::transform::build(&gfx.sim, &self.vec.entities);
                        if let Some(rd) = gfx.vec_scene.rigid_snap_delta(id, &xforms, tol) {
                            crate::vec_snap::slide_entity_world(&mut gfx.sim, bits, rd);
                        }
                        let xforms =
                            ph2d_vec_entities::transform::build(&gfx.sim, &self.vec.entities);
                        gfx.vec_scene
                            .weld_new_shape(id, &xforms, tol, fill_on_close.clone());
                    }
                }
                // Fim do gesto: apaga as guias de alinhamento.
                self.vec.snap_guides.clear();
            }
            // Onda 1: release the group-translate snapshot so
            // the next single-select drag doesn't accidentally
            // pull stale extras along.
            self.group_drag_starts.clear();
            // O instantâneo da moldura redimensionada descreve um gesto que acabou. A
            // limpeza do `advance_gizmo_drag` só corre num `CursorMoved`, e soltar e voltar
            // a pegar não passa necessariamente por um.
            self.frame_resize_start = None;
            // Onda 2 polish: release the global drag-start view
            // so snapshots::publish reverts to the live-union
            // computation for the next frame.
            hero.gizmo.global_view_start = None;
        }
    }
}
