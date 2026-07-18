//! ADR-0114 W7.5 Fase 2 — **o gizmo da POSE da chave** (modo Edit, quadro instanciado).
//!
//! O Enio pediu girar/escalar uma instância. A pose já é um afim ([`Pose`], Fase 1); o
//! que falta é o GESTO — e a decisão central deste módulo é **não reescrever a
//! matemática do gizmo**: o [`ph2d_editor::compute_gizmo_transform`] (testado, com
//! modifiers/snap/contador de voltas) já resolve translate/rotate/scale para um TRS.
//! Então a pose é **reparametrizada como um TRS ancorado no CENTRO da arte**:
//!
//! ```text
//! pose(p) = t_c + R·S·(p − c_local)      com  t_c = pose.apply(c_local)
//! ```
//!
//! onde `c_local` é o centro da bbox CRUA do desenho da chave. Nessa linguagem a chave
//! é exatamente um "sprite" cujo pivô é o centro da arte (`anchor = 0`), o `Transform`
//! local é o TRS da pose e o `parent_world` é o afim do OBJETO — e todo o motor de
//! gizmo (a view, os handles, a conta do delta, a re-ancoragem do canto oposto) é
//! reusado byte a byte. O rotate gira em torno do centro da arte posada (a spec), o
//! scale segura o canto oposto, e o resultado volta a ser um [`Pose`] por
//! [`trs_to_pose`] — o inverso exato de [`pose_trs`].
//!
//! **Seed = sample:** a caixa e o pivô saem da MESMA pose que o render dobra (o
//! `frame_pose` da chave que o `visible_key` resolve — a mesma porta do move do Edit e
//! do `pose_at_cycled` do render). É o antídoto do bug do halo (W7.2).
//!
//! **Por que o gizmo não tem interior (Translate):** o arrasto de canvas do modo Edit
//! JÁ move a instância (`flip_edit_gesture::move_drawing` escreve a pose), e um
//! interior registrado no hit-index tornaria `on_canvas` falso sobre a arte inteira —
//! a seleção de traço morreria. Os handles keyed (`GizmoTarget::FlipPose`) registram
//! só rotate/scale, que é o que falta.

use ph2d_core::{Playhead, Vec2};
use ph2d_ecs::SimWorld;
use ph2d_editor::{GizmoCamera, GizmoModifiers, GizmoSnap, GizmoView, TransformSnapshot};
use ph2d_flip::{FlipDoc, FlipDrawing, FlipObjectId, Frame, LayerId, Pose};
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;

use crate::flip_entities::FlipEntityMap;
use crate::vec_transform::world_transform;

/// O arrasto de POSE em curso: o estado genérico do gizmo + o ALVO da escrita
/// (que o `GizmoDragState` não tem como carregar) + o `c_local` da
/// reparametrização — capturado no Down para que seed (a view) e write (o apply)
/// derivem do MESMO centro.
#[derive(Clone, Copy, Debug)]
pub(crate) struct FlipPoseDrag {
    pub(crate) drag: ph2d_editor::GizmoDragState,
    pub(crate) oid: FlipObjectId,
    pub(crate) lid: LayerId,
    pub(crate) key: Frame,
    pub(crate) c_local: [f32; 2],
}

/// Centro + meia-extensão da bbox **CRUA** (sem pose) dos traços de um desenho.
/// `None` para um desenho sem ponto nenhum.
#[must_use]
pub(crate) fn drawing_center_half(d: &FlipDrawing) -> Option<([f32; 2], [f32; 2])> {
    let mut it = d.strokes.iter().flat_map(|s| s.positions().iter());
    let first = it.next()?;
    let (mut lo, mut hi) = ([first.x, first.y], [first.x, first.y]);
    for p in it {
        lo[0] = lo[0].min(p.x);
        lo[1] = lo[1].min(p.y);
        hi[0] = hi[0].max(p.x);
        hi[1] = hi[1].max(p.y);
    }
    Some((
        [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5],
        [(hi[0] - lo[0]) * 0.5, (hi[1] - lo[1]) * 0.5],
    ))
}

