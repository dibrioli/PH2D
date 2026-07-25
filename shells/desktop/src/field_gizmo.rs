//! O gizmo de canvas de um **FIELD ESPACIAL** dos Motion Nodes (`field.box`, …).
//!
//! Um field espacial mascara/pesa as instâncias por POSIÇÃO, e a posição vive no mesmo
//! espaço de MUNDO que os sprites (as instâncias compõem no canvas via
//! `SpriteRenderer::render_with_extra`). Arrastar `center_x`/`center_y` num slider é
//! caçar a rotação; o idioma dos apps pro (C4D/Cavalry/Houdini mograph) é uma **alça na
//! tela** — mover/girar/escalar o field onde a ação está.
//!
//! # É o ESPELHO do [`crate::flip_selection_gizmo`], com um sink diferente
//!
//! O Flip já resolveu "um drag de gizmo de sprite que escreve num sink que **não é** um
//! `Transform` de entidade": a pose de uma chave (`FlipPose`) e a geometria de uma
//! seleção (`FlipSelection`) são [`ph2d_editor::GizmoTarget`]s próprios, com espaço de id
//! keyed, reconhecidos ANTES do caminho genérico de gizmo. Este módulo é mais um:
//! [`ph2d_editor::GizmoTarget::MotionField`], cujo apply escreve os **params do NÓ**
//! (`center_x`/`center_y`/`rotation`/`width`/`height` via `Graph::set_param`).
//!
//! **Isolamento por construção — a resposta ao *"não vai atrapalhar os sprites?"*:** um
//! field é um NÓ do grafo, não uma entidade ECS, então o gizmo de sprite (`hero.gizmo.view`,
//! chaveado em `hero.gizmo.selection` = bits de entidade) fica **intocado**. A view do
//! field mora no seu próprio slot (`hero.gizmo.field_view`), publicada SÓ com a tool Motion
//! ativa + um field espacial selecionado — modalidade que os torna mutuamente exclusivos
//! (você manipula sprites na tool de move/select, com a Motion desligada; ali o field não
//! tem hit-region nenhuma). Os dois nem compartilham modelo de seleção.
//!
//! **Seed = sample:** a caixa (`field_view`) e a semente do drag (`field_gizmo_down`) leem
//! os params pela MESMA porta que o painel de params usa
//! ([`crate::render_loop::motion_bridge::params::param_value`], override→default), então a
//! alça concorda com os sliders. O writeback ([`params_from`]) é o inverso exato da semente
//! ([`seed_start`]) — um round-trip sob transform identidade devolve os mesmos params (a
//! lição recorrente `feedback_derived_coordinate_seed_must_match_sample`).

use ph2d_editor::screens::layout::CenterSplit;
use ph2d_editor::{
    GizmoCamera, GizmoDragState, GizmoModifiers, GizmoSnap, GizmoTarget, GizmoView,
    TransformSnapshot,
};
use ph2d_host::WindowSize;
use ph2d_nodegraph::graph::NodeId;
use ph2d_nodegraph::node::NodeTypeId;
use ph2d_render::Camera2d;

use crate::motion_state::MotionState;

/// As dims `(w, h)` que a CENA de fato ocupa — o sub-retângulo do split (a porta única
/// [`CenterSplit::scene_viewport`]) quando a tool Motion divide o centro, ou a janela cheia
/// fora do split. ⚠️ **Todo mapeamento mundo↔tela do chrome da cena (a grade do mundo, o
/// gizmo de field e o drag dele) TEM de usar isto**, casando com o `uniform_for_subrect` +
/// `set_viewport` que o render usa (present.rs) — senão a cena renderiza na banda e o
/// chrome projeta a janela cheia, e um ponto de mundo cai em dois lugares (o drift crônico
/// do Motion, 2026-07-25). O sub-retângulo é ancorado em `(0,0)`, então só as DIMS mudam.
#[must_use]
pub(crate) fn scene_window_wh(center_split: CenterSplit, window: WindowSize) -> (f32, f32) {
    let (w, h) = (window.width as f32, window.height as f32);
    center_split.scene_viewport(w, h).map_or((w, h), |r| (r[2], r[3]))
}

/// Os nomes dos params de um field espacial que o gizmo dirige. Uma tabela por tipo de
/// field (v1: só `field.box`) — quando `field.radial_sweep` etc. chegarem, cada um
/// acrescenta uma [`FieldGizmoSpec`], sem tocar a máquina do gizmo. `center`/`rotation`/
/// as extensões são a "Coordinates" da família (doc 63 §0.1); um field NÃO-espacial
/// (`index_range`, por rank) não tem spec e não ganha gizmo.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct FieldGizmoSpec {
    pub(crate) center_x: &'static str,
    pub(crate) center_y: &'static str,
    /// Extensão CHEIA no eixo local X (a meia-extensão do gizmo é `/2`).
    pub(crate) width: &'static str,
    /// Extensão CHEIA no eixo local Y.
    pub(crate) height: &'static str,
    /// Rotação em GRAUS (o `field.box` a consome como `cos_sin_cycles(rotation/360)`).
    pub(crate) rotation: &'static str,
}

