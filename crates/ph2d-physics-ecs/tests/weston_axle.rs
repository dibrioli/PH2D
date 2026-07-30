//! **A talha de WESTON, do lado do ECS** (W-Pulley, W-Weston) — a autoria de um
//! eixo COMPOSTO atravessado duas vezes.
//!
//! O kernel (o peso `R/(R−r)`, a vantagem `2R/(R−r)`, a recusa do retorno maior, o
//! ramo morto) é gateado em `ph2d-physics/tests/pulley_weston.rs`. Aqui ficam as
//! perguntas que só existem deste lado da fronteira:
//!
//! 1. o marcador **mais** um segundo diâmetro fazem a colheita emitir o 2º contato
//!    — e nenhum dos dois sozinho faz;
//! 2. o contato de retorno é a **CAUDA** da rota, por mais alta que seja a `order`
//!    de outra roldana;
//! 3. ele **não recolhe** (um eixo, uma rotação, um termo de recolhimento);
//! 4. os dois contatos são **um** eixo para a ruptura, e o giro desenhado deles é
//!    o mesmo ângulo;
//! 5. a vantagem atravessa a fronteira e um **rewind a re-arma**.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, PulleyWheel,
    RigidBody, WestonAxle,
};

/// O diâmetro de entrada e o de retorno — `R/(R−r) = 4`, e os dois exatos em `f32`.
const R_IN: f32 = 0.5;
const R_RET: f32 = 0.375;

/// A altura do eixo. Alta o bastante para o contrapeso ter espaço — a mesma
/// correção de fixture que o gate de kernel documenta.
const SHEAVE_Y: f32 = 30.0;

