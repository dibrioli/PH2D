//! **A rota que não resolve** — os guardas de degeneração da §10 do plano, medidos
//! (W-Pulley, 2026-07-29).
//!
//! O plano pedia *"cada um com decisão explícita em vez de `NaN` silencioso"*. A
//! medição (`measure_pulley_degenerate.rs`) mostrou que o `NaN` **já estava
//! barrado** no kernel — `rope_route::tangent` recusa quando `inner <= 0` e `route`
//! pula a corda inteira — e que o que faltava era **a jusante**. Estes gates pinam
//! o que já está certo (para não regredir) e o que a wave corrigiu:
//!
//! | configuração | rota | carga (controle −0,460) | finito |
//! |---|---|---|---|
//! | âncora dentro da roldana | `None` | −1,237 | sim |
//! | roldanas sobrepostas | **9,9109** | −1,021 | sim |
//! | roldana dentro da outra | `None` | −2,910 | sim |
//!
//! ⚠️ **Uma das três degenerações do plano NÃO degenera:** duas roldanas
//! sobrepostas do MESMO lado têm tangente externa (`rr` é a DIFERENÇA dos raios),
//! então a rota existe e só encurta. Quem recusa é a sobreposição em lados
//! OPOSTOS, onde `rr` é a soma. A lista do plano tratava as três como iguais.
//!
//! ⚠️ **E a sim não consegue chegar lá** — ver
//! `the_sim_cannot_pull_an_anchor_into_a_wheel`. A degeneração é alcançável só por
//! AUTORIA, que acontece em repouso, e é isso que torna a rota derivada em repouso
//! a fonte certa da cor no overlay.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics::world::rope_route;
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, PulleyWheel,
    RigidBody,
};

/// O elevador das sondas: carga de 3 kg, contrapeso de 1 kg, duas roldanas.
fn rig() -> SimWorld {
    let mut sim = SimWorld::new();
    let mut body = |name: &str, x: f32, y: f32, kind: BodyKind, mass: f32| {
        sim.world_mut().spawn((
            Name::new(name),
            RigidBody { kind },
            Collider {
                shape: ColliderShape::Ball { radius: 0.2 },
                density: mass / (std::f32::consts::PI * 0.04),
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, y)),
        ));
    };
    body("Floor", 0.0, -4.0, BodyKind::Static, 1.0);
    body("Load", -1.5, 2.0, BodyKind::Dynamic, 3.0);
    body("Counter", 1.5, 2.0, BodyKind::Dynamic, 1.0);
    sim.world_mut().spawn((
        Name::new("Rope"),
        PhysicsJoint {
            body_a: stable_name_id("Load"),
            body_b: stable_name_id("Counter"),
            kind: JointKind::Pulley,
            ..PhysicsJoint::of_kind(JointKind::Pulley)
        },
        Transform::from_translation(Vec2::new(-1.5, 2.0)),
    ));
    for (i, x) in [-1.5f32, 1.5].into_iter().enumerate() {
        sim.world_mut().spawn((
            Name::new(format!("Rope Wheel {}", i + 1)),
            PulleyWheel {
                rope: stable_name_id("Rope"),
                order: u16::try_from(i).expect("duas roldanas"),
                radius: 0.3,
                ..Default::default()
            },
            Transform::from_translation(Vec2::new(x, 6.0)),
        ));
    }
    sim
}

fn entity_of(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entidade viva")
}

fn t_of(sim: &mut SimWorld, name: &str) -> Vec2 {
    let e = entity_of(sim, name);
    sim.world().get::<Transform>(e).expect("t").translation
}

fn set_wheel(sim: &mut SimWorld, name: &str, centre: Vec2, radius: f32) {
    let e = entity_of(sim, name);
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) {
        t.translation = centre;
    }
    if let Some(mut w) = sim.world_mut().get_mut::<PulleyWheel>(e) {
        w.radius = radius;
    }
}

fn home(sim: &mut SimWorld) {
    set_wheel(sim, "Rope Wheel 1", Vec2::new(-1.5, 6.0), 0.3);
    set_wheel(sim, "Rope Wheel 2", Vec2::new(1.5, 6.0), 0.3);
}

