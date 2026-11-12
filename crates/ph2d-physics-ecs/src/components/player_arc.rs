//! **A curva do pulo sobre o COMPONENTE** (ficha `docs/GameEngineFuncional/componentes/04_curva_do_pulo.md`):
//! a sensação e a alça escrevem nos campos do `PlatformPlayer` por aqui, e só por aqui.

use ph2d_platformer::arc::{ArcWorld, JumpArcs, jump_arcs, speed_for_apex};
use ph2d_platformer::feel::Feel;

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
        solver_iterations: s.solver_iterations,
    }
}

/// Os passos de integração do mundo por tique ([`ph2d_platformer::JumpConfig::world_steps`]):
/// sub-passos × iterações do solver.
#[must_use]
pub fn world_steps(s: &PhysicsSettings) -> u32 {
    s.substeps.max(1) * s.solver_iterations.max(1)
}

impl PlatformPlayer {
    /// Todas as curvas do canvas, pela lei com os números deste player.
    #[must_use]
    pub fn jump_arcs(&self, w: ArcWorld) -> JumpArcs {
        jump_arcs(&self.config(), w)
    }

    /// Escreve a sensação (seis gravidades); as alturas ficam, e com elas o ápice.
    pub fn set_feel(&mut self, feel: Feel) {
        let g = feel.gravity();
        self.takeoff_gravity = g.takeoff_gravity;
        self.takeoff_speed = g.takeoff_speed;
        self.peak_gravity = g.peak_gravity;
        self.peak_speed = g.peak_speed;
        self.fall_gravity = g.fall_gravity;
        self.cut_gravity = g.cut_gravity;
    }

    /// A alça: o ápice do pulo inteiro vai para `ahead` metros à frente e `up` metros acima do
    /// arranque. Cima = `jump_height` (é o ápice); lado = `speed`. Um alvo sem solução deixa o
    /// número como está.
    pub fn pull_apex(&mut self, ahead: f32, up: f32, w: ArcWorld) {
        if up > 0.0 && up.is_finite() {
            self.jump_height = up;
        }
        let c = self.config();
        if let Some(s) = speed_for_apex(&c, w, ahead) {
            self.speed = s;
        }
    }
}