/// A spec do `field.box`: os nomes canônicos dos seus params (ver
/// `ph2d-node-field-box`). Extensões CHEIAS, rotação em graus.
const BOX_SPEC: FieldGizmoSpec = FieldGizmoSpec {
    center_x: "center_x",
    center_y: "center_y",
    width: "width",
    height: "height",
    rotation: "rotation",
};

/// A [`FieldGizmoSpec`] de um tipo de nó, ou `None` se o nó **não** é um field
/// ESPACIAL (o `index_range` é por rank, sem geometria; o `combine` compõe dois fields,
/// sem geometria própria). Porta única — a view, o down e o gate perguntam à mesma.
#[must_use]
pub(crate) fn spec_for(type_id: NodeTypeId) -> Option<FieldGizmoSpec> {
    if type_id == NodeTypeId::of("field.box") {
        Some(BOX_SPEC)
    } else {
        None
    }
}

/// Embrulha graus em `[-180, 180]` — a faixa declarada do param `rotation` do field
/// (`PARAM_HINTS`). Puro aritmético (`rem_euclid` é o módulo do livro; sem transcendental,
/// HR-5). Não afeta o drag vivo (que usa `start_transform` + cursor); só normaliza o valor
/// ARMAZENADO para bater com o slider do painel.
#[must_use]
fn wrap180(deg: f32) -> f32 {
    (deg + 180.0).rem_euclid(360.0) - 180.0
}

/// Monta a [`GizmoView`] de um field a partir dos seus params — a MESMA álgebra do gizmo
/// de sprite (caixa não-rotacionada center±half, rotação carregada à parte e aplicada em
/// torno do pivô). Para um field o pivô É o centro (âncora zero), então
/// `bbox = center ± half` direto.
// Um builder de `GizmoView` legitimamente carrega a pose (5) + câmera + dims da cena +
// cursor; espelha o `gizmo_view_from` do `vec_gizmo_view` (mesmo domínio). ⚠️ `win_w`/
// `win_h` são as dims da CENA ([`scene_window_wh`]), NÃO da janela cheia — é o que casa o
// gizmo com o `set_viewport` do render sob o split (o fix do drift). O `canvas` (scissor)
// usa as mesmas dims, então o gizmo é recortado na banda em vez de invadir o painel do grafo.
#[allow(clippy::too_many_arguments)]
#[must_use]
fn view_from_params(
    cx: f32,
    cy: f32,
    w: f32,
    h: f32,
    rot_deg: f32,
    camera: &Camera2d,
    win_w: f32,
    win_h: f32,
    last_pointer: (f32, f32),
) -> GizmoView {
    let half = [(w * 0.5).abs(), (h * 0.5).abs()];
    GizmoView {
        bbox_min_world: [cx - half[0], cy - half[1]],
        bbox_max_world: [cx + half[0], cy + half[1]],
        pivot_world: [cx, cy],
        pivot_tool_active: false,
        rotation: rot_deg.to_radians(),
        camera_center: camera.center,
        camera_height_world: camera.height_world,
        window_w: win_w,
        window_h: win_h,
        canvas: ph2d_editor::zones::Rect::new(0.0, 0.0, win_w, win_h),
        cursor_screen: Some(last_pointer),
    }
}

/// O `TransformSnapshot` de PARTIDA (Down) de um field na linguagem do gizmo + a
/// meia-extensão INTRÍNSECA (pré-escala). A escala começa em `1`, a rotação em radianos;
/// `width`/`height` são extensões CHEIAS ⇒ meia = `/2`. O writeback ([`params_from`])
/// multiplica a escala do gizmo por esta meia-extensão congelada.
#[must_use]
fn seed_start(cx: f32, cy: f32, w: f32, h: f32, rot_deg: f32) -> (TransformSnapshot, [f32; 2]) {
    (
        TransformSnapshot {
            translation: [cx, cy],
            rotation: rot_deg.to_radians(),
            scale: [1.0, 1.0],
        },
        [(w * 0.5).abs(), (h * 0.5).abs()],
    )
}

