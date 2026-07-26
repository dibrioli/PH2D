//! **Os knobs do MUNDO** — o que se AJUSTA nele, ao lado do que ele FAZ.
//!
//! Extraído de `world.rs` quando ele passou do cap de 700 LOC, e o corte é por
//! responsabilidade e não por tamanho: tudo aqui é *"o artista (ou o painel de
//! física) virou este número"* — gravidade, sub-passos, iterações, resposta de
//! contato, defaults de corpo, a matriz de camadas. Nada aqui roda no `step`.
//!
//! ⚠️ Nenhum tipo do rapier atravessa esta fronteira: os knobs entram como
//! números simples e o `IntegrationParameters` fica dentro da crate (a mesma
//! regra que o `lib.rs` enuncia para a superfície pública inteira).

use rapier2d::na::Vector2;

use super::PhysicsWorld;
use super::defaults::BodyDefaults;
use super::groups_for;
use super::layers::LayerMatrix;

impl PhysicsWorld {
    /// Override gravity. Useful for top-down 2D (set to zero) or
    /// custom worlds.
    pub fn set_gravity(&mut self, x: f32, y: f32) {
        self.gravity = Vector2::new(x, y);
    }

    /// Contact response tuning, as plain numbers (rapier's
    /// `IntegrationParameters` stays inside this crate).
    ///
    /// - `damping_ratio` — rapier default `5.0`. Its own docs name this as
    ///   the knob to reach for when the simulation should *look stiffer*,
    ///   in preference to raising the contact natural frequency (which
    ///   overshoots and jitters).
    /// - `max_corrective_velocity` — rapier default `10.0` m/s. The ceiling
    ///   on how fast the solver is allowed to push accumulated penetration
    ///   back out.
    ///
    /// ⚠️ These feed the solver, so changing them **changes every
    /// simulation** — including the cross-OS C9 hash. That is a deliberate
    /// product decision, not a free knob.
    pub fn set_contact_response(&mut self, damping_ratio: f32, max_corrective_velocity: f32) {
        self.integration_parameters.contact_damping_ratio = damping_ratio;
        self.integration_parameters
            .normalized_max_corrective_velocity = max_corrective_velocity;
    }

    /// Contact spring frequency, Hz (rapier default `30.0`). rapier's docs:
    /// *"increasing this value will make it so that penetrations get fixed
    /// more quickly at the expense of potential jitter due to overshooting"*.
    pub fn set_contact_frequency(&mut self, hz: f32) {
        self.integration_parameters.contact_natural_frequency = hz;
    }

    /// How many integration sub-steps one [`PhysicsWorld::step`] runs.
    ///
    /// The **only** lever on how deep a fast body is already overlapping the
    /// frame it first touches: that depth is `velocity × dt` and no solver
    /// can undo it after the fact. Halving the sub-step halves the overlap,
    /// at a proportional cost.
    pub fn set_substeps(&mut self, n: u32) {
        self.substeps = n.max(1);
        self.integration_parameters.dt = self.base_dt / self.substeps as f32;
    }

    /// Number of solver iterations per step (rapier default `4`). More
    /// iterations resolve a stack's contacts more completely, at linear cost.
    pub fn set_solver_iterations(&mut self, n: usize) {
        if let Some(n) = std::num::NonZeroUsize::new(n) {
            self.integration_parameters.num_solver_iterations = n;
        }
    }

    /// The world-level values new bodies are born with (damping, sleep).
    pub fn body_defaults(&self) -> BodyDefaults {
        self.body_defaults
    }

    /// Replace the world-level body defaults.
    ///
    /// **Applies to the bodies that already exist, not only to future ones.**
    /// The artist is describing the world in front of them; a drag value that
    /// only reached the next body spawned would be a number that appears to do
    /// nothing. See [`BodyDefaults`] for why these are world settings at all.
    pub fn set_body_defaults(&mut self, d: BodyDefaults) {
        self.body_defaults = d;
        d.apply_to_all(&mut self.bodies);
    }

    /// Which layers collide with which.
    pub fn layer_matrix(&self) -> LayerMatrix {
        self.layer_matrix
    }

    /// Replace the collision-layer matrix.
    ///
    /// **Applies to the colliders that already exist**, for the same reason
    /// [`Self::set_body_defaults`] does: the artist is describing the scene in
    /// front of them, and a rule that only reached the next body spawned would
    /// look like a dead checkbox.
    ///
    /// A collider already carries its own layer — it is `memberships`, a single
    /// bit — so the layer never has to be stored twice or looked up elsewhere.
    pub fn set_layer_matrix(&mut self, matrix: LayerMatrix) {
        self.layer_matrix = matrix;
        for (_, collider) in self.colliders.iter_mut() {
            let membership_bits = collider.collision_groups().memberships.bits();
            let layer = membership_bits.trailing_zeros() as usize;
            collider.set_collision_groups(groups_for(layer, matrix));
        }
    }
}
