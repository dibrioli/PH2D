//! **Fase do quadro: «Convert to Collision Outline»** (dono 09/11) — a composição: a linha → a
//! entidade, a lei em `ph2d_physics_outline_vec::convert` (o desenho Vector vira a «Collision Shape»
//! no lugar dele), a selecção passa à forma e a voz diz o resultado. O undo é o diff do quadro.

use super::*;
use ph2d_i18n::tr;
use ph2d_physics_outline_vec::Refusal;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_convert_outline(&mut self, row: Option<NodeId>) {
        let Some(row) = row else {
            return;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let Some(bits) = gfx.hero_live.as_ref().and_then(|l| l.bridge.entity_for(row)) else {
            return;
        };
        let e = ph2d_ecs::Entity::from_bits(bits);
        match ph2d_physics_outline_vec::convert(gfx.sim.world_mut(), &mut gfx.vec_scene, e) {
            Ok(shape) => {
                if let Some(hero) = gfx.hero_screen.as_mut() {
                    hero.gizmo.replace_selection(Some(shape.to_bits()));
                }
                gfx.toasts
                    .push(Toast::success(tr("shell.hierarchy.converted_to_outline")));
            }
            Err(Refusal::NotAVector) => {
                gfx.toasts
                    .push(Toast::warning(tr("shell.hierarchy.outline_needs_vector")));
            }
            Err(Refusal::NoArea) => {
                gfx.toasts
                    .push(Toast::warning(tr("shell.hierarchy.outline_needs_area")));
            }
        }
    }
}
