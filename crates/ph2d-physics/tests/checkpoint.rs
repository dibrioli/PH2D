//! **The definition of correctness for the scrub ring** (plan §W1.5 gate 1):
//! restoring a checkpoint and replaying forward must land on the *same bits*
//! as replaying the whole way from the start. If it does not, a backwards
//! scrub silently shows the artist a simulation that never happened.
//!
//! This is also the empirical answer to the module's one open question —
//! whether `PhysicsPipeline` (the field `step()` mutates that rapier does not
//! make `Clone`) holds real state. The docs reason that it is scratch; this
//! gate would go red if the reasoning were wrong, at every anchor tick.

use ph2d_physics::{PhysicsCheckpointRing, PhysicsWorld};

/// A scene with enough interaction to have real solver state: bodies in
/// contact with the floor *and* with each other. A checkpoint of a scene in
/// free fall would prove almost nothing — free fall has no contacts, and
/// contacts are exactly what the narrow phase carries across frames.
fn stacked_scene() -> PhysicsWorld {
    let mut w = PhysicsWorld::new();
    w.add_static_cuboid(0.0, 0.0, 50.0, 0.1);
    for i in 0..24 {
        let row = (i / 6) as f32;
        let col = (i % 6) as f32;
        w.add_dynamic_circle(col * 0.55 - 1.4, 0.4 + row * 0.55, 0.25, 1.0);
    }
    w
}

fn replay_from_scratch(ticks: u64) -> [u8; 32] {
    let mut w = stacked_scene();
    for _ in 0..ticks {
        w.step();
    }
    w.deterministic_hash()
}

/// Every tick's hash from 0 to `ticks`, inclusive — the trajectory, not the
/// destination. See the gate below for why the destination alone is not
/// enough of an oracle.
fn truth_trajectory(ticks: u64) -> Vec<[u8; 32]> {
    let mut w = stacked_scene();
    let mut out = vec![w.deterministic_hash()];
    for _ in 0..ticks {
        w.step();
        out.push(w.deterministic_hash());
    }
    out
}

/// **The plan's gate 1.** Restore at an anchor, replay to the target, and the
/// whole path must match — *every tick*, not merely the last one.
///
/// The trajectory oracle is not belt-and-braces: it is the difference between
/// a gate that works and one that does not. Written first as an endpoint
/// comparison, this gate stayed GREEN under a real mutation (a `restore` that
/// dropped the narrow phase), because a settling pile is a **damped** system —
/// it forgets a perturbation and re-converges to the same rest configuration,
/// so tick 137 agreed while the ticks in between did not. A scrub the artist
/// watches is the path, not the endpoint.
#[test]
fn restoring_a_checkpoint_and_replaying_is_bit_exact() {
    const TARGET: u64 = 137;

    let truth = truth_trajectory(TARGET);

    // Anchors deliberately span the interesting regimes: mid-fall (30),
    // first contact (60), settled pile (120), and the target itself.
    for anchor in [30u64, 60, 120, TARGET] {
        let mut w = stacked_scene();
        for _ in 0..anchor {
            w.step();
        }
        let cp = w.checkpoint();
        assert_eq!(cp.step_count(), anchor, "checkpoint records its own tick");

        // Wander far away, so a restore that quietly kept live state would
        // be caught rather than accidentally agreeing.
        for _ in 0..400 {
            w.step();
        }
        w.restore(&cp);
        assert_eq!(w.step_count(), anchor, "restore rewinds the step counter");
        assert_eq!(
            w.deterministic_hash(),
            truth[anchor as usize],
            "restore at tick {anchor} did not reproduce that tick's state"
        );

        for t in (anchor + 1)..=TARGET {
            w.step();
            assert_eq!(
                w.deterministic_hash(),
                truth[t as usize],
                "restoring at tick {anchor} and replaying diverged at tick {t} — the scrub \
                 would show a simulation that never happened"
            );
        }
    }
}

