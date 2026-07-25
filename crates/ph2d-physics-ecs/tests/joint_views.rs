//! **O READOUT de desenho de um joint lê o SOLVER, nunca o componente** (W-J1).
//!
//! O overlay passou a desenhar tudo que um joint é — o tipo, o alcance do
//! limite, o comprimento de repouso, a folga, a deformação, de quem é cada
//! ponta — e a única pergunta arquitetural dessa wave é *de onde saem esses
//! números*. A resposta é a mesma que o `scaled_shape` (W6) já impõe ao
//! contorno do collider: **do que o solver de fato recebeu**.
//!
//! Existem duas fontes possíveis e elas DIVERGEM num caso real: o componente
//! ECS existe para todo joint que o artista autorou, e a ponte só carrega os
//! que conseguiu construir. Um joint cujos corpos não resolvem (um nome
//! renomeado — a exposição que o `stable_name_id` documenta desde o W3) segue
//! autorado e **não está no solver**; desenhá-lo do componente pintaria uma
//! relação que nada está impondo, e o artista veria uma corrente inteira
//! ligada por um elo que não existe.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, RigidBody,
};

/// Um gancho estático, uma prancha dinâmica à direita, presa na ponta ESQUERDA
/// dela — a mesma armação dos gates de âncora, para que "onde na prancha" seja
/// uma afirmação com dentes.
fn pendulum(kind: JointKind) -> SimWorld {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Hook"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.05 },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 6.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Plank"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.5,
                half_y: 0.1,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.5, 5.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Pin"),
        PhysicsJoint {
            body_a: stable_name_id("Hook"),
            body_b: stable_name_id("Plank"),
            kind,
            ..PhysicsJoint::default()
        },
        Transform::from_translation(Vec2::new(0.0, 5.0)),
    ));
    sim
}

fn named(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entity exists")
}

/// **Um joint que o solver NÃO tem não é desenhado** — a porta única em forma
/// observável.
///
/// Renomear um corpo desacopla os joints dele (a exposição que o W3 pinou), e o
/// componente continua lá, autorado, com todos os parâmetros. Mutação: iterar
/// os componentes do ECS em vez do `self.joints` da ponte ⇒ a view do joint
/// morto reaparece e este gate fica VERMELHO.
#[test]
fn a_joint_the_solver_does_not_hold_is_not_drawn() {
    let mut sim = pendulum(JointKind::Pin);
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    assert_eq!(
        bridge.joint_views().count(),
        1,
        "o joint autorado e construído não produziu view — o overlay não teria \
         o que desenhar"
    );

    // O corpo B muda de nome: o joint fica órfão no solver e INTACTO no ECS.
    let plank = named(&mut sim, "Plank");
    *sim.world_mut().get_mut::<Name>(plank).expect("name") = Name::new("Plank2");
    bridge.dispatch(&mut sim, false, 0);

    let joint = named(&mut sim, "Pin");
    assert!(
        sim.world().get::<PhysicsJoint>(joint).is_some(),
        "premissa do gate: o componente autorado sobrevive ao rename"
    );
    assert_eq!(
        bridge.joint_views().count(),
        0,
        "um joint que o solver não conseguiu construir AINDA foi publicado \
         para desenho: o canvas mostraria uma relação que nada está impondo, e \
         o artista veria um elo que não existe"
    );
}

/// **Os parâmetros da view são os que o SOLVER recebeu — e um parâmetro que o
/// tipo ignora não é publicado.**
///
/// A mesma pergunta que `joint_desc` faz antes de entregar ao rapier: um limite
/// sobrado de um tipo anterior não pode ser desenhado como se ainda estivesse
/// em vigor. Sem isto, trocar Pin→Spring deixaria um arco de limite na tela
/// sobre uma mola que não tem limite nenhum.
#[test]
fn the_view_publishes_what_the_solver_was_given_and_nothing_the_kind_ignores() {
    let mut sim = pendulum(JointKind::Pin);
    let joint = named(&mut sim, "Pin");
    {
        let mut j = sim.world_mut().get_mut::<PhysicsJoint>(joint).expect("j");
        j.limits_enabled = true;
        j.limit_min = -0.5;
        j.limit_max = 0.75;
        j.motor_enabled = true;
        j.motor_speed = 2.0;
    }
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);

    let v = bridge.joint_views().next().expect("one view");
    assert_eq!(v.kind, JointKind::Pin);
    assert_eq!(
        v.limits,
        Some([-0.5, 0.75]),
        "o alcance autorado não chegou ao desenho: o arco desenharia outro \
         limite, ou nenhum"
    );
    assert_eq!(v.motor_speed, Some(2.0), "o motor não chegou ao desenho");
    assert_eq!(
        v.length, None,
        "um Pin publicou comprimento: o anel de repouso apareceria em volta de \
         uma dobradiça, que não tem comprimento nenhum"
    );

    // O MESMO joint vira mola: os campos do pino somem, o comprimento aparece.
    {
        let mut j = sim.world_mut().get_mut::<PhysicsJoint>(joint).expect("j");
        j.kind = JointKind::Spring;
        j.rest_length = 1.5;
    }
    bridge.dispatch(&mut sim, false, 0);
    let v = bridge.joint_views().next().expect("one view");
    assert_eq!(v.kind, JointKind::Spring);
    assert_eq!(
        (v.limits, v.motor_speed),
        (None, None),
        "os parâmetros do PINO sobreviveram à troca de tipo: o desenho \
         mostraria paredes e um motor que o solver ignora"
    );
    assert_eq!(
        v.length,
        Some(1.5),
        "o comprimento de repouso não chegou ao desenho: o anel — o número do \
         §12 virando lugar — não existiria"
    );

    // E a corda nomeia o comprimento MÁXIMO pelo mesmo campo, porque a pergunta
    // que ele responde ao desenho ("que raio tem o anel?") é uma só.
    {
        let mut j = sim.world_mut().get_mut::<PhysicsJoint>(joint).expect("j");
        j.kind = JointKind::Rope;
        j.max_length = 2.25;
    }
    bridge.dispatch(&mut sim, false, 0);
    let v = bridge.joint_views().next().expect("one view");
    assert_eq!((v.kind, v.length), (JointKind::Rope, Some(2.25)));
}