/// **Pose → TRS ancorado em `c_local`** (a reparametrização do cabeçalho).
///
/// A translação devolvida é `pose.apply(c_local)` — onde o CENTRO da arte aparece no
/// espaço do objeto —, e rotação/escala saem da parte linear (decomposição 2D padrão:
/// norma da 1ª coluna + `atan2`, com a 2ª escala assinada pelo determinante). Exata
/// para todo afim TRS — que é tudo que uma pose autorada pode ser: ela nasce
/// translação pura (Fase 1) e este gizmo só escreve TRS ([`trs_to_pose`]).
#[must_use]
pub(crate) fn pose_trs(pose: Pose, c_local: [f32; 2]) -> TransformSnapshot {
    let c = pose.coeffs();
    // T1.3.5: transcendental via libm (bit-idêntico cross-OS, HR-5).
    let sx = libm::hypotf(c[0], c[1]);
    let rotation = libm::atan2f(c[1], c[0]);
    let det = c[0] * c[3] - c[1] * c[2];
    let sy = if sx > f32::EPSILON { det / sx } else { c[3] };
    let tc = pose.apply(Vec2::new(c_local[0], c_local[1]));
    TransformSnapshot {
        translation: [tc.x, tc.y],
        rotation,
        scale: [sx, sy],
    }
}

/// **TRS ancorado em `c_local` → Pose** — o inverso exato de [`pose_trs`]:
/// `pose(p) = t_c + R·S·(p − c_local)`, expandido nos 6 coeficientes.
#[must_use]
pub(crate) fn trs_to_pose(s: TransformSnapshot, c_local: [f32; 2]) -> Pose {
    // T1.3.5: transcendental via libm (bit-idêntico cross-OS, HR-5).
    let (sin_r, cos_r) = libm::sincosf(s.rotation);
    let a = s.scale[0] * cos_r;
    let b = s.scale[0] * sin_r;
    let cc = -s.scale[1] * sin_r;
    let d = s.scale[1] * cos_r;
    let tx = s.translation[0] - (a * c_local[0] + cc * c_local[1]);
    let ty = s.translation[1] - (b * c_local[0] + d * c_local[1]);
    Pose::from_coeffs([a, b, cc, d, tx, ty])
}

/// Um passo do arrasto, PURO: o TRS novo sai do motor canônico do gizmo
/// ([`ph2d_editor::compute_gizmo_transform`] — a mesma conta do sprite, com
/// modifiers e snap) e volta a ser pose pelo MESMO `c_local` do Down.
#[must_use]
pub(crate) fn pose_after_drag(
    drag: &ph2d_editor::GizmoDragState,
    cam: &GizmoCamera,
    mods: GizmoModifiers,
    snap: GizmoSnap,
    c_local: [f32; 2],
) -> Pose {
    let new_t = ph2d_editor::compute_gizmo_transform(drag, cam, mods, snap, None);
    trs_to_pose(new_t, c_local)
}

/// O `Transform` do ECS na linguagem do gizmo.
fn snapshot_of(t: ph2d_ecs::Transform) -> TransformSnapshot {
    TransformSnapshot {
        translation: [t.translation.x, t.translation.y],
        rotation: t.rotation,
        scale: [t.scale.x, t.scale.y],
    }
}

/// O alvo resolvido do gizmo de pose: a chave visível instanciada + a geometria da
/// reparametrização (centro/meia-extensão da arte crua) + a pose atual dela.
struct PoseTarget {
    oid: FlipObjectId,
    lid: LayerId,
    key: Frame,
    c_local: [f32; 2],
    h_local: [f32; 2],
    pose: Pose,
}

