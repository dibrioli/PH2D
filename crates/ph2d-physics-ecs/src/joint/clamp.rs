//! **O que um número VÁLIDO é** — o `clamped()` do `PhysicsJoint`.
//!
//! Módulo irmão do `joint.rs` pelo cap de 700 LOC, cortado pelo assunto que a
//! porta já desenhava: *o estado que este joint guarda* × *o que fazer com um
//! estado que chegou impossível*.
//!
//! ⚠️ **Ele é a ÚLTIMA porta antes do rapier**, e não uma cortesia da UI: um
//! componente é `serde` e chega também do arquivo de projeto, então um `NaN` num
//! `stiffness` levaria a pose para `(NaN, NaN)` — e o readback a escreveria no
//! `Transform` **e no hash de determinismo**. A ponte o chama no
//! `bridge::joint_desc`, e toda porta de autoria o chama antes de escrever.

use super::PhysicsJoint;

impl PhysicsJoint {
    pub fn clamped(mut self) -> Self {
        fn finite(v: f32, fallback: f32) -> f32 {
            if v.is_finite() { v } else { fallback }
        }
        let d = Self::default();
        self.limit_min = finite(self.limit_min, d.limit_min);
        self.limit_max = finite(self.limit_max, d.limit_max);
        if self.limit_min > self.limit_max {
            std::mem::swap(&mut self.limit_min, &mut self.limit_max);
        }
        self.motor_speed = finite(self.motor_speed, d.motor_speed);
        // A servo's target flows into rapier's `target_pos`; a NaN there poisons
        // the pose exactly as a NaN stiffness does (measured, above).
        self.motor_target = finite(self.motor_target, d.motor_target);
        self.motor_max_force = finite(self.motor_max_force, d.motor_max_force).max(0.0);
        // A threshold flows into the comparison that decides a break. A NaN there
        // makes EVERY comparison false — the joint would be silently unbreakable
        // with the checkbox ticked — and a negative one makes every comparison
        // true, so it would part on the first frame under its own weight.
        self.break_force = finite(self.break_force, d.break_force).max(0.0);
        self.break_torque = finite(self.break_torque, d.break_torque).max(0.0);
        self.rest_length = finite(self.rest_length, d.rest_length).max(0.0);
        self.stiffness = finite(self.stiffness, d.stiffness).max(0.0);
        self.damping = finite(self.damping, d.damping).max(0.0);
        // rapier's own docs require a rope's distance to be strictly positive.
        self.max_length = finite(self.max_length, d.max_length).max(Self::MIN_LENGTH);
        // A local anchor flows straight into rapier's `local_anchor1/2`; a NaN
        // there poisons the body's pose (the same failure `stiffness = NaN`
        // caused above). Guard each component back to the body's centre.
        self.local_a = [finite(self.local_a[0], 0.0), finite(self.local_a[1], 0.0)];
        self.local_b = [finite(self.local_b[0], 0.0), finite(self.local_b[1], 0.0)];
        self
    }
}
