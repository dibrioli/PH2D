//! ⭐ **A CURVA DO PULO no canvas** (ficha `docs/GameEngineFuncional/componentes/04_curva_do_pulo.md`,
//! dono 10/11: «B, mas com linhas coloridas»): o desenho mora em `ph2d_panel_physics::curva_do_pulo`
//! (tecto de LOC); aqui fica o arrasto da alça [`PointHandleKind::JumpApex`]: cima = Jump Height,
//! lado = Speed.

pub use ph2d_panel_physics::curva_do_pulo::*;

use ph2d_ecs::{Entity, SimWorld};
use ph2d_editor_core::gizmo::PointHandleKind;
use ph2d_physics_ecs::jump_arc::ArcWorld;
use ph2d_physics_ecs::{PhysicsSettings, PlatformPlayer};

use crate::{CanvasCtx, physics_state::PhysicsState};

/// O arrasto da alça em curso.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct JumpArcDrag {
    pub player_bits: u64,
    /// Ápice − cursor no Down: a alça não salta para debaixo do cursor.
    grab: [f32; 2],
    chao: [f32; 2],
    mundo: ArcWorld,
}

/// Um Down numa alça [`PointHandleKind::JumpApex`] abre o arrasto. `false` para qualquer outra.
pub fn open(
    st: &mut PhysicsState,
    sim: &SimWorld,
    settings: &PhysicsSettings,
    hit: (Entity, PointHandleKind),
    cursor: [f32; 2],
) -> bool {
    let (player, kind) = hit;
    if kind != PointHandleKind::JumpApex || ph2d_ecs::is_locked_for_edit(sim.world(), player) {
        return false;
    }
    let Some(d) = desenho(sim, settings, Some(player.to_bits()), true) else {
        return false;
    };
    let a = d.apice();
    st.jump_arc_drag = Some(JumpArcDrag {
        player_bits: player.to_bits(),
        grab: [a[0] - cursor[0], a[1] - cursor[1]],
        chao: d.chao,
        mundo: d.mundo,
    });
    true
}

/// Segue o cursor: escreve Jump Height e Speed no componente (o undo grava o gesto inteiro no soltar).
pub fn advance(st: &PhysicsState, ctx: &mut CanvasCtx<'_>, pointer: (f32, f32)) {
    let Some(d) = st.jump_arc_drag else {
        return;
    };
    let c = ctx.camera.screen_to_world(pointer, ctx.window);
    let alvo = [c[0] + d.grab[0] - d.chao[0], c[1] + d.grab[1] - d.chao[1]];
    let up = ph2d_physics_ecs::jump_arc::up_of_gravity(d.mundo.gravity);
    let lado = [up[1] * d.mundo.facing, -up[0] * d.mundo.facing];
    let acima = alvo[0] * up[0] + alvo[1] * up[1];
    let frente = alvo[0] * lado[0] + alvo[1] * lado[1];
    let e = Entity::from_bits(d.player_bits);
    if let Some(mut p) = ctx.sim.world_mut().get_mut::<PlatformPlayer>(e) {
        p.pull_apex(frente, acima, d.mundo);
        p.sanitize();
    }
}

#[cfg(test)]
#[path = "jump_arc_tests.rs"]
mod tests;
