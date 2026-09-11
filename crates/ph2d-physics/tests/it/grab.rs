//! **A MÃO** (W-Grab) — segurar um corpo enquanto a sim corre.
//!
//! Cada gate é uma frase que o artista pode ver acontecer, e cada um carrega o
//! próprio CONTROLE: *"o corpo seguiu o cursor"* é satisfeito igualmente por
//! *"o corpo foi teleportado"*, então toda afirmação sobre a mola vem ao lado de
//! uma afirmação sobre o que ela ainda tem de respeitar.

use ph2d_physics::{BodyDesc, JointDesc, JointKind, PhysicsWorld, RigidBodyType, ShapeDesc};

/// Um corpo com o preenchimento mínimo — só o que cada gate precisa varia.
fn body(
    world: &mut PhysicsWorld,
    kind: RigidBodyType,
    x: f32,
    y: f32,
    shape: ShapeDesc,
    density: f32,
) -> ph2d_physics::RigidBodyHandle {
    world.spawn_body(BodyDesc {
        body_type: kind,
        x,
        y,
        rotation: 0.0,
        density,
        shape,
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
        offset: [0.0, 0.0],
    })
}

fn pose(world: &PhysicsWorld, h: ph2d_physics::RigidBodyHandle) -> [f32; 2] {
    let p = world.body_pose(h).expect("body alive");
    [p.translation.x, p.translation.y]
}

/// Mundo sem gravidade com uma bola na origem — o palco de quase todo gate aqui,
/// porque o que se mede é o SEGUIDOR e não a queda.
fn floating_ball(density: f32) -> (PhysicsWorld, ph2d_physics::RigidBodyHandle) {
    let mut w = PhysicsWorld::new();
    w.set_gravity(0.0, 0.0);
    let ball = body(
        &mut w,
        RigidBodyType::Dynamic,
        0.0,
        0.0,
        ShapeDesc::Ball { radius: 0.3 },
        density,
    );
    (w, ball)
}

/// Segura o corpo e leva a mão até `to`, dando `ticks` passos.
fn drag_to(
    w: &mut PhysicsWorld,
    h: ph2d_physics::RigidBodyHandle,
    from: [f32; 2],
    to: [f32; 2],
    ticks: u32,
) {
    assert!(w.grab_body(h, from), "a mão tinha de pegar");
    for i in 1..=ticks {
        let t = f32::from(u16::try_from(i).unwrap()) / f32::from(u16::try_from(ticks).unwrap());
        w.move_grab([
            from[0] + (to[0] - from[0]) * t,
            from[1] + (to[1] - from[1]) * t,
        ]);
        w.step();
    }
}

#[test]
fn the_hand_carries_the_body_to_the_cursor() {
    let (mut w, ball) = floating_ball(1.0);
    drag_to(&mut w, ball, [0.0, 0.0], [3.0, 1.0], 30);
    // E depois a mão PARA: um seguidor amortecido assenta no cursor.
    for _ in 0..60 {
        w.move_grab([3.0, 1.0]);
        w.step();
    }
    let p = pose(&w, ball);
    let err = ((p[0] - 3.0).powi(2) + (p[1] - 1.0).powi(2)).sqrt();
    assert!(
        err < 0.01,
        "o corpo tinha de assentar no cursor, err={err:.4}"
    );
    // O CONTROLE: sem mão nenhuma, a mesma bola não vai a lugar nenhum.
    let (mut w2, ball2) = floating_ball(1.0);
    for _ in 0..90 {
        w2.step();
    }
    assert_eq!(pose(&w2, ball2), [0.0, 0.0], "sem mão, nada se move");
}

