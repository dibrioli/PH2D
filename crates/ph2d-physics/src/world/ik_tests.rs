//! Gates e VARREDURAS da IK (W-IK).
//!
//! Módulo FILHO de `ik.rs`, então `use super::*` alcança a superfície privada
//! do wrapper — a mesma razão de `world/tests.rs` existir.
//!
//! As varreduras são `#[ignore]`: elas **imprimem tabelas** e é delas que saem
//! os números escritos em [`IkOptions`]. Rodar:
//!
//! ```text
//! cargo test -p ph2d-physics --release sweep_the_ik -- --ignored --nocapture
//! ```

use super::*;
use rapier2d::dynamics::RigidBodyBuilder;
use rapier2d::geometry::ColliderBuilder;

/// Um elo: caixa de `2·half_x` por `2·half_y` centrada em `(x, y)`.
fn link(w: &mut PhysicsWorld, x: f32, y: f32, half_x: f32, half_y: f32) -> RigidBodyHandle {
    let body = RigidBodyBuilder::dynamic()
        .translation(Vector2::new(x, y))
        .build();
    let h = w.bodies.insert(body);
    w.stamp_defaults(h);
    let col = ColliderBuilder::cuboid(half_x, half_y).density(1.0).build();
    let c = w.colliders.insert_with_parent(col, h, &mut w.bodies);
    w.stamp_layer(c, 0);
    h
}

/// Um Pin puro entre dois pontos locais.
fn pin(anchor_a: [f32; 2], anchor_b: [f32; 2]) -> JointDesc {
    JointDesc {
        kind: JointKind::Pin,
        anchor_a,
        anchor_b,
        limits: None,
        ..JointDesc::default()
    }
}

/// **A cadeia de referência**: gancho estático em `(0,0)` + três elos de 1 m
/// deitados no eixo +X, pinados ponta a ponta. Alcance total 3 m.
///
/// É a mesma forma da cena de smoke, e é isso que faz as varreduras
/// descreverem o produto em vez de um brinquedo.
fn three_link_chain() -> (PhysicsWorld, RigidBodyHandle, Vec<IkLink>, RigidBodyHandle) {
    chain_of(1.0)
}

/// A mesma cadeia de três elos, com elos de `sz` metros. A ESCALA é parâmetro
/// porque a instabilidade do DLS é relativa ao comprimento do elo, e uma
/// fixture de uma escala só não pode ver isso.
fn chain_of(sz: f32) -> (PhysicsWorld, RigidBodyHandle, Vec<IkLink>, RigidBodyHandle) {
    let mut w = PhysicsWorld::new();
    let h = sz * 0.5;
    let (hook, _) = w.add_static_cuboid(0.0, 0.0, sz * 0.1, sz * 0.1);
    let l1 = link(&mut w, h, 0.0, h, sz * 0.1);
    let l2 = link(&mut w, sz + h, 0.0, h, sz * 0.1);
    let l3 = link(&mut w, sz * 2.0 + h, 0.0, h, sz * 0.1);
    let links = vec![
        IkLink {
            parent: hook,
            child: l1,
            joint: pin([0.0, 0.0], [-h, 0.0]),
        },
        IkLink {
            parent: l1,
            child: l2,
            joint: pin([h, 0.0], [-h, 0.0]),
        },
        IkLink {
            parent: l2,
            child: l3,
            joint: pin([h, 0.0], [-h, 0.0]),
        },
    ];
    (w, hook, links, l3)
}

/// Onde a ponta parou, e quão longe ficou do alvo.
fn solve_to(
    w: &PhysicsWorld,
    chain: &mut IkChain,
    tip: RigidBodyHandle,
    target: [f32; 2],
    opts: IkOptions,
) -> ([f32; 2], f32) {
    let poses = w.ik_solve(chain, target, 0.0, opts);
    let p = poses
        .iter()
        .find(|p| p.body == tip)
        .expect("tip is in the tree");
    let d =
        ((p.translation[0] - target[0]).powi(2) + (p.translation[1] - target[1]).powi(2)).sqrt();
    (p.translation, d)
}

