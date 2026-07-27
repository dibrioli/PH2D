//! **A RODA** — o cubo que gira *e* cavalga uma suspensão.
//!
//! Ela é o primeiro tipo do kit a deixar **dois** graus de liberdade livres, e
//! é isso que os gates medem: cada vizinho que quase faz a mesma coisa deixa
//! livre **um** dos dois, então a roda só se define contra os três.
//!
//! | vizinho | deixa livre | o que falta |
//! |---|---|---|
//! | `Pin` | o giro | o cubo não pode subir |
//! | `Slider` | o curso | o cubo não pode girar |
//! | `Rod`/`Spring` | a distância | não há eixo, e o giro é livre nos dois lados |
//!
//! ⚠️ **E há um gate que só existe porque a barra NÃO conseguiu:** o limite de
//! curso de uma roda vale dos **DOIS** lados. O `Rod` mediu que o limite linear
//! ACOPLADO do rapier é unilateral (`// FIXME: handle min limit too.` no solver
//! dele); numa roda nada é acoplado — o `coupled_axes` fica vazio — então o
//! `[min, max]` passa pelo `limit_linear` comum e o batente de compressão MORDE.

use ph2d_physics::{
    BodyDesc, JointDesc, JointKind, MotorDesc, MotorMode, PhysicsWorld, RigidBodyHandle,
    RigidBodyType, ShapeDesc,
};

/// Altura de marcha autorada: o cubo nasce este tanto abaixo do chassi.
const RIDE: f32 = 0.5;
const WHEEL_R: f32 = 0.3;

