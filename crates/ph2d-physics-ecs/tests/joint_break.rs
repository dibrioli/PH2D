//! **The joint that gives way, through the bridge** (W-J7) — the ECS half of
//! what `ph2d-physics/tests/joint_break.rs` gates on the solver side.
//!
//! Three things live here and nowhere else: the checkbox being FOLDED into the
//! `∞` the solver wants, the break coming back as an **entity** rather than a
//! handle, and a rewind un-breaking the joint and re-breaking it at the same
//! tick — which is the whole reason the break is never written into the
//! component.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, MassOverride, PhysicsBridge, PhysicsJoint,
    RigidBody,
};

/// A hook at `(0, 6)` and a `mass` kg load hanging a metre under it on `joint`.
fn rig(joint: PhysicsJoint, mass: f32) -> SimWorld {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Hook"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.05 },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 6.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Load"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.2 },
            ..Collider::default()
        },
        MassOverride(mass),
        Transform::from_translation(Vec2::new(0.0, 5.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Rope"),
        PhysicsJoint {
            body_a: stable_name_id("Hook"),
            body_b: stable_name_id("Load"),
            kind: JointKind::Rope,
            max_length: 1.0,
            ..joint
        },
        Transform::from_translation(Vec2::new(0.0, 6.0)),
    ));
    sim
}

fn load_y(sim: &mut SimWorld) -> f32 {
    let mut q = sim.world_mut().query::<(&Name, &Transform)>();
    let (_, t) = q
        .iter(sim.world())
        .find(|(n, _)| n.as_str() == "Load")
        .expect("Load alive");
    t.translation.y
}

fn run(sim: &mut SimWorld, b: &mut PhysicsBridge, from: u64, ticks: u64) {
    for t in from + 1..=from + ticks {
        b.dispatch(sim, true, t);
    }
}

/// A breakable rope rated at `n` newtons.
fn rated(n: f32) -> PhysicsJoint {
    PhysicsJoint {
        break_enabled: true,
        break_force: n,
        ..PhysicsJoint::default()
    }
}

/// **The checkbox reaches the solver, and clearing it makes the joint
/// unbreakable again.**
///
/// The `∞ = off` of P7 is resolved in the BRIDGE, not stored — so this is where
/// the two halves meet. Its own control in the same gate: the same 10 kg with the
/// box clear has to keep hanging, because "the load fell" is also what a joint
/// that never built looks like.
///
/// ⚠️ **The control carries 15 kg and not 10, and the mutation is why.** The
/// first version used 10 kg on both halves; `joint_desc` passing `j.break_force`
/// unconditionally then SURVIVED, because an unchecked joint still carries the
/// seeded `DEFAULT_BREAK_FORCE` of 100 N and 10 kg is 98.1 — just under it. The
/// control was passing for the wrong reason: *the checkbox was honoured* and
/// *the seed happened to be big enough* looked identical. At 15 kg (147.2 N) the
/// two come apart, and the mutation reddens the control half.
#[test]
fn the_breakable_checkbox_reaches_the_solver_and_clearing_it_undoes_that() {
    let mut sim = rig(rated(50.0), 10.0);
    let mut b = PhysicsBridge::default();
    run(&mut sim, &mut b, 0, 90);
    assert!(
        load_y(&mut sim) < 2.0,
        "98.1 N on a 50 N rope parts it and the load keeps falling, y = {:.2}",
        load_y(&mut sim)
    );

    // The control: box clear, and a load HEAVIER than the seeded default.
    let mut sim = rig(PhysicsJoint::default(), 15.0);
    let mut b = PhysicsBridge::default();
    run(&mut sim, &mut b, 0, 90);
    assert!(
        (load_y(&mut sim) - 5.0).abs() < 0.1,
        "with breaking off the rope holds at y = 5.00, got {:.2}",
        load_y(&mut sim)
    );
}

/// **A break comes back as the ENTITY the artist selects**, with the load it gave
/// at — the toast's whole content.
#[test]
fn a_break_is_reported_as_the_joint_entity_with_its_load() {
    let mut sim = rig(rated(50.0), 10.0);
    let mut b = PhysicsBridge::default();
    let mut seen = None;
    for t in 1..=90 {
        b.dispatch(&mut sim, true, t);
        if let Some(e) = b.joint_breaks().first() {
            seen = Some(*e);
            break;
        }
    }
    let e = seen.expect("the rope reports its own break");
    let name = sim
        .world()
        .get::<Name>(e.joint)
        .map(|n| n.0.to_string())
        .expect("the event names an entity the world knows");
    assert_eq!(name, "Rope", "and it is the JOINT, not one of the bodies");
    assert!(
        e.force > 50.0,
        "carrying the load it gave at: {:.1}",
        e.force
    );
}

