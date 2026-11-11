//! **A curva do pulo sobre o COMPONENTE** (ficha `docs/GameEngineFuncional/componentes/04_curva_do_pulo.md`):
//! a sensação e a alça escrevem nos campos do `PlatformPlayer` por aqui, e só por aqui.

use ph2d_platformer::arc::{ArcWorld, JumpArcs, height_for_apex, jump_arcs, speed_for_apex};
use ph2d_platformer::feel::{Feel, apply_feel};

use super::PlatformPlayer;
use crate::PhysicsSettings;

/// O mundo em que a curva corre: a gravidade do projeto, o tique e os sub-passos do solver.
#[must_use]
pub fn arc_world(s: &PhysicsSettings, facing: f32) -> ArcWorld {
    ArcWorld {
        gravity: [s.gravity_x, s.gravity_y],
        dt: ph2d_physics::PhysicsWorld::DEFAULT_DT,
        facing: if facing < 0.0 { -1.0 } else { 1.0 },
        substeps: s.substeps,
    }
}

impl PlatformPlayer {
    /// Todas as curvas do canvas, pela lei com os números deste player.
    #[must_use]
    pub fn jump_arcs(&self, w: ArcWorld) -> JumpArcs {
        jump_arcs(&self.config(), w)
    }

    /// Escreve a sensação (seis gravidades) e re-resolve as alturas para o ápice ficar onde estava.
    pub fn set_feel(&mut self, feel: Feel, w: ArcWorld) {
        let mut c = self.config();
        apply_feel(&mut c, feel, w);
        let g = feel.gravity();
        self.takeoff_gravity = g.takeoff_gravity;
        self.takeoff_speed = g.takeoff_speed;
        self.peak_gravity = g.peak_gravity;
        self.peak_speed = g.peak_speed;
        self.fall_gravity = g.fall_gravity;
        self.cut_gravity = g.cut_gravity;
        self.jump_height = c.jump.jump_height;
        self.air_jump_height = c.jump.air_jump_height;
    }

    /// A alça: o ápice do pulo inteiro vai para `ahead` metros à frente e `up` metros acima do
    /// arranque. Cima = `jump_height`; lado = `speed`. Um alvo sem solução deixa o número como está.
    pub fn pull_apex(&mut self, ahead: f32, up: f32, w: ArcWorld) {
        let mut c = self.config();
        if let Some(h) = height_for_apex(&c, w, up) {
            self.jump_height = h;
            c.jump.jump_height = h;
        }
        if let Some(s) = speed_for_apex(&c, w, ahead) {
            self.speed = s;
        }
    }
}