/// Os três gestos que o plano nomeia.
fn degenerate(sim: &mut SimWorld, which: usize) {
    match which {
        // A roldana ENGOLE a âncora da carga, que está em (−1,5; 2).
        0 => set_wheel(sim, "Rope Wheel 1", Vec2::new(-1.5, 2.2), 1.0),
        // As duas se SOBREPÕEM (e esta, medida, NÃO degenera).
        1 => set_wheel(sim, "Rope Wheel 2", Vec2::new(-1.2, 6.0), 0.3),
        // Uma DENTRO da outra.
        _ => {
            set_wheel(sim, "Rope Wheel 1", Vec2::new(-1.5, 6.0), 2.0);
            set_wheel(sim, "Rope Wheel 2", Vec2::new(-1.4, 6.0), 0.2);
        }
    }
}

fn run(sim: &mut SimWorld, bridge: &mut PhysicsBridge) -> (Vec2, Vec2) {
    for t in 1..=60u64 {
        bridge.dispatch(sim, true, t);
    }
    (t_of(sim, "Load"), t_of(sim, "Counter"))
}

/// **Nenhuma degeneração produz `NaN`** — o guarda do kernel, pinado pelo lado do
/// produto.
///
/// A recusa vive em `rope_route::tangent` (`inner <= 0`) e a corda inteira é
/// pulada. Se alguém a trocar por um `sqrt` de número negativo, o `NaN` viaja pelo
/// impulso até o `Transform` e envenena o `GlobalTransform` da subárvore — e o
/// `physics_ecs_c9` junto.
#[test]
fn a_degenerate_route_never_produces_a_nan() {
    for which in 0..3 {
        let mut sim = rig();
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, false, 0);
        degenerate(&mut sim, which);
        bridge.dispatch(&mut sim, false, 0);
        let (l, c) = run(&mut sim, &mut bridge);
        assert!(
            l.x.is_finite() && l.y.is_finite() && c.x.is_finite() && c.y.is_finite(),
            "config {which}: a degeneração vazou um non-finite (carga {l:?}, contra {c:?})"
        );
    }
}

/// **E desfazer o gesto devolve a cena EXATAMENTE** — a corda volta a segurar.
///
/// O oráculo é o CONTROLE: a carga tem de pousar onde ela pousaria se ninguém
/// tivesse degenerado nada. Isto é o que prova que a recusa é *transitória* e não
/// deixa estado podre atrás dela (um `L0` selado errado, um lado congelado, uma
/// âncora deslizada).
#[test]
fn a_degenerate_route_recovers_completely() {
    let control = {
        let mut sim = rig();
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, false, 0);
        run(&mut sim, &mut bridge).0.y
    };
    for which in 0..3 {
        let mut sim = rig();
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, false, 0);
        degenerate(&mut sim, which);
        bridge.dispatch(&mut sim, false, 0);
        home(&mut sim);
        bridge.dispatch(&mut sim, false, 0);
        let y = run(&mut sim, &mut bridge).0.y;
        assert!(
            (y - control).abs() < 1.0e-3,
            "config {which}: depois de desfazer, a carga pousou em {y:+.4} contra \
             {control:+.4} do controle — a recusa deixou estado atrás dela"
        );
    }
}

/// **Uma corda que NASCE degenerada é resgatada pelo PISO.**
///
/// Uma cena que chega degenerada (um projeto salvo, um undo) sela o `L0` no default
/// de 1 m com `anchored = true` — um número derivado que ninguém pôde derivar. O
/// piso o levanta para a rota verdadeira assim que a geometria fica válida.
///
/// Mutação (o piso removido): o `L0` fica preso em 1,0 sobre uma rota de 11,965 e a
/// carga acaba em **−6,528** (config 0 e 2) ou é arremessada a **+2,926** (config 1)
/// — contra −0,460 do controle.
#[test]
fn a_rope_born_degenerate_is_rescued_by_the_floor() {
    let control = {
        let mut sim = rig();
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, false, 0);
        run(&mut sim, &mut bridge).0.y
    };
    for which in 0..3 {
        let mut sim = rig();
        degenerate(&mut sim, which);
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, false, 0);
        // E o artista conserta a geometria.
        home(&mut sim);
        bridge.dispatch(&mut sim, false, 0);
        let y = run(&mut sim, &mut bridge).0.y;
        assert!(
            (y - control).abs() < 1.0e-3,
            "config {which}: consertada a geometria, a carga pousou em {y:+.4} contra \
             {control:+.4} do controle — o selo bogus do L0 sobreviveu"
        );
    }
}