/// **A rewind un-breaks it, and it breaks again at the same tick.**
///
/// The reason nothing about a break is written into the component: the world has
/// to stay a function of `(tick, authored rest)`, so scrubbing back before the
/// break has to give back a joint that is holding, and playing forward again has
/// to part it in the same place.
///
/// Mutation: writing `break_enabled = false` (or any break state) back into the
/// component on a break — the replay finds a joint that was never breakable and
/// the second half goes red.
#[test]
fn a_rewind_un_breaks_it_and_it_breaks_again_at_the_same_tick() {
    let mut sim = rig(rated(50.0), 10.0);
    let mut b = PhysicsBridge::default();
    let mut first = None;
    for t in 1..=90 {
        b.dispatch(&mut sim, true, t);
        if !b.joint_breaks().is_empty() && first.is_none() {
            first = Some(t);
        }
    }
    let broke_at = first.expect("it breaks on the way out");
    let fell_to = load_y(&mut sim);

    // Scrub back to the very start: the joint is whole again and the load is
    // where it was authored.
    b.dispatch(&mut sim, true, 0);
    assert!(
        (load_y(&mut sim) - 5.0).abs() < 0.1,
        "a rewind gives back a joint that is holding, y = {:.2}",
        load_y(&mut sim)
    );

    // And replaying reproduces the same break, at the same tick.
    let mut again = None;
    for t in 1..=90 {
        b.dispatch(&mut sim, true, t);
        if !b.joint_breaks().is_empty() && again.is_none() {
            again = Some(t);
        }
    }
    assert_eq!(again, Some(broke_at), "the replay breaks at the same tick");
    assert!(
        (load_y(&mut sim) - fell_to).abs() < 0.01,
        "and lands in the same place: {fell_to:.3} vs {:.3}",
        load_y(&mut sim)
    );
}

/// **The view says the joint is not holding** — the state half, which is what the
/// overlay draws every frame.
#[test]
fn the_view_says_a_parted_joint_is_broken() {
    // ⚠️ Presence AND absence, not "before and after": 10 kg on a 50 N rope
    // parts on the FIRST tick (the rope is taut at spawn), so a temporal claim
    // here would be about how quickly the fixture fails rather than about the
    // flag. The control is the same rig that never breaks.
    let mut sim = rig(rated(50.0), 10.0);
    let mut b = PhysicsBridge::default();
    run(&mut sim, &mut b, 0, 90);
    assert!(
        b.joint_views().any(|v| v.broken),
        "the view reports the break as STATE, not only as an event"
    );

    // 15 kg and not 10, for the reason
    // `the_breakable_checkbox_reaches_the_solver_and_clearing_it_undoes_that`
    // spells out: an unchecked joint still carries the 100 N seed, and 10 kg is
    // just under it, so the control would pass even if the checkbox were ignored.
    let mut sim = rig(PhysicsJoint::default(), 15.0);
    let mut b = PhysicsBridge::default();
    run(&mut sim, &mut b, 0, 90);
    assert!(
        b.joint_views().all(|v| !v.broken),
        "and a joint that is holding never says it is broken"
    );
}

/// **A torque threshold is only handed to a kind that can report one.**
///
/// MEASURED: rapier publishes the reaction of a limited or motorised angular
/// axis and nothing for a LOCKED one, so a Weld's threshold could never fire.
/// The bridge folds `∞` there rather than a number the solver would compare
/// against a reading that is structurally zero — the same reason `limits` asks
/// `has_limits` before passing a range.
///
/// Mutation: dropping `breaks_on_torque()` from `joint_desc` — the Weld row goes
/// red with a finite threshold that can never be crossed.
#[test]
fn a_torque_threshold_is_only_handed_to_a_kind_that_can_report_one() {
    for (kind, expect_finite) in [
        (JointKind::Pin, true),
        (JointKind::Weld, false),
        (JointKind::Slider, false),
        (JointKind::Rope, false),
        (JointKind::Spring, false),
    ] {
        let j = PhysicsJoint {
            kind,
            break_enabled: true,
            break_force: 100.0,
            break_torque: 25.0,
            ..PhysicsJoint::default()
        };
        let desc =
            ph2d_physics_ecs::joint_desc(&j, [0.0, 0.0], [0.0, 0.0], ([1.0, 0.0], [1.0, 0.0]));
        assert_eq!(
            desc.break_torque.is_finite(),
            expect_finite,
            "{kind:?}: only a Pin can be given a torque threshold"
        );
        assert!(
            desc.break_force.is_finite(),
            "{kind:?}: every kind can be given a FORCE threshold"
        );
    }
}

