//! ADR-0114 §4.A — **o gizmo da SELEÇÃO** (modo Edit, arte EXCLUSIVA).
//!
//! O Edit Mode já MOVE a seleção (traço no W6.1, ponto no W8), mas não gira/escala.
//! O caminho do PH2D é o gizmo de sprite agindo sobre a seleção — e a W7.5 (o gizmo
//! da POSE, [`crate::flip_pose_gizmo`]) já resolveu 90% dele. Este módulo é o ESPELHO
//! daquele, com uma única diferença de fundo:
//!
//! - **A pose gizmo** escreve a **pose da chave** (a instância inteira se move como um
//!   bloco rígido; a arte é compartilhada, não se pode deformá-la).
//! - **A selection gizmo** assa o delta na **GEOMETRIA** dos pontos selecionados (arte
//!   exclusiva — deformar é o objetivo).
//!
//! Os dois são **mutuamente exclusivos por `is_instanced`**: numa instância aparece a
//! pose gizmo, numa arte exclusiva com seleção aparece a selection gizmo. Nenhuma
//! precisa de toast — a que não se aplica simplesmente não publica handles.
//!
//! A reparametrização é a MESMA da pose (reusa [`crate::flip_pose_gizmo::pose_trs`] /
//! [`crate::flip_pose_gizmo::trs_to_pose`]): a seleção é um "sprite" cujo pivô é o
//! **centro da bbox dos pontos selecionados** (`c_art`, em coords da ARTE), o
//! `Transform` local é o TRS da pose ancorado nesse centro e o `parent_world` é o
//! afim do OBJETO. Assim o motor canônico do gizmo ([`ph2d_editor::compute_gizmo_transform`],
//! com modifiers/snap/contador de voltas) roda byte a byte.
//!
//! **O bake (o que difere da pose):** o gizmo produz um TRS novo em espaço de OBJETO;
//! a geometria vive em ART. O delta afim ART→ART é
//! `pose⁻¹ ∘ new_aff ∘ start_aff⁻¹ ∘ pose` ([`art_bake_xform`]) — descer o ponto pela
//! pose, aplicar a mudança do frame do gizmo, subir de volta. Numa pose de translação
//! pura (o caso comum de arte exclusiva) isto é a identidade nas duas pontas e o bake
//! reduz a uma rotação/escala/translação em torno de `c_art`. **Seed = sample:** a
//! caixa e o pivô saem da MESMA cadeia `objeto ∘ pose` que o render dobra.
//!
//! **Snapshot no Down, recomputa do snapshot:** as posições de partida de cada ponto
//! selecionado são congeladas no Down; cada Move recomputa `p' = M_art(p₀)` do snapshot
//! — nunca compõe por-frame (deltas compostos driftariam, como no gizmo da pose).
//!
//! **O interior NÃO é um handle — mas a área ARRASTA** (smoke do §4.A, Enio: *"qualquer
//! clique na área do gizmo"*). Registrar um interior no hit-index tornaria `on_canvas`
//! falso sobre a seleção inteira e mataria a re-seleção ali dentro; então quem responde
//! pelo interior é o **down do canvas do Edit**, que já roda lá: errar a tinta dentro da
//! [`grabbable_selection_box`] vira um `Move` do grupo em vez de um marquee
//! (`flip_select::plan_down`). **Tinta primeiro** — clicar num outro traço que passa por
//! dentro da caixa ainda o seleciona. Os handles keyed (`GizmoTarget::FlipSelection`)
//! registram só rotate/scale, que é o que falta ao gesto de canvas.
//!
//! **Vale nos DOIS domínios:** no Stroke a caixa enquadra os traços selecionados; no
//! Point, **os pontos selecionados** — girar/escalar um punhado de âncoras é metade do que
//! o §4.A existe para dar. Só **sem EXTENSÃO não há gizmo** (um ponto único não se
//! rotaciona nem se escalona), e o *"o gizmo do stroke some no `Select: Point`"* vem de
//! entrar no domínio **limpo** (`FlipDrawing::enter_point_domain`), não de recusar o
//! domínio. As regras moram numa função só — [`grabbable_selection_box`], a porta que
//! decide onde a seleção é agarrável. **Se você mexer neste módulo, é ela que decide tudo.**

