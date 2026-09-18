//! **A metade do gizmo de campo que fala com a `App`** — o prólogo que FICA (W2 Fase C).
//!
//! ⚠️ **A LEI mora em [`ph2d_app_motion::field_gizmo`]**: a geometria, o mapeamento caixa↔params,
//! o `scene_camera_window`. O que está aqui é o que toca `self.field_gizmo_drag`,
//! `self.modifiers` e `self.gfx` — três campos da `App`, e nenhum deles é exprimível do outro
//! lado da fronteira sem pedir um sexto método ao [`ph2d_app_host::AppHost`], que o bloco proíbe.
//!
//! ⭐ **É a mesma partição que a `physics` mediu e escreveu:** *o que sai são os CORPOS; o que
//! decide a ordem do quadro e segura o estado do gesto FICA.* Um arrasto é estado de shell —
//! ele começa num pen-down, sobrevive entre quadros e acaba num pen-up.

use ph2d_app_motion::field_gizmo::*;
use ph2d_editor_core::screens::layout::CenterSplit;
use ph2d_editor_core::{
    GizmoCamera, GizmoDragState, GizmoModifiers, GizmoSnap, GizmoTarget, TransformSnapshot,
};
use ph2d_host::WindowSize;

/// **A JANELA DA CENA deste quadro** — o atalho que põe um consumidor na porta sem ele ter
/// de ir buscar o `center_split` à mão.
///
/// ⚠️ **Fora do split ela É a janela**, bit a bit (`CenterSplit::None` devolve `(w, h)`), o
/// que torna a troca de `gfx.surface.size()` por esta chamada uma identidade em toda
/// ferramenta que não divide o centro — e a cura em Motion, que é a única que divide.
pub(crate) fn scene_window_of(gfx: &crate::AppGfx) -> WindowSize {
    // ⚠️ **DELEGA** desde 2026-09-17 — ver o cabeçalho do [`crate::scene_mapping`]: eram TRÊS
    // cópias da mesma conta, e a lei que elas implementam foi violada em ~90 sítios.
    gfx.scene_window()
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
            let p = |name: &str| {
                ph2d_app_motion::motion_bridge::params::param_value(&gfx.motion, nid, name)
            };
            let intrinsic_half = spec.size.half(p);
            let start = seed_start(
                p(spec.center_x),
                p(spec.center_y),
                spec.rotation.map_or(0.0, p),
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
            // `anchor = [0, 0]`: o `start` de um field É o centro dele — a caixa está
            // centrada no próprio pivô, e o termo reduz literalmente ao de antes.
            let pivot = ph2d_editor_core::anchor_pivot_world(
                hit.kind,
                [0.0, 0.0],
                intrinsic_half,
                start,
                ctrl,
            );
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