// ---------------------------------------------------------------- varreduras

/// **O teto de passo, em COMPRIMENTOS DE ELO** — o `clampMag` do DLS.
///
/// ⚠️ Varre o FATOR sobre **três escalas de cadeia** (elos de 0,2 m, 1 m e 5 m),
/// e é essa segunda dimensão que torna a tabela útil: um teto em metros
/// absolutos daria a mesma linha para as três, e a coluna de 0,2 m mostra que
/// isso seria falso.
///
/// Colunas por escala: solves até a ponta chegar a 2% do alcance, e o raio
/// final com alvo MUITO fora de alcance — a cadeia tem de ESTICAR
/// (raio ≈ alcance − meio elo), nunca enrolar.
#[test]
#[ignore = "sweep: prints a table"]
fn sweep_the_ik_step_factor() {
    println!(
        "fator | elo 0.2m: solves / raio(ideal 0.5) | elo 1m: solves / raio(2.5) | elo 5m: solves / raio(12.5)"
    );
    for &f in &[0.25f32, 0.5, 1.0, 2.0, 4.0, 8.0, 1.0e6] {
        let mut cells = Vec::new();
        for &sz in &[0.2f32, 1.0, 5.0] {
            let (w, _, links, tip) = chain_of(sz);
            let mut chain = w.ik_chain(links[0].parent, &links, tip).expect("chain");
            let cap = sz * f;
            // Alvo alcançável: metade do alcance para cima e para o lado.
            let target = [sz * 1.2, sz * 1.8];
            let tol = sz * 0.02;
            let mut n = 999;
            for i in 1..=400 {
                if err_after(&w, &mut chain, tip, target, cap, 1) < tol {
                    n = i;
                    break;
                }
            }
            let (w2, _, links2, tip2) = chain_of(sz);
            let mut c2 = w2.ik_chain(links2[0].parent, &links2, tip2).expect("chain");
            let mut r = 0.0;
            for _ in 0..400 {
                let p = tip_of(
                    &w2.ik_solve_stepped(
                        &mut c2,
                        [sz * 30.0, sz * 0.5],
                        0.0,
                        IkOptions::default(),
                        cap,
                    ),
                    tip2,
                );
                r = (p[0] * p[0] + p[1] * p[1]).sqrt();
            }
            cells.push(format!("{n:4} / {r:7.3}"));
        }
        println!("{f:5.2} | {} | {} | {}", cells[0], cells[1], cells[2]);
    }
}

/// **O `damping` contra a ESTABILIDADE.** A tabela anterior mediu só
/// convergência num alvo alcançável, e amortecimento baixo é rápido justamente
/// onde é perigoso: perto de uma configuração singular (a cadeia esticada). A
/// segunda coluna é essa: o raio final com alvo fora de alcance.
#[test]
#[ignore = "sweep: prints a table"]
fn sweep_the_ik_damping() {
    println!("damping | err 10 solves (alcancavel) | raio 300 solves (inalcancavel, ideal 3,0)");
    for &d in &[0.02f32, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 4.0] {
        let opts = IkOptions {
            damping: d,
            max_iters: 10,
            match_angle: false,
        };
        let (w, _, links, tip) = three_link_chain();
        let mut chain = w.ik_chain(links[0].parent, &links, tip).expect("chain");
        let mut e = 0.0;
        for _ in 0..10 {
            e = tip_err(&w, &mut chain, tip, [1.2, 1.8], opts);
        }
        let (w2, _, links2, tip2) = three_link_chain();
        let mut c2 = w2.ik_chain(links2[0].parent, &links2, tip2).expect("chain");
        let mut r = 0.0;
        for _ in 0..300 {
            let p = tip_of(&w2.ik_solve(&mut c2, [30.0, 0.5], 0.0, opts), tip2);
            r = (p[0] * p[0] + p[1] * p[1]).sqrt();
        }
        println!("{d:7.2} | {e:26.4} | {r:.3}");
    }
}

