//! **Aplicar o motor de um player** ao corpo (W2 do player de plataforma).
//!
//! A LEI mora na `ph2d-platformer` (uma folha pura, sem rapier); a ponte
//! (`ph2d-physics-ecs`) orquestra; e este módulo é a única coisa que **toca o
//! corpo**. A divisão é a mesma de todo o módulo, e existe por um motivo
//! verificável: a ponte declara-se *rapier-free* no próprio `Cargo.toml` (ela
//! só carrega os handles re-exportados), então quem mexe num `RigidBody` tem de
//! ser este wrapper.
//!
//! # As duas grandezas chegam por caminhos diferentes, e é o desenho
//!
//! - **`accel`** (m/s²) vira **IMPULSO** (`a · m · dt`). Multiplicar pela massa
//!   é o que faz o personagem acelerar igual seja qual for a massa dele — e ao
//!   mesmo tempo produz a **força real** que a reação da 3ª lei (W6) vai
//!   devolver ao chão. Um controlador que escrevesse velocidade não teria esse
//!   número para devolver.
//! - **`boost`** (m/s) é somado à velocidade **direto**. É o termo exato: matar
//!   a oscilação da mola, parar no lugar, herdar a velocidade de uma
//!   plataforma.
//!
//! ⚠️ **Impulso, nunca `add_force`.** A força do rapier é CONSTANTE até um
//! `reset_forces` que este pipeline nunca chama — o `world::drag` já pagou esse
//! bug (acumulou ~720× e os terminais saíram não-monotônicos). O primitivo certo
//! é `F·dt`, e é o que todo o resto deste módulo usa.
//!
//! ⚠️ **E o corpo é ACORDADO.** Um corpo adormecido não é integrado, e um player
//! parado adormece — sem isto ele acordaria só quando alguém encostasse nele, o
//! que é a assinatura exata de *"os controles pararam de funcionar"* (a mesma
//! lição, medida, do `move_grab`).

use rapier2d::dynamics::RigidBodyHandle;
use rapier2d::na::Vector2;

use super::PhysicsWorld;

impl PhysicsWorld {
    /// A gravidade do mundo, em m/s².
    ///
    /// A porta simétrica do [`PhysicsWorld::set_gravity`]: a lei do player
    /// precisa dela para **compensá-la** enquanto a mola segura o personagem, e
    /// lê-la daqui é o que impede uma segunda cópia do número de existir.
    #[must_use]
    pub fn gravity(&self) -> [f32; 2] {
        [self.gravity.x, self.gravity.y]
    }

    /// A velocidade linear de um corpo, em mundo. `None` se o handle morreu.
    #[must_use]
    pub fn body_velocity(&self, handle: RigidBodyHandle) -> Option<[f32; 2]> {
        let v = self.bodies.get(handle)?.linvel();
        Some([v.x, v.y])
    }

    /// **A velocidade de um ponto de um corpo** — a translação DELE mais o que a
    /// rotação acrescenta naquele ponto (`v + ω × r`).
    ///
    /// ⚠️ É esta, e não a do centro, que o sensor de chão precisa: um personagem
    /// sobre a ponta de uma plataforma que GIRA está sendo levado por um ponto
    /// que se move, mesmo que o centro dela esteja parado. Ler a do centro faria
    /// a mola lutar contra a rotação da plataforma.
    #[must_use]
    pub fn point_velocity(&self, handle: RigidBodyHandle, point: [f32; 2]) -> Option<[f32; 2]> {
        let b = self.bodies.get(handle)?;
        let v = b.velocity_at_point(&rapier2d::na::Point2::new(point[0], point[1]));
        Some([v.x, v.y])
    }