/// The sibling fixture for the one field whose removal a mutation could not
/// detect: the broad-phase BVH.
///
/// Dropping `broad_phase` from `restore` survived mutation testing, and a
/// survivor is either a missing gate or a null mutation. This fixture is the
/// discriminator: bodies fly apart at speed, so a restored world carrying the
/// *future's* spatial index (or, in the mutation's case, an empty one) would
/// build the wrong collision pairs and diverge immediately. It stays green,
/// which — together with the restore-into-an-empty-world gate below — is the
/// evidence that rapier re-derives the BVH from the colliders each step.
///
/// The conclusion is therefore **measured, not assumed**: the field is kept
/// in the checkpoint anyway (a snapshot's job is to be complete, and the
/// memory is already budgeted), but nobody later has to re-litigate it from
/// prose.
#[test]
fn a_scattering_scene_restores_bit_exactly_too() {
    fn scattering() -> PhysicsWorld {
        let mut w = PhysicsWorld::new();
        w.add_static_cuboid(0.0, -3.0, 200.0, 0.1);
        for i in 0..16 {
            let (h, _) = w.add_dynamic_circle((i % 4) as f32 * 0.3, (i / 4) as f32 * 0.3, 0.2, 1.0);
            let vx = ((i as f32) - 7.5) * 9.0;
            let vy = ((i % 3) as f32 - 1.0) * 7.0;
            if let Some(b) = w.bodies_mut().get_mut(h) {
                b.set_linvel(rapier2d::na::Vector2::new(vx, vy), true);
            }
        }
        w
    }

    const ANCHOR: u64 = 20;
    const TARGET: u64 = 90;

    let mut truth = scattering();
    let mut expect = Vec::new();
    for _ in 0..=TARGET {
        expect.push(truth.deterministic_hash());
        truth.step();
    }

    let mut w = scattering();
    for _ in 0..ANCHOR {
        w.step();
    }
    let cp = w.checkpoint();
    // Let the bodies scatter far, so the live spatial index describes a
    // completely different world than the anchor's.
    for _ in 0..300 {
        w.step();
    }
    w.restore(&cp);
    for t in ANCHOR..=TARGET {
        assert_eq!(
            w.deterministic_hash(),
            expect[t as usize],
            "the scattering scene diverged at tick {t} after a restore at {ANCHOR}"
        );
        w.step();
    }
}

/// A restored world must also keep *simulating* identically, not merely look
/// identical for one frame. A snapshot that dropped, say, the narrow phase
/// would agree on poses at the instant of restore and diverge immediately
/// after — the failure mode that looks like "it works" in a lazy gate.
#[test]
fn a_restored_world_keeps_simulating_identically() {
    let mut w = stacked_scene();
    for _ in 0..80 {
        w.step();
    }
    let cp = w.checkpoint();
    let at_restore = w.deterministic_hash();

    let mut fresh = PhysicsWorld::new();
    fresh.restore(&cp);
    assert_eq!(
        fresh.deterministic_hash(),
        at_restore,
        "restore into an EMPTY world must reproduce the captured state"
    );

    for _ in 0..200 {
        w.step();
        fresh.step();
        assert_eq!(
            w.deterministic_hash(),
            fresh.deterministic_hash(),
            "the restored world diverged while simulating forward"
        );
    }
}

#[test]
fn the_ring_hands_back_the_newest_anchor_at_or_before_the_target() {
    let mut w = stacked_scene();
    let mut ring = PhysicsCheckpointRing::new();
    for tick in 1..=100u64 {
        w.step();
        if ring.should_record(tick) {
            ring.record(tick, w.checkpoint());
        }
    }

    // STRIDE = 10 → anchors at 10, 20, … 100.
    assert_eq!(ring.len(), 10, "one checkpoint per stride");
    assert_eq!(ring.anchor_at_or_before(35).map(|(t, _)| t), Some(30));
    assert_eq!(ring.anchor_at_or_before(30).map(|(t, _)| t), Some(30));
    assert_eq!(
        ring.anchor_at_or_before(9).map(|(t, _)| t),
        None,
        "before the window → rest replay"
    );
    assert_eq!(ring.anchor_at_or_before(999).map(|(t, _)| t), Some(100));
}

