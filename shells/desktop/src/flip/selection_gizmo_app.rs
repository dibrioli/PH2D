//! **O invólucro de shell** do gizmo de seleção (W2/L5 Fase B 2.ª volta, 2026-09-12).
//!
//! A lei vive em [`ph2d_app_flip::selection_gizmo`]. Aqui fica o `impl crate::App`: destrancar
//! o `Option<AppGfx>`, ler os modificadores e entregar o `HeroScreen` por parâmetro.

use ph2d_app_flip::ctx::FlipFrame;

impl crate::App {
    /// Pen-down num handle do gizmo de seleção.
    pub(crate) fn flip_selection_gizmo_down(&mut self, x: f32, y: f32) -> bool {
        let wants_edit = self.flip_wants_edit();
        let ctrl = self.modifiers.control_key() || self.modifiers.super_key();
        let playhead = self.playhead;
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let Some(hero) = gfx.hero_screen.as_ref() else {
            return false;
        };
        let win = gfx.surface.size();
        let f = FlipFrame {
            flip: &mut gfx.flip,
            playhead: &playhead,
            camera: &gfx.camera,
            win,
        };
        ph2d_app_flip::selection_gizmo::gizmo_down(
            &mut self.flip_state,
            &f,
            &gfx.sim,
            hero,
            wants_edit,
            ctrl,
            x,
            y,
        )
    }

    /// Pen-move com um arrasto de seleção aberto.
    pub(crate) fn flip_selection_gizmo_move(&mut self, x: f32, y: f32) -> bool {
        if self.flip_state.selection_drag.is_none() {
            return false;
        }
        let mods = ph2d_editor::GizmoModifiers {
            shift: self.modifiers.shift_key(),
            ctrl: self.modifiers.control_key() || self.modifiers.super_key(),
            alt: self.modifiers.alt_key(),
        };
        let Some(gfx) = self.gfx.as_mut() else {
            // Havia arrasto mas não há janela: consome na mesma (o `return true` de antes,
            // que repunha o `pd` — aqui ele nunca saiu do estado).
            return true;
        };
        let size = gfx.surface.size();
        let cam = ph2d_editor::GizmoCamera {
            center: gfx.camera.center,
            height_world: gfx.camera.height_world,
            window_w: size.width as f32,
            window_h: size.height as f32,
        };
        let snap = gfx
            .hero_screen
            .as_ref()
            .map(|h| ph2d_editor::GizmoSnap {
                move_meters: h.project.snap_move_meters,
                rotate_deg: h.project.snap_rotate_deg,
            })
            .unwrap_or_default();
        let consumed = ph2d_app_flip::selection_gizmo::gizmo_move(
            &mut self.flip_state,
            &mut gfx.flip,
            cam,
            snap,
            mods,
            x,
            y,
        );
        if consumed {
            self.title_dirty = true;
        }
        consumed
    }

    /// Pen-up: fecha o arrasto de seleção. O passo de undo sai do diff pós-frame.
    pub(crate) fn flip_selection_gizmo_up(&mut self) -> bool {
        ph2d_app_flip::selection_gizmo::gizmo_up(&mut self.flip_state)
    }
}
