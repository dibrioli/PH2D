//! **COMO um [`BodyDesc`] vira um corpo do rapier** — irmão de [`super`] pelo teto de 700 LOC,
//! e o corte é por RESPONSABILIDADE: ali mora o MUNDO (o passo, as arenas, as consultas); aqui
//! mora a **construção** de um corpo, ao lado do [`super::collider_build`], que faz o mesmo para
//! a forma.
//!
//! ⚠️ A lei que este ficheiro carrega e que não está em mais lado nenhum: **um eixo congelado não
//! carrega velocidade**. Ver o comentário dentro de [`PhysicsWorld::spawn_body`] — ele é o registo
//! de por que essa lei passou a valer também para a ROTAÇÃO na subida para a rapier 0.31.

use rapier2d::prelude::{LockedAxes, RigidBodyBuilder, RigidBodyHandle};

use crate::rmath::Vector;

use super::desc::BodyDesc;
use super::{PhysicsWorld, collider_build, effector};

impl PhysicsWorld {
    /// Spawn a body of any [`RigidBodyType`] with one attached collider,
    /// from a plain [`BodyDesc`]. The general constructor the ECS bridge
    /// (`ph2d-physics-ecs`) drives — it covers every body×shape combo the
    /// two convenience helpers above don't (dynamic cuboid, static ball,
    /// kinematic, …). Returns the body handle; the bridge reads its pose
    /// back via [`PhysicsWorld::body_pose`].
    ///
    /// Additive — the existing helpers, `step`, and the C9 hash are
    /// untouched, so the cross-OS determinism gate stays byte-identical.
    pub fn spawn_body(&mut self, desc: BodyDesc) -> RigidBodyHandle {
        let body = RigidBodyBuilder::new(desc.body_type)
            .translation(Vector::new(desc.x, desc.y))
            .rotation(desc.rotation)
            // Per-body gravity multiplier (W8). Setting `1.0` explicitly is
            // rapier's own default, so an unscaled body is byte-identical to
            // before this existed; the value survives rewind because it rides
            // the `BodyDesc` the world rebuilds from.
            .gravity_scale(desc.gravity_scale)
            // Dominance group (collision priority). `0` is rapier's own default, so a
            // body authored before this is byte-identical; a higher value makes this
            // body bulldoze lower-dominance ones (infinite relative mass to them). It
            // rides the `BodyDesc`, so a rewind re-arms it.
            .dominance_group(desc.dominance)
            // Initial velocity (W9), applied at build. `[0,0]`/`0` is rapier's
            // own default, so a body authored before this is byte-identical; and
            // because it rides the `BodyDesc`, a rewind to t=0 re-arms the launch.
            //
            // ⚠️ A LOCKED axis drops its velocity component. rapier's `LockedAxes`
            // zeroes the axis's inverse mass/inertia, so no FORCE or TORQUE can move
            // it (gravity on a Y-locked body does nothing) — but
            // `RigidBodyVelocity::integrate` advances the body by its raw `linvel`/
            // `angvel` WITHOUT projecting out the locked axes, so an explicitly-set
            // initial velocity would drift a "frozen" body forever (measured: an
            // X-locked body launched at 3 m/s slid the full 1.5 m in 0.5 s). So a
            // frozen axis carries no velocity — Unity/Godot's Freeze Position fully
            // pins the axis, and this makes the lock authoritative.
            //
            // ⭐⭐ **E a ROTAÇÃO entrou nesta lei em 2026-08-29, na subida para a
            // rapier 0.31 — porque ela deixou de ser a excepção.** O texto acima
            // dizia, até aqui, *«a rapier trata só a rotação como caso especial»*: até
            // à 0.28 o solver anulava sozinho a velocidade angular de um corpo com
            // rotação travada, e por isso só a translação precisava desta projecção.
            // O solver reescrito (0.29) não o faz mais. Medido: um corpo travado com
            // `angvel = 5` girava **2,5 rad em 0,5 s** — a marca de «não gira» tinha
            // deixado de significar isso.
            //
            // ⚠️ *A assimetria nunca foi nossa: era compensação de uma assimetria
            // deles.* Quando ela caiu, a nossa lei ficou meio escrita — e o defeito
            // que isso produz é o pior tipo, porque o CAMPO continua marcado no
            // inspector e o corpo simplesmente deixa de obedecer.
            .linvel(Vector::new(
                if desc.lock_x { 0.0 } else { desc.linvel[0] },
                if desc.lock_y { 0.0 } else { desc.linvel[1] },
            ))
            .angvel(if desc.lock_rotation { 0.0 } else { desc.angvel })
            // Continuous collision detection. `false` is rapier's own default, so
            // a body authored before this is byte-identical; enabling it makes the
            // pipeline sweep this body's motion so a fast one does not tunnel
            // through thin geometry. It rides the `BodyDesc`, so a rewind re-arms it.
            .ccd_enabled(desc.ccd)
            // Constraints (Freeze Rotation / Position X / Position Y). Each flag ORs
            // in its own axis of the SAME `LockedAxes` bitmask; `empty()` (no flag
            // set) is rapier's default, so an unconstrained body is byte-identical.
            // `ROTATION_LOCKED` pins the angular DOF, `TRANSLATION_LOCKED_X/_Y` pin a
            // translation DOF — a body can freeze any combination. Rides the
            // `BodyDesc`, so a rewind re-arms every locked axis.
            .locked_axes({
                let mut axes = LockedAxes::empty();
                if desc.lock_rotation {
                    axes |= LockedAxes::ROTATION_LOCKED;
                }
                if desc.lock_x {
                    axes |= LockedAxes::TRANSLATION_LOCKED_X;
                }
                if desc.lock_y {
                    axes |= LockedAxes::TRANSLATION_LOCKED_Y;
                }
                axes
            })
            .build();
        let handle = self.bodies.insert(body);
        self.stamp_defaults(handle);
        // Per-body damping override (if any), stamped AFTER the global defaults so it
        // wins. `None` (the common case) leaves the body on the global drag, so an
        // un-overridden body is byte-identical to before this existed. Rides the
        // `BodyDesc`, so a rewind re-arms it.
        if let Some(d) = desc.damping {
            self.apply_damping_override(handle, d);
        }
        let collider = collider_build::build_collider(&desc);
        let collider_handle = self
            .colliders
            .insert_with_parent(collider, handle, &mut self.bodies);
        self.stamp_layer(collider_handle, desc.layer as usize);
        // Force zone (W-Area / W-AreaDrag). `zone_effect` is the single door — it
        // refuses a solid collider (an area you cannot enter is not an area, and the
        // narrow phase reports no overlap for it) and an INERT one (no force and no
        // drag: it would touch nothing and only WAKE bodies, so registering it would
        // not be byte-neutral). Kept sorted by handle so two overlapping zones apply
        // in a fixed order.
        if let Some(effect) = effector::zone_effect(&desc) {
            self.effectors.push((handle, effect, desc.shape));
            self.effectors
                .sort_unstable_by_key(|(h, _, _)| h.into_raw_parts());
        }
        handle
    }

