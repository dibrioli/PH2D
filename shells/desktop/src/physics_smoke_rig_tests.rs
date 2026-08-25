//! A sonda da cena 67 + o gate que mantém a mensagem dela honesta (W-Rig).

use super::*;
use ph2d_ecs::{Name, SimWorld};
use ph2d_physics_ecs::{PhysicsBridge, PhysicsJoint, RigidBody};

/// Monta o boneco, roda o rig pela PORTA REAL e devolve o mundo.
///
/// ⚠️ Pela porta, e não semeando corpos à mão: um rig simulado com
/// `RigidBody`+`PhysicsJoint` escritos aqui provaria que a física funciona, que
/// ninguém duvidava, e não que o GERADOR produz o que a cena promete.
fn rigged() -> (SimWorld, usize, usize) {
    let mut sim = SimWorld::new();
    crate::physics_smoke::spawn_floor(sim.world_mut());
    let torso = build_rig_doll(sim.world_mut());

    let reg = {
        let mut r = ph2d_ecs::scene::ComponentRegistry::new();
        ph2d_ecs::scene::register_ecs_components(&mut r);
        ph2d_physics_ecs::register_physics_components(&mut r);
        r
    };
    let queue = ph2d_ecs::scene::EditorCommandQueue::default();
    let plan = crate::joint_rig::plan(&mut sim, &[torso.to_bits()]);
    let out = crate::joint_rig::apply(
        &mut sim,
        &plan,
        ph2d_physics_ecs::JointKind::Pin,
        &queue,
        &reg,
    );
    assert!(out.error.is_none(), "commit: {:?}", out.error);
    (sim, out.bodies, out.joints)
}

/// A pose de MUNDO de uma parte.
///
/// ⚠️ **`Transform` é LOCAL e compõe com o pai** (W5), e num rig toda parte menos
/// a raiz é filha — ler `t.translation` cru mede o OFFSET dentro do tronco, não
/// onde a coisa está. A primeira versão desta sonda fez exatamente isso e
/// reportou o pescoço separando **2,23 m**, um número sobre nada: ele subtraía
/// uma posição de mundo de uma local.
fn pos(sim: &mut SimWorld, name: &str) -> ph2d_core::Vec2 {
    let e = {
        let mut q = sim.world_mut().query::<(ph2d_ecs::Entity, &Name)>();
        q.iter(sim.world())
            .find(|(_, n)| n.as_str() == name)
            .map(|(e, _)| e)
            .expect("parte viva")
    };
    ph2d_ecs::world_transform(sim.world(), e)
        .expect("parte com Transform")
        .translation
}

/// A VIOLAÇÃO da restrição: a distância entre as duas âncoras de um joint, em
/// mundo. Um Pin as prende no MESMO ponto, então este número é zero enquanto o
/// joint segura, e é ele que cresce quando não segura.
///
/// ⚠️ **O oráculo anterior media a distância entre os CENTROS**, e ele mentia a
/// partir do momento em que a âncora foi para a EMENDA: com o pivô no pescoço a
/// cabeça GIRA em torno dele, então a distância entre centros varia de 0,3 a 0,7
/// por pura geometria — 0,35 m que se lê como *"o joint soltou"* e não é. A
/// violação é invariante sob rotação porque ela É a restrição.
fn constraint_gap(sim: &mut SimWorld, joint_name: &str) -> f32 {
    let (j, pj) = {
        let mut q = sim.world_mut().query::<(&Name, &PhysicsJoint)>();
        q.iter(sim.world())
            .find(|(n, _)| n.as_str() == joint_name)
            .map(|(n, p)| (n.as_str().to_string(), *p))
            .expect("joint vivo")
    };
    let _ = j;
    // ⚠️ Resolve pela IDENTIDADE (ADR-0164 F1) — o joint deixou de guardar o hash do nome, e
    // procurar por hash não encontrava o corpo (o `expect("corpo vivo")` disparava).
    let mut anchor = |id: u64, local: [f32; 2]| -> ph2d_core::Vec2 {
        let e = {
            let mut q = sim
                .world_mut()
                .query::<(ph2d_ecs::Entity, &ph2d_ecs::StableId)>();
            q.iter(sim.world())
                .find(|(_, s)| s.0 == id)
                .map(|(e, _)| e)
                .expect("corpo vivo")
        };
        let t = ph2d_ecs::world_transform(sim.world(), e).expect("pose");
        let (s, c) = (t.rotation.sin(), t.rotation.cos());
        ph2d_core::Vec2::new(
            t.translation.x + c * local[0] - s * local[1],
            t.translation.y + s * local[0] + c * local[1],
        )
    };
    let wa = anchor(pj.body_a, pj.local_a);
    let wb = anchor(pj.body_b, pj.local_b);
    (wa - wb).length()
}

