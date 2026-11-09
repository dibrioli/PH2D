//! **A exceção de camadas por forma** (GEF02 B9): `BodyDesc.collide = [include, exclude]` vence a
//! linha da matriz — o `exclude` de qualquer lado ganha; depois a matriz OU o `include` de qualquer
//! lado. Simétrico: o chão também não toca a forma que o exclui.

use ph2d_physics::{BodyDesc, LayerMatrix, PhysicsWorld, RigidBodyType, ShapeDesc};

const CHAO: u8 = 1;

fn desc(
    body_type: RigidBodyType,
    x: f32,
    y: f32,
    shape: ShapeDesc,
    layer: u8,
    collide: [u32; 2],
) -> BodyDesc {
    BodyDesc {
        body_type,
        x,
        y,
        rotation: 0.0,
        density: 1.0,
        shape,
        restitution: 0.0,
        friction: 0.5,
        layer,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        offset: [0.0, 0.0],
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
        shape_pose: [0.0; 3],
        collide,
    }
}

/// Um chão (camada 1) e três bolas (camada 0) com as exceções dadas; devolve o `y` final de cada.
fn cai(matrix: LayerMatrix, collides: [[u32; 2]; 3], chao_primeiro: bool) -> [f32; 3] {
    let mut w = PhysicsWorld::new();
    w.set_layer_matrix(matrix);
    let chao = desc(
        RigidBodyType::Fixed,
        0.0,
        0.0,
        ShapeDesc::Cuboid {
            half_x: 10.0,
            half_y: 0.1,
        },
        CHAO,
        [0; 2],
    );
    if chao_primeiro {
        w.spawn_body(chao);
    }
    let bolas: Vec<_> = collides
        .iter()
        .enumerate()
        .map(|(i, &c)| {
            w.spawn_body(desc(
                RigidBodyType::Dynamic,
                2.0 * i as f32 - 2.0,
                1.0,
                ShapeDesc::Ball { radius: 0.25 },
                0,
                c,
            ))
        })
        .collect();
    if !chao_primeiro {
        w.spawn_body(chao);
    }
    for _ in 0..120 {
        w.step();
    }
    let y = |h| w.bodies().get(h).expect("bola").position().translation.y;
    [y(bolas[0]), y(bolas[1]), y(bolas[2])]
}

#[test]
fn a_forma_que_exclui_o_chao_cai_e_as_outras_repousam() {
    for chao_primeiro in [true, false] {
        let [a, b, c] = cai(
            LayerMatrix::all(),
            [[0, 1 << CHAO], [0; 2], [0; 2]],
            chao_primeiro,
        );
        assert!(a < -2.0, "a forma que exclui o Ground atravessa-o: y={a}");
        assert!(b > 0.2 && c > 0.2, "as irmãs sem exceção repousam: {b} {c}");
    }
}

#[test]
fn o_include_vence_o_nao_da_matriz() {
    let mut m = LayerMatrix::all();
    m.set(0, usize::from(CHAO), false);
    for chao_primeiro in [true, false] {
        let [a, b, _] = cai(m, [[1 << CHAO, 0], [0; 2], [0; 2]], chao_primeiro);
        assert!(
            a > 0.2,
            "o include repousa no chão que a matriz recusa: y={a}"
        );
        assert!(b < -2.0, "sem include a matriz manda: y={b}");
    }
}

#[test]
fn a_exclusao_e_simetrica_o_raio_do_chao_tambem_nao_a_ve() {
    let mut w = PhysicsWorld::new();
    let bola = w.spawn_body(desc(
        RigidBodyType::Fixed,
        0.0,
        2.0,
        ShapeDesc::Ball { radius: 0.5 },
        0,
        [0, 1 << CHAO],
    ));
    w.step();
    // Uma consulta feita da camada do chão não vê a forma que exclui o chão; da camada 0 vê.
    let do_chao = w.cast_ray([0.0, 0.0], [0.0, 1.0], 5.0, None, CHAO);
    let da_zero = w.cast_ray([0.0, 0.0], [0.0, 1.0], 5.0, None, 0);
    assert!(do_chao.is_none(), "o chão viu a forma que o exclui");
    assert!(da_zero.is_some(), "controlo: a camada 0 vê-a");
    let _ = bola;
}