/// **O teto de iterações.** O que UMA chamada compra, que é o que o artista
/// sente por movimento de mouse.
#[test]
#[ignore = "sweep: prints a table"]
fn sweep_the_ik_iterations() {
    println!("iters | err 1 solve | err 5 solves | err 20 solves");
    for &n in &[1usize, 2, 4, 8, 10, 16, 24, 40] {
        let opts = IkOptions {
            damping: 0.1,
            max_iters: n,
            match_angle: false,
        };
        let (w, _, links, tip) = three_link_chain();
        let mut chain = w.ik_chain(links[0].parent, &links, tip).expect("chain");
        let e1 = tip_err(&w, &mut chain, tip, [1.2, 1.8], opts);
        let mut e5 = e1;
        for _ in 0..4 {
            e5 = tip_err(&w, &mut chain, tip, [1.2, 1.8], opts);
        }
        let mut e20 = e5;
        for _ in 0..15 {
            e20 = tip_err(&w, &mut chain, tip, [1.2, 1.8], opts);
        }
        println!("{n:5} | {e1:11.4} | {e5:12.4} | {e20:13.4}");
    }
}

/// **O custo.** Uma IK roda por movimento de mouse, então o número que importa
/// é o de UM solve — e o de CONSTRUIR a árvore, que acontece uma vez por gesto.
#[test]
#[ignore = "sweep: prints a table"]
fn sweep_the_ik_cost() {
    use std::time::Instant;
    for &n in &[3usize, 8, 16, 32] {
        let mut w = PhysicsWorld::new();
        let (hook, _) = w.add_static_cuboid(0.0, 0.0, 0.1, 0.1);
        let mut links = Vec::new();
        let mut prev = hook;
        let mut prev_anchor = [0.0f32, 0.0];
        for i in 0..n {
            let b = link(&mut w, 0.5 + i as f32, 0.0, 0.5, 0.1);
            links.push(IkLink {
                parent: prev,
                child: b,
                joint: pin(prev_anchor, [-0.5, 0.0]),
            });
            prev = b;
            prev_anchor = [0.5, 0.0];
        }
        let tip = prev;
        let t0 = Instant::now();
        let mut chain = w.ik_chain(hook, &links, tip).expect("chain");
        let build = t0.elapsed().as_secs_f64() * 1e3;
        let t1 = Instant::now();
        for k in 0..500 {
            let target = [n as f32 * 0.5, 0.5 + (k % 10) as f32 * 0.05];
            let _ = w.ik_solve(&mut chain, target, 0.0, IkOptions::default());
        }
        let solve = t1.elapsed().as_secs_f64() * 1e3 / 500.0;
        println!("{n:2} elos: build {build:.4} ms | solve {solve:.4} ms");
    }
}

fn tip_of(poses: &[IkPose], tip: RigidBodyHandle) -> [f32; 2] {
    poses
        .iter()
        .find(|p| p.body == tip)
        .expect("tip is in the tree")
        .translation
}

fn tip_err(
    w: &PhysicsWorld,
    chain: &mut IkChain,
    tip: RigidBodyHandle,
    target: [f32; 2],
    opts: IkOptions,
) -> f32 {
    let p = tip_of(&w.ik_solve(chain, target, 0.0, opts), tip);
    ((p[0] - target[0]).powi(2) + (p[1] - target[1]).powi(2)).sqrt()
}

fn err_after(
    w: &PhysicsWorld,
    chain: &mut IkChain,
    tip: RigidBodyHandle,
    target: [f32; 2],
    cap: f32,
    times: usize,
) -> f32 {
    let mut p = [0.0f32, 0.0];
    for _ in 0..times {
        p = tip_of(
            &w.ik_solve_stepped(chain, target, 0.0, IkOptions::default(), cap),
            tip,
        );
    }
    ((p[0] - target[0]).powi(2) + (p[1] - target[1]).powi(2)).sqrt()
}

// -------------------------------------------------------------------- gates

