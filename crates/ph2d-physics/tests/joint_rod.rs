//! **W-Rod — a barra rígida, no seam do motor.**
//!
//! Um rod é a única coisa que este conjunto de tipos não sabia dizer, e o vão é
//! estreito o bastante para passar despercebido: um **Weld** segura a distância
//! *e* congela o giro, uma **Rope** segura só o teto (afrouxa), uma **Spring**
//! não segura nada exatamente. Uma biela precisa da distância rígida com as duas
//! pontas livres — que não é nenhum dos três.
//!
//! Por isso cada gate aqui é um **par**: a afirmação e o tipo que ela tem de
//! distinguir. *"Os corpos ficam a 2 m"* é satisfeito igualmente bem por *"nada
//! se moveu"*, e *"gira livre"* é satisfeito por *"não está preso a nada"*.

use ph2d_physics::{BodyDesc, JointDesc, JointKind, PhysicsWorld, RigidBodyType, ShapeDesc};

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

fn body(
    w: &mut PhysicsWorld,
    kind: RigidBodyType,
    x: f32,
    y: f32,
    shape: ShapeDesc,
    angvel: f32,
    linvel: [f32; 2],
) -> ph2d_physics::RigidBodyHandle {
    w.spawn_body(BodyDesc {
        body_type: kind,
        x,
        y,
        rotation: 0.0,
        density: 1.0,
        shape,
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel,
        angvel,
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

fn pose(w: &PhysicsWorld, h: ph2d_physics::RigidBodyHandle) -> [f32; 2] {
    let p = w.body_pose(h).expect("body alive");
    [p.translation.x, p.translation.y]
}

/// A velocidade angular VIVA — a **taxa**, nunca o angulo.
///
/// `Transform::rotation` wrapa em +/-pi, entao uma prancha que girou 6 rad le
/// `6 - 2pi = -0,28` e um gate ingenuo conclui que ela mal se moveu. Foi
/// exatamente o que a 1a versao deste arquivo fez. A taxa nao tem periodo.
fn angvel(w: &PhysicsWorld, h: ph2d_physics::RigidBodyHandle) -> f32 {
    let idx = h.into_raw_parts().0;
    w.body_snapshots()
        .into_iter()
        .find(|s| s.handle_index == idx)
        .expect("body alive")
        .angvel
}

/// Junta `a` e `b` com âncoras em **MUNDO** — o frame em que o artista aponta.
fn join(
    w: &mut PhysicsWorld,
    a: ph2d_physics::RigidBodyHandle,
    b: ph2d_physics::RigidBodyHandle,
    desc: JointDesc,
) {
    let (la, lb) = w
        .world_to_local_anchors(a, b, desc.anchor_a, desc.anchor_b)
        .expect("bodies alive");
    w.spawn_joint(
        a,
        b,
        JointDesc {
            anchor_a: la,
            anchor_b: lb,
            ..desc
        },
    )
    .expect("joint built");
}

/// **O pêndulo INVERTIDO** — a cena que separa um rod de uma corda, e a razão de
/// ela ser esta e não um peso pendurado.
///
/// Um peso *pendurado* não distingue nada: com a corda esticada os dois tipos
/// seguram igual. O que uma corda **não sabe fazer é empurrar** — então o corpo
/// vai para CIMA da âncora, e a gravidade tenta trazê-lo para perto. A corda
/// afrouxa e ele desce; a barra o segura afastado.
///
/// ⚠️ **O oráculo é a MENOR distância da TRAJETÓRIA, não a final** — e a 1ª
/// versão media a final e ficou vermelha sobre produto correto: um peso que
/// oscila deixa a corda **TESA**, então os dois tipos terminam a 2,0 m. A corda
/// só se distingue *no caminho*, quando o peso passa perto do gancho e ela
/// afrouxa. (A mesma lição que o scrub do W1.5 pagou: um sistema amortecido
/// esquece a perturbação, e o endpoint esquece com ele.)
///
/// A distância é o oráculo, e não a altura, porque um rod deixa o conjunto
/// **girar**: a barra pode tombar de lado (é um equilíbrio instável), e tombar é
/// o rod funcionando. O que ele não pode é encurtar.
fn inverted_pendulum_min_distance(kind: JointKind) -> f32 {
    let mut w = PhysicsWorld::new();
    let hook = body(
        &mut w,
        RigidBodyType::Fixed,
        0.0,
        0.0,
        ShapeDesc::Ball { radius: 0.05 },
        0.0,
        [0.0, 0.0],
    );
    let bob = body(
        &mut w,
        RigidBodyType::Dynamic,
        0.0,
        2.0,
        ShapeDesc::Ball { radius: 0.1 },
        0.0,
        // Um empurraozinho de lado: sem ele o equilibrio instavel fica de pe por
        // simetria exata e a cena nao exercita nada.
        [0.05, 0.0],
    );
    join(
        &mut w,
        hook,
        bob,
        JointDesc {
            kind,
            anchor_a: [0.0, 0.0],
            anchor_b: [0.0, 2.0],
            max_length: 2.0,
            ..Default::default()
        },
    );
    let mut min = dist(pose(&w, hook), pose(&w, bob));
    for _ in 0..180 {
        w.step();
        min = min.min(dist(pose(&w, hook), pose(&w, bob)));
    }
    min
}

#[test]
fn a_rod_holds_the_distance_in_both_directions_where_a_rope_only_caps() {
    let rod = inverted_pendulum_min_distance(JointKind::Rod);
    let rope = inverted_pendulum_min_distance(JointKind::Rope);

    assert!(
        (rod - 2.0).abs() < 0.02,
        "a barra tinha de manter 2 m em TODO instante; encolheu para {rod:.4}"
    );
    // O CONTROLE, na mesma cena: uma corda não empurra, então em algum ponto da
    // queda o peso se aproxima do gancho e ela afrouxa. Sem esta metade, um rod
    // quebrado que simplesmente congelasse tudo passaria.
    assert!(
        rope < 1.5,
        "o controle falhou: uma CORDA tinha de afrouxar em algum instante, mas o \
         mínimo dela foi {rope:.4} m — a cena não distingue os dois tipos"
    );
}

/// **As duas pontas giram** — a metade que separa o rod de um Weld.
///
/// As duas afirmações num gate só porque cada uma sozinha passa sobre o tipo
/// errado: *"a distância é 2 m"* também vale para um Weld, e *"gira"* também vale
/// para um corpo solto.
#[test]
fn a_rod_leaves_both_ends_free_to_turn_where_a_weld_does_not() {
    fn spin(kind: JointKind) -> (f32, f32) {
        let mut w = PhysicsWorld::new();
        let hook = body(
            &mut w,
            RigidBodyType::Fixed,
            0.0,
            0.0,
            ShapeDesc::Ball { radius: 0.05 },
            0.0,
            [0.0, 0.0],
        );
        // Gravidade zero: a queda giraria a barra por conta própria e o número
        // deixaria de ser sobre a LIBERDADE da ponta.
        w.set_gravity(0.0, 0.0);
        let plank = body(
            &mut w,
            RigidBodyType::Dynamic,
            2.0,
            0.0,
            ShapeDesc::Cuboid {
                half_x: 0.3,
                half_y: 0.05,
            },
            3.0,
            [0.0, 0.0],
        );
        join(
            &mut w,
            hook,
            plank,
            JointDesc {
                kind,
                // A âncora no CENTRO da prancha: assim a barra não tem braço de
                // alavanca sobre ela, e o que se mede é a liberdade, não um
                // torque que o próprio vínculo aplicou.
                anchor_a: [0.0, 0.0],
                anchor_b: [2.0, 0.0],
                max_length: 2.0,
                ..Default::default()
            },
        );
        for _ in 0..120 {
            w.step();
        }
        (
            angvel(&w, plank).abs(),
            dist(pose(&w, hook), pose(&w, plank)),
        )
    }

    let (rod_spin, rod_dist) = spin(JointKind::Rod);
    let (weld_spin, _) = spin(JointKind::Weld);

    assert!(
        rod_spin > 2.9,
        "a ponta de um rod gira livre: a prancha foi lançada a 3 rad/s e sem \
         atrito nada a freia; mediu {rod_spin:.4} rad/s"
    );
    assert!(
        (rod_dist - 2.0).abs() < 0.02,
        "e girando ela continua a 2 m; mediu {rod_dist:.4}"
    );
    // O CONTROLE: um Weld congela o giro. Sem ele, "gira" passaria sobre um rod
    // que não estivesse preso a coisa nenhuma.
    assert!(
        weld_spin < 0.05,
        "o controle falhou: um WELD tinha de congelar o giro, mas a prancha \
         segue a {weld_spin:.4} rad/s"
    );
}

/// **Não CEDE sob carga** — a terceira aresta, a que separa o rod de uma MOLA.
///
/// As duas anteriores o separam da corda (segura na compressão) e do weld (as
/// pontas giram); esta é a que torna `ROD_STIFFNESS` load-bearing. Sem ela a
/// mutação que troca a rigidez do rod pela da mola (30) passa em tudo — medido,
/// e foi assim que este gate nasceu: o peso do pêndulo invertido é leve demais
/// para uma mola macia afundar visivelmente.
///
/// ⚠️ **A fixture pendura um corpo PESADO** (r = 0,5 ⇒ 0,785 kg contra os
/// 0,031 do outro gate), porque é a massa que estica uma mola: um oráculo de
/// rigidez medido sob carga leve não pode falhar pelo motivo que alega.
#[test]
fn a_rod_does_not_sag_under_load_where_a_spring_does() {
    fn sag(kind: JointKind) -> f32 {
        let mut w = PhysicsWorld::new();
        let hook = body(
            &mut w,
            RigidBodyType::Fixed,
            0.0,
            0.0,
            ShapeDesc::Ball { radius: 0.05 },
            0.0,
            [0.0, 0.0],
        );
        let load = body(
            &mut w,
            RigidBodyType::Dynamic,
            0.0,
            -2.0,
            ShapeDesc::Ball { radius: 0.5 },
            0.0,
            [0.0, 0.0],
        );
        join(
            &mut w,
            hook,
            load,
            JointDesc {
                kind,
                anchor_a: [0.0, 0.0],
                anchor_b: [0.0, -2.0],
                // O comprimento de um rod e o repouso de uma mola sao o MESMO
                // numero autorado para a mesma cena; so o campo difere por tipo.
                max_length: 2.0,
                rest_length: 2.0,
                ..Default::default()
            },
        );
        for _ in 0..180 {
            w.step();
        }
        dist(pose(&w, hook), pose(&w, load)) - 2.0
    }

    let rod = sag(JointKind::Rod);
    let spring = sag(JointKind::Spring);

    assert!(
        rod.abs() < 0.005,
        "uma barra de 2 m nao pode ceder sob 0,785 kg; esticou {rod:.4} m"
    );
    // O CONTROLE: a mola dos defaults do produto CEDE, e e para isso que ela
    // existe. Sem esta metade, "nao estica" tambem passaria sobre um rod que
    // simplesmente nao esta preso a nada.
    assert!(
        spring > 0.05,
        "o controle falhou: a MOLA tinha de ceder visivelmente sob a mesma \
         carga, mas esticou so {spring:.4} m"
    );
}

/// **Um comprimento impossível não envenena o solver.**
///
/// Irmão do gate do eixo degenerado: um `NaN` não falha alto, ele contamina as
/// poses, o readback e o hash de determinismo. rapier exige a distância de uma
/// corda *estritamente maior que zero*, e um projeto em disco pode trazer
/// qualquer coisa.
#[test]
fn a_rod_with_an_impossible_length_still_produces_finite_poses() {
    for len in [f32::NAN, f32::INFINITY, -3.0, 0.0] {
        let mut w = PhysicsWorld::new();
        let hook = body(
            &mut w,
            RigidBodyType::Fixed,
            0.0,
            0.0,
            ShapeDesc::Ball { radius: 0.05 },
            0.0,
            [0.0, 0.0],
        );
        let bob = body(
            &mut w,
            RigidBodyType::Dynamic,
            0.0,
            -1.0,
            ShapeDesc::Ball { radius: 0.1 },
            0.0,
            [0.0, 0.0],
        );
        join(
            &mut w,
            hook,
            bob,
            JointDesc {
                kind: JointKind::Rod,
                anchor_a: [0.0, 0.0],
                anchor_b: [0.0, -1.0],
                max_length: len,
                ..Default::default()
            },
        );
        for _ in 0..60 {
            w.step();
        }
        let p = pose(&w, bob);
        assert!(
            p[0].is_finite() && p[1].is_finite(),
            "comprimento {len} envenenou a pose: {p:?}"
        );
    }
}
