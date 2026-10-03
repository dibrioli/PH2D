//! **Fase do quadro: O MODO DO OBJECTO ACTIVO** (spec/06 F2) — só a lista das famílias que declaram
//! modos; o quadro é `ph2d_editor_core::screens::hero::mode_drive`.

use super::*;
use ph2d_editor_core::object_mode::ModeRequest;
use ph2d_editor_core::screens::hero::mode_drive::ModeFamily;

/// ⭐ **As famílias que declaram modos**, compiladas.
pub(crate) const MODE_FAMILIES: &[ModeFamily] = &[ph2d_app_painter::paint_mode::FAMILY];

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_object_mode(&mut self, request: Option<ModeRequest>) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sim,
            tools,
            toasts,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        let request = request.or_else(|| ph2d_app_painter::paint_mode::smoke_step(sim, hero));
        let world = sim.world();
        if ph2d_app_components::object_mode::drive(
            world,
            MODE_FAMILIES,
            tools,
            hero,
            toasts,
            request,
        ) {
            self.title_dirty = true;
        }
    }
}