#[test]
fn the_tip_reaches_a_target_inside_the_chains_reach() {
    let (w, _, links, tip) = three_link_chain();
    let mut chain = w.ik_chain(links[0].parent, &links, tip).expect("chain");
    let target = [1.2f32, 1.8];
    let mut d = f32::MAX;
    for _ in 0..20 {
        d = solve_to(&w, &mut chain, tip, target, IkOptions::default()).1;
    }
    assert!(d < 0.02, "tip stopped {d:.4} m from a reachable target");
}

#[test]
fn the_whole_chain_bends_not_just_the_tip() {
    // O ponto inteiro da IK: as juntas do MEIO se movem. Um "solver" que só
    // teleportasse a ponta passaria no gate acima e falharia neste.
    let (w, _, links, tip) = three_link_chain();
    let mut chain = w.ik_chain(links[0].parent, &links, tip).expect("chain");
    let mid = links[0].child;
    let before = w.body_pose(mid).expect("link 1 exists").translation.y;
    for _ in 0..20 {
        let _ = w.ik_solve(&mut chain, [1.2, 1.8], 0.0, IkOptions::default());
    }
    let poses = w.ik_solve(&mut chain, [1.2, 1.8], 0.0, IkOptions::default());
    let after = poses
        .iter()
        .find(|p| p.body == mid)
        .expect("link 1 is in the tree")
        .translation[1];
    assert!(
        (after - before).abs() > 0.2,
        "the first link barely moved ({before:.3} -> {after:.3}); the chain is not bending"
    );
    // E a ponta, que é quem foi arrastada, se moveu mais que o meio.
    let tip_after = poses
        .iter()
        .find(|p| p.body == tip)
        .expect("tip")
        .translation[1];
    assert!(tip_after > after, "the tip should lead the bend");
}

#[test]
fn the_root_does_not_move_when_it_is_static() {
    // Uma raiz FIXA tem zero graus de liberdade: puxar a ponta não pode
    // arrancar o gancho da parede.
    let (w, hook, links, tip) = three_link_chain();
    let mut chain = w.ik_chain(hook, &links, tip).expect("chain");
    let poses = w.ik_solve(&mut chain, [0.5, 2.5], 0.0, IkOptions::default());
    let root = poses.iter().find(|p| p.body == hook);
    if let Some(r) = root {
        assert!(
            r.translation[0].abs() < 1e-4 && r.translation[1].abs() < 1e-4,
            "the static root moved to {:?}",
            r.translation
        );
    }
}

#[test]
fn an_unreachable_target_stretches_toward_it_without_exploding() {
    // O alcance é 3 m. Pedir 30 m é um gesto normal (o artista arrasta para
    // fora); a resposta certa é a cadeia ESTICADA na direção do alvo, com
    // números finitos — não um NaN vindo do pseudo-inverso na configuração
    // singular, que é exatamente onde uma cadeia esticada vive.
    let (w, _, links, tip) = three_link_chain();
    let mut chain = w.ik_chain(links[0].parent, &links, tip).expect("chain");
    let mut pose = [0.0f32, 0.0];
    for _ in 0..40 {
        pose = solve_to(&w, &mut chain, tip, [30.0, 0.5], IkOptions::default()).0;
    }
    assert!(
        pose[0].is_finite() && pose[1].is_finite(),
        "the solve produced {pose:?}"
    );
    let r = (pose[0] * pose[0] + pose[1] * pose[1]).sqrt();
    assert!(
        (2.0..=3.1).contains(&r),
        "the chain should be near full stretch, tip radius = {r:.3}"
    );
}

/// **E ela aponta PARA o alvo.** ⚠️ A metade que faltava, e o gate acima era
/// VERDE sem ela: medido, a cadeia esticava (raio 2,484 de 2,5) e **empacava
/// 28° fora da direção do alvo**, para sempre. Esticar e apontar são duas
/// perguntas, e um oráculo de RAIO não pode ver a segunda.
#[test]
fn an_unreachable_target_is_pointed_at_not_merely_reached_for() {
    let (w, _, links, tip) = three_link_chain();
    let mut chain = w.ik_chain(links[0].parent, &links, tip).expect("chain");
    let target = [30.0f32, 20.0];
    let mut p = [0.0f32, 0.0];
    for _ in 0..40 {
        p = solve_to(&w, &mut chain, tip, target, IkOptions::default()).0;
    }
    let got = p[1].atan2(p[0]);
    let want = target[1].atan2(target[0]);
    assert!(
        (got - want).abs() < 0.05,
        "the stretched chain points at {got:.3} rad, target is at {want:.3}"
    );
}

