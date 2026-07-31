//! O gesto de desenhar um pino de MUNDO no canvas (W-JointWorld).

use super::inspector_joint_world::create_world_pin_at;
use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, JointWorldAnchor, PhysicsBridge, PhysicsJoint,
    RigidBody,
};

fn one_body() -> (SimWorld, ph2d_ecs::Entity) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Name::new("Lamp"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.2 },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 5.0)),
        ))
        .id();
    (sim, e)
}

/// **Soltar no vazio cria um pino de MUNDO, e ele SEGURA.**
///
/// O relato do smoke: *"não aceita desenhar a junta a partir do canvas vazio,
/// apenas de um objeto para outro"*. Este gate é esse gesto.
#[test]
fn releasing_on_empty_canvas_pins_the_body_to_the_world() {
    let (mut sim, body) = one_body();
    let joint = create_world_pin_at(
        &mut sim,
        body.to_bits(),
        JointKind::Pin,
        [0.0, 5.0],
        [0.0, 7.0],
    )
    .expect("o gesto tinha de criar o pino");
    assert!(
        sim.world().get::<JointWorldAnchor>(joint).is_some(),
        "sem o marcador este joint é meio-autorado, e o corpo cai"
    );
    let mut bridge = PhysicsBridge::new();
    for t in 1..=120 {
        bridge.dispatch(&mut sim, true, t);
    }
    let y = {
        let mut q = sim.world_mut().query::<(&Name, &Transform)>();
        q.iter(sim.world())
            .find(|(n, _)| n.as_str() == "Lamp")
            .map(|(_, t)| t.translation.y)
            .expect("corpo vivo")
    };
    assert!(
        y > 4.0,
        "a lâmpada tinha de ficar pendurada; ela caiu para {y:.4}"
    );
}

/// **Um Pin nasce SEM TRANCO** — o pivô é onde o artista soltou, e o corpo não
/// é arrancado para lá.
///
/// ⚠️ É a razão de a política de âncora ser copiada do irmão de dois corpos em
/// vez de simplificada: num tipo que COMPARTILHA um ponto os dois lados são o
/// MESMO lugar, então ancorar A no ponto do *press* faria o solver puxar o corpo
/// até o ponto do *release* no primeiro passo.
#[test]
fn a_world_pin_does_not_yank_the_body_on_creation() {
    let (mut sim, body) = one_body();
    create_world_pin_at(
        &mut sim,
        body.to_bits(),
        JointKind::Pin,
        [0.1, 5.1],
        [0.0, 7.0],
    )
    .expect("pino");
    let mut bridge = PhysicsBridge::new();
    // Dois tiques só: o tranco, se houver, acontece no primeiro.
    for t in 1..=2 {
        bridge.dispatch(&mut sim, true, t);
    }
    let p = {
        let mut q = sim.world_mut().query::<(&Name, &Transform)>();
        q.iter(sim.world())
            .find(|(n, _)| n.as_str() == "Lamp")
            .map(|(_, t)| [t.translation.x, t.translation.y])
            .expect("corpo vivo")
    };
    assert!(
        (p[0] - 0.0).abs() < 0.1 && (p[1] - 5.0).abs() < 0.1,
        "o corpo foi ARRANCADO para o ponto do release: está em {p:?}, e nasceu em [0, 5]"
    );
}

/// **A POLIA é recusada no GESTO**, não só no reconcile — senão o artista cria
/// um objeto que nasce dormente e não diz por quê.
#[test]
fn a_pulley_cannot_be_pinned_to_the_world() {
    let (mut sim, body) = one_body();
    assert!(
        create_world_pin_at(
            &mut sim,
            body.to_bits(),
            JointKind::Pulley,
            [0.0, 5.0],
            [0.0, 7.0]
        )
        .is_none(),
        "uma corda puxa as DUAS pontas — uma delas no cenário é outra máquina"
    );
    let mut q = sim.world_mut().query::<&PhysicsJoint>();
    assert_eq!(
        q.iter(sim.world()).count(),
        0,
        "a recusa não pode deixar um joint para trás"
    );
}

/// **O release no VAZIO chega à porta do pino de mundo** — arch-gate.
///
/// ⚠️ `joint_draw_release` exige `gfx` (janela + GPU), então **nenhum teste de
/// unidade o alcança**: a decisão de rotear o vazio mora lá dentro, e sem este
/// gate os três gates acima ficariam verdes sobre um gesto que continua
/// recusando — que é exatamente o defeito que o smoke reportou.
#[test]
fn the_canvas_release_on_empty_routes_to_the_world_pin() {
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/joint_draw.rs"))
        .expect("joint_draw.rs");
    assert!(
        src.contains("inspector_joint_world::create_world_pin_at"),
        "o release do canvas não chama a porta do pino de mundo — soltar no vazio \
         continua sendo uma recusa"
    );
    // E a recusa que ELE substituiu não pode voltar a existir.
    assert!(
        !src.contains("world pins are not a thing yet"),
        "a recusa antiga ainda está na tela, e ela agora é FALSA"
    );
    // ⚠️ Controle positivo: se este arquivo deixar de ser o do gesto, as duas
    // asserções acima viram vácuo — elas só sabem dizer o que NÃO está lá.
    assert!(
        src.contains("fn joint_draw_release"),
        "este gate está lendo o arquivo errado"
    );
}
