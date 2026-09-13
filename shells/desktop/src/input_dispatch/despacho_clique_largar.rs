//! **O clique, o LARGAR do botão primário** — ramos do `on_mouse_input` ([`super`]): a borracha que resolve a
//! seleção, o clique que colapsa a multi-seleção, o fim do arrasto do gizmo e a solda das pontas. Os corpos
//! MUDARAM-SE verbatim (`line/input-dispatch`, 2026-09-13) e correm no sítio da chamada, pela mesma ordem.
//!
//! ⚠️ Cada ramo re-empresta `gfx`/`hero` com o MESMO guarda do braço de onde saiu (`if let … && let …`): sem
//! eles o braço não fazia nada, e o ramo também não. A chamada seguinte só é legal porque o empréstimo MORRE
//! antes dela (NLL) — nada do ramo volta a ler `gfx`/`hero` depois de chamar o próximo.

use super::*;

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

    /// O largar: fecha a âncora de junta, resolve a borracha e o clique que colapsa a multi-seleção, e segue
    /// para o fim do arrasto.
    pub(super) fn ramo_gizmo_largar(&mut self, evt: PointerEvent) {
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
        {
            // W-J2: close any joint-anchor drag. The write already landed
            // on the last Move; `post_frame_undo` records the whole
            // gesture as one global step once the button is up. Written
            // as a field (not a `&mut self` method) because `gfx`/`hero`
            // are borrowed from `self.gfx` for the whole of this arm —
            // disjoint fields are fine, a second `&mut self` is not.
            self.physics.joint_anchor_drag = None;
            // Fase 0f: resolve the rubber-band rect — pick every
            // sprite whose world bbox intersects, then apply
            // replace or add depending on `add_mode` (Shift held
            // at Down). A click that didn't drift more than 4 px
            // is treated as a bare click on empty: clear
            // selection if !add_mode, else preserve.
            if let Some(rb) = self.rubber_band.take() {
                let dx = rb.current_screen.0 - rb.anchor_screen.0;
                let dy = rb.current_screen.1 - rb.anchor_screen.1;
                let moved = (dx * dx + dy * dy) > 16.0; // > 4 px
                if moved {
                    let window_size = gfx.surface.size();
                    let world_a = gfx.camera.screen_to_world(rb.anchor_screen, window_size);
                    let world_b = gfx.camera.screen_to_world(rb.current_screen, window_size);
                    let rmin = [world_a[0].min(world_b[0]), world_a[1].min(world_b[1])];
                    let rmax = [world_a[0].max(world_b[0]), world_a[1].max(world_b[1])];
                    let vec_view = ph2d_vec_entities::entities::view_state_for_pick(
                        &gfx.sim,
                        &self.vec.entities,
                        &self.vec.view_derived,
                    );
                    let mut bits = crate::vec_gizmo_view::pick_in_world_rect(
                        &gfx.sim,
                        &gfx.vec_scene,
                        &self.vec.live_drawn,
                        &vec_view,
                        &self.vec.entities,
                        rmin,
                        rmax,
                    );
                    // ADR-0114/ADR-0111: o marquee também pega objetos Flip pela
                    // bbox de mundo.
                    bits.extend(ph2d_app_flip::gizmo_view::pick_in_world_rect(
                        &gfx.sim,
                        &gfx.flip,
                        &self.flip_state.entities,
                        rmin,
                        rmax,
                    ));
                    bits.extend(ph2d_render::pick_sprites_in_world_rect(
                        gfx.present.world_mut(),
                        rmin,
                        rmax,
                    ));
                    // ⚠️ **A TRAVA DO PAINTER também fecha o LAÇO.** O guarda do pick
                    // trata do clique; a borracha é a outra porta por onde a multi-seleção
                    // entra, e o Enio nomeou-a: *"não permita a seleção de múltiplas
                    // imagens se o painter está ativo"*. Uma trava que só cobre o gesto
                    // óbvio ensina o artista a usar o outro.
                    if ph2d_app_painter::painter_lock::locked_entity(&gfx.tools, hero).is_some() {
                        gfx.toasts
                            .push(Toast::warning(ph2d_app_painter::painter_lock::REFUSAL));
                        self.pending_ui_sound = Some(crate::ui_sound::UiSound::Refuse);
                    } else {
                        if !rb.add_mode {
                            hero.gizmo.clear_all_selection();
                        }
                        for b in bits {
                            hero.gizmo.add_to_selection(b);
                        }
                    }
                    // Sync the panel header label to the new
                    // primary (Fase 0e parity).
                    let primary = hero.gizmo.selection;
                    if let Some(entry) = resolve_live_entry(gfx.hero_live.as_ref(), primary) {
                        hero.selection = Some(ph2d_editor_core::HeroSelection {
                            label: entry.name.clone(),
                            kind: entry.badge.clone().unwrap_or_else(|| "ENT".to_string()),
                            world_pos: (0.0, 0.0),
                        });
                    } else if primary.is_none() {
                        hero.selection = None;
                    }
                    self.title_dirty = true;
                } else if !rb.add_mode {
                    // Bare click on empty = clear selection.
                    hero.gizmo.clear_all_selection();
                    hero.selection = None;
                    self.title_dirty = true;
                }
            }
            // Onda 2 hotfix: resolve a pending click-vs-drag
            // decision. `pending_single_replace` is Some when
            // the user Down'd on a multi-selected sprite. If
            // the cursor stayed within ~4 px of the Down point
            // until now (a click, not a drag), collapse the
            // multi-selection to just that sprite. If it moved
            // past the threshold, the open Translate drag has
            // already group-translated the selection; just
            // clear the pending state.
            if let Some((bits, (dx0, dy0))) = self.pending_single_replace.take() {
                let dx = evt.x - dx0;
                let dy = evt.y - dy0;
                // 12 px tolerance — trackpads have micro
                // tremor and acceleration that can move
                // the cursor a few px even on what feels
                // like a stationary click.
                if (dx * dx + dy * dy) <= 144.0 {
                    hero.gizmo.replace_selection(Some(bits));
                    // Sync the panel header label to the new
                    // primary so the Hierarchy highlight
                    // matches the canvas immediately.
                    if let Some(entry) = resolve_live_entry(gfx.hero_live.as_ref(), Some(bits)) {
                        hero.selection = Some(ph2d_editor_core::HeroSelection {
                            label: entry.name.clone(),
                            kind: entry.badge.clone().unwrap_or_else(|| "ENT".to_string()),
                            world_pos: (0.0, 0.0),
                        });
                    }
                    self.title_dirty = true;
                }
            }
            self.ramo_gizmo_largar_fim();
        }
    }
}
