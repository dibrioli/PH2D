//! **A sonda da cena 54** — os números que a mensagem afirma, medidos sobre as
//! MESMAS peças que o artista abre.
//!
//! `#[ignore]` como as irmãs: ela imprime uma tabela, não afirma uma. Roda com
//! `cargo test -p ph2d-host-desktop --bin ph2d-host-desktop probe_smoke_54 -- --ignored --nocapture`.
//!
//! Mais dois gates NÃO-ignorados, porque a cena tem duas afirmações que só ela
//! pode falsificar e que a mensagem faz em letras grandes: **o ombro estático é a
//! raiz** e **o joelho limitado não dobra ao contrário**.

use super::spawn_props;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{IkOptions, PhysicsBridge};

fn scene() -> (SimWorld, PhysicsBridge) {
    let mut sim = SimWorld::new();
    spawn_props(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    // UM dispatch, com o relógio PARADO: é o estado em que o artista abre a cena,
    // e é o único em que posar é legal. Um `playing: true` aqui mediria a cena
    // caindo, que não é a cena.
    bridge.dispatch(&mut sim, false, 0);
    (sim, bridge)
}

fn by_name(sim: &SimWorld, want: &str) -> Entity {
    let mut q = sim.world().try_query::<(Entity, &Name)>().unwrap();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == want)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("a cena 54 não tem `{want}`"))
}

fn pos(sim: &SimWorld, e: Entity) -> [f32; 2] {
    let t = sim.world().get::<Transform>(e).unwrap();
    [t.translation.x, t.translation.y]
}

/// Arrasta `tip` até `target` por `n` solves e devolve as poses finais por nome.
fn pose_to(
    bridge: &mut PhysicsBridge,
    tip: Entity,
    target: [f32; 2],
    n: usize,
) -> Vec<(Entity, [f32; 2], f32)> {
    assert!(bridge.ik_begin(tip), "a cadeia da ponta tem de montar");
    let mut last = Vec::new();
    for _ in 0..n {
        last = bridge.ik_move(target, 0.0, IkOptions::default());
    }
    bridge.ik_end();
    last
}

fn of(poses: &[(Entity, [f32; 2], f32)], e: Entity) -> ([f32; 2], f32) {
    poses
        .iter()
        .find(|(x, _, _)| *x == e)
        .map(|(_, p, r)| (*p, *r))
        .expect("o corpo está na árvore")
}

#[test]
#[ignore = "sonda de medição: imprime os números da cena 54"]
fn probe_smoke_54() {
    let (sim, mut bridge) = scene();

    println!("BRACO (ombro estatico em -7,0 / 2,5; mao comeca em -3,5 / 2,5):");
    let hand = by_name(&sim, "Hand");
    let elbow = by_name(&sim, "Forearm");
    let shoulder = by_name(&sim, "Shoulder");
    for (label, target) in [
        ("mao para cima  (-5,5 / 4,5)", [-5.5f32, 4.5]),
        ("mao dobrada    (-6,5 / 3,5)", [-6.5, 3.5]),
        ("mao MUITO longe (30 / 20)  ", [30.0, 20.0]),
    ] {
        let poses = pose_to(&mut bridge, hand, target, 40);
        let (h, _) = of(&poses, hand);
        let (e, _) = of(&poses, elbow);
        let (s, _) = of(&poses, shoulder);
        let d = ((h[0] - target[0]).powi(2) + (h[1] - target[1]).powi(2)).sqrt();
        let reach = ((h[0] + 7.0).powi(2) + (h[1] - 2.5).powi(2)).sqrt();
        println!(
            "  {label}: mao ({:.2}, {:.2}) err {d:.3} | cotovelo ({:.2}, {:.2}) | ombro ({:.2}, {:.2}) | alcance {reach:.2}",
            h[0], h[1], e[0], e[1], s[0], s[1]
        );
    }

    println!("PERNA (joelho limitado a 0..2 rad; canela comeca em 0,5 / 2,5):");
    let shin = by_name(&sim, "Shin");
    for (label, target) in [
        ("canela para o lado PERMITIDO (0,0 / 1,0)", [0.0f32, 1.0]),
        ("canela para o lado PROIBIDO  (0,0 / 4,0)", [0.0, 4.0]),
    ] {
        let poses = pose_to(&mut bridge, shin, target, 40);
        let (p, r) = of(&poses, shin);
        println!("  {label}: ({:.2}, {:.2}) rot {r:+.3} rad", p[0], p[1]);
    }

    println!("PERNA: coordenada da junta num anel DENTRO do alcance (r = 1,0):");
    {
        let shin = by_name(&sim, "Shin");
        let thigh = by_name(&sim, "Thigh");
        let hip = pos(&sim, by_name(&sim, "Hip"));
        for i in 0..12 {
            let a = std::f32::consts::TAU * i as f32 / 12.0;
            let target = [hip[0] + 1.0 * a.cos(), hip[1] + 1.0 * a.sin()];
            let poses = pose_to(&mut bridge, shin, target, 40);
            let (_, sr) = of(&poses, shin);
            let (_, tr) = of(&poses, thigh);
            println!(
                "  alvo {:5.1} deg -> coord {:+.3} rad",
                a.to_degrees(),
                sr - tr
            );
        }
    }

    println!("COBRA (sem ancora; cabeca comeca em 7,5 / 2,5):");
    let head = by_name(&sim, "Snake3");
    let tail = by_name(&sim, "Snake0");
    let t0 = pos(&sim, tail);
    let poses = pose_to(&mut bridge, head, [7.5, 6.0], 40);
    let (h, _) = of(&poses, head);
    let (t, _) = of(&poses, tail);
    println!(
        "  cabeca para (7,5 / 6,0): cabeca ({:.2}, {:.2}) | cauda ({:.2}, {:.2}), andou {:.2} m",
        h[0],
        h[1],
        t[0],
        t[1],
        ((t[0] - t0[0]).powi(2) + (t[1] - t0[1]).powi(2)).sqrt()
    );
}

