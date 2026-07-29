//! **O ÍMÃ do eixo de uma roldana MONTADA** (W-Pulley W6) — os pontos a que a
//! alça de centro cola, e o controle que mantém a recusa antiga de pé.
//!
//! # O que esta wave corrige
//!
//! O gesto de arrasto trazia uma isenção escrita à mão: *"uma roldana não
//! pertence a corpo nenhum — não há a que colar"*. Era verdade quando foi
//! escrita, e o **W3 a falsificou** — uma roldana montada tem corpo, o eixo dela
//! é `corpo · local`, e aquele corpo tem collider. É a forma exata de *uma
//! condição que enumera seus leitores*: ela enumerava os donos de collider da
//! época, e o dono novo nasceu fora da lista.
//!
//! # O preço, medido antes da correção
//!
//! Sonda `measure_pulley_wheel_snap`: um erro de mão de **0,02 m** ao mirar a
//! quina do bloco assa `local = [0,62, 0,27]` e o eixo carrega **0,0283 m** de
//! desvio para sempre — o bloco anda e leva o erro junto, sem nada corrigi-lo
//! depois. E o desvio é **invisível**: o eixo acompanha o bloco corretamente, só
//! não está na quina. O alcance do ímã (14 px) vale 0,052 m a `height_world` 4 e
//! 0,207 m a 16, então o erro real cai DENTRO dele.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics::ShapeDesc;
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, JointSide, PhysicsBridge, PhysicsJoint,
    PulleyWheel, RigidBody,
};

/// Meias-extensões do bloco que carrega o eixo, e a pose dele — declaradas aqui
/// porque os pontos esperados saem DESTA geometria, nunca de uma chamada ao
/// produto: um oráculo que usa a função sob teste para computar o que espera é
/// sempre verde.
const HALF: [f32; 2] = [0.60, 0.25];
const AT: [f32; 2] = [0.0, 2.0];

/// Um bloco RETANGULAR com uma roldana montada nele (a *cadernal móvel*), as duas
/// pontas da corda, e um Pin comum entre o bloco e a carga.
///
/// ⚠️ **A caixa é deliberada:** é a única forma com **nove** pontos — quinas E
/// meios de aresta —, e é onde um ímã tem o que fazer (numa bola os cardinais
/// estão visíveis no contorno; numa caixa a quina é o ponto que a mão não
/// acerta). O Pin existe para o gate da porta COMPARTILHADA: ele dá ao MESMO
/// corpo um segundo perguntador.
fn rig(mount: &str) -> SimWorld {
    let mut sim = SimWorld::new();
    let mut ball = |name: &str, x: f32, y: f32, kind: BodyKind| {
        sim.world_mut().spawn((
            Name::new(name),
            RigidBody { kind },
            Collider {
                shape: ColliderShape::Ball { radius: 0.2 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, y)),
        ));
    };
    ball("Dead", -1.0, 6.0, BodyKind::Static);
    ball("Haul", 1.5, 4.0, BodyKind::Dynamic);
    sim.world_mut().spawn((
        Name::new("Block"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: HALF[0],
                half_y: HALF[1],
            },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(AT[0], AT[1])),
    ));
    sim.world_mut().spawn((
        Name::new("Rope"),
        PhysicsJoint {
            body_a: stable_name_id("Dead"),
            body_b: stable_name_id("Haul"),
            kind: JointKind::Pulley,
            ..PhysicsJoint::of_kind(JointKind::Pulley)
        },
        Transform::from_translation(Vec2::new(-1.0, 6.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Link"),
        PhysicsJoint {
            body_a: stable_name_id("Block"),
            body_b: stable_name_id("Haul"),
            ..PhysicsJoint::default()
        },
        Transform::from_translation(Vec2::new(AT[0], AT[1])),
    ));
    sim.world_mut().spawn((
        Name::new("Rope Wheel 1"),
        PulleyWheel {
            rope: stable_name_id("Rope"),
            order: 0,
            radius: 0.3,
            body: stable_name_id(mount),
            ..Default::default()
        },
        Transform::from_translation(Vec2::new(0.25, 2.15)),
    ));
    sim
}

fn named(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entidade viva")
}

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    (b[0] - a[0]).hypot(b[1] - a[1])
}