/// **A marca d'água é o número que se DIGITA: ela sobrevive a uma pausa e só
/// zera num rewind** (W-J7b).
///
/// O pico do wrapper é por TICK e um tranco acaba antes de dar para ler; sem a
/// marca d'água da CORRIDA, ajustar um teto é busca binária sem sinal de retorno
/// — o report que abriu esta wave. E o tempo de vida dela é a metade que decide:
/// o artista **pausa exatamente para ler o número**, então limpá-lo no `hold`
/// apagaria a resposta no instante em que ela é pedida.
///
/// Mutação: `discard_joint_peaks` chamado também no `hold` — a 2ª metade cai com
/// o pico em 0,00 depois da pausa.
#[test]
fn the_high_water_mark_survives_a_pause_and_only_a_rewind_clears_it() {
    let mut sim = rig(PhysicsJoint::default(), 5.0);
    let mut b = PhysicsBridge::default();
    run(&mut sim, &mut b, 0, 60);
    let peak = b.joint_views().next().expect("uma view").peak.force;
    assert!(
        (peak - 49.05).abs() < 0.5,
        "5 kg pendurados marcam ~49,05 N, marcou {peak:.2}"
    );

    // PAUSA (o toggle Physics DESARMADO): a corrida acabou, mas o que ela mediu
    // continua verdadeiro — e é agora que o artista olha.
    //
    // ⚠️ **`hold`, não `dispatch(.., false, ..)`.** A 1ª versão deste gate usava
    // o `false` do `dispatch`, que é *pausado* e cai no ramo `settle`; o `hold` —
    // o toggle Physics da barra — é outra coisa, e a mutação que limpava a marca
    // d'água ali **sobreviveu** porque a fixture nunca o executava.
    b.hold(&mut sim, 60);
    let after_hold = b.joint_views().next().expect("view").peak.force;
    assert!(
        (after_hold - peak).abs() < 1e-3,
        "uma pausa nao apaga a medicao da corrida: {peak:.2} -> {after_hold:.2}"
    );

    // REWIND: uma corrida nova comeca aqui, e a marca velha nao descreve mais nada.
    b.dispatch(&mut sim, true, 0);
    assert!(
        b.joint_views().next().expect("view").peak.force < 1.0,
        "um rewind zera a marca d'agua"
    );
}

/// **Num joint rompido a marca d'água CONGELA na carga que o partiu** — e é ela
/// que o readout mostra, porque a carga viva de um rompido é zero.
///
/// Sem caso especial nenhum: o wrapper pula um joint desabilitado, então ele
/// para de contribuir e o `max` fica onde estava.
#[test]
fn a_broken_joints_high_water_freezes_at_the_load_that_broke_it() {
    let mut sim = rig(rated(50.0), 10.0);
    let mut b = PhysicsBridge::default();
    run(&mut sim, &mut b, 0, 90);
    let v = b.joint_views().next().expect("view");
    assert!(v.broken, "a premissa: ela rompeu");
    assert!(
        v.load.force < 1e-3,
        "um rompido nao segura nada: carga viva {:.3}",
        v.load.force
    );
    assert!(
        v.peak.force > 50.0,
        "e a marca d'agua guarda o que cruzou: {:.1} N",
        v.peak.force
    );
    // E ela nao anda mais: mais 60 ticks de queda livre nao mexem no numero.
    let frozen = v.peak.force;
    run(&mut sim, &mut b, 90, 60);
    assert!(
        (b.joint_views().next().expect("view").peak.force - frozen).abs() < 1e-3,
        "congelada"
    );
}

