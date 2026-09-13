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

    /// A seleção que o pick decide: modificadores, a trava do Painter, o colapso adiado e a borracha armada.
    pub(super) fn ramo_gizmo_pick_selecao(
        &mut self,
        evt: PointerEvent,
        picked: Option<u64>,
        world_pos: [f32; 2],
    ) {
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
        {
            // Fase 0d: read modifier state at click time —
            // Shift adds to the selection, Cmd/Ctrl toggles,
            // bare click replaces (legacy default). Modifier
            // clicks skip drag setup since the user is
            // adjusting selection, not moving sprites.
            let shift_held = self.modifiers.shift_key();
            let cmd_held = self.modifiers.super_key() || self.modifiers.control_key();
            // Smart-click preservation: bare click on a
            // sprite that's already inside an active multi-
            // selection KEEPS the whole set (user intends
            // to interact with the group — e.g. drag the
            // group or run a tool — not collapse to single).
            let preserves_multi = picked
                .is_some_and(|bits| hero.gizmo.selected_len() > 1 && hero.gizmo.is_selected(bits));
            // Drag-setup skip: modifier clicks adjust the
            // selection but should not start a gizmo drag
            // (the user is curating, not moving). Bare-
            // click in a multi-selection DOES start a drag
            // (group translate via the clicked sprite as
            // pivot, Onda 1).
            let is_modifier_click = picked.is_some() && (shift_held || cmd_held);
            // **A TRAVA DO PAINTER, na porta do CANVAS** (Enio, 2026-08-19). Enquanto
            // ele tem um documento aberto, um clique noutra sprite não a troca debaixo
            // do pincel — e um clique com modificador não abre uma segunda seleção.
            //
            // ⚠️ Sai por `picked = None` em vez de por um `return`: o resto deste ramo
            // é o caminho do **clique vazio**, que limpa e não seleciona outra — a lei
            // permite-o de propósito (recusar faria o `Esc` e o canvas parecerem
            // partidos). *Recusar a troca não é recusar o clique.*
            let painter_locked = ph2d_app_painter::painter_lock::locked_entity(&gfx.tools, hero);
            let picked = match picked {
                Some(bits)
                    if ph2d_app_painter::painter_lock::decide(
                        painter_locked,
                        Some(bits),
                        cmd_held || shift_held,
                    ) == ph2d_app_painter::painter_lock::Decision::Refuse =>
                {
                    gfx.toasts
                        .push(Toast::warning(ph2d_app_painter::painter_lock::REFUSAL));
                    self.pending_ui_sound = Some(crate::ui_sound::UiSound::Refuse);
                    None
                }
                other => other,
            };
            if let Some(bits) = picked {
                if cmd_held || shift_held {
                    // Onda 1: unify Shift + Cmd as toggle on
                    // the canvas. Click on a sprite already
                    // in the selection → removes JUST that
                    // one. Click on a sprite outside → adds.
                    // The Hierarchy panel keeps Shift = range
                    // (list-style UX); the canvas has no
                    // natural linear order, so toggle is the
                    // sane semantic for both modifiers.
                    hero.gizmo.toggle_in_selection(bits);
                } else if preserves_multi {
                    // Onda 2 hotfix: bare click on a sprite
                    // already in the multi-selection DEFERS
                    // the decision to PointerUp. If the user
                    // drags from here, the open Translate
                    // drag becomes a group translate (Onda 1
                    // semantics preserved). If they release
                    // without dragging, Up `replace_selection`
                    // collapses the multi to just this sprite
                    // (Enio: "se há multiplas sprites
                    // selecionas e eu clicar com botão
                    // esquerdo em uma delas, todas as outras
                    // devem ser desselecionadas").
                    self.pending_single_replace = Some((bits, (evt.x, evt.y)));
                } else {
                    hero.gizmo.replace_selection(Some(bits));
                }
            } else {
                // Empty click — Fase 0f: defer to PointerKind::Up
                // so we can distinguish "bare click on empty"
                // (= clear selection) from "start of a rubber-
                // band box-select drag" (= keep selection
                // until release, then resolve against the
                // dragged rect). Cmd on empty stays a no-op
                // (preserves built-up multi-selection). Shift
                // on empty starts an additive rubber-band.
                if !cmd_held {
                    self.rubber_band = Some(crate::app_state::RubberBandState {
                        anchor_screen: (evt.x, evt.y),
                        current_screen: (evt.x, evt.y),
                        add_mode: shift_held,
                    });
                }
            }
            self.ramo_gizmo_pick_arrasta(evt, picked, is_modifier_click, world_pos);
        }
    }
}