/// Roda `ticks` da sim e devolve `(queda do tronco, violação do pescoço)`.
fn run(ticks: u64) -> (f32, f32) {
    let (mut sim, _, _) = rigged();
    let y0 = pos(&mut sim, "Torso").y;

    let mut bridge = PhysicsBridge::new();
    for t in 1..=ticks {
        ph2d_physics_ecs::resolve_body_names(sim.world_mut());
        bridge.dispatch(&mut sim, true, t);
    }
    let y1 = pos(&mut sim, "Torso").y;
    (y0 - y1, constraint_gap(&mut sim, "Torso : Head"))
}

/// A sonda da cena 67.
///
/// `cargo test -p ph2d-host-desktop --bins probe_smoke_67 -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate"]
fn probe_smoke_67() {
    let (mut sim, bodies, joints) = rigged();
    println!("\n=== CENA 67 — o rig sai da hierarquia ===");
    println!("corpos criados: {bodies} | joints criados: {joints}");

    let mut q = sim.world_mut().query::<(&Name, &PhysicsJoint)>();
    let mut names: Vec<String> = q
        .iter(sim.world())
        .map(|(n, _)| n.as_str().to_string())
        .collect();
    names.sort();
    for n in &names {
        println!("  joint: {n}");
    }
    for (secs, ticks) in [(1.0, 60u64), (3.0, 180)] {
        let (drop, drift) = run(ticks);
        println!("t={secs:.0}s  queda do tronco {drop:.2} m  violacao do pescoco {drift:.5} m");
    }
    // Onde cada joint PENSA que está a âncora, contra onde a emenda de fato é.
    {
        let (mut sim2, _, _) = rigged();
        let mut q = sim2
            .world_mut()
            .query::<(&Name, &PhysicsJoint, &ph2d_ecs::Transform)>();
        let mut rows: Vec<(String, f32, f32)> = q
            .iter(sim2.world())
            .map(|(n, _, t)| (n.as_str().to_string(), t.translation.x, t.translation.y))
            .collect();
        rows.sort_by(|a, b| a.0.cmp(&b.0));
        for (n, x, y) in rows {
            println!("  ancora {n:>14}: ({x:.2}, {y:.2})");
        }
        for n in ["Torso", "Head", "ArmL"] {
            let p = pos(&mut sim2, n);
            println!("  MUNDO t=0 {n:>6}: ({:.2}, {:.2})", p.x, p.y);
        }
    }
    // Onde cada parte pousa, em MUNDO — o retrato do ragdoll.
    let (mut sim, _, _) = rigged();
    let mut bridge = PhysicsBridge::new();
    for t in 1..=180u64 {
        ph2d_physics_ecs::resolve_body_names(sim.world_mut());
        bridge.dispatch(&mut sim, true, t);
    }
    for n in ["Torso", "Head", "ArmL", "ArmR", "LegL", "LegR"] {
        let p = pos(&mut sim, n);
        println!("  {n:>6}: ({:.2}, {:.2})", p.x, p.y);
    }
}

