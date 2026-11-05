//! **O arrasto do gizmo, o CÁLCULO** — ramos do `advance_gizmo_drag` ([`super`]): o pivô que se muda
//! (`MovePivot`, com o snap do Ctrl), a câmera, os modificadores e o snap que dão o `new_t`, e o que se deriva
//! dele — a escala uniforme de uma selecção rodada, os factores e a rotação contínua pela costura do `atan2`. Os
//! corpos MUDARAM-SE verbatim (`line/input-dispatch`, 2026-09-13) e correm no sítio da chamada, pela mesma ordem.

use crate::Transform;

impl crate::App {
    /// Do `new_t` aos valores da escrita: a escala uniforme de uma selecção rodada, os factores, a rotação que atravessa
    /// a costura do `atan2` sem saltar, o `delta_rot` — e a escrita.
    pub(super) fn ramo_gizmo_escala_e_rotacao(
        &mut self,
        drag: ph2d_editor_core::GizmoDragState,
        entity: ph2d_ecs::Entity,
        new_t: ph2d_editor_core::TransformSnapshot,
        is_scale: bool,
    ) {
        if let Some(gfx) = self.gfx.as_mut() {
            // Local-pivot multi-select: cada um escala em torno do próprio pivô. Global: a fórmula do
            // pivô partilhado, como os outros. Seleção rodada: escala proporcional, porque a não
            // uniforme de um filho rodado é um SHEAR que `Transform.scale` não representa (Enio 2026-06-03).
            // Eixo a zero: `gizmo/escala.rs`, e a decisão em docs/CartoesUI/10 (F7).
            let partida = ph2d_editor_core::gizmo::partida_do_grupo(drag)
                .start_transform
                .scale;
            let new_t = if !self.group_drag_starts.is_empty()
                && matches!(
                    drag.kind,
                    ph2d_editor_core::GizmoDragKind::ScaleCorner { .. }
                        | ph2d_editor_core::GizmoDragKind::ScaleEdge { .. }
                )
                // A rotação de MUNDO (pai·local): um filho de pai rodado também está rodado.
                && (drag.parent_world.rotation + drag.start_transform.rotation != 0.0
                    || self.group_drag_starts.iter().any(|s| {
                        s.parent_world.rotation + s.start_transform.rotation != 0.0
                    })) {
                ph2d_editor_core::TransformSnapshot {
                    scale: ph2d_editor_core::gizmo::uniforme_do_grupo(partida, new_t.scale),
                    ..new_t
                }
            } else {
                new_t
            };
            let [factor_x, factor_y] =
                ph2d_editor_core::gizmo::fator_do_grupo(partida, new_t.scale);
            let new_t = ph2d_editor_core::TransformSnapshot {
                scale: ph2d_editor_core::gizmo::escala_do_primario(&drag, new_t.scale),
                ..new_t
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

    /// A câmera da cena, os modificadores vivos, o snap (vetorial com guias, de grelha, ou nenhum) e o `new_t` do
    /// gizmo — com a rotação zerada para um arrasto GLOBAL e reposta a seguir.
    pub(super) fn ramo_gizmo_calcular(
        &mut self,
        drag: ph2d_editor_core::GizmoDragState,
        is_scale_drag: bool,
        vec_scale_snap: bool,
        vec_cfg: ph2d_vec_edit::snap::SnapConfig,
    ) {
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
        {
            let window_size = ph2d_app_motion::field_gizmo::scene_camera_window(
                hero.view.center_split,
                gfx.surface.size(),
            );
            let cam = ph2d_editor_core::GizmoCamera {
                center: gfx.camera.center,
                height_world: gfx.camera.height_world,
                window_w: window_size.width as f32,
                window_h: window_size.height as f32,
            };
            // M14.7 D: sample winit's tracked modifier state (updated
            // on ModifiersChanged). Shift / Ctrl / Alt feed AR lock +
            // snap + mirror-anchor. On macOS we treat Cmd as Ctrl
            // (industry convention for snap-to-grid).
            let mods = ph2d_editor_core::GizmoModifiers {
                shift: self.modifiers.shift_key(),
                ctrl: self.modifiers.control_key() || self.modifiers.super_key(),
                alt: self.modifiers.alt_key(),
            };
            let snap = ph2d_editor_core::GizmoSnap {
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
            let is_scale = is_scale_drag;
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
            let is_global_drag = matches!(drag.target, ph2d_editor_core::GizmoTarget::Global);
            let drag_for_math = if is_global_drag {
                let mut d = drag;
                d.start_transform.rotation = 0.0;
                d
            } else {
                drag
            };
            // Escalar uma sprite com skew corre no espaço sem skew e volta pela afim (`sprite_box`).
            let skew_m = ph2d_editor_core::gizmo::drag_skew_model(
                &drag,
                ph2d_ecs::world_skew(gfx.sim.world(), entity),
            );
            let skew_inv = ph2d_editor_core::gizmo::model_inverse(&skew_m)
                .unwrap_or(ph2d_editor_core::gizmo::MODEL_IDENTITY);
            let (para_o_desenho, do_desenho) = (
                |w: [f32; 2]| ph2d_editor_core::gizmo::model_apply(&skew_m, w),
                |w: [f32; 2]| ph2d_editor_core::gizmo::model_apply(&skew_inv, w),
            );
            let drag_for_math = ph2d_editor_core::gizmo::partida_do_grupo(drag_for_math);
            let drag_for_math =
                ph2d_editor_core::gizmo::drag_in_skew_frame(drag_for_math, &skew_m, &cam);
            let new_t = if is_scale && vec_scale_snap {
                // Encaixa o CANTO arrastado (cursor) nas outras formas + grade e
                // publica as guias — mesmo motor do translate, mas quem aplica é o
                // gizmo (o cursor encaixado dirige a razão de escala, pivô fixo). O
                // bloco interno solta os borrows do closure antes de gravar as guias.
                let targets = &self.vec.snap_targets;
                let mut guides: Vec<ph2d_vec_render::Guide> = Vec::new();
                let snap_state = &mut hero.grid.snap_state;
                let t = {
                    let mut snap_closure = |w: [f32; 2]| -> [f32; 2] {
                        let w = para_o_desenho(w);
                        let p = [f64::from(w[0]), f64::from(w[1])];
                        let mut grid = |q: [f64; 2]| crate::vec_snap::ask_grid(snap_state, q);
                        let r = ph2d_vec_edit::snap::snap(&[p], targets, vec_cfg, Some(&mut grid));
                        guides = crate::vec_snap::guides_of(&r);
                        let s = r.apply(p);
                        do_desenho([s[0] as f32, s[1] as f32])
                    };
                    ph2d_editor_core::compute_gizmo_transform(
                        &drag_for_math,
                        &cam,
                        mods,
                        snap,
                        Some(&mut snap_closure),
                    )
                };
                self.vec.snap_guides = guides;
                t
            } else if is_scale {
                let snap_state = &mut hero.grid.snap_state;
                let mut snap_closure = |w: [f32; 2]| -> [f32; 2] {
                    do_desenho(snap_state.snap_world(para_o_desenho(w), sprite_half_rendered))
                };
                ph2d_editor_core::compute_gizmo_transform(
                    &drag_for_math,
                    &cam,
                    mods,
                    snap,
                    Some(&mut snap_closure),
                )
            } else {
                ph2d_editor_core::compute_gizmo_transform(&drag_for_math, &cam, mods, snap, None)
            };
            // Restore the primary's actual rotation: in Global
            // drags `compute_gizmo_transform` returned a rotation
            // computed against the zeroed start, so we shift it
            // back by the primary's original start rotation. For
            // non-Global drags this is a no-op.
            let new_t = if is_global_drag {
                ph2d_editor_core::TransformSnapshot {
                    rotation: drag.start_transform.rotation
                        + (new_t.rotation - drag_for_math.start_transform.rotation),
                    ..new_t
                }
            } else {
                new_t
            };
            let new_t = ph2d_editor_core::TransformSnapshot {
                translation: ph2d_editor_core::gizmo::skew_translation_out(
                    &skew_m,
                    drag.parent_world,
                    new_t.translation,
                ),
                ..new_t
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
            self.ramo_gizmo_escala_e_rotacao(drag, entity, new_t, is_scale);
        }
    }

    /// `MovePivot`: o pivô vai para o cursor e o quad fica parado no mundo (âncora compensada); com Ctrl, encaixa
    /// no centro, cantos e meios de aresta do quad e no centro do conteúdo opaco.
    pub(super) fn ramo_gizmo_mover_pivo(
        &mut self,
        drag: ph2d_editor_core::GizmoDragState,
        ctrl: bool,
        content_center: Option<[f32; 2]>,
    ) {
        if let Some(gfx) = self.gfx.as_mut() {
            // TOOL_PIVOT: relocate the pivot to the cursor while the
            // sprite's quad stays world-fixed (compensating anchor).
            // CTRL snaps to the quad center / corners / edge mids +
            // the content-bbox center (`content_center`).
            let window_size = crate::field_gizmo_host::scene_window_of(gfx);
            let entity = ph2d_ecs::Entity::from_bits(drag.entity_bits);
            let raw_world = gfx.camera.screen_to_world(drag.cursor_screen, window_size);
            // Os candidatos nascem na pose de mundo SEM skew; com skew vão ao desenho pela afim, e o
            // alvo volta por ela antes da conta da âncora (`sprite_box`).
            let skew_m = ph2d_editor_core::gizmo::drag_skew_model(
                &drag,
                ph2d_ecs::world_skew(gfx.sim.world(), entity),
            );
            let mundo = ph2d_editor_core::compose_snapshot(drag.parent_world, drag.start_transform);
            let target = if ctrl {
                let half_world = gfx
                    .sim
                    .world()
                    .get::<ph2d_render::Sprite>(entity)
                    .map(|s| {
                        [
                            s.size[0] * mundo.scale[0] * 0.5,
                            s.size[1] * mundo.scale[1] * 0.5,
                        ]
                    })
                    .unwrap_or([0.0, 0.0]);
                let cands = ph2d_editor_core::pivot_snap_candidates(
                    drag.pivot_world,
                    mundo.rotation,
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
                for c in cands.into_iter().chain(content_center) {
                    let c = ph2d_editor_core::gizmo::model_apply(&skew_m, c);
                    consider(c, &mut best, &mut best_d2);
                }
                best
            } else {
                raw_world
            };
            let target = ph2d_editor_core::gizmo::model_inverse(&skew_m).map_or(target, |inv| {
                ph2d_editor_core::gizmo::model_apply(&inv, target)
            });
            let (new_translation, new_anchor) = ph2d_editor_core::move_pivot_transform(
                drag.start_transform,
                drag.pivot_world,
                target,
                drag.parent_world,
            );
            let new_translation = ph2d_editor_core::gizmo::skew_translation_out(
                &skew_m,
                drag.parent_world,
                new_translation,
            );
            if let Some(mut t) = gfx.sim.world_mut().get_mut::<Transform>(entity) {
                t.translation = ph2d_core::Vec2::new(new_translation[0], new_translation[1]);
            }
            if let Some(mut s) = gfx.sim.world_mut().get_mut::<ph2d_render::Sprite>(entity) {
                s.anchor = new_anchor;
            }
        }
    }
}