/// **A SONDA da cena 49** — os números que a mensagem do smoke afirma, medidos
/// sobre a mesma armação antes de ela ser escrita.
///
/// `#[ignore]`: é medição, não gate. Rodar com
/// `cargo test -p ph2d-physics-ecs --test joint_break -- --ignored --nocapture`.
#[test]
#[ignore = "probe for the smoke scene's numbers"]
fn probe_scene_49() {
    const RATING: f32 = 60.0;
    // A: a mesma corda, três cargas.
    println!("\n-- A: uma corda de {RATING} N, tres cargas");
    for mass in [1.0_f32, 5.0, 10.0] {
        let mut sim = rig(rated(RATING), mass);
        let mut b = PhysicsBridge::default();
        run(&mut sim, &mut b, 0, 120);
        let broke = b.joint_views().any(|v| v.broken);
        println!(
            "   {mass:>4} kg = {:>6.2} N  ->  {}   (y = {:.2})",
            mass * 9.81,
            if broke { "PARTE " } else { "segura" },
            load_y(&mut sim)
        );
    }
    // B: a corrente — SIMULADA, não somada à mão: um transiente no assentamento
    // pode cruzar um teto que a aritmética estática diz que não é cruzado, e é
    // exatamente esse o número que a mensagem do smoke promete.
    println!("\n-- B: a corrente (3 elos de 1 kg + bigorna de 4 kg), teto {RATING} N");
    // ⚠️ Varridas quatro armações antes de escolher. A que a cena usa é a
    // ÚLTIMA: elos PESADOS e bigorna leve, pendurados TESOS. As outras três
    // ensinariam a coisa errada —
    //   * elos leves (1 kg) + bigorna pesada: TRÊS elos partem no tick 1, porque
    //     o transiente de assentamento vale ~1,3× a carga estática e a diferença
    //     entre um elo e o seguinte é só o peso de um elo. Para que só o de cima
    //     passe, o elo tem de pesar mais que ~0,75 da bigorna.
    //   * espaçamento FROUXO (0,85 com corda de 1,0): o tranco de 15 cm lê
    //     **1022 N** e **4090 N** — o pico de impacto, que aqui é visível porque
    //     cai entre sub-passos. Vira uma cena sobre impacto, que é justamente o
    //     que esta feature NÃO mede.
    for (link_mass, anvil, spacing, rating) in [
        (1.0_f32, 4.0_f32, 1.0_f32, 60.0_f32),
        (1.0, 4.0, 0.85, 60.0),
        (4.0, 1.0, 0.85, 120.0),
        (4.0, 1.0, 1.0, 120.0),
    ] {
        chain_probe(link_mass, anvil, spacing, rating);
    }

    // C: a porta.
    println!("\n-- C: a porta (6 kg, braco de 1 m)");
    println!(
        "   torque = {:.2} N.m contra o teto de 20   ->  PARTE",
        6.0 * 9.81
    );
    println!(
        "   forca  = {:.2} N contra o teto de 1000  ->  nao e ela",
        6.0 * 9.81
    );
    // D: o par à mão.
    println!(
        "\n-- D: o par a mao: 4 kg = {:.2} N contra um teto de 20 -> PARTE\n",
        4.0 * 9.81
    );
}

fn chain_probe(link_mass: f32, anvil: f32, spacing: f32, rating: f32) {
    println!("   elos {link_mass} kg, bigorna {anvil} kg, espacamento {spacing}, teto {rating} N");
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Chain Hook"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.08 },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.5, 8.5)),
    ));
    let ys = [
        8.5 - spacing,
        8.5 - spacing * 2.0,
        8.5 - spacing * 3.0,
        8.5 - spacing * 4.0,
    ];
    for (i, (y, mass)) in [
        (ys[0], link_mass),
        (ys[1], link_mass),
        (ys[2], link_mass),
        (ys[3], anvil),
    ]
    .iter()
    .enumerate()
    {
        let name = if i == 3 {
            "Anvil 4kg".to_string()
        } else {
            format!("Link {}", i + 1)
        };
        sim.world_mut().spawn((
            Name::new(name),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.2,
                    half_y: 0.2,
                },
                ..Collider::default()
            },
            MassOverride(*mass),
            Transform::from_translation(Vec2::new(0.5, *y)),
        ));
    }
    for (n, a, b, y) in [
        ("Chain 1", "Chain Hook", "Link 1", 8.5_f32),
        ("Chain 2", "Link 1", "Link 2", ys[0]),
        ("Chain 3", "Link 2", "Link 3", ys[1]),
        ("Chain 4", "Link 3", "Anvil 4kg", ys[2]),
    ] {
        sim.world_mut().spawn((
            Name::new(n.to_string()),
            PhysicsJoint {
                body_a: stable_name_id(a),
                body_b: stable_name_id(b),
                kind: JointKind::Rope,
                max_length: 1.0,
                break_enabled: true,
                break_force: rating,
                ..PhysicsJoint::default()
            },
            Transform::from_translation(Vec2::new(0.5, y)),
        ));
    }
    let mut b = PhysicsBridge::default();
    let mut broke: Vec<String> = Vec::new();
    for t in 1..=180 {
        b.dispatch(&mut sim, true, t);
        for e in b.joint_breaks() {
            let name = sim
                .world()
                .get::<Name>(e.joint)
                .map_or_else(|| "?".into(), |n| n.0.to_string());
            broke.push(format!("{name} @ tick {t}, {:.1} N", e.force));
        }
    }
    println!("   partiram: {broke:?}");
}