use ph2d_core::{Playhead, Vec2};
use ph2d_ecs::SimWorld;
use ph2d_editor::{GizmoCamera, GizmoModifiers, GizmoSnap, GizmoView, TransformSnapshot};
use ph2d_flip::{DrawingId, FlipDoc, FlipDrawing, FlipObjectId, LayerId, Pose};
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_vec_scene::Xform;

use crate::flip_entities::FlipEntityMap;
use crate::flip_pose_gizmo::{pose_trs, trs_to_pose};
use crate::flip_transform::key_xform;
use crate::vec_transform::world_transform;

/// Qual anel de um traço o ponto snapshotado pertence: a polilinha principal ou um
/// dos buracos. Os buracos só entram no snapshot quando o traço INTEIRO está
/// selecionado (senão não andam — o mesmo critério do `translate_selected_points`).
#[derive(Clone, Copy, Debug)]
pub(crate) enum Ring {
    Main,
    Hole(usize),
}

/// A posição de PARTIDA (Down) de um ponto selecionado, em coords da ARTE. Cada Move
/// recomputa a partir daqui — nunca do estado vivo.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SelPoint {
    pub(crate) si: usize,
    pub(crate) ring: Ring,
    pub(crate) pi: usize,
    pub(crate) p0: Vec2,
}

/// O arrasto de SELEÇÃO em curso: o estado genérico do gizmo + o ALVO da escrita + o
/// `c_art`/pose da reparametrização + o snapshot dos pontos. O `Vec` torna isto
/// não-`Copy` (≠ `FlipPoseDrag`), então o `move` faz `take`-e-restaura.
#[derive(Clone, Debug)]
pub(crate) struct FlipSelectionDrag {
    pub(crate) drag: ph2d_editor::GizmoDragState,
    pub(crate) oid: FlipObjectId,
    pub(crate) did: DrawingId,
    pub(crate) pose: Pose,
    pub(crate) start: TransformSnapshot,
    pub(crate) points: Vec<SelPoint>,
}

/// Centro + meia-extensão da bbox dos pontos **SELECIONADOS** de um desenho, em coords
/// da ARTE. `None` quando nada está selecionado (o gizmo não abre). Domínio unificado:
/// `point_selected` já resolve Curve (broadcast do estado do traço) e Point (por-ponto).
#[must_use]
pub(crate) fn selection_center_half(d: &FlipDrawing) -> Option<([f32; 2], [f32; 2])> {
    let mut acc: Option<([f32; 2], [f32; 2])> = None;
    for s in &d.strokes {
        let pts = s.positions();
        for (i, p) in pts.iter().enumerate() {
            if s.point_selected(i) {
                acc = Some(match acc {
                    None => ([p.x, p.y], [p.x, p.y]),
                    Some((lo, hi)) => (
                        [lo[0].min(p.x), lo[1].min(p.y)],
                        [hi[0].max(p.x), hi[1].max(p.y)],
                    ),
                });
            }
        }
    }
    let (lo, hi) = acc?;
    Some((
        [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5],
        [(hi[0] - lo[0]) * 0.5, (hi[1] - lo[1]) * 0.5],
    ))
}

