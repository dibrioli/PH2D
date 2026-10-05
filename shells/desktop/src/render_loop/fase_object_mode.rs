//! **Fase do quadro: O MODO DO OBJECTO ACTIVO** (spec/06 F2/F3) — só a lista das famílias que
//! declaram modos, cada uma com o que empresta; o quadro é `ph2d_editor_core::screens::hero::mode_drive`.

use super::*;
use ph2d_editor_core::object_mode::ModeRequest;
use ph2d_editor_core::screens::hero::mode_drive::ModeFamily;

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
            flip,
            vec_scene,
            ..
        } = FrameGfx::of(gfx);
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        let request = request
            .or_else(|| ph2d_app_painter::paint_mode::smoke_step(sim, hero))
            .or_else(|| ph2d_app_vec::vector_mode::smoke_step(&mut self.vec, vec_scene, sim, hero));
        ph2d_app_flip::flip_mode::smoke_step(&self.flip_state, hero);
        // ⭐ **As famílias que declaram modos**, compiladas.
        let mut paint = ph2d_app_painter::paint_mode::Family;
        let mut flip = ph2d_app_flip::flip_mode::Family::new(&mut self.flip_state, flip);
        let mut vector = ph2d_app_vec::vector_mode::Family::new(&mut self.vec, sim);
        let families: &mut [&mut dyn ModeFamily] = &mut [&mut paint, &mut flip, &mut vector];
        let world = sim.world();
        if ph2d_app_components::object_mode::drive(world, families, tools, hero, toasts, request) {
            self.title_dirty = true;
        }
    }
}
