//! **A metade da FRONTEIRA** dos gates da cena `physics_smoke_joint_anim`: os que exercitam o
//! **assador desta shell** (o `TimelineScene` do `physics/bake.rs`) sobre uma cena que vive na
//! [`ph2d_app_physics::physics_smoke_joint_anim`].
//!
//! ⚠️ Um `#[cfg(test)]` é invisível do outro lado da fronteira de crate (HOWTO §2),
//! e o corte é **por quem o teste EXERCITA, não por quem ele nomeia** — os outros
//! gates da mesma cena ficaram com o sujeito, na crate.

use crate::physics::bake::TimelineScene;
use ph2d_app_physics::physics_smoke_joint_anim::*;
use ph2d_ecs::{Entity, Name, Transform};
use ph2d_timeline::TimelineDoc;

/// O passo fixo do assador — o mesmo que o `physics/bake.rs` usa.
const DT: f64 = 1.0 / 60.0;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::PhysicsBridge;

fn named(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("corpo vivo")
}

fn staged() -> (SimWorld, TimelineDoc, [Entity; 4], PhysicsBridge) {
    let mut sim = SimWorld::new();
    let joints = build_joint_anim_scene(sim.world_mut());
    // A costura NOME → IDENTIDADE (ADR-0164 F1) — o roteador das cenas fá-la no produto.
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    let mut doc = TimelineDoc::new();
    author_joint_anim_tracks(&mut doc, joints);
    (sim, doc, joints, PhysicsBridge::new())
}

fn pose(sim: &mut SimWorld, name: &str) -> [f32; 3] {
    let e = named(sim, name);
    let t = sim.world().get::<Transform>(e).expect("transform");
    [t.translation.x, t.translation.y, t.rotation]
}

fn play(
    sim: &mut SimWorld,
    doc: &mut TimelineDoc,
    bridge: &mut PhysicsBridge,
    watch: &str,
    ticks: u64,
) -> Vec<[f32; 3]> {
    let mut out = vec![pose(sim, watch)];
    for t in 1..=ticks {
        let mut scene = TimelineScene { doc, fixed_dt: DT };
        bridge.dispatch_with_scene(sim, true, t, &mut scene);
        out.push(pose(sim, watch));
    }
    out
}

/// **A régua é o que esta wave existe para provar:** um scrub mostra a pose
/// daquele tick, não a do fim.
///
/// ⚠️ É o gate de produto do roteiro. Ele passa pela shell inteira — o mesmo
/// `TimelineScene` que o `render_loop` usa —, então cobre a costura que os
/// gates da `ph2d-physics-ecs` não alcançam: as tracks de verdade, o
/// `apply_from_doc` de verdade, os quatro canais de verdade.
#[test]
fn scrubbing_the_ruler_shows_the_pose_of_that_tick() {
    let (mut sim, mut doc, _, mut bridge) = staged();
    let played = play(&mut sim, &mut doc, &mut bridge, "ServoArm", 240);

    for &t in &[173u64, 111, 67] {
        let mut scene = TimelineScene {
            doc: &mut doc,
            fixed_dt: DT,
        };
        bridge.dispatch_with_scene(&mut sim, false, t, &mut scene);
        let got = pose(&mut sim, "ServoArm");
        let want = played[t as usize];
        let d = (got[2] - want[2]).abs();
        assert!(
            d < 1e-3,
            "scrub para o tick {t} tem de mostrar a pose do play: {want:?} contra {got:?} (delta {d})"
        );
    }
}

/// **A sonda.** `cargo test -p ph2d-host-desktop --release probe_smoke_78 --
/// --ignored --nocapture`
///
/// ⚠️ **A linha do GIRO é ruído, e de propósito:** `Transform::rotation` wrapa em
/// ±π, então uma pá que dá voltas devolve um número sem sentido para quem lê a
/// tabela (a lição que o W-AreaTorque pagou). Ela fica porque o que a cena
/// promete ali é *acelera*, que é uma TAXA — e o oráculo dela é o olho, no smoke.
#[test]
#[ignore = "sonda de medição"]
fn probe_smoke_78() {
    println!("\n=== cena 78 — a maquina animada (4 s de play) ===");
    for (watch, label) in [
        ("ServoArm", "servo (rotacao)"),
        ("CtrlArm", "CONTROLE (rotacao)"),
        ("WinchLoad", "guincho (altura)"),
        ("MuscleWeight", "musculo (altura)"),
        ("SpinBlade", "giro (rotacao)"),
    ] {
        let (mut sim, mut doc, _, mut bridge) = staged();
        let p = play(&mut sim, &mut doc, &mut bridge, watch, 240);
        let idx = [0usize, 60, 120, 180, 240];
        let col = if watch.contains("Load") || watch.contains("Weight") {
            1
        } else {
            2
        };
        let vals: Vec<String> = idx.iter().map(|&i| format!("{:.3}", p[i][col])).collect();
        println!("  {label:<22} t=0,1,2,3,4s -> [{}]", vals.join(", "));
    }
}

/// **O braço com track varre; o CONTROLE ao lado não.**
///
/// O oráculo da cena inteira, e ele não conhece função nenhuma da wave: dois
/// braços idênticos, mesmo motor, mesma força — só um tem keys. Sem o controle
/// a cena não distingue *o alvo foi animado* de *tudo se mexe sozinho*.
#[test]
fn the_keyed_arm_sweeps_and_the_control_beside_it_does_not() {
    let (mut sim, mut doc, _, mut bridge) = staged();
    let keyed = play(&mut sim, &mut doc, &mut bridge, "ServoArm", 240);
    let swing = keyed.iter().map(|p| p[2]).fold(f32::NEG_INFINITY, f32::max)
        - keyed.iter().map(|p| p[2]).fold(f32::INFINITY, f32::min);

    let (mut sim2, mut doc2, _, mut bridge2) = staged();
    let ctrl = play(&mut sim2, &mut doc2, &mut bridge2, "CtrlArm", 240);
    let ctrl_swing = ctrl.iter().map(|p| p[2]).fold(f32::NEG_INFINITY, f32::max)
        - ctrl.iter().map(|p| p[2]).fold(f32::INFINITY, f32::min);

    assert!(
        swing > 1.5,
        "o braco com track tem de varrer: excursao {swing:.3} rad"
    );
    assert!(
        ctrl_swing < 0.2,
        "o CONTROLE nao tem track e nao pode varrer: excursao {ctrl_swing:.3} rad"
    );
}

/// **A carga do guincho SOBE** — o teto da corda é keyframado, e ela obedece.
#[test]
fn the_winch_reels_the_load_in() {
    let (mut sim, mut doc, _, mut bridge) = staged();
    let p = play(&mut sim, &mut doc, &mut bridge, "WinchLoad", 240);
    let climb = p[240][1] - p[0][1];
    assert!(
        climb > 1.2,
        "a carga tem de subir com o teto da corda encurtando: {climb:.3} m"
    );
}