/// **A caixa AGARRÁVEL da seleção**, em coords da ARTE (`(centro, meia-extensão)`) —
/// `None` **exatamente** quando o gizmo da seleção não é publicado.
///
/// Uma pergunta, dois consumidores: a [`selection_view`] a **desenha** e o down do Edit
/// (`flip_select::flip_edit_canvas_down`) a torna **arrastável**. Se fossem duas funções,
/// divergiriam — e o artista veria uma caixa que não pega, ou pegaria onde não vê. É a
/// mesma família do BUGS #18 (a costura), e a razão de isto ser uma porta só.
///
/// **Vale nos DOIS domínios** — no Stroke a caixa enquadra os traços selecionados; no
/// Point, **os pontos selecionados** (2+). *"O gizmo do stroke deve sumir no `Select:
/// Point`"* (Enio) é entregue por `FlipDrawing::enter_point_domain`, que entra **limpo**:
/// sem seleção não há caixa. Recusar o domínio inteiro seria tirar o gizmo de **múltiplos
/// pontos**, que é metade do que o §4.A existe para dar.
///
/// **Duas recusas:**
/// - **Arte INSTANCIADA** — a instância é do gizmo de POSE (arte compartilhada não deforma).
/// - **Seleção sem EXTENSÃO** — um ponto único (ou N coincidentes) **não se rotaciona nem
///   se escalona**, e os 8 handles empilhados sobre ele roubariam justamente o clique que o
///   MOVE (Enio). O realce do ponto já diz que ele está selecionado; o arrasto dele é o
///   gesto do W8. O limiar é o **zero exato** — a meia-extensão de um ponto é `0.0` por
///   construção, então não há épsilon a inventar
///   (`feedback_a_threshold_must_live_where_the_domain_is_empty`).
#[must_use]
pub(crate) fn grabbable_selection_box(d: &FlipDrawing) -> Option<([f32; 2], [f32; 2])> {
    if d.is_instanced() {
        return None;
    }
    let (c, h) = selection_center_half(d)?;
    (h[0] > 0.0 || h[1] > 0.0).then_some((c, h))
}

/// O ponto (coords da ARTE) está DENTRO da caixa agarrável da seleção? É o que faz um
/// clique no **vazio dentro do gizmo** virar um MOVE em vez de um marquee (Enio, smoke do
/// §4.A: *"qualquer clique na área do gizmo"*). Sem gizmo publicado, sempre `false` — a
/// área não existe.
#[must_use]
pub(crate) fn selection_box_contains(d: &FlipDrawing, p: Vec2) -> bool {
    grabbable_selection_box(d)
        .is_some_and(|(c, h)| (p.x - c[0]).abs() <= h[0] && (p.y - c[1]).abs() <= h[1])
}

/// O `Transform` do ECS na linguagem do gizmo (espelho do `flip_pose_gizmo`).
fn snapshot_of(t: ph2d_ecs::Transform) -> TransformSnapshot {
    TransformSnapshot {
        translation: [t.translation.x, t.translation.y],
        rotation: t.rotation,
        scale: [t.scale.x, t.scale.y],
    }
}

/// O afim de um TRS puro (sem ancoragem), reusando a math testada do gizmo da pose
/// ([`trs_to_pose`] com `c_local = 0` = a translação vai crua) + [`key_xform`]. É o
/// `T·R·S` na convenção do [`Pose`]/[`Xform`].
fn trs_affine(t: TransformSnapshot) -> Xform {
    key_xform(trs_to_pose(t, [0.0, 0.0]))
}

/// **O delta afim ART→ART do arrasto**: `pose⁻¹ ∘ new_aff ∘ start_aff⁻¹ ∘ pose`.
///
/// `start`/`new_t` são TRS em espaço de OBJETO (o `parent_world` do drag é o afim do
/// objeto), e a geometria vive em ART; a pose (`key_xform`) faz a ponte ART↔objeto. A
/// conjugação pela pose garante que sob uma pose girada a geometria ande na direção
/// certa (regra-mãe #10) — numa pose de translação pura as duas pontas se cancelam e
/// sobra `new_aff ∘ start_aff⁻¹` (uma rotação/escala/translação em torno de `c_art`).
///
/// Pose/afim degenerado devolve identidade (`inverse` = `None`) — travar seria pior.
#[must_use]
fn art_bake_xform(pose: Pose, start: TransformSnapshot, new_t: TransformSnapshot) -> Xform {
    let pose_aff = key_xform(pose);
    let pose_inv = pose_aff.inverse().unwrap_or(Xform::IDENTITY);
    let start_inv = trs_affine(start).inverse().unwrap_or(Xform::IDENTITY);
    let new_aff = trs_affine(new_t);
    // `X.then(&Y)` = aplica X, depois Y = `Y ∘ X`. Ordem: pose, start⁻¹, new, pose⁻¹.
    pose_aff.then(&start_inv).then(&new_aff).then(&pose_inv)
}