/// **A MEDIÇÃO que justifica o `MotorModel::AccelerationBased`.**
///
/// A mão é uma FERRAMENTA: o artista não quer lutar contra a massa para
/// reposicionar um caixote, então 1 kg e 25 kg têm de seguir IGUAL. A mola do
/// ARTISTA (`JointKind::Spring`, `ForceBased`) é uma mola FÍSICA e faz o
/// oposto, o que é o correto para ela.
///
/// Duas leis, duas coisas, e é este gate que impede alguém de "simplificar"
/// unificando as duas.
///
/// ⚠️ **O oráculo é a TRAJETÓRIA, não um endpoint, e a 1ª versão errou por
/// isso:** eu afirmei *"o pesado anda MENOS"* e medi 1,99 (leve) contra 2,52
/// (pesado) — o pesado tinha andado **mais**, porque com ganhos force-based a
/// razão `d/m` cai com a massa e o corpo de 25 kg fica **sub-amortecido**: ele
/// passa do alvo e volta. "Mass-dependent" não quer dizer *mais lento*, quer
/// dizer *outra trajetória*; então é a divergência máxima entre as duas curvas
/// que se mede.
#[test]
fn the_hand_is_mass_independent_and_the_artists_spring_is_not() {
    // A trajetória de meio segundo, com a mão parada em 2 m.
    let hand = |density: f32| {
        let (mut w, ball) = floating_ball(density);
        assert!(w.grab_body(ball, [0.0, 0.0]));
        (0..60)
            .map(|_| {
                w.move_grab([2.0, 0.0]);
                w.step();
                pose(&w, ball)[0]
            })
            .collect::<Vec<_>>()
    };
    // O mesmo, pela mola do ARTISTA para uma âncora estática em 2 m.
    let spring = |density: f32| {
        let (mut w, ball) = floating_ball(density);
        let anchor = body(
            &mut w,
            RigidBodyType::Fixed,
            2.0,
            0.0,
            ShapeDesc::Ball { radius: 0.05 },
            1.0,
        );
        w.spawn_joint(
            anchor,
            ball,
            JointDesc {
                kind: JointKind::Spring,
                rest_length: 0.0,
                stiffness: PhysicsWorld::GRAB_STIFFNESS,
                damping: PhysicsWorld::GRAB_DAMPING,
                ..JointDesc::default()
            },
        )
        .expect("joint");
        (0..60)
            .map(|_| {
                w.step();
                pose(&w, ball)[0]
            })
            .collect::<Vec<_>>()
    };
    let divergence = |a: &[f32], b: &[f32]| {
        a.iter()
            .zip(b)
            .map(|(x, y)| (x - y).abs())
            .fold(0.0_f32, f32::max)
    };
    let hand_div = divergence(&hand(1.0), &hand(25.0));
    assert!(
        hand_div < 0.01,
        "a MÃO tem de ser mass-independent: divergência máx = {hand_div:.4} m"
    );
    let spring_div = divergence(&spring(1.0), &spring(25.0));
    assert!(
        spring_div > 0.5,
        "a mola do ARTISTA é física e depende da massa: divergência máx = {spring_div:.4} m"
    );
}

/// O que separa a mão de um teleporte: ela entra pelo SOLVER, então o contato
/// ainda vale. Com os ganhos que shipam, a penetração medida sob um puxão de 5 m
/// para dentro da parede é ~5 mm (a tabela vive em `GRAB_STIFFNESS`).
#[test]
fn the_hand_cannot_drag_a_body_through_a_wall() {
    let mut w = PhysicsWorld::new();
    w.set_gravity(0.0, 0.0);
    // Parede de x=1 a x=3.
    w.add_static_cuboid(2.0, 0.0, 1.0, 2.0);
    let ball = body(
        &mut w,
        RigidBodyType::Dynamic,
        0.5,
        0.0,
        ShapeDesc::Ball { radius: 0.5 },
        1.0,
    );
    assert!(w.grab_body(ball, [0.5, 0.0]));
    for _ in 0..30 {
        w.move_grab([6.0, 0.0]);
        w.step();
    }
    let x = pose(&w, ball)[0];
    assert!(
        x < 0.6,
        "a parede tinha de segurar; a bola parou em x={x:.3}"
    );
    // O CONTROLE: sem parede, o MESMO puxão leva a bola para longe — senão este
    // gate ficaria verde sobre uma mão que não puxa nada.
    let (mut w2, ball2) = floating_ball(1.0);
    assert!(w2.grab_body(ball2, [0.0, 0.0]));
    for _ in 0..30 {
        w2.move_grab([6.0, 0.0]);
        w2.step();
    }
    assert!(
        pose(&w2, ball2)[0] > 3.0,
        "sem parede a mão leva a bola: x={:.3}",
        pose(&w2, ball2)[0]
    );
}