    /// **Reafina um corpo VIVO no lugar** — gémeo de [`PhysicsWorld::retune_joint`]: a curva muda
    /// o atrito, a densidade, a forma… a meio da simulação e o corpo obedece sem ser recriado (os
    /// contactos e a velocidade ficam; o ring de checkpoints vale). O colisor sai do MESMO
    /// [`collider_build::build_collider`] que o `spawn_body` usa, e copia-se dele.
    ///
    /// A pose (`x`/`y`/`rotation`) e a velocidade INICIAL (`linvel`/`angvel`) são da criação e não
    /// se tocam. ⚠️ O desmonte do `desc` é exaustivo: um campo novo no [`BodyDesc`] é erro de
    /// compilação aqui até alguém decidir o que ele faz a um corpo vivo.
    ///
    /// Devolve `false` quando o handle não nomeia um corpo vivo com colisor (o chamador recria).
    pub fn retune_body(&mut self, handle: RigidBodyHandle, desc: &BodyDesc) -> bool {
        let BodyDesc {
            body_type,
            x: _,
            y: _,
            rotation: _,
            density,
            shape: _,
            restitution: _,
            friction: _,
            layer,
            is_sensor: _,
            gravity_scale,
            linvel: _,
            angvel: _,
            ccd,
            lock_rotation,
            offset: _,
            lock_x,
            lock_y,
            mass_override,
            dominance,
            material: _,
            damping,
            one_way: _,
            effector: _,
        } = *desc;
        let Some(&collider_handle) = self.bodies.get(handle).and_then(|b| b.colliders().first())
        else {
            return false;
        };
        if let Some(b) = self.bodies.get_mut(handle) {
            b.set_body_type(body_type, true);
            b.set_gravity_scale(gravity_scale, true);
            b.set_dominance_group(dominance);
            b.enable_ccd(ccd);
            let mut axes = LockedAxes::empty();
            if lock_rotation {
                axes |= LockedAxes::ROTATION_LOCKED;
            }
            if lock_x {
                axes |= LockedAxes::TRANSLATION_LOCKED_X;
            }
            if lock_y {
                axes |= LockedAxes::TRANSLATION_LOCKED_Y;
            }
            b.set_locked_axes(axes, true);
            // Um eixo congelado não carrega velocidade — a lei do `spawn_body`, a meio do voo.
            let v = b.linvel();
            b.set_linvel(
                Vector::new(if lock_x { 0.0 } else { v.x }, if lock_y { 0.0 } else { v.y }),
                true,
            );
            if lock_rotation {
                b.set_angvel(0.0, true);
            }
        }
        self.stamp_defaults(handle);
        if let Some(d) = damping {
            self.apply_damping_override(handle, d);
        }
        let fresh = collider_build::build_collider(desc);
        if let Some(c) = self.colliders.get_mut(collider_handle) {
            c.set_shape(fresh.shared_shape().clone());
            match mass_override {
                Some(m) => c.set_mass(m),
                None => c.set_density(density),
            }
            c.set_restitution(fresh.restitution());
            c.set_friction(fresh.friction());
            c.set_restitution_combine_rule(fresh.restitution_combine_rule());
            c.set_friction_combine_rule(fresh.friction_combine_rule());
            c.set_sensor(fresh.is_sensor());
            c.set_active_collision_types(fresh.active_collision_types());
            // Sem pai, a posição do construtor É a relativa ao pai (`insert_with_parent` adopta-a).
            c.set_position_wrt_parent(*fresh.position());
            c.user_data = fresh.user_data;
            c.set_active_hooks(fresh.active_hooks());
        }
        self.stamp_layer(collider_handle, layer as usize);
        self.effectors.retain(|(h, _, _)| *h != handle);
        if let Some(effect) = effector::zone_effect(desc) {
            self.effectors.push((handle, effect, desc.shape));
            self.effectors
                .sort_unstable_by_key(|(h, _, _)| h.into_raw_parts());
        }
        true
    }
}
