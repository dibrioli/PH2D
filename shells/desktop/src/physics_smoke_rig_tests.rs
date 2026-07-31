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
    let (bodies, joints, _) = crate::joint_rig::apply(
        &mut sim,
        &plan,
        ph2d_physics_ecs::JointKind::Pin,
        &queue,
        &reg,
    );
    ph2d_ecs::scene::apply_editor_commands(sim.world_mut(), &queue, &reg).expect("commit");
    (sim, bodies, joints)
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

/// Roda `ticks` da sim e devolve `(queda do tronco, deriva do pescoço)`.
///
/// ⚠️ A deriva é a MUDANÇA da distância tronco↔cabeça, não a distância — um Pin
/// segura os dois num ponto, e o que um joint quebrado faria é deixá-la CRESCER.
/// A distância em si é geometria do boneco e não diz nada sobre o rig.
fn run(ticks: u64) -> (f32, f32) {
    let (mut sim, _, _) = rigged();
    let y0 = pos(&mut sim, "Torso").y;
    let d0 = (pos(&mut sim, "Head") - pos(&mut sim, "Torso")).length();

    let mut bridge = PhysicsBridge::new();
    for t in 1..=ticks {
        bridge.dispatch(&mut sim, true, t);
    }
    let y1 = pos(&mut sim, "Torso").y;
    let d1 = (pos(&mut sim, "Head") - pos(&mut sim, "Torso")).length();
    (y0 - y1, (d1 - d0).abs())
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
        println!("t={secs:.0}s  queda do tronco {drop:.2} m  deriva do pescoço {drift:.4} m");
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
        drift <= MEASURED_NECK_DRIFT + 0.02,
        "o pescoço separou {drift:.4} m — a mensagem promete {MEASURED_NECK_DRIFT:.2}, e um \
         joint que não segura é um rig que não rigou"
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