/// **A cena demonstra o que a mensagem afirma:** um clique, seis corpos, cinco
/// joints — e nenhum deles pendura um braço no GRUPO.
#[test]
fn scene_67_rigs_the_whole_doll_in_one_click() {
    let (mut sim, bodies, joints) = rigged();
    assert_eq!(bodies, DOLL_PARTS, "corpos criados");
    assert_eq!(joints, DOLL_JOINTS, "joints criados");

    let arms = {
        let mut q = sim.world_mut().query::<(ph2d_ecs::Entity, &Name)>();
        q.iter(sim.world())
            .find(|(_, n)| n.as_str() == "Arms")
            .map(|(e, _)| e)
            .expect("o grupo continua na cena")
    };
    assert!(
        sim.world().get::<RigidBody>(arms).is_none(),
        "o grupo virou osso — ele não tem desenho, e um collider invisível de \
         meio metro no meio do boneco é o que o `Add` daria a ele"
    );

    let mut q = sim.world_mut().query::<(&Name, &PhysicsJoint)>();
    let names: Vec<String> = q
        .iter(sim.world())
        .map(|(n, _)| n.as_str().to_string())
        .collect();
    assert!(
        names.iter().all(|n| n.starts_with("Torso : ")),
        "algum joint não pendura do tronco: {names:?}"
    );
}

/// **E os números da mensagem são os que a sonda mede.**
///
/// Uma cena que afirma metros que ninguém mediu é a forma exata de mentir
/// devagar — a cena 66 já teve um oráculo periódico corrigido por esta linha.
#[test]
fn the_message_of_scene_67_quotes_measured_numbers() {
    let (drop, drift) = run(180);
    assert!(
        (drop - MEASURED_TORSO_DROP).abs() < 0.35,
        "a mensagem diz que o tronco cai {MEASURED_TORSO_DROP:.1} m e a sonda mede {drop:.2}"
    );
    assert!(
        drift <= MEASURED_JOINT_GAP,
        "a restrição do pescoço foi violada em {drift:.5} m — a mensagem promete \
         menos de {MEASURED_JOINT_GAP:.4}, e um joint que não segura é um rig que \
         não rigou"
    );
}

/// **O boneco DESABA** — o controle da cena. Sem os joints as partes cairiam
/// igual e o rig não teria demonstrado nada; o que se vê é o corpo se dobrar
/// sobre o piso, então o tronco tem de parar ACIMA dele e as partes têm de sair
/// da pilha vertical em que começaram.
#[test]
fn the_rigged_doll_collapses_onto_the_floor_instead_of_falling_through() {
    let (mut sim, _, _) = rigged();
    let mut bridge = PhysicsBridge::new();
    for t in 1..=180 {
        ph2d_physics_ecs::resolve_body_names(sim.world_mut());
        bridge.dispatch(&mut sim, true, t);
    }
    let torso = pos(&mut sim, "Torso");
    assert!(
        torso.y > -1.0,
        "o tronco atravessou o piso (y = {:.2})",
        torso.y
    );
    let head = pos(&mut sim, "Head");
    assert!(
        (head.x - torso.x).abs() > 0.05 || (head.y - torso.y).abs() < 0.65,
        "o boneco continua em pé e empilhado — a cena não mostra um ragdoll \
         (cabeça em {head:?}, tronco em {torso:?})"
    );
}

/// **SONDA DO RESET** (report do Enio, 2026-07-31): o Reset devolve o conjunto?
///
/// `cargo test -p ph2d-host-desktop --bins probe_reset_67 -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate"]
fn probe_reset_67() {
    let (mut sim, _, _) = rigged();
    let names = ["Torso", "Head", "ArmL", "ArmR", "LegL", "LegR"];
    let before: Vec<ph2d_core::Vec2> = names.iter().map(|n| pos(&mut sim, n)).collect();

    let mut bridge = PhysicsBridge::new();
    for t in 1..=120u64 {
        ph2d_physics_ecs::resolve_body_names(sim.world_mut());
        bridge.dispatch(&mut sim, true, t);
    }
    let collapsed: Vec<ph2d_core::Vec2> = names.iter().map(|n| pos(&mut sim, n)).collect();

    // RESET = o relógio volta ao tique 0, PARADO (o que o botão faz).
    bridge.dispatch(&mut sim, false, 0);
    let after: Vec<ph2d_core::Vec2> = names.iter().map(|n| pos(&mut sim, n)).collect();

    println!("\n=== RESET da cena 67 ===");
    println!(
        "{:>6} | {:>16} | {:>16} | {:>16} | erro",
        "parte", "autorado", "desabado", "pos-reset"
    );
    for i in 0..names.len() {
        let err = (after[i] - before[i]).length();
        println!(
            "{:>6} | ({:>6.2},{:>6.2}) | ({:>6.2},{:>6.2}) | ({:>6.2},{:>6.2}) | {err:.3} m",
            names[i],
            before[i].x,
            before[i].y,
            collapsed[i].x,
            collapsed[i].y,
            after[i].x,
            after[i].y
        );
    }
    // E um SEGUNDO dispatch parado — o frame seguinte ao Reset.
    bridge.dispatch(&mut sim, false, 0);
    println!("--- depois de um 2o frame parado ---");
    for (i, n) in names.iter().enumerate() {
        let p = pos(&mut sim, n);
        println!(
            "{n:>6} | ({:>6.2},{:>6.2}) | erro {:.3} m",
            p.x,
            p.y,
            (p - before[i]).length()
        );
    }
}