/// **As âncoras e as poses são VIVAS** — o desenho segue a corrente que balança,
/// não o lugar onde o artista largou o pivô.
///
/// É a razão pela qual o overlay nunca leu o `Transform` do joint (W3): ele é a
/// âncora AUTORADA e nada o reescreve durante o play. Mutação: publicar o
/// `Transform` no lugar da leitura do solver ⇒ os números param no tick 0.
#[test]
fn the_anchors_and_poses_are_live() {
    let mut sim = pendulum(JointKind::Pin);
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let at_rest = bridge.joint_views().next().expect("view");

    // Deixa o pêndulo balançar: o corpo B gira e translada.
    for t in 1..=40 {
        bridge.dispatch(&mut sim, false, t);
    }
    let swung = bridge.joint_views().next().expect("view");

    let moved =
        (swung.centre_b[0] - at_rest.centre_b[0]).hypot(swung.centre_b[1] - at_rest.centre_b[1]);
    assert!(
        moved > 0.05,
        "o centro do corpo B andou {moved:.4} m em 40 ticks de queda: a view \
         está congelada no estado autorado, e o desenho ficaria onde o artista \
         largou o pivô enquanto a corrente balança embora"
    );
    assert!(
        (swung.angle_b - at_rest.angle_b).abs() > 0.01,
        "a rotação viva do corpo B não mudou — a agulha do arco de limite \
         apontaria sempre para o mesmo lugar"
    );
    // A âncora do lado estático não anda: é o controle que separa "a view é
    // viva" de "a view é ruído".
    let anchor_drift =
        (swung.anchor_a[0] - at_rest.anchor_a[0]).hypot(swung.anchor_a[1] - at_rest.anchor_a[1]);
    assert!(
        anchor_drift < 1e-3,
        "a âncora no corpo ESTÁTICO andou {anchor_drift:.4} m — nada a moveu"
    );
}

/// **Dois corpos KINEMATIC afastados pela animação abrem o vão** — o único
/// caminho MEDIDO até a marca vermelha, e a razão dela existir.
///
/// Os impulse joints do rapier são rígidos: carga não os estica (medido — um
/// pino segurando 500× a massa, e outro levando um martelo de 400×, abriram
/// **0,00000 m**). Mas um joint **não move um corpo kinematic** (massa
/// infinita), então dois corpos curva-dirigidos que a animação separa ficam
/// separados, com o pino desenhado por cima como se ainda os prendesse. É
/// exatamente o estado em que o W-BakeJoint deixa um rig assado, e é o que a
/// marca torna visível.
#[test]
fn two_curve_driven_bodies_pulled_apart_break_the_constraint_visibly() {
    let mut sim = SimWorld::new();
    for (name, x) in [("A", 0.0f32), ("B", 0.0)] {
        sim.world_mut().spawn((
            Name::new(name.to_string()),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.2 },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, 5.0)),
        ));
    }
    sim.world_mut().spawn((
        Name::new("P".to_string()),
        PhysicsJoint {
            body_a: stable_name_id("A"),
            body_b: stable_name_id("B"),
            kind: JointKind::Pin,
            ..PhysicsJoint::default()
        },
        Transform::from_translation(Vec2::new(0.0, 5.0)),
    ));
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);

    // A "curva" afasta B — o que uma track assada faz todo frame.
    let b = named(&mut sim, "B");
    sim.world_mut()
        .get_mut::<Transform>(b)
        .expect("t")
        .translation
        .x = 1.5;
    for t in 1..=30 {
        bridge.dispatch(&mut sim, false, t);
    }

    let v = bridge.joint_views().next().expect("view");
    let gap = (v.anchor_b[0] - v.anchor_a[0]).hypot(v.anchor_b[1] - v.anchor_a[1]);
    assert!(
        gap > 1.0,
        "os dois corpos foram afastados 1,5 m e as âncoras do pino ficaram a \
         {gap:.4} m: o overlay desenharia um pino perfeitamente normal sobre \
         dois objetos que já não estão presos um ao outro"
    );
}

/// **Um pino em repouso tem as duas âncoras no MESMO ponto** — e é isso que faz
/// da separação uma medida útil em vez de ruído.
///
/// O overlay pinta o vão em vermelho a partir de 1 px de TELA; este gate mede o
/// resíduo do solver assentado, que é o número que aquele limiar precisa
/// superar. Medido: 0,0000 m.
#[test]
fn a_resting_pin_has_no_gap_between_its_anchors() {
    let mut sim = pendulum(JointKind::Pin);
    let mut bridge = PhysicsBridge::new();
    for t in 0..=90 {
        bridge.dispatch(&mut sim, false, t);
    }
    let v = bridge.joint_views().next().expect("view");
    let gap = (v.anchor_b[0] - v.anchor_a[0]).hypot(v.anchor_b[1] - v.anchor_a[1]);
    assert!(
        gap < 1e-3,
        "um pino assentado abriu {gap:.5} m entre as âncoras — a marca \
         vermelha de deformação estaria acesa o tempo todo, e um alarme que \
         sempre toca não é lido"
    );
}
