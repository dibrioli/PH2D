//! ⭐ **A mola que segue um alvo** — o espaço que abre, o cartão que assenta, a lista que recolhe.
//!
//! A LEI (rigidez, amortecimento, ou nenhuma) é a do hero: `UiMotion::law(Role::Travel)` — o
//! carácter que o artista escolheu e o «movimento reduzido», que a anula (tudo chega no quadro).
//! Mudar o alvo a meio herda a velocidade: um cartão que se mexe e volta a mexer não dá saltos.

use ph2d_spring::Spring;

/// O passo de integração (o da `ph2d-spring`).
const STEP: f64 = 1.0 / 240.0;
/// Abaixo disto (px e px/s) a mola assentou.
const SETTLE: f64 = 0.05;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Follow {
    pub pos: f64,
    pub vel: f64,
    pub target: f64,
    /// Já recebeu um alvo (o primeiro chega sem animar: o que aparece não voa de zero).
    pub seeded: bool,
}

impl Follow {
    /// Põe o alvo; o primeiro é a posição.
    pub fn aim(&mut self, target: f64) {
        if !self.seeded {
            *self = Self {
                pos: target,
                vel: 0.0,
                target,
                seeded: true,
            };
        }
        self.target = target;
    }

    /// Salta para `pos` (o cartão largado parte de onde a mão o deixou).
    pub fn jump(&mut self, pos: f64) {
        self.pos = pos;
        self.vel = 0.0;
        self.seeded = true;
    }

    /// Avança `dt` segundos. `None` = sem mola (movimento reduzido): chega já. `true` = em voo.
    pub fn advance(&mut self, dt: f64, law: Option<Spring>) -> bool {
        let Some(s) = law.map(Spring::clamped) else {
            self.pos = self.target;
            self.vel = 0.0;
            return false;
        };
        let (w, z) = (s.stiffness, s.damping);
        let mut left = dt.clamp(0.0, 0.25);
        while left > 0.0 {
            let h = left.min(STEP);
            let a = w * w * (self.target - self.pos) - 2.0 * z * w * self.vel;
            self.vel += a * h;
            self.pos += self.vel * h;
            left -= h;
        }
        let moving = (self.pos - self.target).abs() > SETTLE || self.vel.abs() > SETTLE;
        if !moving {
            self.pos = self.target;
            self.vel = 0.0;
        }
        moving
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_a_law_it_arrives_in_the_same_frame() {
        let mut f = Follow::default();
        f.aim(0.0);
        f.aim(120.0);
        assert!(!f.advance(1.0 / 60.0, None));
        assert!((f.pos - 120.0).abs() < f64::EPSILON);
    }

    #[test]
    fn with_a_law_it_travels_and_settles_on_the_target() {
        let mut f = Follow::default();
        f.aim(0.0);
        f.aim(100.0);
        let law = Some(Spring::default());
        assert!(f.advance(1.0 / 60.0, law), "o 1.º quadro ainda está em voo");
        assert!(f.pos > 0.0 && f.pos < 100.0, "a meio do caminho: {}", f.pos);
        for _ in 0..240 {
            f.advance(1.0 / 60.0, law);
        }
        assert!(
            (f.pos - 100.0).abs() < f64::EPSILON,
            "assenta no alvo: {}",
            f.pos
        );
    }

    #[test]
    fn the_first_target_is_the_position() {
        let mut f = Follow::default();
        f.aim(42.0);
        assert!(!f.advance(1.0 / 60.0, Some(Spring::default())));
        assert!((f.pos - 42.0).abs() < f64::EPSILON);
    }
}
