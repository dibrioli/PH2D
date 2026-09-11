//! **A sonda da cena 55** — os números que a mensagem afirma, medidos sobre as
//! MESMAS peças que o artista abre.
//!
//! `#[ignore]` como as irmãs: ela imprime uma tabela, não afirma uma. Roda com
//! `cargo test -p ph2d-host-desktop --bin ph2d-host-desktop probe_smoke_55 -- --ignored --nocapture`.
//!
//! Mais dois gates NÃO-ignorados, porque a cena faz em letras grandes duas
//! afirmações que só ela pode falsificar: **Rig leva o ombro e Links não**, e **a
//! peça que a FK move é rígida**.

use super::spawn_props;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{DragReach, PhysicsBridge};

fn scene() -> (SimWorld, PhysicsBridge) {
    let mut sim = SimWorld::new();
    spawn_props(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    // UM dispatch, com o relógio PARADO: é o estado em que o artista abre a cena,
    // e é o único em que posar é legal.
    bridge.dispatch(&mut sim, false, 0);
    (sim, bridge)
}

fn by_name(sim: &SimWorld, want: &str) -> Entity {
    let mut q = sim.world().try_query::<(Entity, &Name)>().unwrap();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == want)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("a cena 55 não tem `{want}`"))
}

fn pos(sim: &SimWorld, e: Entity) -> [f32; 2] {
    let t = sim.world().get::<Transform>(e).unwrap();
    [t.translation.x, t.translation.y]
}

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

/// Os nomes que um arrasto carrega sob cada política.
fn carried(sim: &mut SimWorld, seed: &str, reach: DragReach) -> Vec<String> {
    let e = by_name(sim, seed);
    let group = ph2d_physics_ecs::jointed_by(sim.world_mut(), &[e], reach);
    let mut names: Vec<String> = group
        .iter()
        .filter_map(|&g| sim.world().get::<Name>(g).map(|n| n.as_str().to_string()))
        .collect();
    names.sort();
    names
}

#[test]
#[ignore = "sonda de medição: imprime os números da cena 55"]
fn probe_smoke_55() {
    let (mut sim, mut bridge) = scene();

    println!("BRACO -- o que cada modo de arrasto CARREGA (semente: a mao):");
    for (label, reach) in [("Rig  ", DragReach::Whole), ("Links", DragReach::Dynamic)] {
        println!("  {label}: {:?}", carried(&mut sim, "Hand", reach));
    }

    println!("PERNA -- a FK girando a COXA em torno do quadril:");
    let thigh = by_name(&sim, "Thigh");
    let shin = by_name(&sim, "Shin");
    let hip = pos(&sim, by_name(&sim, "Hip"));
    let t0 = pos(&sim, thigh);
    let s0 = pos(&sim, shin);
    println!(
        "  repouso: coxa ({:.2}, {:.2}) canela ({:.2}, {:.2}) dist {:.3}",
        t0[0],
        t0[1],
        s0[0],
        s0[1],
        dist(t0, s0)
    );
    assert!(bridge.fk_begin(&sim, thigh, t0));
    // O cursor a 90° em torno da âncora do quadril.
    let r = dist(hip, t0);
    let poses = bridge.fk_move([hip[0], hip[1] + r]);
    let t1 = poses.iter().find(|(e, _, _)| *e == thigh).unwrap().1;
    let s1 = poses.iter().find(|(e, _, _)| *e == shin).unwrap().1;
    println!(
        "  girada 90 graus: coxa ({:.2}, {:.2}) canela ({:.2}, {:.2}) dist {:.3}",
        t1[0],
        t1[1],
        s1[0],
        s1[1],
        dist(t1, s1)
    );
    println!(
        "  viagem: coxa {:.2} m | canela {:.2} m",
        dist(t0, t1),
        dist(s0, s1)
    );
    bridge.fk_end();

    println!("PERNA -- a FK girando a CANELA em torno do joelho (a coxa fica):");
    assert!(bridge.fk_begin(&sim, shin, s0));
    let knee = [3.0f32, 2.0];
    let rr = dist(knee, s0);
    let poses = bridge.fk_move([knee[0], knee[1] + rr]);
    println!(
        "  conjunto movido: {:?}",
        poses
            .iter()
            .filter_map(|(e, _, _)| sim.world().get::<Name>(*e).map(|n| n.as_str().to_string()))
            .collect::<Vec<_>>()
    );
    let s2 = poses.iter().find(|(e, _, _)| *e == shin).unwrap().1;
    println!(
        "  canela ({:.2}, {:.2}), viajou {:.2} m",
        s2[0],
        s2[1],
        dist(s0, s2)
    );
    bridge.fk_end();
}

/// **`Rig` leva o ombro estático e `Links` não** — a diferença que o passo 3 e o
/// passo 4 da mensagem afirmam, e a razão de os dois modos existirem.
#[test]
fn rig_carries_the_static_shoulder_and_links_leaves_it() {
    let (mut sim, _bridge) = scene();
    let whole = carried(&mut sim, "Hand", DragReach::Whole);
    let dynamic = carried(&mut sim, "Hand", DragReach::Dynamic);
    assert!(
        whole.iter().any(|n| n == "Shoulder"),
        "o modo Rig tem de levar a ancora junto: {whole:?}"
    );
    assert!(
        !dynamic.iter().any(|n| n == "Shoulder"),
        "o modo Links tem de DEIXAR a ancora: {dynamic:?}"
    );
    // E os três elos móveis vão nos dois.
    for arm in ["UpperArm", "Forearm", "Hand"] {
        assert!(whole.iter().any(|n| n == arm), "Rig perdeu {arm}");
        assert!(dynamic.iter().any(|n| n == arm), "Links perdeu {arm}");
    }
}

/// **A peça que a FK move é RÍGIDA, e o pai não vai junto.**
///
/// As duas metades num gate só porque uma sem a outra passa: mover tudo satisfaz
/// a rigidez, e mover só o elo pego satisfaz "o pai ficou".
#[test]
fn the_fk_swings_a_rigid_piece_and_leaves_the_parent() {
    let (sim, mut bridge) = scene();
    let thigh = by_name(&sim, "Thigh");
    let shin = by_name(&sim, "Shin");
    let hip = by_name(&sim, "Hip");
    let (t0, s0, h0) = (pos(&sim, thigh), pos(&sim, shin), pos(&sim, hip));

    assert!(bridge.fk_begin(&sim, thigh, t0));
    let r = dist(h0, t0);
    let poses = bridge.fk_move([h0[0], h0[1] + r]);
    let t1 = poses.iter().find(|(e, _, _)| *e == thigh).unwrap().1;
    let s1 = poses.iter().find(|(e, _, _)| *e == shin).unwrap().1;

    assert!(
        (dist(t1, s1) - dist(t0, s0)).abs() < 1e-3,
        "a peça esticou: {:.4} contra {:.4}",
        dist(t1, s1),
        dist(t0, s0)
    );
    assert!(
        !poses.iter().any(|(e, _, _)| *e == hip),
        "o quadril é o PAI: ele não pode estar no conjunto que a FK move"
    );
}