/// Os pontos a transformar: os selecionados de cada traço (main), MAIS os buracos dos
/// traços INTEIROS selecionados (`all_points_selected` — o mesmo critério do
/// `translate_selected_points`; buracos não têm seleção própria e só andam com o anel
/// externo). Cada um carrega a posição de PARTIDA, em coords da ARTE.
#[must_use]
fn snapshot_selected_points(drawing: &FlipDrawing) -> Vec<SelPoint> {
    let mut out = Vec::new();
    for (si, s) in drawing.strokes.iter().enumerate() {
        for (pi, p) in s.positions().iter().enumerate() {
            if s.point_selected(pi) {
                out.push(SelPoint {
                    si,
                    ring: Ring::Main,
                    pi,
                    p0: *p,
                });
            }
        }
        if s.all_points_selected() {
            for (h, hole) in s.holes.iter().enumerate() {
                for (pi, p) in hole.iter().enumerate() {
                    out.push(SelPoint {
                        si,
                        ring: Ring::Hole(h),
                        pi,
                        p0: *p,
                    });
                }
            }
        }
    }
    out
}

/// O alvo resolvido do gizmo de seleção: a chave visível de arte EXCLUSIVA com pelo
/// menos um ponto selecionado + a geometria da reparametrização (centro/meia-extensão
/// dos pontos selecionados) + a pose atual.
struct SelTarget {
    oid: FlipObjectId,
    did: DrawingId,
    c_local: [f32; 2],
    h_local: [f32; 2],
    pose: Pose,
}

/// O alvo do gizmo de seleção — `None` fora dele: sem chave visível, ou fora do que a
/// [`grabbable_selection_box`] admite (arte instanciada · sem seleção · seleção sem
/// extensão). É o inverso exato do `flip_pose_gizmo::pose_target` (que exige instância).
#[must_use]
fn selection_target(
    flip: &FlipDoc,
    playhead: &Playhead,
    active_layer: Option<LayerId>,
) -> Option<SelTarget> {
    let (oid, lid, key, did) = crate::flip_select::visible_key(flip, playhead, active_layer)?;
    let obj = flip.object(oid)?;
    let drawing = obj.drawing(did)?;
    // A MESMA porta que o down do Edit usa para decidir o interior arrastável — desenhar
    // uma caixa que não pega (ou pegar onde não se vê) seria o bug de sempre.
    let (c_local, h_local) = grabbable_selection_box(drawing)?;
    Some(SelTarget {
        oid,
        did,
        c_local,
        h_local,
        pose: obj.frame_pose(lid, key),
    })
}

/// Os insumos do lado do `App` para a [`selection_view`] (espelho de `PoseViewInputs`).
#[derive(Clone, Copy)]
pub(crate) struct SelectionViewInputs<'a> {
    pub(crate) playhead: &'a Playhead,
    pub(crate) active_layer: Option<LayerId>,
    pub(crate) last_pointer: (f32, f32),
}

