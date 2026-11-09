//! As consultas levam a exceção de QUEM PERGUNTA (GEF02 B9): o raio, o personagem e a varredura
//! do próprio corpo concordam com os contactos. E a camada de linha vazia alcança-se por `include`.
use crate::world::PhysicsWorld;
use crate::world::character::CharacterParams;
use crate::world::desc::BodyDesc;
use crate::world::layers::LayerMatrix;
use crate::world::shape::ShapeDesc;
use rapier2d::dynamics::{RigidBodyHandle, RigidBodyType};

const GROUND: u8 = 1;

fn desc(
    body_type: RigidBodyType,
    y: f32,
    shape: ShapeDesc,
    layer: u8,
    collide: [u32; 2],
) -> BodyDesc {
    BodyDesc {
        body_type,
        x: 0.0,
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
        lock_rotation: true,
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
        offset: [0.0, 0.0],
    }
}

/// Chão (camada `GROUND`, topo em `y = 0,5`) e um corpo dinâmico da camada 0 com `collide`.
fn scene(m: LayerMatrix, collide: [u32; 2]) -> (PhysicsWorld, RigidBodyHandle) {
    let mut w = PhysicsWorld::new();
    w.set_layer_matrix(m);
    let floor = ShapeDesc::Cuboid {
        half_x: 10.0,
        half_y: 0.5,
    };
    w.spawn_body(desc(RigidBodyType::Fixed, 0.0, floor, GROUND, [0; 2]));
    let me = w.spawn_body(desc(
        RigidBodyType::Dynamic,
        1.0,
        ShapeDesc::Cuboid {
            half_x: 0.2,
            half_y: 0.2,
        },
        0,
        collide,
    ));
    w.step();
    (w, me)
}

/// O que as TRÊS consultas e os contactos dizem sobre o chão: `(raio, personagem, varredura, fica)`.
fn verdicts(m: LayerMatrix, collide: [u32; 2]) -> (bool, bool, bool, bool) {
    let (mut w, me) = scene(m, collide);
    let ray = w
        .cast_ray_for_body([0.0, 1.0], [0.0, -1.0], 2.0, me, None, 0)
        .is_some();
    let params = CharacterParams {
        up: [0.0, 1.0],
        snap_distance: 0.0,
        max_slope_deg: 45.0,
        step_height: 0.0,
    };
    let moved = w.move_character(me, [0.0, -1.0], params, None, 0, &mut Vec::new());
    let blocked = moved.translation[1] > -0.9;
    let swept = w.sweep_body(me, [0.0, -1.0], 1.0, 0).is_some();
    for _ in 0..120 {
        w.step();
    }
    let stays = w.body_pose(me).expect("corpo").translation.y > 0.0;
    (ray, blocked, swept, stays)
}

fn all_bits(v: (bool, bool, bool, bool)) -> bool {
    v.0 && v.1 && v.2 && v.3
}

fn no_bits(v: (bool, bool, bool, bool)) -> bool {
    !v.0 && !v.1 && !v.2 && !v.3
}

#[test]
fn query_um_player_que_exclui_o_chao_atravessa_o_e_o_raio_concorda() {
    let m = LayerMatrix::all();
    assert!(
        all_bits(verdicts(m, [0; 2])),
        "controlo: sem exceção o chão segura e é visto"
    );
    let v = verdicts(m, [0, 1 << GROUND]);
    assert!(
        no_bits(v),
        "o exclude da forma do próprio corpo vale nas consultas como nos contactos: {v:?}"
    );
}

#[test]
fn query_um_player_que_inclui_uma_camada_proibida_fica_de_pe_nela() {
    let mut m = LayerMatrix::all();
    m.set(0, usize::from(GROUND), false);
    assert!(
        no_bits(verdicts(m, [0; 2])),
        "controlo: a matriz proíbe o par"
    );
    let v = verdicts(m, [1 << GROUND, 0]);
    assert!(
        all_bits(v),
        "o include do corpo vence o «não» da matriz: {v:?}"
    );
}

#[test]
fn query_o_include_de_uma_camada_de_linha_vazia_vale() {
    let mut m = LayerMatrix::all();
    for j in 0..crate::world::layers::MAX_LAYERS {
        m.set(usize::from(GROUND), j, false);
    }
    assert_eq!(m.row(usize::from(GROUND)), 0);
    assert!(
        no_bits(verdicts(m, [0; 2])),
        "controlo: a linha vazia não colide com nada"
    );
    let v = verdicts(m, [1 << GROUND, 0]);
    assert!(
        all_bits(v),
        "o include alcança a camada de linha vazia: {v:?}"
    );
}

/// A cura da linha vazia não abre nada a quem não tem exceção: duas camadas vazias não se tocam e
/// uma camada cheia não toca uma vazia — nem por contacto nem por consulta.
#[test]
fn query_linha_vazia_sem_excecao_continua_sem_ninguem() {
    let mut m = LayerMatrix::all();
    for j in 0..crate::world::layers::MAX_LAYERS {
        m.set(usize::from(GROUND), j, false);
        m.set(2, j, false);
    }
    let mut w = PhysicsWorld::new();
    w.set_layer_matrix(m);
    let floor = ShapeDesc::Cuboid {
        half_x: 10.0,
        half_y: 0.5,
    };
    w.spawn_body(desc(RigidBodyType::Fixed, 0.0, floor, GROUND, [0; 2]));
    let small = ShapeDesc::Cuboid {
        half_x: 0.2,
        half_y: 0.2,
    };
    let empty = w.spawn_body(desc(RigidBodyType::Dynamic, 1.0, small, 2, [0; 2]));
    let full = w.spawn_body(desc(RigidBodyType::Dynamic, 1.0, small, 0, [0; 2]));
    w.step();
    for layer in [0u8, 2] {
        assert!(
            w.cast_ray([3.0, 1.0], [0.0, -1.0], 2.0, None, layer)
                .is_none(),
            "a consulta da camada {layer} não vê a linha vazia"
        );
    }
    for _ in 0..120 {
        w.step();
    }
    for b in [empty, full] {
        assert!(
            w.body_pose(b).expect("corpo").translation.y < 0.0,
            "atravessou"
        );
    }
}
