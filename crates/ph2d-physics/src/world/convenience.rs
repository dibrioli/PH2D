//! The two convenience constructors — a dynamic ball and a static cuboid.
//!
//! Split out of `world.rs` for the LOC cap, `impl PhysicsWorld` here like the other
//! submodules. Fixtures and early games reach for these; a full-fidelity caller uses
//! [`PhysicsWorld::spawn_body`] with a [`super::desc::BodyDesc`], or `bodies_mut` and
//! the rapier builders directly.

use rapier2d::dynamics::{RigidBodyBuilder, RigidBodyHandle};
use rapier2d::geometry::{ColliderBuilder, ColliderHandle};
use rapier2d::na::Vector2;

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
            .translation(Vector2::new(x, y))
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
            .translation(Vector2::new(x, y))
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
}