    /// **Aplicar o motor** ao corpo: a aceleração como impulso, o boost como
    /// escrita de velocidade.
    ///
    /// No-op silencioso para handle morto — o chamador é um laço por-entidade e
    /// não deve ter de perguntar antes.
    pub fn apply_player_motor(
        &mut self,
        handle: RigidBodyHandle,
        accel: [f32; 2],
        boost: [f32; 2],
    ) {
        let dt = self.dt();
        let Some(body) = self.bodies.get_mut(handle) else {
            return;
        };
        // Ver o aviso do módulo: um player parado adormece, e um corpo dormindo
        // não é integrado.
        body.wake_up(true);
        let mass = body.mass();
        if accel != [0.0, 0.0] && mass.is_finite() && mass > 0.0 {
            body.apply_impulse(
                Vector2::new(accel[0] * mass * dt, accel[1] * mass * dt),
                true,
            );
        }
        if boost != [0.0, 0.0] {
            let v = *body.linvel();
            body.set_linvel(Vector2::new(v.x + boost[0], v.y + boost[1]), true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A aceleração é resistida pela MASSA na hora de virar impulso, então o
    /// resultado em velocidade é o mesmo para qualquer massa — que é o que um
    /// controlador quer, e é o que dá o número certo para a reação da W6.
    #[test]
    fn the_same_accel_moves_any_mass_the_same() {
        let mut w = PhysicsWorld::new();
        w.set_gravity(0.0, 0.0);
        let (light, _) = w.add_dynamic_circle(0.0, 0.0, 0.5, 1.0);
        let (heavy, _) = w.add_dynamic_circle(5.0, 0.0, 0.5, 50.0);

        w.apply_player_motor(light, [10.0, 0.0], [0.0, 0.0]);
        w.apply_player_motor(heavy, [10.0, 0.0], [0.0, 0.0]);

        let vl = w.body_velocity(light).unwrap();
        let vh = w.body_velocity(heavy).unwrap();
        assert!(
            (vl[0] - vh[0]).abs() < 1.0e-5,
            "a mesma aceleracao tem de dar a mesma velocidade: leve {} pesado {}",
            vl[0],
            vh[0]
        );
        assert!(vl[0] > 0.0, "e ela tem de mover: {}", vl[0]);
    }

    /// O boost é escrita DIRETA — soma na velocidade, sem passar pela massa.
    #[test]
    fn the_boost_is_written_straight_onto_the_velocity() {
        let mut w = PhysicsWorld::new();
        w.set_gravity(0.0, 0.0);
        let (b, _) = w.add_dynamic_circle(0.0, 0.0, 0.5, 7.0);
        w.apply_player_motor(b, [0.0, 0.0], [3.0, -2.0]);
        let v = w.body_velocity(b).unwrap();
        assert!(
            (v[0] - 3.0).abs() < 1.0e-6 && (v[1] + 2.0).abs() < 1.0e-6,
            "{v:?}"
        );
    }

    /// ⚠️ Um corpo ADORMECIDO volta a ser integrado — sem isto um player parado
    /// para de responder aos controles.
    #[test]
    fn the_motor_wakes_a_sleeping_body() {
        let mut w = PhysicsWorld::new();
        w.set_gravity(0.0, 0.0);
        let (b, _) = w.add_dynamic_circle(0.0, 0.0, 0.5, 1.0);
        for _ in 0..240 {
            w.step();
        }
        assert!(
            w.bodies().get(b).unwrap().is_sleeping(),
            "a fixture precisa de um corpo DORMINDO para o gate significar algo"
        );
        w.apply_player_motor(b, [10.0, 0.0], [0.0, 0.0]);
        assert!(!w.bodies().get(b).unwrap().is_sleeping());
    }

    /// A velocidade de um PONTO inclui o que a rotação acrescenta ali.
    #[test]
    fn a_spinning_platform_carries_its_edge() {
        let mut w = PhysicsWorld::new();
        w.set_gravity(0.0, 0.0);
        let (b, _) = w.add_dynamic_circle(0.0, 0.0, 1.0, 1.0);
        w.bodies_mut().get_mut(b).unwrap().set_angvel(2.0, true);

        let centre = w.point_velocity(b, [0.0, 0.0]).unwrap();
        let edge = w.point_velocity(b, [1.0, 0.0]).unwrap();
        assert!(centre[1].abs() < 1.0e-6, "o centro nao anda: {centre:?}");
        assert!(
            (edge[1] - 2.0).abs() < 1.0e-4,
            "a borda a 1 m com omega=2 anda a 2 m/s: {edge:?}"
        );
    }
}