#[test]
fn a_zero_damping_is_clamped_rather_than_dividing_by_a_singular_matrix() {
    let o = IkOptions {
        damping: 0.0,
        max_iters: 0,
        match_angle: false,
    }
    .clamped();
    assert_eq!(o.damping, IkOptions::MIN_DAMPING);
    assert_eq!(o.max_iters, IkOptions::MIN_ITERS);
    let nan = IkOptions {
        damping: f32::NAN,
        ..IkOptions::default()
    }
    .clamped();
    assert_eq!(nan.damping, IkOptions::default().damping);
}

#[test]
fn a_tip_that_is_the_root_is_refused() {
    // Arrastar a raiz é uma TRANSLAÇÃO (a W-JG), não uma IK — e uma raiz não
    // tem junta pai a resolver, então o handle nem existe.
    let (w, hook, links, _) = three_link_chain();
    assert!(w.ik_chain(hook, &links, hook).is_none());
    assert!(w.ik_chain(hook, &[], links[0].child).is_none());
}

#[test]
fn a_non_dynamic_link_is_refused_instead_of_panicking() {
    // `Multibody::forward_kinematics` tem um `assert_eq!` para isto, e um
    // pânico dentro de um arrasto derruba o app com a arte por salvar.
    let mut w = PhysicsWorld::new();
    let (hook, _) = w.add_static_cuboid(0.0, 0.0, 0.1, 0.1);
    let (wall, _) = w.add_static_cuboid(1.0, 0.0, 0.1, 0.1);
    let links = vec![IkLink {
        parent: hook,
        child: wall,
        joint: pin([0.0, 0.0], [0.0, 0.0]),
    }];
    assert!(w.ik_chain(hook, &links, wall).is_none());
}

#[test]
fn only_rigid_joints_are_links() {
    assert!(is_rigid_link(JointKind::Pin));
    assert!(is_rigid_link(JointKind::Weld));
    assert!(is_rigid_link(JointKind::Slider));
    // Uma mola e uma corda são SOFT: a pose delas é resultado de forças, não
    // de coordenadas generalizadas. Elas são fronteiras da travessia.
    assert!(!is_rigid_link(JointKind::Spring));
    assert!(!is_rigid_link(JointKind::Rope));
}

#[test]
fn a_hinge_limit_is_honoured_while_posing() {
    // Um cotovelo que não dobra para trás na simulação não pode dobrar para
    // trás ao ser posado — senão a pose autorada é uma que o Play desfaz no
    // primeiro tick.
    let mut w = PhysicsWorld::new();
    let (hook, _) = w.add_static_cuboid(0.0, 0.0, 0.1, 0.1);
    let l1 = link(&mut w, 0.5, 0.0, 0.5, 0.1);
    let mut j = pin([0.0, 0.0], [-0.5, 0.0]);
    // Só pode girar para CIMA, e pouco.
    j.limits = Some([0.0, 0.3]);
    let links = vec![IkLink {
        parent: hook,
        child: l1,
        joint: j,
    }];
    let mut chain = w.ik_chain(hook, &links, l1).expect("chain");
    // Puxa bem para BAIXO — a direção proibida.
    let mut y = 0.0;
    for _ in 0..30 {
        y = solve_to(&w, &mut chain, l1, [0.0, -3.0], IkOptions::default()).0[1];
    }
    assert!(
        y > -0.2,
        "the limited hinge folded the wrong way to y={y:.3}"
    );
}

