//! **O clique, o GIZMO** — ramos do `on_mouse_input` ([`super`]), corpos verbatim pela mesma ordem: modificador, o hit
//! (alça, traço, arte Flip, Translate chaveado), pivô, âncoras de junta, roldana, e a cadeia alça/pick. ⚠️ Cada ramo
//! re-empresta `gfx`/`hero` com o MESMO guarda do braço, e chama o seguinte depois do último uso deles (NLL).

use super::*;
use ph2d_i18n::tr;

/// **O que o hit de um pen-down disse**, calculado UMA vez e lido pelos ramos seguintes (o pivô, as âncoras, a
/// cadeia). Os campos têm exactamente os nomes das variáveis que eram no `on_mouse_input`: cada ramo desestrutura-o
/// na primeira linha, e o corpo mudou-se sem uma letra diferente. Oito `Copy` na pilha — nada aloca por evento.
#[derive(Clone, Copy)]
pub(super) struct AlvoDoClique {
    pub(super) hit_id: Option<ph2d_editor_core::NodeId>,
    pub(super) gizmo_kind: Option<ph2d_editor_core::GizmoDragKind>,
    pub(super) effective_target: ph2d_editor_core::GizmoTarget,
    pub(super) effective_kind: Option<ph2d_editor_core::GizmoDragKind>,
    pub(super) is_specific_handle: bool,
    pub(super) over_open_vec_stroke: bool,
    pub(super) over_flip_art: bool,
    pub(super) is_keyed_translate: bool,
}

