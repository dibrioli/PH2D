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
use ph2d_editor_core::{GizmoCamera, GizmoModifiers, GizmoSnap, GizmoTarget};
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
            let Some(hit_id) = hero.chrome_hit(x, y) else {
                return false;
            };
            let Some(hit) = hero.gizmo.gizmo_hit_map.get(&hit_id).copied() else {
                return false;
            };
            if hit.target != GizmoTarget::MotionField {
                return false;
            }
            // ⚠️ As dims da CENA (o sub-retângulo do split), não a janela cheia — senão o cursor
            // mapeia para um mundo diferente do que o gizmo é PINTADO (o drift do chrome).
            let (sw, sh) = scene_window_wh(hero.view.center_split, gfx.surface.size());
            let scene_cam = GizmoCamera {
                center: gfx.camera.center,
                height_world: gfx.camera.height_world,
                window_w: sw,
                window_h: sh,
            };
            // ⭐ A semente (no espaço do objecto Motion em edição) é da crate: [`drag_do_down`].
            let Some(fgd) = drag_do_down(&gfx.motion, hit.kind, &scene_cam, (x, y), ctrl) else {
                return false;
            };
            fgd
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
