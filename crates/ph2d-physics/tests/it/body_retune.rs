//! **Reafinar um corpo vivo descreve o MESMO corpo que criá-lo** (F30 §C) — o gémeo do
//! `joint_retune.rs`, pela mesma razão: duas portas a construir o mesmo corpo divergem em silêncio.
//!
//! Cada linha muda UM campo do `BodyDesc`: o mundo A cria o corpo antigo e reafina-o antes do 1.º
//! `step`; o mundo B cria-o já novo. As poses têm de ser IGUAIS bit a bit depois de 240 passos.
//! ⚠️ Antes do 1.º passo de propósito: a meio, o reafinado guarda a velocidade e os contactos (a
//! feature) e o recriado zera-os.

use ph2d_physics::{
    AreaEffect, BodyDesc, CoefficientCombineRule, DampingDesc, PhysicsWorld, RigidBodyHandle,
    RigidBodyType, ShapeDesc,
};

fn bola() -> BodyDesc {
    BodyDesc {
        body_type: RigidBodyType::Dynamic,
        x: 0.0,
        y: 3.0,
        rotation: 0.3,
        density: 1.0,
        shape: ShapeDesc::Ball { radius: 0.25 },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [2.0, 0.0],
        angvel: 1.0,
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
    }
}

fn zona() -> BodyDesc {
    BodyDesc {
        body_type: RigidBodyType::Fixed,
        x: 0.0,
        y: 1.5,
        rotation: 0.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        shape: ShapeDesc::Cuboid {
            half_x: 3.0,
            half_y: 1.0,
        },
        is_sensor: true,
        ..bola()
    }
}

fn chao() -> BodyDesc {
    BodyDesc {
        body_type: RigidBodyType::Fixed,
        y: 0.0,
        rotation: 0.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        shape: ShapeDesc::Cuboid {
            half_x: 20.0,
            half_y: 0.1,
        },
        ..bola()
    }
}

/// O mundo da prova: o chão, a zona (sensor) e a bola; devolve o handle de `quem` (0 = bola, 1 = zona).
fn mundo(bola_d: BodyDesc, zona_d: BodyDesc) -> (PhysicsWorld, [RigidBodyHandle; 3]) {
    let mut w = PhysicsWorld::new();
    // A camada 3 não colide com a 0 (o chão, a caixa): é assim que a camada se vê.
    let mut m = ph2d_physics::LayerMatrix::all();
    m.set(3, 0, false);
    w.set_layer_matrix(m);
    w.spawn_body(chao());
    let z = w.spawn_body(zona_d);
    // A caixa no caminho da bola: é no choque que a massa, a dominância e o material se veem.
    let c = w.spawn_body(BodyDesc {
        x: 1.2,
        y: 0.35,
        rotation: 0.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        shape: ShapeDesc::Cuboid {
            half_x: 0.25,
            half_y: 0.25,
        },
        ..bola()
    });
    let b = w.spawn_body(bola_d);
    (w, [b, z, c])
}

fn poses(w: &mut PhysicsWorld, hs: [RigidBodyHandle; 3]) -> Vec<[u32; 3]> {
    let mut out = Vec::new();
    for _ in 0..240 {
        w.step();
        for h in hs {
            let p = w.body_pose(h).expect("vivo");
            out.push([
                p.translation.x.to_bits(),
                p.translation.y.to_bits(),
                p.rotation.angle().to_bits(),
            ]);
        }
    }
    out
}

#[test]
fn reafinar_um_corpo_vivo_e_o_mesmo_que_cria_lo() {
    let regra = CoefficientCombineRule::Max;
    // (nome, quem: 0 = bola, 1 = zona, o desc novo)
    let casos: Vec<(&str, usize, BodyDesc)> = vec![
        ("body_type", 0, BodyDesc { body_type: RigidBodyType::KinematicVelocityBased, ..bola() }),
        ("density", 0, BodyDesc { density: 7.0, ..bola() }),
        ("shape", 0, BodyDesc { shape: ShapeDesc::Cuboid { half_x: 0.4, half_y: 0.2 }, ..bola() }),
        ("restitution", 0, BodyDesc { restitution: 0.9, ..bola() }),
        ("friction", 0, BodyDesc { friction: 0.0, ..bola() }),
        ("layer", 0, BodyDesc { layer: 3, ..bola() }),
        ("is_sensor", 0, BodyDesc { is_sensor: true, ..bola() }),
        ("gravity_scale", 0, BodyDesc { gravity_scale: -0.5, ..bola() }),
        ("lock_rotation", 0, BodyDesc { lock_rotation: true, ..bola() }),
        ("offset", 0, BodyDesc { offset: [0.3, -0.1], ..bola() }),
        ("lock_x", 0, BodyDesc { lock_x: true, ..bola() }),
        ("lock_y", 0, BodyDesc { lock_y: true, ..bola() }),
        ("mass_override", 0, BodyDesc { mass_override: Some(9.0), ..bola() }),
        ("dominance", 0, BodyDesc { dominance: 5, ..bola() }),
        (
            "material",
            0,
            BodyDesc {
                material: ph2d_physics::CombineRules { restitution: regra, friction: regra },
                restitution: 0.4,
                ..bola()
            },
        ),
        (
            "damping",
            0,
            BodyDesc {
                damping: Some(DampingDesc { linear: 2.0, angular: 3.0, replace: true }),
                ..bola()
            },
        ),
        ("one_way", 0, BodyDesc { one_way: true, ..bola() }),
        (
            "effector",
            1,
            BodyDesc {
                effector: Some(AreaEffect {
                    force: [0.0, 30.0],
                    drag: 1.0,
                    density: 0.0,
                    form_drag: 0.0,
                    torque: 0.0,
                    world_axes: true,
                    falloff: 0.0,
                    mirror: AreaEffect::UNMIRRORED,
                }),
                ..zona()
            },
        ),
    ];
    // A CCD só se vê num tiro: a bola a 600 m/s contra a caixa atravessa-a sem ela.
    let tiro = BodyDesc {
        x: -3.0,
        y: 0.35,
        rotation: 0.0,
        linvel: [600.0, 0.0],
        angvel: 0.0,
        gravity_scale: 0.0,
        ..bola()
    };
    let mut todos: Vec<(&str, usize, BodyDesc, BodyDesc)> = casos
        .into_iter()
        .map(|(n, q, novo)| (n, q, if q == 0 { bola() } else { zona() }, novo))
        .collect();
    todos.push(("ccd", 0, tiro, BodyDesc { ccd: true, ..tiro }));
    let par = |q: usize, d: BodyDesc| if q == 0 { (d, zona()) } else { (bola(), d) };
    let mut mudos = Vec::new();
    for (nome, quem, antigo, novo) in todos {
        let (b_d, z_d) = par(quem, novo);
        let (mut criado, hc) = mundo(b_d, z_d);
        let (b_d, z_d) = par(quem, antigo);
        let (mut reafinado, hr) = mundo(b_d, z_d);
        assert!(reafinado.retune_body(hr[quem], &novo), "{nome}: o handle não pegou");
        let (a, b) = (poses(&mut criado, hc), poses(&mut reafinado, hr));
        assert_eq!(a, b, "{nome}: o corpo reafinado não é o corpo criado com o mesmo desc");
        let (mut velho, hv) = mundo(b_d, z_d);
        if poses(&mut velho, hv) == a {
            mudos.push(nome);
        }
    }
    assert!(mudos.is_empty(), "a mudança não se vê nesta cena — as linhas não provam nada: {mudos:?}");
}