/// The ring is only an accelerator: scrubbing *through* it must produce the
/// same bits as the slow path. This is the gate the plan's mutation targets —
/// an `anchor_at_or_before` that returns an anchor **after** the target makes
/// the replay count underflow or the state arrive from the future.
#[test]
fn scrubbing_through_the_ring_matches_the_slow_path() {
    let mut w = stacked_scene();
    let mut ring = PhysicsCheckpointRing::new();
    for tick in 1..=200u64 {
        w.step();
        if ring.should_record(tick) {
            ring.record(tick, w.checkpoint());
        }
    }

    for target in [12u64, 47, 100, 155, 199] {
        let truth = replay_from_scratch(target);

        let mut scrubbed = stacked_scene();
        let replayed = match ring.anchor_at_or_before(target) {
            Some((anchor, cp)) => {
                scrubbed.restore(cp);
                target - anchor
            }
            None => target,
        };
        assert!(
            replayed <= ph2d_physics::checkpoint_stride(),
            "scrub to {target} replayed {replayed} steps — the ring is not bounding the work"
        );
        for _ in 0..replayed {
            scrubbed.step();
        }

        assert_eq!(
            scrubbed.deterministic_hash(),
            truth,
            "scrub to tick {target} via the ring disagrees with a full replay"
        );
    }
}

/// The byte cap, not a count cap: the window shrinks for a heavy scene
/// instead of growing the bill (ADR-0117's lesson — a count is a multiplier).
#[test]
fn the_ring_evicts_to_stay_inside_its_byte_budget() {
    let mut w = stacked_scene();
    let mut ring = PhysicsCheckpointRing::new();
    // Far more ticks than the budget can hold at any scene size.
    for tick in 1..=20_000u64 {
        if ring.should_record(tick) {
            ring.record(tick, w.checkpoint());
        }
        w.step();
    }
    assert!(
        ring.approx_bytes() <= ph2d_physics::checkpoint_budget_bytes(),
        "ring grew to {} bytes, past its {} byte budget",
        ring.approx_bytes(),
        ph2d_physics::checkpoint_budget_bytes()
    );
    assert!(
        !ring.is_empty(),
        "eviction must never empty the ring entirely"
    );
}

/// **The neutral point is rapier's own, and that is measured.**
///
/// `Collider` gained `restitution`/`friction` in W2, and `spawn_body` now
/// calls `.restitution()`/`.friction()` where it previously called neither.
/// The claim in the component docs — that a body authored before those fields
/// existed simulates byte-identically — is only true if the defaults are the
/// values rapier was already using.
///
/// So this compares the two paths that must agree: `spawn_body` at its
/// defaults against `add_dynamic_circle`, which has never set either field and
/// never will. Same fixture, same hash, or the defaults are wrong and every
/// existing scene silently changed behaviour on load.
#[test]
fn the_new_collider_defaults_are_the_ones_rapier_already_used() {
    use ph2d_physics::{BodyDesc, ShapeDesc};

    let mut untouched = PhysicsWorld::new();
    untouched.add_static_cuboid(0.0, 0.0, 50.0, 0.1);
    for i in 0..12 {
        untouched.add_dynamic_circle(
            (i % 4) as f32 * 0.6 - 0.9,
            1.0 + (i / 4) as f32 * 0.7,
            0.25,
            1.0,
        );
    }

    let mut described = PhysicsWorld::new();
    described.spawn_body(BodyDesc {
        body_type: ph2d_physics::RigidBodyType::Fixed,
        x: 0.0,
        y: 0.0,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Cuboid {
            half_x: 50.0,
            half_y: 0.1,
        },
        // NOT `..Default::default()` — the point is to state the constants and
        // let the gate fail if they stop matching rapier.
        restitution: ph2d_physics_ecs_defaults::RESTITUTION,
        friction: ph2d_physics_ecs_defaults::FRICTION,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
        offset: [0.0, 0.0],
    });
    for i in 0..12 {
        described.spawn_body(BodyDesc {
            body_type: ph2d_physics::RigidBodyType::Dynamic,
            x: (i % 4) as f32 * 0.6 - 0.9,
            y: 1.0 + (i / 4) as f32 * 0.7,
            rotation: 0.0,
            density: 1.0,
            shape: ShapeDesc::Ball { radius: 0.25 },
            restitution: ph2d_physics_ecs_defaults::RESTITUTION,
            friction: ph2d_physics_ecs_defaults::FRICTION,
            layer: 0,
            is_sensor: false,
            gravity_scale: 1.0,
            linvel: [0.0, 0.0],
            angvel: 0.0,
            ccd: false,
            lock_rotation: false,
            lock_x: false,
            lock_y: false,
            mass_override: None,
            dominance: 0,
            material: Default::default(),
            damping: None,
            one_way: false,
            effector: None,
            offset: [0.0, 0.0],
        });
    }

    for _ in 0..240 {
        untouched.step();
        described.step();
        assert_eq!(
            untouched.deterministic_hash(),
            described.deterministic_hash(),
            "spawning at the declared defaults diverged from the path that never set \
             restitution/friction — the 'byte-identical for existing scenes' claim is false"
        );
    }
}

