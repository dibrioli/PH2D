//! **A POLIA, do lado do ECS** (W-Pulley) — a ponte roteia um tipo que o rapier
//! não tem.
//!
//! O kernel (a corda inextensível, a talha, o clamp de só-puxa, a massa efetiva
//! exata) é gateado em `ph2d-physics/tests/pulley.rs`. Aqui ficam as perguntas
//! que só existem deste lado da fronteira:
//!
//! 1. a ponte entrega ao passe de polias a corda que o componente descreve — e
//!    **não** ao `ImpulseJointSet`;
//! 2. uma polia nova SEMEIA as roldanas e o comprimento da corda da pose de
//!    repouso, pelo mesmo sentinela `anchored` das âncoras;
//! 3. um rewind a re-arma;
//! 4. o `Active` a solta;
//! 5. ela conduz o grupo articulado como qualquer outro joint.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, RigidBody,
};

/// Onde os dois corpos nascem: lado a lado, na mesma altura.
const START_Y: f32 = 2.0;
const SPAN: f32 = 4.0;

/// Um elevador com contrapeso: a carga à esquerda, o contrapeso à direita,
/// ligados por uma corda que sobe até uma roldana sobre cada um.
fn rig(kind: JointKind, load: f32, counterweight: f32, active: bool) -> SimWorld {
    let mut sim = SimWorld::new();
    let mut body = |name: &str, x: f32, density: f32| {
        sim.world_mut().spawn((
            Name::new(name),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.2 },
                density,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, START_Y)),
        ));
    };
    // Densidade escolhida para a massa sair no número que o nome promete.
    let area = std::f32::consts::PI * 0.2 * 0.2;
    body("Load", -SPAN / 2.0, load / area);
    body("Counterweight", SPAN / 2.0, counterweight / area);
    sim.world_mut().spawn((
        Name::new("Rope"),
        PhysicsJoint {
            body_a: stable_name_id("Load"),
            body_b: stable_name_id("Counterweight"),
            kind,
            active,
            ..PhysicsJoint::of_kind(kind)
        },
        Transform::from_translation(Vec2::new(-SPAN / 2.0, START_Y)),
    ));
    sim
}

fn y_of(sim: &mut SimWorld, name: &str) -> f32 {
    let mut q = sim.world_mut().query::<(&Name, &Transform)>();
    q.iter(sim.world())
        .find(|(n, _)| n.as_str() == name)
        .map(|(_, t)| t.translation.y)
        .expect("body alive")
}

fn joint_of(sim: &mut SimWorld) -> PhysicsJoint {
    let mut q = sim.world_mut().query::<(&Name, &PhysicsJoint)>();
    q.iter(sim.world())
        .find(|(n, _)| n.as_str() == "Rope")
        .map(|(_, j)| *j)
        .expect("joint alive")
}

fn entity_named(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entity alive")
}

fn run(sim: &mut SimWorld, bridge: &mut PhysicsBridge, ticks: u64) {
    for t in 1..=ticks {
        bridge.dispatch(sim, false, t);
    }
}

/// **A ponte entrega a polia — e a corda ergue o contrapeso.**
///
/// O controle é o MESMO rig com o mesmo par de massas e sem corda nenhuma: ali o
/// contrapeso apenas cai.
#[test]
fn the_bridge_folds_a_pulley_and_the_counterweight_rises() {
    let mut sim = rig(JointKind::Pulley, 4.0, 1.0, true);
    let mut bridge = PhysicsBridge::new();
    run(&mut sim, &mut bridge, 60);
    let load = y_of(&mut sim, "Load");
    let cw = y_of(&mut sim, "Counterweight");
    assert!(load < START_Y - 0.5, "a carga tinha de descer: {load:.4}");
    assert!(cw > START_Y + 0.5, "o contrapeso tinha de SUBIR: {cw:.4}");
    // O que um lado desce, o outro sobe.
    let fell = START_Y - load;
    let rose = cw - START_Y;
    assert!(
        (fell - rose).abs() < 0.02,
        "a corda esticou {:.4} m em {fell:.4} m de percurso",
        (fell - rose).abs()
    );
}