/// Soltar não zera a velocidade — é isso que faz de soltar em movimento um
/// ARREMESSO, e é metade da razão de existir do gesto.
#[test]
fn letting_go_keeps_the_velocity_so_you_can_throw() {
    let (mut w, ball) = floating_ball(1.0);
    drag_to(&mut w, ball, [0.0, 0.0], [4.0, 0.0], 30);
    let at_release = pose(&w, ball)[0];
    w.release_grab();
    for _ in 0..30 {
        w.step();
    }
    let after = pose(&w, ball)[0];
    assert!(
        after > at_release + 1.0,
        "o corpo tinha de continuar viajando: soltou em {at_release:.3}, parou em {after:.3}"
    );
    // O CONTROLE: pegar e soltar SEM mover não arremessa nada.
    let (mut w2, ball2) = floating_ball(1.0);
    assert!(w2.grab_body(ball2, [0.0, 0.0]));
    w2.step();
    w2.release_grab();
    for _ in 0..30 {
        w2.step();
    }
    assert!(
        pose(&w2, ball2)[0].abs() < 1e-4,
        "sem gesto não há arremesso: x={:.6}",
        pose(&w2, ball2)[0]
    );
}

/// Um corpo adormecido não é integrado, então uma mão que não acorda pareceria
/// quebrada exactamente nos corpos que já assentaram — que são a maioria dos que
/// se quer cutucar.
#[test]
fn a_sleeping_body_wakes_in_the_hand() {
    let (mut w, ball) = floating_ball(1.0);
    // Sem gravidade e sem velocidade, o rapier o adormece.
    for _ in 0..300 {
        w.step();
    }
    assert!(
        w.bodies().get(ball).expect("alive").is_sleeping(),
        "a fixture precisa CONTER o fenômeno: a bola tinha de estar dormindo"
    );
    drag_to(&mut w, ball, [0.0, 0.0], [2.0, 0.0], 30);
    assert!(
        pose(&w, ball)[0] > 1.0,
        "a mão tinha de acordar e carregar a bola: x={:.3}",
        pose(&w, ball)[0]
    );
}

/// Um joint não move corpo estático nem kinematic (massa infinita), então
/// oferecer a mão ali seria um gesto morto — a recusa é o que deixa o chamador
/// seguir o caminho de sempre.
#[test]
fn the_hand_refuses_a_static_or_a_kinematic_body() {
    let mut w = PhysicsWorld::new();
    let stat = body(
        &mut w,
        RigidBodyType::Fixed,
        0.0,
        0.0,
        ShapeDesc::Cuboid {
            half_x: 1.0,
            half_y: 1.0,
        },
        1.0,
    );
    let kin = body(
        &mut w,
        RigidBodyType::KinematicPositionBased,
        3.0,
        0.0,
        ShapeDesc::Cuboid {
            half_x: 1.0,
            half_y: 1.0,
        },
        1.0,
    );
    let dyn_ = body(
        &mut w,
        RigidBodyType::Dynamic,
        6.0,
        0.0,
        ShapeDesc::Ball { radius: 0.5 },
        1.0,
    );
    assert!(!w.grab_body(stat, [0.0, 0.0]), "estático recusa");
    assert!(w.grabbed_body().is_none(), "nada ficou pego");
    assert!(!w.grab_body(kin, [3.0, 0.0]), "kinematic recusa");
    assert!(w.grabbed_body().is_none(), "nada ficou pego");
    assert!(w.grab_body(dyn_, [6.0, 0.0]), "dinâmico aceita");
    assert!(w.grabbed_body().is_some(), "e fica pego");
}

/// Soltar devolve o mundo à contagem de antes: a âncora invisível e a mola saem
/// juntas. Sem isto uma sessão de cutucões acumula tralha que entra em todo
/// checkpoint e em todo hash.
#[test]
fn letting_go_removes_the_plumbing() {
    let (mut w, ball) = floating_ball(1.0);
    let (bodies, joints) = (w.body_snapshots().len(), w.joint_count());
    assert!(w.grab_body(ball, [0.0, 0.0]));
    assert_eq!(w.body_snapshots().len(), bodies + 1, "a âncora existe");
    assert_eq!(w.joint_count(), joints + 1, "a mola existe");
    w.release_grab();
    assert_eq!(w.body_snapshots().len(), bodies, "a âncora saiu");
    assert_eq!(w.joint_count(), joints, "a mola saiu");
    assert!(w.grabbed_body().is_none());
    // Soltar duas vezes é no-op (o caminho comum de todo release de botão).
    w.release_grab();
    assert_eq!(w.body_snapshots().len(), bodies);
}