/// **SONDA DAS LIMITES:** quanto cada junta gira sem batente, e o que um batente
/// de `X` graus faria. O numero da wave sai daqui, nao de intuicao.
#[test]
#[ignore = "measurement, not a gate"]
fn probe_rig_limits() {
    use ph2d_physics_ecs::PhysicsJoint;
    let names = [
        "Torso : Head",
        "Torso : ArmL",
        "Torso : ArmR",
        "Torso : LegL",
        "Torso : LegR",
    ];
    for deg in [f32::INFINITY, 90.0, 60.0, 45.0, 30.0] {
        let (mut sim, _, _) = rigged();
        if deg.is_finite() {
            let ents: Vec<ph2d_ecs::Entity> = {
                let mut q = sim.world_mut().query::<(ph2d_ecs::Entity, &PhysicsJoint)>();
                q.iter(sim.world()).map(|(e, _)| e).collect()
            };
            for e in ents {
                let mut j = sim.world_mut().get_mut::<PhysicsJoint>(e).unwrap();
                j.limits_enabled = true;
                j.limit_min = -deg.to_radians();
                j.limit_max = deg.to_radians();
            }
        }
        let mut bridge = PhysicsBridge::new();
        let mut worst: Vec<f32> = vec![0.0; names.len()];
        let t0 = pos(&mut sim, "Torso");
        for t in 1..=180u64 {
            ph2d_physics_ecs::resolve_body_names(sim.world_mut());
            bridge.dispatch(&mut sim, true, t);
            for (i, n) in names.iter().enumerate() {
                let rel = rel_angle(&mut sim, n).to_degrees().abs();
                worst[i] = worst[i].max(rel);
            }
        }
        let travel = (pos(&mut sim, "Torso") - t0).length();
        let label = if deg.is_finite() {
            format!("+/-{deg:.0} graus")
        } else {
            "SEM batente".to_string()
        };
        println!(
            "{label:>14} | max relativo: {} | queda {travel:.2} m",
            worst
                .iter()
                .map(|v| format!("{v:5.0}"))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
}

/// O angulo do FILHO relativo ao PAI, radianos.
fn rel_angle(sim: &mut SimWorld, joint_name: &str) -> f32 {
    use ph2d_physics_ecs::PhysicsJoint;
    let pj = {
        let mut q = sim.world_mut().query::<(&Name, &PhysicsJoint)>();
        q.iter(sim.world())
            .find(|(n, _)| n.as_str() == joint_name)
            .map(|(_, p)| *p)
            .expect("joint vivo")
    };
    let mut rot = |name_id: u64| -> f32 {
        let e = {
            let mut q = sim.world_mut().query::<(ph2d_ecs::Entity, &Name)>();
            q.iter(sim.world())
                .find(|(_, n)| ph2d_ecs::stable_name_id(n.as_str()) == name_id)
                .map(|(e, _)| e)
                .expect("corpo vivo")
        };
        ph2d_ecs::world_transform(sim.world(), e)
            .expect("pose")
            .rotation
    };
    let a = rot(pj.body_a);
    let b = rot(pj.body_b);
    let mut d = b - a;
    while d > std::f32::consts::PI {
        d -= std::f32::consts::TAU;
    }
    while d < -std::f32::consts::PI {
        d += std::f32::consts::TAU;
    }
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    d
}