/// A `GizmoView` da SELEÇÃO — `None` fora do alvo. O chamador (render_loop) já gateia
/// tool Flip + modo Edit. A caixa é o OBB da seleção sob `objeto ∘ pose` (a MESMA
/// composição TRS do render): centro em `parent ∘ t_c`, meia-extensão
/// `h_local ⊙ |escala composta|`, rotação composta. O pivô É o centro.
#[must_use]
pub(crate) fn selection_view(
    sim: &SimWorld,
    flip: &FlipDoc,
    map: &FlipEntityMap,
    inputs: SelectionViewInputs<'_>,
    camera: &Camera2d,
    window_size: WindowSize,
) -> Option<GizmoView> {
    let t = selection_target(flip, inputs.playhead, inputs.active_layer)?;
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
    /// Pen-DOWN num handle do gizmo de seleção. `true` = arrasto aberto (consumido — o
    /// caminho genérico de gizmo e o Edit não veem este clique). Reconhece o alvo pelo
    /// `gizmo_hit_map` (`GizmoTarget::FlipSelection`); os handles só existem quando a
    /// `selection_view` foi publicada neste frame, então a pré-condição já está provada
    /// pela pintura.
    pub(crate) fn flip_selection_gizmo_down(&mut self, x: f32, y: f32) -> bool {
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
        if hit.target != ph2d_editor::GizmoTarget::FlipSelection {
            return false;
        }
        let Some(t) = selection_target(&gfx.flip, &playhead, active_layer) else {
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
        let Some(drawing) = gfx.flip.object(t.oid).and_then(|o| o.drawing(t.did)) else {
            return false;
        };
        let points = snapshot_selected_points(drawing);
        let parent = snapshot_of(world_transform(&gfx.sim, e));
        let start = pose_trs(t.pose, t.c_local);
        let world_snap = ph2d_editor::compose_snapshot(parent, start);
        let win = gfx.surface.size();
        let world_pos = gfx.camera.screen_to_world((x, y), win);
        // Rotate pivota no centro da seleção; scale, no canto/borda OPOSTOS (ou no
        // centro com Ctrl) — a mesma política do sprite/pose.
        let pivot = ph2d_editor::anchor_pivot_world(hit.kind, t.h_local, world_snap, ctrl);
        self.flip_selection_drag = Some(FlipSelectionDrag {
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
                target: ph2d_editor::GizmoTarget::FlipSelection,
                parent_world: parent,
                turns: 0,
            },
            oid: t.oid,
            did: t.did,
            pose: t.pose,
            start,
            points,
        });
        // A seleção vira o alvo do gesto — o "alvo vivo" sai de cena (regra do down do Edit).
        self.flip_live_clear();
        true
    }

    /// Pen-MOVE com um arrasto de seleção aberto: recomputa cada ponto do snapshot do
    /// Down pelo delta afim do gizmo e o escreve na geometria. `true` = consumido.
    /// (`take`-e-restaura porque o snapshot é um `Vec`, não `Copy`.)
    pub(crate) fn flip_selection_gizmo_move(&mut self, x: f32, y: f32) -> bool {
        let Some(mut pd) = self.flip_selection_drag.take() else {
            return false;
        };
        let mods = GizmoModifiers {
            shift: self.modifiers.shift_key(),
            ctrl: self.modifiers.control_key() || self.modifiers.super_key(),
            alt: self.modifiers.alt_key(),
        };
        let Some(gfx) = self.gfx.as_mut() else {
            self.flip_selection_drag = Some(pd);
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
        // O cursor avança ATRAVÉS do drag (o contador de voltas do Rotate mora aí).
        pd.drag.advance_cursor((x, y), &cam);
        let new_t = ph2d_editor::compute_gizmo_transform(&pd.drag, &cam, mods, snap, None);
        let m = art_bake_xform(pd.pose, pd.start, new_t);
        if let Some(dr) = gfx
            .flip
            .object_mut(pd.oid)
            .and_then(|o| o.drawing_mut(pd.did))
        {
            for sp in &pd.points {
                let q = m.apply([f64::from(sp.p0.x), f64::from(sp.p0.y)]);
                let np = Vec2::new(q[0] as f32, q[1] as f32);
                match sp.ring {
                    Ring::Main => {
                        if let Some(p) = dr
                            .strokes
                            .get_mut(sp.si)
                            .and_then(|s| s.positions_mut().get_mut(sp.pi))
                        {
                            *p = np;
                        }
                    }
                    Ring::Hole(h) => {
                        if let Some(p) = dr
                            .strokes
                            .get_mut(sp.si)
                            .and_then(|s| s.holes.get_mut(h))
                            .and_then(|hole| hole.get_mut(sp.pi))
                        {
                            *p = np;
                        }
                    }
                }
            }
        }
        self.flip_selection_drag = Some(pd);
        self.title_dirty = true;
        true
    }

    /// Pen-UP: fecha o arrasto de seleção. `true` = havia um. O passo de undo sai do
    /// diff pós-frame, como todo gesto do Flip.
    pub(crate) fn flip_selection_gizmo_up(&mut self) -> bool {
        self.flip_selection_drag.take().is_some()
    }
}

#[cfg(test)]
#[path = "flip_selection_gizmo_tests.rs"]
mod tests;
