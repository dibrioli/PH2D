//! **A metade CORPO do reconcile** — as entidades com [`RigidBody`] viram corpos
//! do rapier, e somem com elas.
//!
//! Irmão do [`super::joints`] e do [`super::rope`], e o corte é o mesmo assunto
//! visto por peça: aqui *o que um CORPO é para o solver*. Nasceu do cap de 700 LOC
//! quando a arena de roldanas entrou no `bridge.rs`.

use super::*;

impl PhysicsBridge {
    /// Spawn bodies for new physics entities, remove bodies for despawned
    /// ones. New bodies are created in a **stable (entity-sorted) order** so
    /// the rapier handle assignment — and thus the whole simulation — does
    /// not depend on ECS archetype/insertion order (HR-5, the same sort
    /// `propagate_transforms` uses for roots). Structural only; it does not
    /// touch existing bodies' poses (that is `step`/`settle`).
    pub(super) fn reconcile_structure(&mut self, sim: &SimWorld) {
        self.seen.clear();
        self.to_spawn.clear();
        self.to_remove.clear();

        let world = sim.world();
        // Take the cached query out so the loop body can push into the
        // other scratch fields without a borrow clash.
        let mut q = self.query.take().expect("query built in dispatch");
        let at_rest = self.last_stepped == 0;
        for (e, rb, col, _local) in q.iter(world) {
            self.seen.push(e);
            // ⚠️ WORLD, not the local `Transform` the query just handed us. The
            // solver has no hierarchy: a body authored under a parent must be
            // DESCRIBED where it is actually drawn, or it spawns (and rests, and
            // collides) at its local coordinates read as world ones.
            let Some(t) = space::world_transform(world, e, &mut self.chain) else {
                continue;
            };
            // Optional per-body gravity multiplier (W8); absent = full gravity.
            // Read here because this is the half holding the `World`; folded into
            // the `BodyDesc` so it survives rewind.
            let gravity_scale = world
                .get::<crate::GravityScale>(e)
                .map_or(crate::GravityScale::NEUTRAL, |g| g.0);
            // Optional authored initial velocity (W9); absent = at rest. Folded
            // into the `BodyDesc` for the same reason — a rewind re-arms it.
            let iv = world
                .get::<crate::InitialVelocity>(e)
                .copied()
                .unwrap_or(crate::InitialVelocity::REST);
            // Optional CCD marker (W-CCD); its PRESENCE is the flag (absent =
            // discrete, rapier's default). Folded into the `BodyDesc` so a rewind
            // re-arms it, exactly like the two above.
            let ccd = world.get::<crate::Ccd>(e).is_some();
            // Optional LockRotation marker (Freeze Rotation); presence is the flag.
            let lock_rotation = world.get::<crate::LockRotation>(e).is_some();
            // Optional LockPositionX/Y markers (Freeze Position); each presence is a
            // flag that ORs its axis into the same `LockedAxes` fold, so a rewind
            // re-arms it exactly like the rotation lock above.
            let lock_x = world.get::<crate::LockPositionX>(e).is_some();
            let lock_y = world.get::<crate::LockPositionY>(e).is_some();
            // Optional MassOverride (W-Mass); its VALUE is the explicit mass in kg
            // (`None` = auto, density-derived). Folded into the `BodyDesc` so a
            // rewind re-arms it, exactly like gravity scale.
            let mass_override = world.get::<crate::MassOverride>(e).map(|m| m.0);
            // Optional Dominance (W-Dominance); its VALUE is the collision priority
            // (absent = neutral `0`). Folded in and rides the `BodyDesc` for rewind.
            let dominance = world.get::<crate::Dominance>(e).map_or(0, |d| d.0);
            // Optional MaterialCombine (W-Material); absent = both `Average`. The
            // collider's friction/restitution combine policy, folded in and riding
            // the `BodyDesc` for rewind like the rest.
            let material = world
                .get::<crate::MaterialCombine>(e)
                .copied()
                .unwrap_or_default();
            // Optional DampingOverride (W-Damping); absent = the world default drag.
            // Folded into the `BodyDesc` for rewind, AND re-stamped each dispatch by
            // `restamp_damping` so a mid-play global-drag change cannot clobber it.
            let damping = world.get::<crate::DampingOverride>(e).copied();
            // Optional OneWayPlatform marker (W-OneWay); its PRESENCE is the flag. A
            // collider property (a platform is usually Static), folded in and riding
            // the `BodyDesc` for the rewind like the rest.
            let one_way = world.get::<crate::OneWayPlatform>(e).is_some();
            // Optional AreaEffector (W-Area); its VALUE is the force in newtons the
            // zone applies to whatever overlaps it. Folded in and riding the
            // `BodyDesc` for the rewind; the wrapper refuses it on a solid collider.
            // Optional AreaEffector / AreaDrag (W-Area, W-AreaDrag); the VALUES are
            // what the area does to whatever overlaps it. Two components, bundled here
            // into the one `AreaEffect` the world takes — they are separate on this
            // side because a component blob is positional and a second FIELD would be
            // a schema bump, while a second COMPONENT is additive.
            let zone_force = world.get::<crate::AreaEffector>(e).map(|a| a.force);
            let zone_drag = world.get::<crate::AreaDrag>(e).map(|d| d.0);
            let zone_density = world.get::<crate::AreaBuoyancy>(e).map(|b| b.0);
            let zone_form = world.get::<crate::AreaFormDrag>(e).map(|f| f.0);
            // The rotational half (W-AreaTorque): a fifth optional component, folded into
            // the same `AreaEffect` bundle for the same reason as its siblings.
            let zone_torque = world.get::<crate::AreaTorque>(e).map(|t| t.0);
            // The FRAME of the force (W-AreaFrame): a sixth optional component, and a
            // MARKER — its presence pins the force to world axes, its absence (the
            // default) authors it in the zone's own frame so turning the sensor turns
            // the wind.
            //
            // ⚠️ It is deliberately NOT part of `any` — a marker alone describes the
            // frame of a force that is not there — but that exclusion is HYGIENE, not
            // correctness, and a mutation proved it: putting it in `any` leaves every
            // gate green, because `effector::zone_effect` refuses a wholly inert zone
            // anyway (zero force, zero torque, no drag). Same shape as the two refusals
            // that function documents about itself, and worth writing down for the same
            // reason: ask what a layer buys ALONE and be willing to answer "nothing"
            // ([[feedback_layered_defenses_need_per_layer_gates]]). What it does buy is
            // a `BodyDesc` that does not claim to be a zone when it is not.
            let zone_world_axes = world.get::<crate::AreaForceWorldAxes>(e).is_some();
            // How much of the push reaches the far side (W-AreaFalloff): a seventh
            // optional component, folded into the same bundle. Like the marker above it is
            // deliberately NOT part of `any` — a falloff is a MODIFIER, so a zone carrying
            // nothing else attenuates nothing and is not a zone.
            //
            // ⚠️ And, like the marker above, that exclusion is HYGIENE rather than
            // correctness — a mutation proved it: putting it in `any` leaves both gates
            // green, because `effector::zone_effect` refuses the wholly inert zone anyway.
            // I first wrote here that "the reason is not merely hygiene"; the measurement
            // says otherwise, and the honest version is the one that survives being tested
            // ([[feedback_layered_defenses_need_per_layer_gates]]). What it buys is the
            // same thing its sibling buys: a `BodyDesc` that does not claim to be a zone.
            let zone_falloff = world.get::<crate::AreaFalloff>(e).map(|f| f.0);
            let any = zone_force.is_some()
                || zone_drag.is_some()
                || zone_density.is_some()
                || zone_form.is_some()
                || zone_torque.is_some();
            let effector = any.then(|| ph2d_physics::AreaEffect {
                force: zone_force.unwrap_or([0.0, 0.0]),
                drag: zone_drag.unwrap_or(0.0),
                density: zone_density.unwrap_or(0.0),
                form_drag: zone_form.unwrap_or(0.0),
                torque: zone_torque.unwrap_or(0.0),
                world_axes: zone_world_axes,
                falloff: zone_falloff.unwrap_or(0.0),
                // A lateralidade do frame é função da POSE, não dos componentes, então ela
                // não se decide aqui: `scale::body_desc` a dobra ao lado da linha que já
                // dobra a escala sincada no offset (W-Offset), que é a mesma regra.
                mirror: ph2d_physics::AreaEffect::UNMIRRORED,
            });
            let desc = crate::scale::body_desc(
                rb,
                col,
                &t,
                gravity_scale,
                iv.linvel,
                iv.angvel,
                ccd,
                lock_rotation,
                lock_x,
                lock_y,
                mass_override,
                dominance,
                material,
                damping,
                one_way,
                effector,
            );
            match self.bodies.get(&e) {
                None => self.to_spawn.push((e, desc, rb.kind)),
                // **The rest pose is the authored pose at tick 0** — read, not
                // remembered. Without this, `rest` froze at the pose the body
                // happened to be spawned with, so moving an object and pressing
                // Reset threw the artist's placement away and jumped it back to
                // where it first appeared. It also covers a shape or density
                // edited at tick 0 (the Inspector, W2): re-describing the body
                // is one rule instead of a growing list of fields to watch.
                Some(b) if at_rest && b.rest != desc => {
                    self.to_spawn.push((e, desc, rb.kind));
                    self.to_remove.push(e);
                }
                Some(_) => {}
            }
        }
        self.query = Some(q);

        // Stale bodies: entity no longer carries the components. O(N²) is
        // fine at W1 body counts and allocates nothing; only fires on
        // despawn (steady state leaves `to_remove` empty).
        for (&e, _) in self.bodies.iter() {
            if !self.seen.contains(&e) {
                self.to_remove.push(e);
            }
        }
        // Any structural change makes every cached state a snapshot of a
        // DIFFERENT world: restoring one would hand back rapier handles that
        // no longer address the entities this bridge is holding, and the
        // wrong pose would be published without anything looking broken.
        // Asked once, for both directions.
        if !self.to_spawn.is_empty() || !self.to_remove.is_empty() {
            self.ring.clear();
        }

        self.to_remove.sort_unstable_by_key(|e| e.to_bits());
        for i in 0..self.to_remove.len() {
            let e = self.to_remove[i];
            if let Some(b) = self.bodies.remove(&e) {
                self.world.remove_body(b.handle);
            }
        }
        self.to_remove.clear();

        // Deterministic spawn order (HR-5): sort by entity bits so handle
        // assignment is a pure function of the entity set.
        self.to_spawn.sort_unstable_by_key(|(e, _, _)| e.to_bits());
        for i in 0..self.to_spawn.len() {
            let (e, desc, kind) = self.to_spawn[i];
            let handle = self.world.spawn_body(desc);
            // `desc` was built from the Transform as it is RIGHT NOW, before
            // any stepping — that is the rest state a rewind replays from.
            self.bodies.insert(
                e,
                BodyRef {
                    handle,
                    kind,
                    rest: desc,
                },
            );
        }
        self.to_spawn.clear();
    }
}