/// Existe candidato a menos de 1e-4 de `p`?
fn offers(cands: &[[f32; 2]], p: [f32; 2]) -> bool {
    cands.iter().any(|c| dist(*c, p) < 1.0e-4)
}

/// **O eixo de uma roldana montada pode mirar os nove pontos do bloco.**
///
/// Este é o gate que nasceu VERMELHO: antes da wave a porta não existia, e o
/// gesto de arrasto pulava o ímã por um ramo `if drag.kind.is_wheel()`.
///
/// Mutação: fazer a porta devolver `0` para uma roldana com corpo (o ramo antigo)
/// derruba as nove asserções de uma vez.
#[test]
fn the_axle_of_a_mounted_wheel_can_aim_at_the_blocks_nine_points() {
    let mut sim = rig("Block");
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let wheel = named(&mut sim, "Rope Wheel 1");

    let mut out = [[0.0f32; 2]; ShapeDesc::MAX_SNAP_POINTS];
    let n = bridge.wheel_snap_targets(&sim, wheel, &mut out);
    assert_eq!(
        n, 9,
        "uma caixa oferece nove pontos, e a porta devolveu {n}"
    );

    let (hx, hy) = (HALF[0], HALF[1]);
    let (cx, cy) = (AT[0], AT[1]);
    for (what, p) in [
        ("o centro", [cx, cy]),
        ("a quina de cima à esquerda", [cx - hx, cy + hy]),
        ("a quina de cima à direita", [cx + hx, cy + hy]),
        ("a quina de baixo à esquerda", [cx - hx, cy - hy]),
        ("a quina de baixo à direita", [cx + hx, cy - hy]),
        ("o meio do topo", [cx, cy + hy]),
        ("o meio da direita", [cx + hx, cy]),
        ("o meio da base", [cx, cy - hy]),
        ("o meio da esquerda", [cx - hx, cy]),
    ] {
        assert!(
            offers(&out[..n], p),
            "o ímã tinha de oferecer {what} ({p:?}); os candidatos são {:?}",
            &out[..n]
        );
    }
}

/// **O CONTROLE — uma roldana de CENÁRIO não tem nada a que colar**, e a isenção
/// que o gesto trazia continua correta *para ela*.
///
/// ⚠️ **A recusa passou a ser aritmética em vez de um ramo:** a porta devolve
/// zero, e `nearest_within` sobre uma fatia vazia é `None`. Um ramo
/// `if is_wheel()` no chamador é justamente o que apodreceu — ele enumerava quem
/// tinha collider.
#[test]
fn a_scenery_wheel_offers_nothing_to_snap_to() {
    let mut sim = rig("Não Existe");
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let wheel = named(&mut sim, "Rope Wheel 1");
    let mut out = [[0.0f32; 2]; ShapeDesc::MAX_SNAP_POINTS];
    assert_eq!(
        bridge.wheel_snap_targets(&sim, wheel, &mut out),
        0,
        "uma roldana sem corpo resolvido não tem collider — logo não tem candidato"
    );

    // E uma entidade que não é roldana nenhuma também não: a porta é perguntada
    // pelo gesto, e um gesto pode terminar sobre um alvo que morreu.
    let block = named(&mut sim, "Block");
    assert_eq!(
        bridge.wheel_snap_targets(&sim, block, &mut out),
        0,
        "sem `PulleyWheel` não há eixo a colar"
    );
}