/// O alvo do gizmo de pose: a chave visível da camada ativa, **se** o desenho dela é
/// uma INSTÂNCIA (arte compartilhada — é a pose que move; arte exclusiva segue no
/// arrasto-geometria do Edit e não abre gizmo).
#[must_use]
fn pose_target(
    flip: &FlipDoc,
    playhead: &Playhead,
    active_layer: Option<LayerId>,
) -> Option<PoseTarget> {
    let (oid, lid, key, did) = crate::flip_select::visible_key(flip, playhead, active_layer)?;
    let obj = flip.object(oid)?;
    let drawing = obj.drawing(did)?;
    if !drawing.is_instanced() {
        return None;
    }
    let (c_local, h_local) = drawing_center_half(drawing)?;
    Some(PoseTarget {
        oid,
        lid,
        key,
        c_local,
        h_local,
        pose: obj.frame_pose(lid, key),
    })
}

/// A `GizmoView` da POSE da chave visível — `None` fora do alvo (sem instância, sem
/// arte, entidade morta). O chamador (render_loop) já gateia tool Flip + modo Edit.
///
/// A caixa é o OBB da arte crua sob `objeto ∘ pose` (mesma composição TRS do render):
/// centro em `parent ∘ t_c`, meia-extensão `h_local ⊙ |escala composta|`, rotação
/// composta. O pivô É o centro (spec do Enio: girar em torno da arte posada).
/// Os insumos do lado do `App` para a [`pose_view`] — agrupados para a chamada no
/// render_loop não carregar meia dúzia de escalares soltos.
#[derive(Clone, Copy)]
pub(crate) struct PoseViewInputs<'a> {
    pub(crate) playhead: &'a Playhead,
    pub(crate) active_layer: Option<LayerId>,
    pub(crate) last_pointer: (f32, f32),
}

#[must_use]
pub(crate) fn pose_view(
    sim: &SimWorld,
    flip: &FlipDoc,
    map: &FlipEntityMap,
    inputs: PoseViewInputs<'_>,
    camera: &Camera2d,
    window_size: WindowSize,
) -> Option<GizmoView> {
    let t = pose_target(flip, inputs.playhead, inputs.active_layer)?;
    let e = map
        .get(&t.oid)
        .map(|&b| ph2d_ecs::Entity::from_bits(b))
        .filter(|e| sim.world().get_entity(*e).is_ok())?;
    let parent = snapshot_of(world_transform(sim, e));
    let start = pose_trs(t.pose, t.c_local);
    let world = ph2d_editor::compose_snapshot(parent, start);
    let half = [
        (t.h_local[0] * world.scale[0]).abs(),
        (t.h_local[1] * world.scale[1]).abs(),
    ];
    let (cx, cy) = (world.translation[0], world.translation[1]);
    Some(GizmoView {
        bbox_min_world: [cx - half[0], cy - half[1]],
        bbox_max_world: [cx + half[0], cy + half[1]],
        pivot_world: [cx, cy],
        pivot_tool_active: false,
        rotation: world.rotation,
        camera_center: camera.center,
        camera_height_world: camera.height_world,
        window_w: window_size.width as f32,
        window_h: window_size.height as f32,
        canvas: ph2d_editor::zones::Rect::new(
            0.0,
            0.0,
            window_size.width as f32,
            window_size.height as f32,
        ),
        cursor_screen: Some(inputs.last_pointer),
    })
}

