//! Os construtores de conveniência — uma bola dinâmica, um cuboide estático e
//! a ÂNCORA DE MUNDO (W-JointWorld), que é da mesma família: um corpo que se
//! pede por um ponto em vez de por um `BodyDesc` inteiro.
//!
//! Split out of `world.rs` for the LOC cap, `impl PhysicsWorld` here like the other
//! submodules. Fixtures and early games reach for these; a full-fidelity caller uses
//! [`PhysicsWorld::spawn_body`] with a [`super::desc::BodyDesc`], or `bodies_mut` and
//! the rapier builders directly.

use crate::rmath::Vector;
use rapier2d::dynamics::{RigidBodyBuilder, RigidBodyHandle};
use rapier2d::geometry::{ColliderBuilder, ColliderHandle};

use super::PhysicsWorld;

impl PhysicsWorld {
    /// Spawn a dynamic rigid body at `(x, y)` with a circle collider
    /// of `radius` and `density`. Convenience for fixtures + early
    /// games; full-fidelity callers reach for [`PhysicsWorld::bodies_mut`]
    /// and the rapier builders directly.
    pub fn add_dynamic_circle(
        &mut self,
        x: f32,
        y: f32,
        radius: f32,
        density: f32,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let body = RigidBodyBuilder::dynamic()
            .translation(Vector::new(x, y))
            .build();
        let body_handle = self.bodies.insert(body);
        self.stamp_defaults(body_handle);
        let collider = ColliderBuilder::ball(radius).density(density).build();
        let collider_handle =
            self.colliders
                .insert_with_parent(collider, body_handle, &mut self.bodies);
        self.stamp_layer(collider_handle, 0);
        (body_handle, collider_handle)
    }

    /// Spawn a static cuboid (e.g. a floor or wall). `half_x` and
    /// `half_y` are HALF-EXTENTS (rapier convention).
    pub fn add_static_cuboid(
        &mut self,
        x: f32,
        y: f32,
        half_x: f32,
        half_y: f32,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let body = RigidBodyBuilder::fixed()
            .translation(Vector::new(x, y))
            .build();
        let body_handle = self.bodies.insert(body);
        self.stamp_defaults(body_handle);
        let collider = ColliderBuilder::cuboid(half_x, half_y).build();
        let collider_handle =
            self.colliders
                .insert_with_parent(collider, body_handle, &mut self.bodies);
        self.stamp_layer(collider_handle, 0);
        (body_handle, collider_handle)
    }

    /// **Um ponto fixo do MUNDO com que um joint possa se prender** (W-JointWorld).
    ///
    /// rapier **não tem âncora de mundo**: todo joint é corpo↔corpo, e a resposta
    /// deste repo já era um corpo `fixed` no ponto — é o que a
    /// [`Self::grab_body_with`] faz desde a W-Grab para segurar um objeto com o
    /// mouse. Esta porta dá a esse mesmo corpo um ciclo de vida **autorado** em
    /// vez de um por-gesto.
    ///
    /// ⚠️ **Entra direto na arena, sem `insert_body`** — a mesma razão da âncora
    /// da mão: o `insert_body` estampa os defaults de mundo (damping, sono), que
    /// não querem dizer nada para um corpo que **nunca é integrado**, e esta
    /// âncora não tem entidade, então nada a lê de volta nem a desenha.
    ///
    /// Some pela porta de sempre — [`Self::remove_body`] —, que já limpa os
    /// joints presos a ela. Não há door de remoção própria de propósito: duas
    /// portas para *"tire este corpo do mundo"* divergem na primeira que alguém
    /// esquecer de atualizar.
    pub fn spawn_world_anchor(&mut self, point: [f32; 2]) -> RigidBodyHandle {
        self.bodies.insert(
            RigidBodyBuilder::fixed()
                .translation(Vector::new(point[0], point[1]))
                .build(),
        )
    }
}