/// O pivô guardado no pen-down: o CANTO oposto à mão, sempre, mesmo com Ctrl premido. A âncora é viva
/// (`live_anchor`) e o centro deriva-se da pose; o canto só se sabe aqui. Compõe o pai: filho de pai
/// rodado não calcula o pivô como raiz. Lei: `shells/desktop/tests/it/the_scale_anchor_follows_the_live_modifier.rs`.
pub(super) fn pivo_da_alca(
    gkind: ph2d_editor_core::GizmoDragKind,
    (anchor, half): ([f32; 2], [f32; 2]),
    parent_world: ph2d_editor_core::TransformSnapshot,
    snap: ph2d_editor_core::TransformSnapshot,
    start_world: [f32; 2],
) -> [f32; 2] {
    let world_snap = ph2d_editor_core::compose_snapshot(parent_world, snap);
    ph2d_editor_core::grip_pivot_world(gkind, anchor, half, world_snap, start_world)
}

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
            let window_size =
                crate::scene_mapping::janela(hero.view.center_split, gfx.surface.size());
            let start_world = gfx.camera.screen_to_world((evt.x, evt.y), window_size);
            // A semeadura do grupo saiu deste bloco (W-JG): pede `&mut gfx.sim` e o `t` empresta
            // o mundo. Sem drag aberto não há grupo a semear, daí o sinalizador.
            let mut opened_drag = false;
            if let Some(t) = gfx.sim.world().get::<Transform>(entity) {
                let snap = ph2d_editor_core::TransformSnapshot {
                    translation: [t.translation.x, t.translation.y],
                    rotation: t.rotation,
                    scale: [t.scale.x, t.scale.y],
                };
                // O mundo do pai: compute_gizmo_transform desfaz nele o delta antes de escrever
                // no Transform LOCAL.
                let pw = ph2d_ecs::parent_world_transform(gfx.sim.world(), entity);
                let parent_world = ph2d_editor_core::TransformSnapshot {
                    translation: [pw.translation.x, pw.translation.y],
                    rotation: pw.rotation,
                    scale: [pw.scale.x, pw.scale.y],
                };
                // ⚠️ As DUAS metades: sem o `anchor`, o pivô desloca-se de `anchor ⊙ scale`
                // e a borda oposta caminha numa moldura fora do centro.
                let (sprite_anchor_intrinsic, sprite_half_intrinsic) =
                    gizmo_anchor_half(&gfx.sim, &gfx.vec_scene, &gfx.flip, entity);
                // Individual: a âncora do objecto. Global: o centro da caixa do grupo.
                let pivot = if let ph2d_editor_core::GizmoTarget::Global = effective_target
                    && let Some(gv) = hero.gizmo.global_view.as_ref()
                {
                    [
                        (gv.bbox_min_world[0] + gv.bbox_max_world[0]) * 0.5,
                        (gv.bbox_min_world[1] + gv.bbox_max_world[1]) * 0.5,
                    ]
                } else {
                    pivo_da_alca(
                        gkind,
                        (sprite_anchor_intrinsic, sprite_half_intrinsic),
                        parent_world,
                        snap,
                        start_world,
                    )
                };
                // A vista global da partida: o gizmo do grupo roda com ele, em vez de ser a
                // união alinhada aos eixos, que cresceria ao rodar.
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
                // A partida de cada OUTRO selecionado, para qualquer tipo de arrasto.
                // W-JG: num Translate em repouso com ALT, o rig articulado entra junto, pela
                // mesma porta do pick de canvas (`ph2d_app_physics::joint_rig_drag`).
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

    /// A cadeia do pen-down: o pivô, a âncora ou a roldana já abertos · a alça de um alvo · o pick de canvas.
    pub(super) fn ramo_gizmo_cadeia(
        &mut self,
        evt: PointerEvent,
        menu_open_before: bool,
        alvo: AlvoDoClique,
        began_pivot: bool,
        began_joint_anchor: bool,
        began_wheel_select: bool,
    ) -> bool {
        let AlvoDoClique {
            hit_id,
            gizmo_kind,
            effective_target,
            effective_kind,
            is_specific_handle,
            over_open_vec_stroke,
            over_flip_art,
            is_keyed_translate,
        } = alvo;
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
        {
            if began_pivot || began_joint_anchor || began_wheel_select {
                // Pivot or joint-anchor drag opened; Move events drive it.
            } else if is_specific_handle
                && !over_open_vec_stroke
                && !over_flip_art
                && let Some(gkind) = effective_kind
                && let Some(entity_bits) = match effective_target {
                    ph2d_editor_core::GizmoTarget::ExtraIndividual(bits) => Some(bits),
                    _ => hero.gizmo.selection,
                }
            {
                if self.ramo_gizmo_alca(evt, gkind, entity_bits, effective_target) {
                    return true;
                }
            } else if hero.chrome_panel_at(evt.x, evt.y).is_none()
                && !menu_open_before
                && (hit_id.is_none()
                    || matches!(gizmo_kind, Some(ph2d_editor_core::GizmoDragKind::Translate))
                    || hit_id == Some(ph2d_editor_core::gizmo::ids::GIZMO_PIVOT)
                    || is_keyed_translate
                    || over_open_vec_stroke
                    || over_flip_art)
            {
                self.ramo_gizmo_pick(evt, gizmo_kind);
            }
        }
        false
    }

    /// O pivô (a ferramenta Pivot sobre o selecionado), a âncora de junta pelo mapa do pintor e a roldana —
    /// e depois a cadeia.
    pub(super) fn ramo_gizmo_pivo_e_ancora(
        &mut self,
        evt: PointerEvent,
        menu_open_before: bool,
        alvo: AlvoDoClique,
    ) -> bool {
        let AlvoDoClique { hit_id, .. } = alvo;
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
        {
            // TOOL_PIVOT begin: with the Pivot tool active, a click on the selected sprite (or
            // its pivot dot) opens a MovePivot drag instead of the pick / scale path.
            let pivot_tool_active = hero.store.button_state(ph2d_editor_core::ids::TOOL_PIVOT)
                == Some(ph2d_editor_core::widget::ButtonState::Pressed);
            let mut began_pivot = false;
            if pivot_tool_active
                && hero.chrome_panel_at(evt.x, evt.y).is_none()
                && !menu_open_before
                && let Some(entity_bits) = hero.gizmo.selection
            {
                let entity = ph2d_ecs::Entity::from_bits(entity_bits);
                let window_size =
                    crate::scene_mapping::janela(hero.view.center_split, gfx.surface.size());
                let world_pos = gfx.camera.screen_to_world((evt.x, evt.y), window_size);
                let on_pivot_dot = hit_id == Some(ph2d_editor_core::gizmo::ids::GIZMO_PIVOT);
                // ADR-0111: uma forma vetorial ou um objeto Flip também é
                // agarrável pela arte (não só pelo dot do pivô).
                let on_object =
                    ph2d_render::pick_sprite_at_world(gfx.present.world_mut(), world_pos)
                        == Some(entity_bits)
                        || ph2d_app_vec::vec_gizmo_view::contains_world(
                            &gfx.sim,
                            &gfx.vec_scene,
                            &self.vec.live_drawn,
                            &self.vec.view_derived,
                            entity,
                            world_pos,
                            ph2d_app_vec::vec_gizmo_view::stroke_hit_r(&gfx.camera, window_size),
                        )
                        || ph2d_app_flip::gizmo_view::contains_world(
                            &gfx.sim,
                            &gfx.flip,
                            entity,
                            world_pos,
                            ph2d_app_flip::gizmo_view::stroke_hit_r(&gfx.camera, window_size),
                        );
                if (on_pivot_dot || on_object)
                    && !ph2d_ecs::is_locked_for_edit(gfx.sim.world(), entity)
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
                    let (anchor, half) =
                        gizmo_anchor_half(&gfx.sim, &gfx.vec_scene, &gfx.flip, entity);
                    // Invariant quad center = pivot + R·(anchor ⊙ scale).
                    let ax = anchor[0] * snap_t.scale[0];
                    let ay = anchor[1] * snap_t.scale[1];
                    // T1.3.5 cross-OS bit-identical.
                    let (sin_r, cos_r) = libm::sincosf(snap_t.rotation);
                    let quad_center = [
                        snap_t.translation[0] + ax * cos_r - ay * sin_r,
                        snap_t.translation[1] + ax * sin_r + ay * cos_r,
                    ];
                    hero.gizmo.drag = Some(ph2d_editor_core::GizmoDragState {
                        kind: ph2d_editor_core::GizmoDragKind::MovePivot,
                        entity_bits,
                        start_screen: (evt.x, evt.y),
                        cursor_screen: (evt.x, evt.y),
                        start_transform: snap_t,
                        pivot_world: quad_center,
                        start_cursor_world: world_pos,
                        sprite_half_intrinsic: half,
                        anchor_is_center: false,
                        target: ph2d_editor_core::GizmoTarget::PrimaryIndividual,
                        parent_world,
                        turns: 0,
                    });
                    began_pivot = true;
                }
            }
            // Joint-anchor handles: a Down on either dot opens the anchor drag for that END
            // (`ph2d_app_physics::joint_anchor_drag`); one door writes both locals. Which joint
            // comes from the painter's MAP, not the selection (W-J2b).
            let anchor_hit = hit_id.and_then(|id| {
                ph2d_app_physics::overlay::point_gizmo::resolve_anchor_hit(
                    &hero.gizmo.point_hit_map,
                    id,
                )
            });
            let mut began_joint_anchor = false;
            if let Some((joint, kind)) = anchor_hit
                && hero.chrome_panel_at(evt.x, evt.y).is_none()
                && !menu_open_before
            {
                let opened = ph2d_app_physics::joint_anchor_drag::open_drag(
                    &gfx.physics,
                    &gfx.sim,
                    &gfx.camera,
                    crate::scene_mapping::janela(hero.view.center_split, gfx.surface.size()),
                    joint,
                    (evt.x, evt.y),
                    kind,
                );
                if opened.is_some() {
                    // Disjoint field write: `gfx`/`hero` borrow
                    // `self.gfx`, this is `self.physics.joint_anchor_drag`.
                    self.physics.joint_anchor_drag = opened;
                    began_joint_anchor = true;
                    // **Grabbing a handle SELECTS its joint**: the press also puts §12 (Physics
                    // Joint) on screen, the same two lines W-JointCreate uses after a Join.
                    hero.gizmo.selection = Some(joint.to_bits());
                    hero.gizmo.extra_selection.clear();
                }
            }
            // **SELECIONAR UMA ROLDANA CLICANDO NELA** — ver `select_wheel_at`.
            let began_wheel_select = !began_pivot
                && !began_joint_anchor
                && anchor_hit.is_none()
                && self.show_colliders
                && hero.chrome_panel_at(evt.x, evt.y).is_none()
                && !menu_open_before
                && select_wheel_at(
                    &gfx.physics,
                    &gfx.camera,
                    crate::scene_mapping::janela(hero.view.center_split, gfx.surface.size()),
                    hero,
                    (evt.x, evt.y),
                );
            if self.ramo_gizmo_cadeia(
                evt,
                menu_open_before,
                alvo,
                began_pivot,
                began_joint_anchor,
                began_wheel_select,
            ) {
                return true;
            }
        }
        false
    }

    /// O pen-down do botão primário no canvas: o toque com modificador (alterna a seleção e devolve) e o que o
    /// hit diz — a alça, o traço aberto, a arte Flip, o Translate chaveado —, que segue num `AlvoDoClique`.
    pub(super) fn ramo_gizmo_premido(&mut self, evt: PointerEvent, menu_open_before: bool) -> bool {
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
        {
            // Shift/Cmd in the canvas ALWAYS means selection-adjustment: a modifier click
            // never opens a scale-handle drag, even over a handle.
            let shift_held_early = self.modifiers.shift_key();
            let cmd_held_early = self.modifiers.super_key() || self.modifiers.control_key();
            if (shift_held_early || cmd_held_early)
                && hero.chrome_panel_at(evt.x, evt.y).is_none()
                && !menu_open_before
            {
                // ⭐ **A porta ÚNICA do pick de objecto** — vetor, depois Flip, depois
                // sprites, na ordem de z que o artista vê. Esta lista existia copiada
                // aqui e no clique simples, e o realce de proveniência ia ser a terceira.
                let ppm_for_pick = hero.project.pixels_per_meter;
                let mut pw = crate::hover_highlight::PickWorld {
                    window_size: crate::scene_mapping::janela(
                        hero.view.center_split,
                        gfx.surface.size(),
                    ),
                    sim: &gfx.sim,
                    vec_scene: &gfx.vec_scene,
                    flip: &gfx.flip,
                    frame_order: &gfx.frame_order,
                    present: &mut gfx.present,
                    camera: &gfx.camera,
                    pixels_per_meter: ppm_for_pick,
                    camera_icon_px: hero.project.camera_icon_px,
                    camera_frames: &self.undo.camera_no_canvas.molduras,
                    motion: Some(ph2d_app_motion::motion_pick::Alvo::new(
                        &gfx.motion,
                        gfx.surface.gpu(),
                        &mut self.motion_shell.clique,
                    )),
                };
                let hits = crate::hover_highlight::pick_objects_at(
                    &mut pw,
                    &self.vec.entities,
                    &self.vec.view_derived,
                    &self.vec.live_drawn,
                    &self.flip_state.entities,
                    (evt.x, evt.y),
                );
                if let Some(bits) = hits.first().copied() {
                    hero.gizmo.toggle_in_selection(bits);
                    let primary = hero.gizmo.selection;
                    if let Some(entry) = resolve_live_entry(gfx.hero_live.as_ref(), primary) {
                        hero.selection = Some(ph2d_editor_core::HeroSelection {
                            label: entry.name.clone(),
                            kind: entry.badge.clone().unwrap_or_else(|| {
                                tr("shell.despacho_clique_gizmo.ent").to_string()
                            }),
                            world_pos: (0.0, 0.0),
                        });
                    } else if primary.is_none() {
                        hero.selection = None;
                    }
                    self.title_dirty = true;
                    return true;
                }
                // Modifier on empty canvas → fall through to
                // existing cascade so a Shift-drag can still
                // open an additive rubber-band.
            }
            let hit_id = hero.chrome_hit(evt.x, evt.y);
            let gizmo_kind = hit_id.and_then(ph2d_editor_core::gizmo_kind_for_id);
            // Onda 2C: hit_map fills in for handles with non-canonical ids (extras + global);
            // the primary keeps canonical ids.
            let hit_map_entry: Option<ph2d_editor_core::GizmoHit> =
                hit_id.and_then(|id| hero.gizmo.gizmo_hit_map.get(&id).copied());
            let effective_target = hit_map_entry
                .map(|h| h.target)
                .unwrap_or(ph2d_editor_core::GizmoTarget::PrimaryIndividual);
            let effective_kind = hit_map_entry.map(|h| h.kind).or(gizmo_kind);
            let is_specific_handle = matches!(
                effective_kind,
                Some(ph2d_editor_core::GizmoDragKind::ScaleCorner { .. })
                    | Some(ph2d_editor_core::GizmoDragKind::ScaleEdge { .. })
                    | Some(ph2d_editor_core::GizmoDragKind::Rotate)
            );
            // Sobre o TRAÇO de uma forma vetorial aberta, o arrasto é Translate dela: a caixa
            // fina daria as alças ao clique, e arrastar a linha a escalaria. Alças de quina
            // fora do traço seguem escalando.
            let over_open_vec_stroke = {
                let window_size = gfx.scene_window();
                let world_pos = gfx.camera.screen_to_world((evt.x, evt.y), window_size);
                let vec_view = ph2d_vec_entities::entities::view_state_for_pick(
                    &gfx.sim,
                    &self.vec.entities,
                    &self.vec.view_derived,
                );
                let hits = ph2d_app_vec::vec_gizmo_view::pick_all_at_world(
                    &gfx.sim,
                    &gfx.vec_scene,
                    &self.vec.live_drawn,
                    &vec_view,
                    &self.vec.entities,
                    world_pos,
                    ph2d_app_vec::vec_gizmo_view::stroke_hit_r(&gfx.camera, window_size),
                );
                hits.first().is_some_and(|&bits| {
                    gfx.sim
                        .world()
                        .get::<ph2d_ecs::VecPathRef>(ph2d_ecs::Entity::from_bits(bits))
                        .is_some_and(|vp| {
                            gfx.vec_scene
                                .paths()
                                .iter()
                                .any(|p| p.id == vp.0 && !p.closed)
                        })
                })
            };
            // Idem para a arte Flip: uma nuvem de traços não tem interior, então
            // o gizmo de sprite colapsaria e os handles roubariam o clique.
            // Sobre a arte ⇒ o arrasto é Translate dela (cai no canvas-pick).
            let over_flip_art = {
                let window_size = gfx.scene_window();
                let world_pos = gfx.camera.screen_to_world((evt.x, evt.y), window_size);
                !ph2d_app_flip::gizmo_view::pick_all_at_world(
                    &gfx.sim,
                    &gfx.flip,
                    &self.flip_state.entities,
                    world_pos,
                    ph2d_app_flip::gizmo_view::stroke_hit_r(&gfx.camera, window_size),
                )
                .is_empty()
            };
            // Keyed Translate = click on the bbox interior of an extra or the global gizmo (hashed
            // ids, unknown to `gizmo_kind_for_id`). Treated as a multi-select translate through the
            // canvas-pick branch below; without this guard those clicks fall through to nothing.
            let is_keyed_translate = hit_map_entry
                .map(|h| matches!(h.kind, ph2d_editor_core::GizmoDragKind::Translate))
                .unwrap_or(false);
            // O que o hit disse, para os ramos que seguem (`despacho_clique_gizmo::AlvoDoClique`).
            let alvo = AlvoDoClique {
                hit_id,
                gizmo_kind,
                effective_target,
                effective_kind,
                is_specific_handle,
                over_open_vec_stroke,
                over_flip_art,
                is_keyed_translate,
            };
            if self.ramo_gizmo_pivo_e_ancora(evt, menu_open_before, alvo) {
                return true;
            }
        }
        false
    }
}