/// **Uma polia nova semeia as roldanas e a corda da pose de REPOUSO.**
///
/// Mesmo sentinela das âncoras (`anchored`): a polia é *montada* onde o artista
/// pôs os corpos, uma vez, e depois disso mover um corpo não re-deriva nada.
#[test]
fn a_fresh_pulley_seeds_its_wheels_and_its_rope_from_the_rest_pose() {
    let mut sim = rig(JointKind::Pulley, 1.0, 1.0, true);
    let before = joint_of(&mut sim);
    assert!(!before.anchored, "uma polia nova nasce por semear");
    assert_eq!(before.wheel_a, [0.0, 0.0], "zero é 'ainda não semeado'");

    let mut bridge = PhysicsBridge::new();
    run(&mut sim, &mut bridge, 1);
    let j = joint_of(&mut sim);
    assert!(j.anchored, "o primeiro reconcile tinha de semear");
    // As roldanas ficam DIRETAMENTE ACIMA de cada corpo, meia distância acima.
    let lift = SPAN / 2.0;
    assert!(
        (j.wheel_a[0] - (-SPAN / 2.0)).abs() < 1.0e-4
            && (j.wheel_a[1] - (START_Y + lift)).abs() < 1.0e-4,
        "roldana A em {:?}",
        j.wheel_a
    );
    assert!(
        (j.wheel_b[0] - (SPAN / 2.0)).abs() < 1.0e-4
            && (j.wheel_b[1] - (START_Y + lift)).abs() < 1.0e-4,
        "roldana B em {:?}",
        j.wheel_b
    );
    // E a corda nasce EXATAMENTE esticada: os dois ramos medem o levante.
    assert!(
        (j.max_length - 2.0 * lift).abs() < 1.0e-3,
        "a corda nasceu com {:.4} m, e o vão é {:.4}",
        j.max_length,
        2.0 * lift
    );
}

/// **Um rewind re-arma a polia.** A tabela é reconstruída do estado autorado
/// todo dispatch, então um scrub para trás e de volta reproduz o percurso.
#[test]
fn a_rewind_re_arms_the_pulley() {
    let mut sim = rig(JointKind::Pulley, 4.0, 1.0, true);
    let mut bridge = PhysicsBridge::new();
    run(&mut sim, &mut bridge, 45);
    let straight = y_of(&mut sim, "Counterweight");

    bridge.dispatch(&mut sim, false, 0);
    run(&mut sim, &mut bridge, 45);
    let replayed = y_of(&mut sim, "Counterweight");
    assert!(
        (straight - replayed).abs() < 1.0e-3,
        "o replay tinha de reproduzir: {straight:.5} vs {replayed:.5}"
    );
}

/// **`Active` desmarcado solta a corda** — sem apagar nada dela.
#[test]
fn an_inactive_pulley_lets_go() {
    let mut sim = rig(JointKind::Pulley, 4.0, 1.0, false);
    let mut bridge = PhysicsBridge::new();
    run(&mut sim, &mut bridge, 45);
    let cw = y_of(&mut sim, "Counterweight");
    assert!(
        cw < START_Y,
        "com a corda solta o contrapeso apenas cai: {cw:.4}"
    );
    // E os números seguem autorados: soltar não é apagar.
    assert!(joint_of(&mut sim).max_length > 0.0);
}

/// **Virar uma polia tira o joint do SOLVER.**
///
/// Um Pin segura os dois corpos juntos rigidamente; trocado para Pulley ele tem
/// de sair do `ImpulseJointSet` — senão o vínculo antigo continuaria valendo por
/// baixo, e o artista teria dois vínculos onde autorou um.
#[test]
fn switching_a_pin_to_a_pulley_takes_it_out_of_the_solver() {
    let mut sim = rig(JointKind::Pin, 4.0, 1.0, true);
    let mut bridge = PhysicsBridge::new();
    run(&mut sim, &mut bridge, 30);
    // Um Pin junta os dois corpos no mesmo ponto: eles caem JUNTOS.
    let pinned = (y_of(&mut sim, "Load") - y_of(&mut sim, "Counterweight")).abs();
    assert!(pinned < 0.5, "o pin tinha de juntar os dois: {pinned:.4}");

    let e = entity_named(&mut sim, "Rope");
    if let Some(mut j) = sim.world_mut().get_mut::<PhysicsJoint>(e) {
        j.kind = JointKind::Pulley;
        // Re-semear, como o gesto de trocar o tipo faz.
        j.anchored = false;
    }
    run(&mut sim, &mut bridge, 60);
    let apart = (y_of(&mut sim, "Load") - y_of(&mut sim, "Counterweight")).abs();
    assert!(
        apart > 1.0,
        "a polia manda os dois para lados opostos; distância {apart:.4}"
    );
}

/// **Uma polia conduz o grupo articulado** — assar uma ponta puxa a outra, como
/// em qualquer outro joint.
#[test]
fn a_pulley_conducts_the_jointed_group() {
    let mut sim = rig(JointKind::Pulley, 4.0, 1.0, true);
    let load = entity_named(&mut sim, "Load");
    let group = ph2d_physics_ecs::jointed_group(sim.world_mut(), &[load]);
    assert_eq!(group.len(), 2, "a corda liga os dois corpos: {group:?}");
}