/// Os params do field a partir do `TransformSnapshot` novo do gizmo + a meia-extensão
/// intrínseca — o inverso exato de [`seed_start`], e o espelho do writeback de entidade
/// (que escreve translation/rotation/scale de uma vez): centro = translation, extensões =
/// `2·meia·escala` (`abs` — um field é simétrico, então flip é no-op e uma extensão nunca
/// é negativa), rotação = radianos→graus embrulhada em `[-180, 180]`.
#[must_use]
fn params_from(new_t: TransformSnapshot, intrinsic_half: [f32; 2]) -> (f32, f32, f32, f32, f32) {
    (
        new_t.translation[0],
        new_t.translation[1],
        (2.0 * intrinsic_half[0] * new_t.scale[0]).abs(),
        (2.0 * intrinsic_half[1] * new_t.scale[1]).abs(),
        wrap180(new_t.rotation.to_degrees()),
    )
}

/// O nó selecionado no grafo SE ele for um field espacial (+ sua [`FieldGizmoSpec`]).
/// `None` quando: nada selecionado (ou multi-seleção), o nó não é um field espacial, ou o
/// nó sumiu do grafo. Porta única para a view, o down e o gate.
#[must_use]
pub(crate) fn selected_field(motion: &MotionState) -> Option<(NodeId, FieldGizmoSpec)> {
    let nid = crate::render_loop::motion_bridge::params::selected_motion_node().map(NodeId)?;
    let type_id = motion.doc.graph.node(nid)?.type_id();
    Some((nid, spec_for(type_id)?))
}

/// A [`GizmoView`] do field espacial selecionado, ou `None`. O chamador (render_loop) já
/// gateia a tool Motion ativa; aqui recusa quando não há um field espacial selecionado. Lê
/// os params pela MESMA porta do painel (`param_value`), então a caixa concorda com os
/// sliders (seed = sample).
#[must_use]
pub(crate) fn field_view(
    motion: &MotionState,
    camera: &Camera2d,
    win_w: f32,
    win_h: f32,
    last_pointer: (f32, f32),
) -> Option<GizmoView> {
    let (nid, spec) = selected_field(motion)?;
    let p = |name: &str| crate::render_loop::motion_bridge::params::param_value(motion, nid, name);
    Some(view_from_params(
        p(spec.center_x),
        p(spec.center_y),
        p(spec.width),
        p(spec.height),
        p(spec.rotation),
        camera,
        win_w,
        win_h,
        last_pointer,
    ))
}

/// O arrasto de gizmo de field em curso: o estado genérico do gizmo + o NÓ alvo + a spec
/// + a meia-extensão intrínseca congelada no Down. `Copy` como o [`GizmoDragState`] (nada
/// de `Vec`, ≠ `FlipSelectionDrag`).
#[derive(Copy, Clone, Debug)]
pub(crate) struct FieldGizmoDrag {
    pub(crate) drag: GizmoDragState,
    pub(crate) node: NodeId,
    pub(crate) spec: FieldGizmoSpec,
    pub(crate) intrinsic_half: [f32; 2],
}

/// O núcleo do writeback de um arrasto de field, SEM a janela: avança o cursor pelo drag
/// (o contador de voltas do Rotate mora aí), recomputa o TRS pelo motor canônico e escreve
/// os CINCO params do NÓ. ⚠️ **A única escrita é `motion.doc.graph.set_param`** — esta
/// função nem recebe um `SimWorld`, então por CONSTRUÇÃO não pode tocar nenhum `Transform`
/// de entidade (a prova de que o gizmo de field não interfere na manipulação de sprites).
/// Devolve os params escritos. Porta única do [`crate::App::field_gizmo_move`] e do gate.
fn apply_field_drag(
    motion: &mut MotionState,
    fgd: &mut FieldGizmoDrag,
    cursor: (f32, f32),
    cam: &GizmoCamera,
    mods: GizmoModifiers,
    snap: GizmoSnap,
) -> (f32, f32, f32, f32, f32) {
    fgd.drag.advance_cursor(cursor, cam);
    let new_t = ph2d_editor::compute_gizmo_transform(&fgd.drag, cam, mods, snap, None);
    let (cx, cy, w, h, rot) = params_from(new_t, fgd.intrinsic_half);
    let g = &mut motion.doc.graph;
    g.set_param(fgd.node, fgd.spec.center_x, cx);
    g.set_param(fgd.node, fgd.spec.center_y, cy);
    g.set_param(fgd.node, fgd.spec.width, w);
    g.set_param(fgd.node, fgd.spec.height, h);
    g.set_param(fgd.node, fgd.spec.rotation, rot);
    motion.pump.mark_dirty();
    (cx, cy, w, h, rot)
}