/// **O ombro ESTÁTICO é a raiz, e não se move.** A mensagem afirma isso em
/// letras grandes; se o plano escolhesse outra raiz, o passo 3 estaria mentindo.
#[test]
fn the_static_shoulder_is_the_root_of_the_arm() {
    let (sim, bridge) = scene();
    let plan = bridge
        .ik_plan(by_name(&sim, "Hand"))
        .expect("a mão tem cadeia");
    assert_eq!(plan.root, by_name(&sim, "Shoulder"));
    assert_eq!(plan.edges.len(), 3, "ombro -> braco -> antebraco -> mao");
}

/// **O joelho limitado nunca sai da faixa autorada** — a segunda afirmação
/// grande da cena, e a que o `inverse_kinematics` do rapier erra sozinho.
///
/// ⚠️ **A 1ª versão deste gate media a rotação de MUNDO da canela num alvo só, e
/// SOBREVIVEU à mutação que apaga a projeção de limites** — porque aquele alvo
/// dobrava o joelho para o lado PERMITIDO: a fixture não continha o fenômeno.
/// O limite é sobre a **coordenada da junta** (canela relativa à coxa), então é
/// ela que se mede, e sobre um ANEL de alvos em vez de um ponto — assim o gate
/// não depende de eu adivinhar qual direção é a proibida.
#[test]
fn the_limited_knee_never_leaves_its_authored_range() {
    let (sim, mut bridge) = scene();
    let shin = by_name(&sim, "Shin");
    let thigh = by_name(&sim, "Thigh");
    // A faixa autorada na cena, com folga para o resíduo do solver.
    const MIN: f32 = -0.15;
    const MAX: f32 = 2.15;
    let hip = pos(&sim, by_name(&sim, "Hip"));
    for i in 0..12 {
        let a = std::f32::consts::TAU * i as f32 / 12.0;
        // ⚠️ **Raio 1,0 e não 2,5, e a 2ª versão deste gate morreu por isso:** o
        // alcance da perna é 1,5, então um anel a 2,5 é todo INALCANÇÁVEL e a
        // cadeia só estica — o joelho fica em ~0 e nada o testa. Medido, a
        // mutação que apaga a projeção deixava o anel de 2,5 VERDE e o de 1,0
        // com −1,825 rad em quatro alvos. *A fixture tem de conter o fenômeno.*
        let target = [hip[0] + 1.0 * a.cos(), hip[1] + 1.0 * a.sin()];
        let poses = pose_to(&mut bridge, shin, target, 40);
        let (_, shin_rot) = of(&poses, shin);
        let (_, thigh_rot) = of(&poses, thigh);
        // A diferença de ângulos, trazida para (−π, π]: um wrap aqui viraria um
        // falso positivo, e a lição de que coordenada que wrapa é oráculo ruim
        // já custou uma wave nesta linha.
        let mut coord = shin_rot - thigh_rot;
        while coord > std::f32::consts::PI {
            coord -= std::f32::consts::TAU;
        }
        while coord <= -std::f32::consts::PI {
            coord += std::f32::consts::TAU;
        }
        assert!(
            (MIN..=MAX).contains(&coord),
            "puxando a canela para {target:?} o joelho foi a {coord:.3} rad, \
             fora da faixa autorada [0, 2]"
        );
    }
}