fn body(
    w: &mut PhysicsWorld,
    kind: RigidBodyType,
    x: f32,
    y: f32,
    shape: ShapeDesc,
    density: f32,
) -> RigidBodyHandle {
    w.spawn_body(BodyDesc {
        body_type: kind,
        x,
        y,
        rotation: 0.0,
        density,
        shape,
        restitution: 0.0,
        friction: 1.0,
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

/// Um carro de UMA roda, apoiado no chão — **a suspensão só comprime com a roda
/// APOIADA**, e é por isso que o chão não é decoração da fixture: solto no ar o
/// par cai junto e a distância entre eles nunca muda (a fixture que a primeira
/// sonda desta wave usou, e que media zero em toda rigidez).
fn car(
    kind: JointKind,
    stiffness: f32,
    limits: Option<[f32; 2]>,
    motor: Option<MotorDesc>,
    chassis_density: f32,
) -> (PhysicsWorld, RigidBodyHandle, RigidBodyHandle) {
    let mut w = PhysicsWorld::new();
    body(
        &mut w,
        RigidBodyType::Fixed,
        0.0,
        -0.5,
        ShapeDesc::Cuboid {
            half_x: 50.0,
            half_y: 0.5,
        },
        1.0,
    );
    let chassis = body(
        &mut w,
        RigidBodyType::Dynamic,
        0.0,
        WHEEL_R + RIDE,
        ShapeDesc::Cuboid {
            half_x: 1.0,
            half_y: 0.2,
        },
        chassis_density,
    );
    let hub = body(
        &mut w,
        RigidBodyType::Dynamic,
        0.0,
        WHEEL_R,
        ShapeDesc::Ball { radius: WHEEL_R },
        1.0,
    );
    let (la, lb) = w
        .world_to_local_anchors(chassis, hub, [0.0, WHEEL_R], [0.0, WHEEL_R])
        .expect("bodies alive");
    w.spawn_joint(
        chassis,
        hub,
        JointDesc {
            kind,
            anchor_a: la,
            anchor_b: lb,
            // A suspensão é VERTICAL.
            axis_a: [0.0, 1.0],
            axis_b: [0.0, 1.0],
            stiffness,
            damping: JointDesc::WHEEL_DAMPING,
            limits,
            motor,
            ..Default::default()
        },
    )
    .expect("joint");
    (w, chassis, hub)
}

/// Um chassi **ESTÁTICO** com a roda pendurada nele — a única configuração em
/// que o peso ESTICA a suspensão em vez de comprimi-la, e portanto a única em
/// que o batente de extensão (`min`) é quem segura.
fn hanging_wheel(
    stiffness: f32,
    limits: [f32; 2],
) -> (PhysicsWorld, RigidBodyHandle, RigidBodyHandle) {
    let mut w = PhysicsWorld::new();
    let chassis = body(
        &mut w,
        RigidBodyType::Fixed,
        0.0,
        WHEEL_R + RIDE,
        ShapeDesc::Cuboid {
            half_x: 1.0,
            half_y: 0.2,
        },
        1.0,
    );
    let hub = body(
        &mut w,
        RigidBodyType::Dynamic,
        0.0,
        WHEEL_R,
        ShapeDesc::Ball { radius: WHEEL_R },
        1.0,
    );
    let (la, lb) = w
        .world_to_local_anchors(chassis, hub, [0.0, WHEEL_R], [0.0, WHEEL_R])
        .expect("bodies alive");
    w.spawn_joint(
        chassis,
        hub,
        JointDesc {
            kind: JointKind::Wheel,
            anchor_a: la,
            anchor_b: lb,
            axis_a: [0.0, 1.0],
            axis_b: [0.0, 1.0],
            stiffness,
            damping: JointDesc::WHEEL_DAMPING,
            limits: Some(limits),
            ..Default::default()
        },
    )
    .expect("joint");
    (w, chassis, hub)
}

fn pose(w: &PhysicsWorld, h: RigidBodyHandle) -> [f32; 3] {
    let p = w.body_pose(h).expect("body alive");
    [p.translation.x, p.translation.y, p.rotation.angle()]
}

/// A separação chassi↔cubo ao longo da suspensão, e quanto ela variou.
fn settle(
    w: &mut PhysicsWorld,
    chassis: RigidBodyHandle,
    hub: RigidBodyHandle,
    ticks: usize,
) -> (f32, f32, f32) {
    let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
    for _ in 0..ticks {
        w.step();
        let r = pose(w, chassis)[1] - pose(w, hub)[1];
        lo = lo.min(r);
        hi = hi.max(r);
    }
    (pose(w, chassis)[1] - pose(w, hub)[1], lo, hi)
}

/// **A suspensão CEDE, e um pino não.** O primeiro dos dois graus de liberdade.
#[test]
fn a_wheel_gives_along_its_axis_where_a_pin_holds_the_hub_fast() {
    let (mut w, c, h) = car(
        JointKind::Wheel,
        JointDesc::WHEEL_STIFFNESS,
        None,
        None,
        1.0,
    );
    let (wheel_ride, _, _) = settle(&mut w, c, h, 240);
    let (mut w, c, h) = car(JointKind::Pin, JointDesc::WHEEL_STIFFNESS, None, None, 1.0);
    let (pin_ride, _, _) = settle(&mut w, c, h, 240);

    let sag = RIDE - wheel_ride;
    assert!(
        sag > 0.02,
        "o peso do chassi tinha de afundar a suspensão; ela cedeu {sag:.4} m"
    );
    assert!(
        (RIDE - pin_ride).abs() < 0.005,
        "o controle falhou: um PINO não tem por onde ceder, mas a altura de marcha \
         dele foi de {RIDE:.3} para {pin_ride:.4}"
    );
}

/// **O cubo GIRA, e um trilho não deixa.** O segundo grau de liberdade — e é o
/// que separa a roda do `Slider`, que é o outro tipo com eixo e curso.
///
/// ⚠️ Lê a velocidade ANGULAR, nunca a rotação: `rotation` wrapa em ±π, e um
/// cubo tocado por um motor dá várias voltas — a coordenada vira ruído acima de
/// uma revolução.
#[test]
fn a_wheel_spins_where_a_slider_forbids_it() {
    let drive = Some(MotorDesc {
        mode: MotorMode::Velocity,
        speed: -6.0,
        target: 0.0,
        max_force: 40.0,
    });
    let spin_of = |kind: JointKind| {
        let (mut w, _c, hub) = car(kind, JointDesc::WHEEL_STIFFNESS, None, drive, 1.0);
        for _ in 0..120 {
            w.step();
        }
        w.body_snapshots()
            .iter()
            .find(|s| s.handle_index == hub.into_raw_parts().0)
            .map(|s| s.angvel)
            .unwrap_or(0.0)
    };
    let wheel = spin_of(JointKind::Wheel);
    let slider = spin_of(JointKind::Slider);
    assert!(
        wheel < -3.0,
        "o cubo tinha de girar no sentido pedido (-6 rad/s); mediu {wheel:.4}"
    );
    assert!(
        slider.abs() < 0.5,
        "o controle falhou: um TRILHO proíbe rotação relativa, mas o cubo girou a \
         {slider:.4} rad/s"
    );
}

/// **O curso tem os DOIS batentes — e é a EXTENSÃO que prova a bilateralidade.**
///
/// É o gate que justifica a construção inteira: numa roda o `coupled_axes` fica
/// vazio, então `[min, max]` vai pelo `limit_linear` bilateral em vez do
/// `limit_linear_coupled` unilateral (`// FIXME: handle min limit too.`, que lê
/// só `limits[1]` e sai com `impulse_bounds = [0, ∞]`) — o beco que matou o
/// desenho original do [`JointKind::Rod`].
///
/// ⚠️ **Só a metade da EXTENSÃO distingue as duas construções, e chegar a isso
/// custou duas correções deste gate.** (1) A primeira versão usava faixa
/// SIMÉTRICA e uma mutação que acopla os eixos **passou**: um limite acoplado é
/// sobre a MAGNITUDE (`dist = ‖lin_jac‖`, não-negativa), então `|x| ≤ s` e
/// `−s ≤ x ≤ s` são a mesma restrição — o gate comparava dois desenhos
/// indistinguíveis e chamava isso de prova. (2) Assimétrica, ele nasceu VERMELHO
/// e o número denunciou o resto: a suspensão parou no `max` e não no `min`,
/// porque **comprimir é POSITIVO** neste eixo (o cubo sobe em direção ao chassi)
/// — o comentário que eu tinha escrito afirmava o contrário, e a faixa simétrica
/// não podia revelá-lo. Com o sinal certo, o `max` é lido pelas DUAS construções
/// e só o `min` separa: um limite de magnitude deixaria a roda pendurada descer
/// até `−max`.
#[test]
fn a_wheels_travel_has_both_stops_and_the_droop_one_proves_it() {
    let soft = 60.0;
    let (mut w, c, h) = car(JointKind::Wheel, soft, None, None, 4.0);
    let (free, _, _) = settle(&mut w, c, h, 240);
    let free_sag = RIDE - free;
    assert!(
        free_sag > 0.25,
        "a fixture não contém o fenômeno: sem batente o chassi tinha de afundar \
         bem além dele, e afundou {free_sag:.4} m"
    );

    // (a) COMPRESSÃO — o batente que o peso do chassi encontra. `min` fica bem
    // FROUXO para não ser ele a segurar.
    for stop in [0.20_f32, 0.05, 0.01] {
        let (mut w, c, h) = car(JointKind::Wheel, soft, Some([-0.30, stop]), None, 4.0);
        let (end, lo, _hi) = settle(&mut w, c, h, 240);
        let sag = RIDE - end;
        assert!(
            (sag - stop).abs() < 0.01,
            "o batente de compressão de {stop:.2} m tinha de segurar a suspensão ali; \
             ela parou em {sag:.4}"
        );
        // ⚠️ E ela nunca o atravessa NO CAMINHO — o mínimo da trajetória, não só
        // o repouso: um batente que só segura depois de assentar deixaria o
        // chassi mergulhar através dele e voltar.
        assert!(
            RIDE - lo < stop + 0.02,
            "a suspensão atravessou o batente de {stop:.2} m no caminho (mergulhou \
             {:.4} m)",
            RIDE - lo
        );
    }

    // (b) EXTENSÃO — a metade que só um limite BILATERAL tem. O chassi é ESTÁTICO
    // e a roda PENDURA nele: agora é o peso do cubo que estica a suspensão, e
    // quem tem de segurar é o `min`. Um limite de magnitude (`|x| ≤ max`) leria
    // só o `max` e deixaria a roda descer muito mais.
    //
    // ⚠️ A mola é FRACA aqui de propósito: com a rigidez normal ela sozinha
    // segura o cubo antes de qualquer batente razoável (medido: estica 0,1634 m
    // a k=60), e o gate mediria a MOLA achando que media o batente. A k=10 ela
    // estica 0,7016 m, então os dois batentes abaixo mordem de verdade.
    let limp = 10.0;
    let (mut w, c, h) = hanging_wheel(limp, [-10.0, 10.0]);
    let (natural, _, _) = settle(&mut w, c, h, 240);
    assert!(
        natural - RIDE > 0.5,
        "a fixture não contém o fenômeno: sem batente a roda tinha de descer bem \
         além dele, e desceu {:.4} m",
        natural - RIDE
    );
    for droop in [0.30_f32, 0.10] {
        let (mut w, c, h) = hanging_wheel(limp, [-droop, 0.05]);
        let (end, _lo, _hi) = settle(&mut w, c, h, 240);
        let stretch = end - RIDE;
        assert!(
            (stretch - droop).abs() < 0.01,
            "o batente de EXTENSÃO de {droop:.2} m tinha de segurar a roda pendurada; \
             ela desceu {stretch:.4}"
        );
    }
}

/// **O motor dirige o GIRO, não a suspensão** — a escolha de eixo que
/// `motor_axis` faz, medida pelo que ela impede.
///
/// Uma roda carrega uma mola (que o rapier modela como motor em `LinX`) **e** um
/// motor (em `AngX`), e os dois não colidem porque são eixos diferentes. Se o
/// motor tivesse ido para o eixo linear ele **sobrescreveria** a mola — que é
/// exatamente a razão de uma `Spring` não ter motor nenhum.
#[test]
fn the_motor_drives_the_spin_and_leaves_the_suspension_alone() {
    let drive = Some(MotorDesc {
        mode: MotorMode::Velocity,
        speed: -6.0,
        target: 0.0,
        max_force: 40.0,
    });
    let (mut w, c, h) = car(
        JointKind::Wheel,
        JointDesc::WHEEL_STIFFNESS,
        None,
        None,
        1.0,
    );
    let (quiet, _, _) = settle(&mut w, c, h, 240);
    let (mut w, c, h) = car(
        JointKind::Wheel,
        JointDesc::WHEEL_STIFFNESS,
        None,
        drive,
        1.0,
    );
    let (driven, _, _) = settle(&mut w, c, h, 240);
    assert!(
        (quiet - driven).abs() < 0.02,
        "a altura de marcha não podia depender do motor; parada {quiet:.4} contra \
         {driven:.4} com tração"
    );
}

/// **Uma roda com números impossíveis ainda produz poses finitas.**
///
/// A porta de sanidade que todo tipo desta linha tem: `NaN`/negativo não falha
/// alto, ele envenena a pose, o readback e daí o hash de determinismo.
#[test]
fn a_wheel_with_an_impossible_spring_still_produces_finite_poses() {
    for stiffness in [0.0_f32, -1.0, f32::MAX] {
        let (mut w, c, h) = car(JointKind::Wheel, stiffness, None, None, 1.0);
        for _ in 0..120 {
            w.step();
        }
        for handle in [c, h] {
            let p = pose(&w, handle);
            assert!(
                p.iter().all(|v| v.is_finite()),
                "stiffness {stiffness} produziu uma pose não-finita: {p:?}"
            );
        }
    }
}