/// Um clique sem arrasto **não move nada**, e é o que torna a mão inofensiva
/// para quem só quis selecionar durante o play.
///
/// Também é o gate que prova que `rest_length = 0` não produz direção indefinida:
/// com a separação nula o erro é nulo, a mola não empurra, e a pose fica
/// bit-idêntica em vez de NaN.
#[test]
fn a_click_without_a_drag_does_not_move_the_body() {
    let (mut w, ball) = floating_ball(1.0);
    let before = pose(&w, ball);
    assert!(w.grab_body(ball, before));
    for _ in 0..60 {
        w.step();
    }
    let after = pose(&w, ball);
    assert_eq!(before, after, "um clique parado tem de ser bit-idêntico");
    assert!(after[0].is_finite() && after[1].is_finite(), "e finito");
}

/// Uma mão só: pegar com a mão ocupada solta a anterior. O alternativo (duas
/// molas) é uma feature de dois cursores que ninguém pediu, e recusar em silêncio
/// deixaria o gesto seguinte inerte sem dizer por quê.
#[test]
fn grabbing_again_lets_go_of_the_first_body() {
    let (mut w, first) = floating_ball(1.0);
    let second = body(
        &mut w,
        RigidBodyType::Dynamic,
        5.0,
        0.0,
        ShapeDesc::Ball { radius: 0.3 },
        1.0,
    );
    let (bodies, joints) = (w.body_snapshots().len(), w.joint_count());
    assert!(w.grab_body(first, [0.0, 0.0]));
    assert!(w.grab_body(second, [5.0, 0.0]));
    assert_eq!(w.body_snapshots().len(), bodies + 1, "UMA âncora, não duas");
    assert_eq!(w.joint_count(), joints + 1, "UMA mola, não duas");
    // E é o SEGUNDO que a mão carrega.
    for _ in 0..60 {
        w.move_grab([5.0, 3.0]);
        w.step();
    }
    assert!(pose(&w, second)[1] > 2.0, "o segundo veio");
    assert!(pose(&w, first)[1].abs() < 1e-4, "o primeiro ficou");
}

/// **Um corpo que ADORMECEU na mão volta a segui-la** — o gate que faltava, e o
/// que ele pinou é uma linha que eu tinha REMOVIDO por medir o build errado.
///
/// Um corpo na mão dorme se a mão ficar parada (medido: tick 119 de mão imóvel),
/// e um corpo dormindo não é integrado. Sem o `wake_up` do `move_grab`, segurar
/// quieto por dois segundos e voltar a arrastar move o cursor e **mais nada** —
/// medido, o corpo fica em `x = 0,000` com a mão a 3 m. Mover a âncora com
/// `wake_up = true` não resolve: ela é FIXA.
///
/// ⚠️ A sonda que me convenceu do contrário sampleava a cada 100 ticks e rodava
/// sobre o build que **ainda tinha** a linha — era ela que mantinha o corpo
/// acordado. *Uma medição de inércia tem de rodar sobre o build SEM o suspeito.*
#[test]
fn a_body_that_slept_in_the_hand_follows_it_again() {
    let (mut w, ball) = floating_ball(1.0);
    assert!(w.grab_body(ball, [0.0, 0.0]));
    // Dois segundos SEM chamada nenhuma de `move_grab` — que é exatamente o que a
    // shell faz quando o artista segura o botão e não mexe o mouse (ela só chama
    // no evento de Move). É esta a janela em que o corpo adormece.
    for _ in 0..150 {
        w.step();
    }
    assert!(
        w.bodies().get(ball).expect("alive").is_sleeping(),
        "a fixture precisa CONTER o fenômeno: o corpo tinha de adormecer"
    );
    // E agora a mão anda.
    for _ in 0..90 {
        w.move_grab([3.0, 0.0]);
        w.step();
    }
    assert!(
        pose(&w, ball)[0] > 2.9,
        "a mão tinha de voltar a carregar o corpo: x={:.3}",
        pose(&w, ball)[0]
    );
}
