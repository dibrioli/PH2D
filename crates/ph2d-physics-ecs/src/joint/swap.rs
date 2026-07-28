//! **O gesto de trocar as duas pontas de um joint** (W-J8).
//!
//! Irmão de `joint.rs`, separado dele pelo cap de 700 LOC, e o corte é por
//! assunto: aqui *o que trocar as pontas SIGNIFICA*, lá *o estado que este joint
//! guarda*. É um assunto com uma tabela medida atrás dele, e ela mora com a
//! função que a produziu.

use super::PhysicsJoint;

impl PhysicsJoint {
    /// **This joint with its two ends exchanged** — the *Swap A↔B* gesture
    /// (W-J8), and a **behaviour-preserving** one.
    ///
    /// ## What has to move, and why it is more than the two names
    ///
    /// Every per-body fact travels with its body: the two **anchors** are stored
    /// in each body's own frame, so exchanging the pair without exchanging them
    /// would re-glue the pin somewhere it never was. (`axis_a`/`axis_b` are not
    /// stored — the bridge derives them per reconcile from the joint's own
    /// rotation against each body — so they follow for free.)
    ///
    /// ## And every SIGNED quantity is measured from A to B, so it negates
    ///
    /// MEASURED (`ph2d-physics/tests/measure_joint_pair.rs`), same rig twice:
    ///
    /// | quantity | authored | bare swap | this |
    /// |---|---|---|---|
    /// | pin: load y | −1.0000 | −1.0000 | −1.0000 |
    /// | rope: load y | −2.0000 | −2.0000 | −2.0000 |
    /// | motor: wheel ω | 4.0000 | **−4.0000** | 4.0000 |
    /// | servo: wheel rot | 44.9998° | **−44.9998°** | 44.9998° |
    /// | limit: plank rot | −11.4592° | **−34.3775°** | −11.4592° |
    /// | slider: carriage y | −0.3000 | **−1.2000** | −0.3000 |
    ///
    /// A bare swap reverses the motor and mirrors the range (`[min, max]` is the
    /// range of `θb − θa`, which becomes `[−max, −min]`). Compensating reproduces
    /// the authored column in every row — so **a swap changes which end is called
    /// A, and nothing else**.
    ///
    /// ⚠️ **"Every row" is the table above, and it was measured before the
    /// [`JointKind::Wheel`] existed.** On a DRIVEN wheel with ground contact the
    /// preservation is of SENSE, not bit-exact: the car still drives the same way
    /// (that is gated), but it travels 5.60 m where the authored one travels 7.92
    /// in 4 s, and the relative spin differs by ~1.5% in 1 s. rapier's solver is
    /// not symmetric in body order, and friction amplifies the difference. A bare
    /// swap, by contrast, is the EXACT mirror at every horizon tried — which is
    /// what makes "the compensation only flips the drive back" a measured claim
    /// rather than a reading of this code.
    ///
    /// ⚠️ **And on a Wheel the swap is the least visible it ever is**, which is
    /// how a smoke reported it as *"não funcionou"*: the anchors coincide, so the
    /// pivot dot does not move; the constraint is symmetric in A/B, so nothing
    /// physical changes; and what is left is the two name rows exchanging and the
    /// overlay's solid/dashed ownership lines swapping. ⛔ **The obvious answer —
    /// *let the swap actually move the motor to the other body* — is MEASURED and
    /// impossible:** a wheel joint does not designate a wheel (both anchors are
    /// one point, the axis is the same world direction in both frames), so which
    /// body spins is decided by mass and ground contact. Uncompensated, the
    /// button's only effect is to drive the car backwards, which is the sign of
    /// `motor_speed` under a name that does not say so.
    ///
    /// ⚠️ **That it changes nothing physical is the point, not a reason to doubt
    /// the button.** What it changes is real and visible: the two rows exchange,
    /// the display pivot follows the OTHER body (`sync_joint_pivots` derives it
    /// from A), the overlay's ownership lines exchange (A solid, B dashed), and
    /// each eyedropper re-picks the other end. Uncompensated, the button would
    /// instead be the one that silently reverses a hinge you spent an hour tuning.
    ///
    /// ⚠️ **`anchored` stays true.** The locals are still exactly right, only
    /// re-labelled — marking it un-anchored would send the anchors back through
    /// the seed policy (a Spring's B end goes to the body's CENTRE) and throw away
    /// where the artist put them. This is the one authoring gesture on this
    /// component that must NOT re-seed.
    #[must_use]
    pub fn swapped(self) -> Self {
        Self {
            body_a: self.body_b,
            body_b: self.body_a,
            local_a: self.local_b,
            local_b: self.local_a,
            limit_min: -self.limit_max,
            limit_max: -self.limit_min,
            motor_speed: -self.motor_speed,
            motor_target: -self.motor_target,
            ..self
        }
    }
}