/// Mirror of `ph2d_physics_ecs::Collider`'s constants. Restated here rather
/// than imported because `ph2d-physics` must not depend on the bridge crate
/// (rapier stays below the ECS, never above it) — and if the two ever drift,
/// the gate above is what notices.
mod ph2d_physics_ecs_defaults {
    pub const RESTITUTION: f32 = 0.0;
    pub const FRICTION: f32 = 0.5;
}

/// **O que um guincho RECOLHEU viaja no checkpoint** (W-Pulley W2).
///
/// A regra do módulo — *config não é capturada* — poderia ler o recolhido como
/// config, porque ele nasce de um número autorado (a `motor_speed` da roldana).
/// Não é: ele é a **INTEGRAL** dessa taxa ao longo do run, estado simulado tanto
/// quanto uma velocidade. Sem ele no checkpoint, restaurar o tique 40 devolve o
/// mundo daquele tique com o guincho onde ele está **agora**, e o scrub passa a
/// depender de o ring ter ou não o âncora — a mesma classe do defeito que o
/// `SceneAtTick` fechou no W4b.
///
/// O oráculo é a **TRAJETÓRIA** e não o destino, pela razão que o gate no topo
/// deste arquivo pagou: um sistema amortecido esquece a perturbação. Aqui a
/// carga sobe monotonicamente, então o destino até serviria — e é exatamente por
/// isso que a trajetória é o hábito, não a exceção.
#[test]
fn a_winch_carries_what_it_reeled_through_a_checkpoint() {
    use ph2d_physics::world::pulley::PulleyDesc;
    use ph2d_physics::world::rope_route::RopeWheel;

    fn scene() -> (PhysicsWorld, PulleyDesc) {
        let mut w = PhysicsWorld::new();
        let (anchor, _) = w.add_static_cuboid(-4.0, 8.0, 0.1, 0.1);
        let (load, _) = w.add_dynamic_circle(0.0, 2.0, 0.2, 8.0);
        let wheel = RopeWheel {
            centre: [0.0, 8.0],
            radius: 0.5,
            side: 1,
        };
        let probe = PulleyDesc {
            id: 7,
            body_a: anchor,
            body_b: load,
            local_a: [0.0, 0.0],
            local_b: [0.0, 0.0],
            wheel_start: 0,
            wheel_count: 1,
            total_length: 1.0e9,
            motor_rate: 1.0,
        };
        w.set_pulleys(vec![probe], vec![wheel]);
        let span = w.pulley_span(&probe).expect("rota válida");
        let d = PulleyDesc {
            total_length: span,
            ..probe
        };
        w.set_pulleys(vec![d], vec![wheel]);
        (w, d)
    }

    const TARGET: u64 = 90;
    let truth: Vec<[u8; 32]> = {
        let (mut w, _) = scene();
        let mut out = vec![w.deterministic_hash()];
        for _ in 0..TARGET {
            w.step();
            out.push(w.deterministic_hash());
        }
        out
    };

    for anchor in [20u64, 45, TARGET] {
        let (mut w, d) = scene();
        for _ in 0..anchor {
            w.step();
        }
        let reeled_at_anchor = w.pulley_reeled(&d);
        let cp = w.checkpoint();
        // Deixa o mundo correr BEM além do âncora — é o que o produto faz: o
        // ring guarda o passado e o mundo vivo já está no futuro.
        for _ in 0..TARGET {
            w.step();
        }
        assert!(
            w.pulley_reeled(&d) > reeled_at_anchor + 0.5,
            "a fixture não continha o fenômeno: o guincho não recolheu depois do âncora"
        );
        w.restore(&cp);
        assert!(
            (w.pulley_reeled(&d) - reeled_at_anchor).abs() < 1.0e-6,
            "restaurar devolveu {} de corda recolhida, e no âncora era {reeled_at_anchor}",
            w.pulley_reeled(&d)
        );
        for tick in anchor..TARGET {
            w.step();
            assert_eq!(
                w.deterministic_hash(),
                truth[(tick + 1) as usize],
                "âncora {anchor}: o tique {} divergiu do replay do zero",
                tick + 1
            );
        }
    }
}