#[test]
fn the_chain_reports_its_size() {
    let (w, _, links, tip) = three_link_chain();
    let chain = w.ik_chain(links[0].parent, &links, tip).expect("chain");
    // Três elos + a raiz.
    assert_eq!(chain.len(), 4);
    assert!(!chain.is_empty());
}

#[test]
fn the_step_cap_is_a_property_of_the_chain_not_a_number_in_metres() {
    // Um teto absoluto daria o MESMO número às duas, e é assim que a proteção
    // some numa escala e não na outra.
    let (w1, _, l1, t1) = chain_of(0.2);
    let (w5, _, l5, t5) = chain_of(5.0);
    let c1 = w1.ik_chain(l1[0].parent, &l1, t1).expect("chain");
    let c5 = w5.ik_chain(l5[0].parent, &l5, t5).expect("chain");
    let ratio = c5.max_step / c1.max_step;
    assert!(
        (ratio - 25.0).abs() < 0.5,
        "the cap should scale with the link span (25x), got {ratio:.2}x"
    );
}

#[test]
fn a_small_chain_stretches_toward_an_unreachable_target_too() {
    // O gate irmão do de 1 m. É ELE que morre se o teto voltar a ser absoluto:
    // 0,25 m sobre elos de 0,2 m é um teto que não segura nada.
    let (w, _, links, tip) = chain_of(0.2);
    let mut chain = w.ik_chain(links[0].parent, &links, tip).expect("chain");
    let mut p = [0.0f32, 0.0];
    for _ in 0..200 {
        p = solve_to(&w, &mut chain, tip, [6.0, 0.1], IkOptions::default()).0;
    }
    let r = (p[0] * p[0] + p[1] * p[1]).sqrt();
    assert!(
        (0.4..=0.62).contains(&r),
        "a 0.2 m-link chain should reach full stretch (~0.5), got {r:.3}"
    );
}

#[test]
fn a_degenerate_chain_still_has_a_usable_step_cap() {
    // Todos os elos ancorados no MESMO ponto: vão zero. Um teto zero congelaria
    // a ferramenta em vez de a proteger.
    let mut w = PhysicsWorld::new();
    let (hook, _) = w.add_static_cuboid(0.0, 0.0, 0.1, 0.1);
    let a = link(&mut w, 0.0, 0.0, 0.1, 0.1);
    let b = link(&mut w, 0.0, 0.0, 0.1, 0.1);
    let links = vec![
        IkLink {
            parent: hook,
            child: a,
            joint: pin([0.0, 0.0], [0.0, 0.0]),
        },
        IkLink {
            parent: a,
            child: b,
            joint: pin([0.0, 0.0], [0.0, 0.0]),
        },
    ];
    let chain = w.ik_chain(hook, &links, b).expect("chain");
    assert!(chain.max_step >= PhysicsWorld::MIN_IK_STEP_M);
}

/// **Alvo fora de alcance: a cadeia aponta PARA ele?** O gate irmão afirma o
/// RAIO (ela estica); este afirma a DIREÇÃO, que é a metade que faltava.
#[test]
#[ignore = "sweep: prints a table"]
fn sweep_the_unreachable_direction() {
    println!("solves | raio | angulo da cadeia | angulo do alvo | erro");
    let (w, _, links, tip) = three_link_chain();
    let mut chain = w.ik_chain(links[0].parent, &links, tip).expect("chain");
    let target = [30.0f32, 20.0];
    let want = target[1].atan2(target[0]);
    for n in [1usize, 5, 10, 20, 40, 100, 400] {
        let (w2, _, l2, t2) = three_link_chain();
        let mut c2 = w2.ik_chain(l2[0].parent, &l2, t2).expect("chain");
        let mut p = [0.0f32, 0.0];
        for _ in 0..n {
            p = solve_to(&w2, &mut c2, t2, target, IkOptions::default()).0;
        }
        let got = p[1].atan2(p[0]);
        println!(
            "{n:6} | {:.3} | {:14.3} | {want:14.3} | {:.3}",
            (p[0] * p[0] + p[1] * p[1]).sqrt(),
            got,
            (got - want).abs()
        );
    }
    let _ = &mut chain;
}