impl crate::App {
    /// Pen-DOWN num handle do gizmo de pose. `true` = arrasto de pose aberto
    /// (consumido — o caminho genérico de gizmo e o Edit não veem este clique).
    ///
    /// Reconhece o alvo pelo `gizmo_hit_map` (`GizmoTarget::FlipPose`) — os handles só
    /// existem no hit-index quando a `pose_view` foi publicada neste frame (tool Flip +
    /// modo Edit + quadro instanciado), então a pré-condição já está provada pela
    /// pintura.
    pub(crate) fn flip_pose_gizmo_down(&mut self, x: f32, y: f32) -> bool {
        if !self.flip_wants_edit() {
            return false;
        }
        let playhead = self.playhead;
        let active_layer = self.flip_active_layer;
        let ctrl = self.modifiers.control_key() || self.modifiers.super_key();
        let Some(gfx) = self.gfx.as_ref() else {
            return false;
        };
        let Some(hero) = gfx.hero_screen.as_ref() else {
            return false;
        };
        let Some(hit_id) = hero.hit_index.hit(x, y) else {
            return false;
        };
        let Some(hit) = hero.gizmo.gizmo_hit_map.get(&hit_id).copied() else {
            return false;
        };
        if hit.target != ph2d_editor::GizmoTarget::FlipPose {
            return false;
        }
        let Some(t) = pose_target(&gfx.flip, &playhead, active_layer) else {
            return false;
        };
        let Some(e) = self
            .flip_entities
            .get(&t.oid)
            .map(|&b| ph2d_ecs::Entity::from_bits(b))
            .filter(|e| gfx.sim.world().get_entity(*e).is_ok())
        else {
            return false;
        };
        let parent = snapshot_of(world_transform(&gfx.sim, e));
        let start = pose_trs(t.pose, t.c_local);
        let world_snap = ph2d_editor::compose_snapshot(parent, start);
        let win = gfx.surface.size();
        let world_pos = gfx.camera.screen_to_world((x, y), win);
        // Rotate pivota no centro da arte (= a translação do TRS); scale, no canto/
        // borda OPOSTOS (ou no centro com Ctrl) — a mesma política do sprite.
        let pivot = ph2d_editor::anchor_pivot_world(hit.kind, t.h_local, world_snap, ctrl);
        self.flip_pose_drag = Some(FlipPoseDrag {
            drag: ph2d_editor::GizmoDragState {
                kind: hit.kind,
                entity_bits: e.to_bits(),
                start_screen: (x, y),
                cursor_screen: (x, y),
                start_transform: start,
                pivot_world: pivot,
                start_cursor_world: world_pos,
                sprite_half_intrinsic: t.h_local,
                anchor_is_center: ctrl,
                target: ph2d_editor::GizmoTarget::FlipPose,
                parent_world: parent,
                turns: 0,
            },
            oid: t.oid,
            lid: t.lid,
            key: t.key,
            c_local: t.c_local,
        });
        true
    }

    /// Pen-MOVE com um arrasto de pose aberto: recomputa a pose da chave a partir do
    /// snapshot do Down (nunca do estado vivo — deltas compostos por frame driftariam)
    /// e a escreve pelo choke point `set_frame_pose`. `true` = consumido.
    pub(crate) fn flip_pose_gizmo_move(&mut self, x: f32, y: f32) -> bool {
        let Some(mut pd) = self.flip_pose_drag else {
            return false;
        };
        let mods = GizmoModifiers {
            shift: self.modifiers.shift_key(),
            ctrl: self.modifiers.control_key() || self.modifiers.super_key(),
            alt: self.modifiers.alt_key(),
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return true;
        };
        let size = gfx.surface.size();
        let cam = GizmoCamera {
            center: gfx.camera.center,
            height_world: gfx.camera.height_world,
            window_w: size.width as f32,
            window_h: size.height as f32,
        };
        let snap = gfx
            .hero_screen
            .as_ref()
            .map(|h| GizmoSnap {
                move_meters: h.project.snap_move_meters,
                rotate_deg: h.project.snap_rotate_deg,
            })
            .unwrap_or_default();
        // O cursor avança ATRAVÉS do drag (o contador de voltas do Rotate mora aí —
        // pular isto reintroduz o salto de 2π no corte do atan2).
        pd.drag.advance_cursor((x, y), &cam);
        let pose = pose_after_drag(&pd.drag, &cam, mods, snap, pd.c_local);
        if let Some(obj) = gfx.flip.object_mut(pd.oid) {
            obj.set_frame_pose(pd.lid, pd.key, pose);
        }
        self.flip_pose_drag = Some(pd);
        self.title_dirty = true;
        true
    }

    /// Pen-UP: fecha o arrasto de pose. `true` = havia um (consumido). O passo de
    /// undo sai do diff pós-frame, como todo gesto do Flip (`post_frame_undo` espera
    /// o botão soltar).
    pub(crate) fn flip_pose_gizmo_up(&mut self) -> bool {
        self.flip_pose_drag.take().is_some()
    }
}

#[cfg(test)]
#[path = "flip_pose_gizmo_tests.rs"]
mod tests;