/// **O eixo e a âncora de um joint colam nos MESMOS pontos do MESMO corpo.**
///
/// A colocação é uma função só (`body_snap_targets`), e este gate é o que a
/// mantém assim: duas cópias colariam o pino e o eixo em pontos diferentes do
/// mesmo collider, e **nada na tela diria qual dos dois está errado**.
///
/// Mutação: dar à porta da roldana a sua própria colocação (esquecendo o offset
/// do collider, digamos) derruba isto sem tocar em nenhum gate do joint.
#[test]
fn the_axle_and_a_joint_anchor_snap_to_the_same_points_of_the_same_body() {
    let mut sim = rig("Block");
    // Um offset e uma rotação, para o gate ver a colocação inteira em vez de
    // um caso onde `local` e mundo coincidem.
    let block = named(&mut sim, "Block");
    {
        let mut t = sim.world_mut().get_mut::<Transform>(block).expect("t");
        t.rotation = 0.4;
    }
    {
        let mut c = sim.world_mut().get_mut::<Collider>(block).expect("c");
        c.offset = [0.1, -0.2];
    }
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);

    let wheel = named(&mut sim, "Rope Wheel 1");
    let link = named(&mut sim, "Link");
    let mut from_wheel = [[0.0f32; 2]; ShapeDesc::MAX_SNAP_POINTS];
    let mut from_joint = [[0.0f32; 2]; ShapeDesc::MAX_SNAP_POINTS];
    let nw = bridge.wheel_snap_targets(&sim, wheel, &mut from_wheel);
    // O `Link` tem o bloco na ponta A.
    let nj = bridge.joint_snap_targets(&sim, link, JointSide::A, &mut from_joint);
    assert_eq!(nw, nj, "a mesma caixa tem de oferecer a mesma contagem");
    assert!(nw > 0, "a fixture tem de conter o fenômeno");
    for i in 0..nw {
        assert!(
            dist(from_wheel[i], from_joint[i]) < 1.0e-6,
            "o ponto {i} divergiu: o eixo vê {:?} e a âncora vê {:?}",
            from_wheel[i],
            from_joint[i]
        );
    }
}

/// **Os candidatos seguem o collider RESOLVIDO** — a forma escalada (W6) no
/// offset autorado (W-Offset), que é a que o solver colide e a que o contorno
/// desenha.
///
/// Sem isto o ímã colaria o eixo onde a caixa foi *digitada* em vez de onde ela
/// está — e o artista veria a roda pousar fora do contorno que ele está olhando.
/// ⚠️ **O OFFSET entra nesta fixture de propósito.** A 1ª versão desta suíte
/// deixava os dois números em zero, e a mutação que apaga o offset da colocação
/// compartilhada **passava aqui inteira** — só o gate irmão do joint a pegava.
/// Uma colocação partilhada precisa de um gate independente em CADA lado, senão o
/// lado que não a mede vai junto em silêncio quando alguém lhe der uma cópia.
#[test]
fn the_snap_targets_follow_the_scaled_and_offset_collider() {
    let mut sim = rig("Block");
    let block = named(&mut sim, "Block");
    {
        let mut t = sim.world_mut().get_mut::<Transform>(block).expect("t");
        t.scale = Vec2::new(2.0, 3.0);
    }
    {
        let mut c = sim.world_mut().get_mut::<Collider>(block).expect("c");
        c.offset = [0.15, -0.35];
    }
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let wheel = named(&mut sim, "Rope Wheel 1");
    let mut out = [[0.0f32; 2]; ShapeDesc::MAX_SNAP_POINTS];
    let n = bridge.wheel_snap_targets(&sim, wheel, &mut out);
    assert_eq!(n, 9, "uma caixa escalada segue sendo uma caixa");
    // ⚠️ **O offset também escala** — ele é uma POSIÇÃO no corpo (W-Offset), e o
    // `scaled_shape` o multiplica pela escala junto com a forma.
    let (ox, oy) = (0.15 * 2.0, -0.35 * 3.0);
    // O centro do collider é o corpo mais o offset resolvido, e a quina de cima à
    // direita é ele mais a meia-extensão escalada.
    let centre = [AT[0] + ox, AT[1] + oy];
    assert!(
        offers(&out[..n], centre),
        "o centro do collider tinha de estar em {centre:?} (offset resolvido); \
         candidatos {:?}",
        &out[..n]
    );
    let want = [centre[0] + HALF[0] * 2.0, centre[1] + HALF[1] * 3.0];
    assert!(
        offers(&out[..n], want),
        "a quina tinha de estar em {want:?} (caixa 2x3 no offset); candidatos {:?}",
        &out[..n]
    );
}
