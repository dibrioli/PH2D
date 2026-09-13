//! **O clique, o PICK de canvas** — ramos do `on_mouse_input` ([`super`]): o que está sob o dedo (vetor, Flip,
//! sprites, pela porta única), o ciclo de cliques repetidos, a seleção com e sem modificador, a trava do
//! Painter, e o arrasto que o pick abre. Os corpos MUDARAM-SE verbatim (`line/input-dispatch`, 2026-09-13) e
//! correm no sítio da chamada, pela mesma ordem.
//!
//! ⚠️ Cada ramo re-empresta `gfx`/`hero` com o MESMO guarda do braço de onde saiu (`if let … && let …`), e
//! chama o seguinte só depois do último uso deles (NLL) — é isso que deixa o texto correr em sequência.

use super::*;

impl crate::App {
    /// O arrasto que o pick abre (a mão, a pose, o Translate e o rig articulado) e o rótulo da seleção.
    pub(super) fn ramo_gizmo_pick_arrasta(
        &mut self,
        evt: PointerEvent,
        picked: Option<u64>,
        is_modifier_click: bool,
        world_pos: [f32; 2],
    ) {
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
        {
            if let Some(bits) = picked
                && !is_modifier_click
            {
                let entity = ph2d_ecs::Entity::from_bits(bits);
                let locked = ph2d_ecs::is_locked_for_edit(gfx.sim.world(), entity);
                // W-Grab: com o relógio ANDANDO e a física armada, um
                // press num corpo dinâmico é a **MÃO**, não um arrasto
                // de autoria — o mesmo relógio que decide se o Alt
                // carrega o rig (W-JG, condição 2) decide isto, do
                // outro lado. Pegou ⇒ nenhum arrasto de gizmo abre
                // (`ph2d_app_physics::body_grab` explica por que os dois juntos
                // seriam um gesto inerte cavalgando um vivo).
                // W-JointTools: qual gesto de POSE este press abre,
                // se algum. Uma pergunta só, feita à porta que também
                // decide o alcance do arrasto — é dela que sai o Alt
                // significar *leve o rig inteiro* nos cinco modos.
                let gesture = self
                    .physics
                    .interaction
                    .joint
                    .gesture(self.modifiers.alt_key());
                let grabbed = !locked
                    && (ph2d_app_physics::body_grab::take_hold(
                        &mut gfx.physics,
                        &self.physics.interaction,
                        entity,
                        world_pos,
                        self.playhead.is_playing(),
                        self.timeline.flags.simulate_physics,
                    )
                    // W-IK: e com o relógio PARADO e o modo IK em
                    // mãos, o mesmo press é a **cinemática inversa**
                    // — arrastar a ponta dobra a cadeia. A mesma
                    // consequência da mão (pegou ⇒ nenhum arrasto de
                    // gizmo abre), pela mesma razão: dois gestos
                    // sobre o mesmo `Transform` no mesmo frame é o
                    // de trás vencendo em silêncio.
                    || ph2d_app_physics::body_pose::take_pose(
                        &mut gfx.physics,
                        gesture == Some(ph2d_physics_ecs::JointGesture::Ik),
                        entity,
                        self.playhead.is_playing(),
                    )
                    // W-FK: e no modo FK o press gira o elo em torno
                    // da PRÓPRIA junta, levando os descendentes.
                    || ph2d_app_physics::body_fk::take_fk(
                        &mut gfx.physics,
                        &gfx.sim,
                        gesture == Some(ph2d_physics_ecs::JointGesture::Fk),
                        entity,
                        world_pos,
                        self.playhead.is_playing(),
                    ));
                // Ver a nota gêmea no sítio da alça (W-JG): a semeadura
                // precisa de `&mut gfx.sim` e mora fora do bloco do `t`.
                let mut opened_drag = false;
                if !grabbed
                    && !locked
                    && let Some(t) = gfx.sim.world().get::<Transform>(entity)
                {
                    let snap_t = ph2d_editor_core::TransformSnapshot {
                        translation: [t.translation.x, t.translation.y],
                        rotation: t.rotation,
                        scale: [t.scale.x, t.scale.y],
                    };
                    let pw = ph2d_ecs::parent_world_transform(gfx.sim.world(), entity);
                    let parent_world = ph2d_editor_core::TransformSnapshot {
                        translation: [pw.translation.x, pw.translation.y],
                        rotation: pw.rotation,
                        scale: [pw.scale.x, pw.scale.y],
                    };
                    let pivot = [t.translation.x, t.translation.y];
                    hero.gizmo.drag = Some(ph2d_editor_core::GizmoDragState {
                        kind: ph2d_editor_core::GizmoDragKind::Translate,
                        entity_bits: bits,
                        start_screen: (evt.x, evt.y),
                        cursor_screen: (evt.x, evt.y),
                        start_transform: snap_t,
                        pivot_world: pivot,
                        start_cursor_world: world_pos,
                        sprite_half_intrinsic: [0.0, 0.0],
                        anchor_is_center: false,
                        target: ph2d_editor_core::GizmoTarget::PrimaryIndividual,
                        parent_world,
                        turns: 0,
                    });
                    opened_drag = true;
                }
                if opened_drag {
                    // Onda 1 + 2C.4: snapshot every OTHER
                    // selected sprite's full start_transform
                    // (skip the drag's own primary — its
                    // snapshot lives on GizmoDragState).
                    // Canvas pick always opens a Translate
                    // drag; the same snapshots also feed
                    // future scale/rotate handles on extras +
                    // global (advance_gizmo_drag dispatches
                    // by drag.kind + drag.target).
                    //
                    // W-JG: um pick de canvas SEMPRE abre um
                    // Translate, então aqui as três condições do
                    // rig se reduzem às duas do relógio e do Alt.
                    let selected: Vec<u64> = hero.gizmo.iter_selected().collect();
                    let carry_reach = if self.playhead.is_playing() {
                        None
                    } else {
                        self.physics
                            .interaction
                            .joint
                            .drag_reach(self.modifiers.alt_key())
                    };
                    ph2d_app_physics::joint_rig_drag::seed_group_drag_starts(
                        &mut self.group_drag_starts,
                        &mut gfx.sim,
                        bits,
                        &selected,
                        carry_reach,
                    );
                }
            }
            // ADR-0029 Phase C.2: live entries owned by the
            // Hierarchy panel crate; reach via the public
            // thread-local snapshot. With multi-select the
            // label mirrors the primary; the count is
            // surfaced via hero.gizmo.selected_len() at
            // paint time (Fase 0e polish).
            let primary = hero.gizmo.selection;
            // O ciclo corrente fica atado a ESTA seleção — ver `cycle_pick_selection`.
            self.cycle_pick_selection = primary;
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
        }
    }
}