impl crate::App {
    /// Pen-DOWN num handle do gizmo de field. `true` = arrasto aberto (consumido — o
    /// caminho genérico de gizmo e o resto do canvas não veem este clique). Reconhece o
    /// alvo pelo `gizmo_hit_map` ([`GizmoTarget::MotionField`]); os handles só existem
    /// quando a [`field_view`] foi publicada neste frame, então a pré-condição já está
    /// provada pela pintura. Abre o bracket de undo (um arrasto = um passo, como um drag de
    /// nó).
    pub(crate) fn field_gizmo_down(&mut self, x: f32, y: f32) -> bool {
        if !self.motion_tool_active() {
            return false;
        }
        let ctrl = self.modifiers.control_key() || self.modifiers.super_key();
        let fgd = {
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
            if hit.target != GizmoTarget::MotionField {
                return false;
            }
            let Some((nid, spec)) = selected_field(&gfx.motion) else {
                return false;
            };
            let p =
                |name: &str| crate::render_loop::motion_bridge::params::param_value(&gfx.motion, nid, name);
            let (start, intrinsic_half) = seed_start(
                p(spec.center_x),
                p(spec.center_y),
                p(spec.width),
                p(spec.height),
                p(spec.rotation),
            );
            // ⚠️ O `world_pos` do drag TEM de usar as dims da CENA (o sub-retângulo do
            // split), não a janela cheia — senão o cursor mapeia pra um mundo diferente do
            // que o gizmo é PINTADO (o mesmo drift do chrome). A `GizmoCamera` do
            // sub-retângulo espelha o `set_viewport` do render.
            let (sw, sh) = scene_window_wh(hero.view.center_split, gfx.surface.size());
            let scene_cam = GizmoCamera {
                center: gfx.camera.center,
                height_world: gfx.camera.height_world,
                window_w: sw,
                window_h: sh,
            };
            let world_pos = scene_cam.screen_to_world((x, y));
            // Rotate pivota no centro; scale, no canto/borda OPOSTOS (ou no centro com
            // Ctrl) — a mesma política do sprite/pose. `parent_world` = identidade: um
            // field não tem pai, e o param JÁ é de mundo, então o `world_snap` é o `start`.
            let pivot = ph2d_editor::anchor_pivot_world(hit.kind, intrinsic_half, start, ctrl);
            FieldGizmoDrag {
                drag: GizmoDragState {
                    kind: hit.kind,
                    // Sentinela: o writeback é por-param (`field_gizmo_move`), então este
                    // drag NUNCA lê nem escreve uma entidade. Só existe para o dispatch
                    // reconhecer que há um field-drag aberto.
                    entity_bits: 0,
                    start_screen: (x, y),
                    cursor_screen: (x, y),
                    start_transform: start,
                    pivot_world: pivot,
                    start_cursor_world: world_pos,
                    sprite_half_intrinsic: intrinsic_half,
                    anchor_is_center: ctrl,
                    target: GizmoTarget::MotionField,
                    parent_world: TransformSnapshot::IDENTITY,
                    turns: 0,
                },
                node: nid,
                spec,
                intrinsic_half,
            }
        };
        self.field_gizmo_drag = Some(fgd);
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.motion.history.begin(&gfx.motion.doc);
        }
        true
    }

    /// Pen-MOVE com um arrasto de field aberto: recomputa o TRS pelo cursor (o mesmo motor
    /// canônico, com modifiers/snap/contador de voltas) e escreve os params do NÓ. `true` =
    /// consumido. `FieldGizmoDrag` é `Copy`, então avança-se a cópia e regrava-se (o
    /// `advance_cursor` conta as voltas do Rotate).
    pub(crate) fn field_gizmo_move(&mut self, x: f32, y: f32) -> bool {
        let Some(mut fgd) = self.field_gizmo_drag else {
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
        // As MESMAS dims da CENA que o `down` usou e que o gizmo é pintado (o fix do drift).
        let (sw, sh) = scene_window_wh(
            gfx.hero_screen
                .as_ref()
                .map_or(CenterSplit::None, |h| h.view.center_split),
            size,
        );
        let cam = GizmoCamera {
            center: gfx.camera.center,
            height_world: gfx.camera.height_world,
            window_w: sw,
            window_h: sh,
        };
        let snap = gfx
            .hero_screen
            .as_ref()
            .map(|h| GizmoSnap {
                move_meters: h.project.snap_move_meters,
                rotate_deg: h.project.snap_rotate_deg,
            })
            .unwrap_or_default();
        apply_field_drag(&mut gfx.motion, &mut fgd, (x, y), &cam, mods, snap);
        self.field_gizmo_drag = Some(fgd);
        true
    }

    /// Pen-UP: fecha o arrasto de field e commita o passo de undo. `true` = havia um.
    pub(crate) fn field_gizmo_up(&mut self) -> bool {
        if self.field_gizmo_drag.take().is_none() {
            return false;
        }
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.motion.history.commit_if_changed(&gfx.motion.doc);
        }
        true
    }
}

#[cfg(test)]
#[path = "field_gizmo_tests.rs"]
mod tests;