/// **O rig da talha, autorado pelo ECS.**
///
/// `weston` anexa o marcador; `extra_order` acrescenta uma roldana boba com uma
/// `order` ALTA, para o gate da cauda ter o que derrotar.
fn rig(weston: bool, extra_order: Option<u16>) -> SimWorld {
    let mut sim = SimWorld::new();
    let mut body = |name: &str, x: f32, y: f32, kind: BodyKind, density: f32| {
        sim.world_mut().spawn((
            Name::new(name),
            RigidBody { kind },
            Collider {
                shape: ColliderShape::Ball { radius: 0.2 },
                density,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, y)),
        ));
    };
    // A área de uma bola de 0,2 é ~0,1257 m²; a densidade dá a massa.
    body("Dead", -0.8, SHEAVE_Y, BodyKind::Static, 1.0);
    // 8 kg = o equilíbrio previsto contra 1 kg com vantagem 8.
    body("Block", 0.0, 4.0, BodyKind::Dynamic, 8.0 / 0.125_663_7);
    body("Haul", 0.8, 6.0, BodyKind::Dynamic, 1.0 / 0.125_663_7);
    sim.world_mut().spawn((
        Name::new("Rope"),
        PhysicsJoint {
            body_a: stable_name_id("Haul"),
            body_b: stable_name_id("Dead"),
            kind: JointKind::Pulley,
            ..PhysicsJoint::of_kind(JointKind::Pulley)
        },
        Transform::from_translation(Vec2::new(0.8, 6.0)),
    ));
    // O eixo composto, no cenário.
    let axle = sim.world_mut().spawn((
        Name::new("Sheave"),
        PulleyWheel {
            rope: stable_name_id("Rope"),
            order: 0,
            radius: R_IN,
            radius_out: R_RET,
            ..Default::default()
        },
        Transform::from_translation(Vec2::new(0.0, SHEAVE_Y)),
    ));
    if weston {
        let e = axle.id();
        sim.world_mut().entity_mut(e).insert(WestonAxle);
    }
    // A cadernal MÓVEL, montada no bloco.
    sim.world_mut().spawn((
        Name::new("Block Sheave"),
        PulleyWheel {
            rope: stable_name_id("Rope"),
            order: 1,
            radius: 0.15,
            body: stable_name_id("Block"),
            ..Default::default()
        },
        Transform::from_translation(Vec2::new(0.0, 4.0)),
    ));
    if let Some(order) = extra_order {
        // Uma roldana com a `order` mais alta que qualquer outra: se a cauda fosse um
        // sentinela de `order`, ela empataria com o retorno e a rota sairia no
        // desempate por NOME — em silêncio.
        sim.world_mut().spawn((
            Name::new("Late Wheel"),
            PulleyWheel {
                rope: stable_name_id("Rope"),
                order,
                radius: 0.1,
                ..Default::default()
            },
            Transform::from_translation(Vec2::new(-0.4, SHEAVE_Y - 1.0)),
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

fn y_of(sim: &mut SimWorld, name: &str) -> f32 {
    let e = entity_of(sim, name);
    sim.world()
        .get::<Transform>(e)
        .map_or(f32::NAN, |t| t.translation.y)
}

/// A arena — a MESMA lista que o solver e o desenho leem.
fn arena(b: &PhysicsBridge) -> Vec<ph2d_physics_ecs::rope_route::RopeWheel> {
    b.pulley_wheel_arena().to_vec()
}

/// **O marcador MAIS o segundo diâmetro emitem o 2º contato — e nenhum dos dois
/// sozinho.**
///
/// ⚠️ Mutação: colher o retorno sem perguntar `radius_out > 0` faz uma roldana
/// marcada e sem 2º diâmetro emitir um contato de raio ZERO na cauda — que a rota
/// recusa como par, então o marcador viraria um nó extra inerte na corda.
#[test]
fn the_marker_and_a_second_diameter_together_emit_the_return_contact() {
    // Com os dois: três contatos, o último no diâmetro de retorno.
    let mut sim = rig(true, None);
    let mut b = PhysicsBridge::new();
    b.dispatch(&mut sim, false, 0);
    let w = arena(&b);
    assert_eq!(w.len(), 3, "entrada + cadernal + retorno");
    assert_eq!(w[2].radius, R_RET, "o último contato é o diâmetro pequeno");
    assert!(
        ph2d_physics_ecs::rope_route::axle_pair(&w, 0).is_some(),
        "e os dois contatos do eixo formam um par"
    );

    // Sem o marcador: duas roldanas, e a entrada carrega o `radius_out` (o TAMBOR).
    let mut sim = rig(false, None);
    let mut b = PhysicsBridge::new();
    b.dispatch(&mut sim, false, 0);
    let w = arena(&b);
    assert_eq!(w.len(), 2, "sem marcador não há contato de retorno");
    assert_eq!(
        w[0].radius_out,
        Some(R_RET),
        "e o 2º diâmetro fica NO nó — é o tambor adjacente do W4"
    );
    assert_eq!(w[0].axle, 0, "eixo próprio");

    // Com o marcador e SEM 2º diâmetro: inerte.
    let mut sim = rig(true, None);
    let e = entity_of(&mut sim, "Sheave");
    if let Some(mut pw) = sim.world_mut().get_mut::<PulleyWheel>(e) {
        pw.radius_out = 0.0;
    }
    let mut b = PhysicsBridge::new();
    b.dispatch(&mut sim, false, 0);
    let w = arena(&b);
    assert_eq!(w.len(), 2, "o marcador sozinho não emite contato nenhum");
    assert!(w.iter().all(|x| x.axle == 0));
}

/// **O contato de retorno é a CAUDA**, por mais alta que seja a `order` de outra
/// roldana.
///
/// ⚠️ Mutação: pôr o retorno num sentinela de `order` (`u16::MAX`) em vez de num
/// campo próprio da chave faz esta fixture empatar — e o desempate por NOME decide a
/// rota em silêncio. Aqui a `Late Wheel` tem `order` 65535.
#[test]
fn the_return_contact_is_the_tail_whatever_order_anyone_authors() {
    let mut sim = rig(true, Some(u16::MAX));
    let mut b = PhysicsBridge::new();
    b.dispatch(&mut sim, false, 0);
    let w = arena(&b);
    assert_eq!(w.len(), 4, "entrada + cadernal + a boba + retorno");
    assert_eq!(
        w[3].radius, R_RET,
        "o retorno é o ÚLTIMO nó da corda, depois até de uma order 65535"
    );
    let (first, second) =
        ph2d_physics_ecs::rope_route::axle_pair(&w, 0).expect("o par sobrevive à intrusa");
    assert_eq!((first, second), (0, 3));
}

/// **Um eixo, um termo de RECOLHIMENTO** — o retorno não recolhe.
///
/// Comportamental e não por leitura de campo: a ponte não publica a taxa da corda, e
/// o que importa é o que a carga FAZ. Com as duas pontas presas o único lugar em que
/// a corda recolhida cabe é o trecho abraçado, então a subida é `ω·R/(2·peso)` —
/// cinemática pura (o guincho é onipotente, medido).
///
/// ⚠️ Mutação: dar ao retorno o mesmo `reel_rate` da entrada faz a taxa virar
/// `ω(R+r)` em vez de `ω·R` — a mesma volta contada duas vezes —, e a carga sobe
/// **1,75×** mais depressa.
#[test]
fn the_return_contact_does_not_reel() {
    const OMEGA_DEG: f32 = 180.0;
    let omega = OMEGA_DEG.to_radians();
    let mut sim = rig(true, None);
    // As DUAS pontas presas: sem isso o recolhimento se reparte com o contrapeso e a
    // medição fica confundida — a lição que o gate irmão do kernel pagou.
    let haul = entity_of(&mut sim, "Haul");
    if let Some(mut rb) = sim.world_mut().get_mut::<RigidBody>(haul) {
        rb.kind = BodyKind::Static;
    }
    let e = entity_of(&mut sim, "Sheave");
    if let Some(mut pw) = sim.world_mut().get_mut::<PulleyWheel>(e) {
        pw.motor_speed = omega;
    }
    let mut b = PhysicsBridge::new();
    // Deixa o transitório de esticamento passar e mede a TAXA em regime.
    for t in 0..=60 {
        b.dispatch(&mut sim, t > 0, t);
    }
    let y0 = y_of(&mut sim, "Block");
    for t in 61..=120 {
        b.dispatch(&mut sim, true, t);
    }
    let rate = y_of(&mut sim, "Block") - y0;
    // `2 · R/(R−r)` é a sensibilidade do orçamento à altura da carga.
    let expected = omega * R_IN / (2.0 * (R_IN / (R_IN - R_RET)));
    assert!(
        (rate - expected).abs() < 0.1 * expected,
        "a corda recolhe ω·R e a carga sobe isso dividido por 2·peso: \
         esperado {expected:.4} m/s, deu {rate:.4} — somar o retorno daria {:.4}",
        omega * (R_IN + R_RET) / (2.0 * (R_IN / (R_IN - R_RET)))
    );
}

/// **Os dois contatos são UM eixo** — mesma identidade para a ruptura, e o giro
/// desenhado é o MESMO ângulo nos dois anéis.
///
/// ⚠️ Mutação: integrar o giro também no retorno avança o ângulo da entidade duas
/// vezes por tique, e com o raio errado (`v/r` em vez de `v/R`) — dois anéis
/// concêntricos girando em velocidades diferentes, que é o desenho de um eixo que não
/// existe.
#[test]
fn a_compound_axle_is_one_identity_and_one_rotation() {
    let mut sim = rig(true, None);
    let mut b = PhysicsBridge::new();
    b.dispatch(&mut sim, false, 0);
    let w = arena(&b);
    assert_eq!(w[0].id, w[2].id, "um eixo, uma identidade — uma ruptura");
    // Deixa a corda correr, para haver giro a comparar.
    for t in 1..40 {
        b.dispatch(&mut sim, true, t);
    }
    let spins = b.pulley_wheel_spins();
    assert_eq!(spins.len(), 3);
    assert_eq!(
        spins[0], spins[2],
        "os dois anéis do mesmo eixo estão no mesmo ângulo"
    );
    assert_ne!(spins[0], 0.0, "e o eixo de fato girou");
}

/// **A vantagem atravessa a fronteira, e um REWIND a re-arma.**
///
/// 8 kg contra 1 kg é o equilíbrio da vantagem 8 (`2R/(R−r)` com `R = 0,5` e
/// `r = 0,375`), então a carga fica quase parada; sem o par ela DESCE, e essa é a
/// metade que faz o gate morder.
///
/// ⚠️ Mutação: a ponte não anexar o `axle` colapsa a vantagem em 2 e a carga desce
/// nos dois braços.
#[test]
fn the_advantage_crosses_the_bridge_and_a_rewind_re_arms_it() {
    for (weston, expect_still) in [(true, true), (false, false)] {
        let mut sim = rig(weston, None);
        let mut b = PhysicsBridge::new();
        let y0 = {
            b.dispatch(&mut sim, false, 0);
            y_of(&mut sim, "Block")
        };
        for t in 1..=60 {
            b.dispatch(&mut sim, true, t);
        }
        let moved = y_of(&mut sim, "Block") - y0;
        if expect_still {
            assert!(
                moved.abs() < 0.15,
                "com o par de eixo 8 kg equilibram 1 kg; a carga andou {moved:.4}"
            );
        } else {
            assert!(
                moved < -0.15,
                "sem o par a vantagem é 2 e 8 kg DESCEM; a carga andou {moved:.4}"
            );
        }
        // E o rewind: voltar ao tique 0 reconstrói o mundo do repouso AUTORADO, então
        // o par tem de renascer com ele.
        b.dispatch(&mut sim, false, 0);
        let w = arena(&b);
        assert_eq!(
            ph2d_physics_ecs::rope_route::axle_pair(&w, 0).is_some(),
            weston,
            "um rewind re-arma o eixo composto (ou a ausência dele)"
        );
    }
}
/// **As CORDAS sobrevivem a um rewind** — e este gate nasceu VERMELHO sobre um bug
/// PRÉ-EXISTENTE, achado ao afirmar que a Weston sobrevive a um scrub.
///
/// A tabela de polias vive DENTRO do `PhysicsWorld`, e o `rebuild_from_rest` o
/// substituía por um novo sem reinstalá-la — e o laço de replay roda no MESMO
/// chamado. Medido: depois de um Reset a arena voltava **VAZIA**, então um scrub para
/// um tique intermediário replayava **sem corda nenhuma** e a carga caía livre.
///
/// ⚠️ **Ficou calado porque `target == 0` replaya ZERO passos** — o Reset, que é o
/// caso comum e o único que os smokes fazem. É por isso que este gate faz as duas
/// coisas: o Reset (a arena tem de estar de pé) e depois um scrub para o MEIO, com o
/// ring já limpo, onde o replay de fato acontece.
///
/// ⚠️ Mutação: tirar o `swap_pulleys` de volta no `rebuild_from_rest` deixa a arena
/// vazia e a carga cai ~1,2 m em 30 tiques de queda livre.
#[test]
fn the_ropes_survive_a_rewind_and_the_replay_runs_with_them() {
    let mut sim = rig(true, None);
    let mut b = PhysicsBridge::new();
    b.dispatch(&mut sim, false, 0);
    let y_rest = y_of(&mut sim, "Block");
    for t in 1..=60 {
        b.dispatch(&mut sim, true, t);
    }
    // (1) O Reset. Sem passos a replayar, mas a arena TEM de estar de pé — é ela que
    // o próximo scrub vai usar, e é ela que o desenho lê.
    b.dispatch(&mut sim, false, 0);
    let w = arena(&b);
    assert_eq!(w.len(), 3, "o Reset não pode apagar as roldanas da corda");
    assert!(
        w[1].body.is_some(),
        "e o eixo MONTADO tem de apontar um corpo do mundo NOVO, não um handle órfão"
    );
    assert!(
        ph2d_physics_ecs::rope_route::axle_pair(&w, 0).is_some(),
        "o eixo composto renasce com a corda"
    );
    // (2) O scrub para o MEIO, com o ring limpo pelo Reset ⇒ miss ⇒ rebuild + replay
    // de 30 passos. Com a corda de pé a carga fica onde o equilíbrio a deixa; sem
    // ela, 30 tiques de queda livre valem ~1,2 m.
    b.dispatch(&mut sim, false, 30);
    let moved = y_of(&mut sim, "Block") - y_rest;
    assert!(
        moved.abs() < 0.3,
        "o replay do scrub rodou COM as cordas: a carga andou {moved:.4} m \
         (queda livre de 30 tiques valeria ~1,2 m)"
    );
}