/// **A SIMULAÇÃO não consegue puxar uma âncora para dentro de uma roldana** — e é
/// esta a premissa que decide de onde vem a cor do overlay.
///
/// A corda puxa ao longo da **tangente**, que por construção fica fora do círculo,
/// então o vínculo não pode levar a própria âncora ao centro. Medido com um
/// GUINCHO enrolando por 240 tiques (o único jeito de a corda arrastar a âncora, já
/// que subir só afrouxa): a âncora chega a **0,6789 m** de um centro de raio 0,5 e
/// nunca entra.
///
/// ⚠️ **Se este gate sangrar, a decisão de desenho tem de ser revisitada:** o
/// overlay colore pela rota derivada em REPOUSO, o que é correto exactamente porque
/// a degeneração só é alcançável por autoria. Uma corda que degenerasse no meio da
/// corrida precisaria da verdade VIVA — e o solver hoje não a publica (ele pula com
/// um `continue` e não registra nada).
#[test]
fn the_sim_cannot_pull_an_anchor_into_a_wheel() {
    const R: f32 = 0.5;
    let mut sim = SimWorld::new();
    let mut body = |name: &str, x: f32, y: f32, kind: BodyKind, mass: f32| {
        sim.world_mut().spawn((
            Name::new(name),
            RigidBody { kind },
            Collider {
                shape: ColliderShape::Ball { radius: 0.2 },
                density: mass / (std::f32::consts::PI * 0.04),
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, y)),
        ));
    };
    body("Floor", 0.0, -8.0, BodyKind::Static, 1.0);
    body("Load", -1.5, 2.0, BodyKind::Dynamic, 0.5);
    body("Counter", 1.5, 2.0, BodyKind::Dynamic, 6.0);
    sim.world_mut().spawn((
        Name::new("Rope"),
        PhysicsJoint {
            body_a: stable_name_id("Load"),
            body_b: stable_name_id("Counter"),
            kind: JointKind::Pulley,
            ..PhysicsJoint::of_kind(JointKind::Pulley)
        },
        Transform::from_translation(Vec2::new(-1.5, 2.0)),
    ));
    for (i, x) in [-1.5f32, 1.5].into_iter().enumerate() {
        sim.world_mut().spawn((
            Name::new(format!("Rope Wheel {}", i + 1)),
            PulleyWheel {
                rope: stable_name_id("Rope"),
                order: u16::try_from(i).expect("duas"),
                radius: R,
                // O guincho, na roldana alinhada com a âncora da carga.
                motor_speed: if i == 0 { 4.0 } else { 0.0 },
                ..Default::default()
            },
            Transform::from_translation(Vec2::new(x, 6.0)),
        ));
    }

    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let mut closest = f32::INFINITY;
    let mut ever_degenerate = false;
    for t in 1..=240u64 {
        bridge.dispatch(&mut sim, true, t);
        let v = bridge.joint_views().next().expect("uma view");
        let arena = bridge.pulley_wheel_arena();
        let s = v.wheel_start as usize;
        let wheels = &arena[s..s + v.wheel_count as usize];
        let c = wheels[0].centre;
        closest = closest.min((v.anchor_a[0] - c[0]).hypot(v.anchor_a[1] - c[1]));
        let mut segs = Vec::new();
        ever_degenerate |= rope_route::route(v.anchor_a, v.anchor_b, wheels, &mut segs).is_none();
    }
    // O CONTROLE POSITIVO: sem um guincho que de fato enrola, a carga nem se
    // aproxima e o número abaixo não significaria nada.
    assert!(
        closest < 2.0,
        "a fixture não aproximou a âncora da roldana ({closest:.4} m) — o guincho não \
         está enrolando, e o limite abaixo passaria de graça"
    );
    assert!(
        closest > R,
        "a âncora ENTROU na roldana ({closest:.4} m contra raio {R}): a sim passou a \
         alcançar a degeneração, e a cor do overlay não pode mais sair da rota de repouso"
    );
    assert!(
        !ever_degenerate,
        "a rota degenerou durante a corrida — mesma conclusão do limite acima"
    );
}
