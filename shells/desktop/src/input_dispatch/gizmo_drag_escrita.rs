//! **O arrasto do gizmo, a ESCRITA** — ramos do `advance_gizmo_drag` ([`super`]), corpos verbatim pela mesma ordem: o
//! grupo que roda e escala no mundo, a moldura, o fluxo, a peça simples. ⚠️ A cadeia `grupo / moldura / fluxo /
//! simples` fica CONTÍGUA num ramo só: dois gates leem as janelas entre os `} else if` dela.

use crate::Transform;

/// **Os valores que a escrita do arrasto lê**, com os nomes exactos das variáveis do `advance_gizmo_drag` (o ramo
/// desestrutura-o na 1.ª linha, e o corpo mudou-se sem uma letra diferente): oito `Copy` na pilha, nada aloca.
#[derive(Clone, Copy)]
pub(super) struct EscritaDoGizmo {
    pub(super) drag: ph2d_editor_core::GizmoDragState,
    pub(super) entity: ph2d_ecs::Entity,
    pub(super) new_t: ph2d_editor_core::TransformSnapshot,
    pub(super) is_scale: bool,
    pub(super) factor_x: f32,
    pub(super) factor_y: f32,
    pub(super) delta_rot: f32,
    pub(super) is_rot_or_scale: bool,
}

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
            // A conta de cada membro (e o skew sob a escala de mundo) é a `gizmo::group_member_pose`.
            let movimento = ph2d_editor_core::gizmo::GroupMotion {
                pivot: drag.pivot_world,
                delta_rot,
                factor: [factor_x, factor_y],
                global: matches!(drag.target, ph2d_editor_core::GizmoTarget::Global),
            };
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
                // O pai de AGORA (um antepassado seleccionado já pode ter sido escrito neste quadro).
                let live_parent = ph2d_ecs::parent_world_transform(gfx.sim.world(), member);
                let live_parent = ph2d_editor_core::TransformSnapshot {
                    translation: [live_parent.translation.x, live_parent.translation.y],
                    rotation: live_parent.rotation,
                    scale: [live_parent.scale.x, live_parent.scale.y],
                };
                let Some(&vivo) = gfx.sim.world().get::<Transform>(member) else {
                    continue;
                };
                let skew = [vivo.skew_x, vivo.skew_y];
                let mundo = ph2d_ecs::world_skew(gfx.sim.world(), member);
                let pose = ph2d_editor_core::gizmo::group_member_pose(
                    start_local,
                    start_parent,
                    live_parent,
                    [mundo[0] - skew[0], mundo[1] - skew[1]],
                    (
                        ph2d_editor_core::TransformSnapshot {
                            translation: [vivo.translation.x, vivo.translation.y],
                            rotation: vivo.rotation,
                            scale: [vivo.scale.x, vivo.scale.y],
                        },
                        skew,
                    ),
                    movimento,
                );
                if let Some(mut t) = gfx.sim.world_mut().get_mut::<Transform>(member) {
                    t.translation = ph2d_core::Vec2::new(pose.translation[0], pose.translation[1]);
                    t.rotation = pose.rotation;
                    t.scale = ph2d_core::Vec2::new(pose.scale[0], pose.scale[1]);
                    t.skew_x = pose.skew[0];
                    t.skew_y = pose.skew[1];
                }
            }
        }
    }

    /// A escrita do arrasto: o grupo, a moldura que redimensiona sem escalar, o fluxo que reordena, ou a peça simples
    /// (confinada à folha) com os extras de um Translate.
    pub(super) fn ramo_gizmo_escrita(&mut self, escrita: EscritaDoGizmo) {
        let EscritaDoGizmo {
            drag,
            entity,
            new_t,
            is_scale,
            factor_x,
            factor_y,
            delta_rot,
            is_rot_or_scale,
        } = escrita;
        if let Some(gfx) = self.gfx.as_mut() {
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
                self.ramo_gizmo_escrita_grupo(drag, delta_rot, factor_x, factor_y);
            } else if is_scale
                && let Some(start) = self.frame_resize_start.as_ref()
                && start.is_for(drag.entity_bits)
            {
                // **UMA MOLDURA REDIMENSIONA; ela não ESCALA** (corolário do W3).
                //
                // ⚠️ E o `Transform` NÃO é escrito — é a metade que importa. A pose de um pai é
                // herdada por todo descendente (é isso que um grafo de cena é), então escrevê-la
                // ESTICA os filhos: a tipografia achata e a regra de âncora nunca corre, porque
                // a moldura não mudou de CAIXA, mudou de ESCALA. O `W`/`H` do painel já fazia o
                // certo; isto leva a alça à mesma porta.
                //
                // A razão é ABSOLUTA contra o `start_transform` (o gizmo recomputa-a a cada
                // `CursorMoved`), e é por isso que a porta repõe o instantâneo antes de escalar.
                let (ssx, ssy) = (drag.start_transform.scale[0], drag.start_transform.scale[1]);
                let fx = if ssx.abs() > 1e-6 {
                    f64::from(new_t.scale[0] / ssx)
                } else {
                    1.0
                };
                let fy = if ssy.abs() > 1e-6 {
                    f64::from(new_t.scale[1] / ssy)
                } else {
                    1.0
                };
                crate::vec_frame_resize::apply(
                    &mut gfx.sim,
                    &mut gfx.vec_scene,
                    start,
                    drag.pivot_world,
                    fx,
                    fy,
                );
            } else if matches!(drag.kind, ph2d_editor_core::GizmoDragKind::Translate)
                && crate::layout_reorder::flow_parent(&gfx.sim, entity).is_some()
            {
                // **DENTRO de um fluxo, arrastar é REORDENAR** (ADR-0153, corolário).
                //
                // ⚠️ E o `Transform` NÃO é escrito, o que é a metade que importa: a posição de
                // um filho colocado é derivada por frame, então a escrita seria invisível — e
                // ainda assim contaria, porque o undo deste editor regista por DIFF do mundo
                // ECS. O artista teria a forma a saltar de volta para a fila **e** um passo de
                // undo por cima.
                //
                // O cursor é lido em MUNDO porque a régua que decide o slot está em mundo (o
                // passe publica-a lá); converter de novo seria a segunda resposta a *"onde
                // está o dedo?"*.
                let window_size = crate::field_gizmo_host::scene_window_of(gfx);
                let cursor = gfx.camera.screen_to_world(drag.cursor_screen, window_size);
                crate::layout_reorder::drop_at(&mut gfx.sim, &self.layout_live, entity, cursor);
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
                // **UMA PEÇA NÃO SAI DA FOLHA PELO ARRASTO** (Enio 2026-08-19).
                //
                // ⚠️ Confina-se DEPOIS de escrever, e não antes: o que o gizmo calcula é para
                // onde o dedo aponta, e essa é a resposta certa à pergunta dele. Torcer a
                // entrada faria a peça arrastar-se com um desvio — o dedo num sítio e a peça
                // noutro — enquanto corrigir a saída faz o que o artista lê: ela acompanha o
                // dedo e **encosta** na borda.
                //
                // ⚠️ E vale para rotação e escala também, não só para Translate: crescer uma
                // peça encostada à borda empurra-a para fora tanto quanto arrastá-la. A porta
                // é a mesma; ela não faz nada quando a entidade não é filha de uma folha.
                ph2d_sheet_bounds::confine(&mut gfx.sim, entity);
                // Multi-selection TRANSLATE: rigid-body shift — add the
                // dragged primary's world delta to every extra's start
                // translation, converted into each extra's LOCAL frame via
                // inverse-parent (Enio 2026-05-26: child of a rotated
                // parent in the group moved along the local axis, not
                // world). Rotate/scale never reach here (handled by the
                // unified branch above); MovePivot stays primary-only (its
                // own branch writes Sprite.anchor).
                if !self.group_drag_starts.is_empty()
                    && matches!(drag.kind, ph2d_editor_core::GizmoDragKind::Translate)
                {
                    let dx = new_t.translation[0] - drag.start_transform.translation[0];
                    let dy = new_t.translation[1] - drag.start_transform.translation[1];
                    for snap in self.group_drag_starts.iter().copied() {
                        let extra_entity = ph2d_ecs::Entity::from_bits(snap.entity_bits);
                        let st = snap.start_transform;
                        let [dx_l, dy_l] =
                            ph2d_editor_core::world_delta_to_local(snap.parent_world, dx, dy);
                        if let Some(mut t) = gfx.sim.world_mut().get_mut::<Transform>(extra_entity)
                        {
                            t.translation = ph2d_core::Vec2::new(
                                st.translation[0] + dx_l,
                                st.translation[1] + dy_l,
                            );
                            t.rotation = st.rotation;
                            t.scale = ph2d_core::Vec2::new(st.scale[0], st.scale[1]);
                        }
                        // ⚠️ **Cada extra confina-se sozinho.** Uma seleção múltipla pode ter
                        // peças de folhas DIFERENTES (ou nenhuma): a fronteira é do pai de
                        // cada uma, não do arrasto. Confinar o grupo como bloco rígido pararia
                        // as cinco porque uma chegou à borda — e as outras quatro não têm nada
                        // a ver com essa borda.
                        ph2d_sheet_bounds::confine(&mut gfx.sim, extra_entity);
                    }
                }
            }
        }
    }
}
