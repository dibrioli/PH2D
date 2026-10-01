//! As funções puras da ponte da navegação: a forma de um obstáculo e o raio de um agente.

use super::*;

fn desc(shape: ShapeDesc, x: f32, y: f32, rot: f32, offset: [f32; 2]) -> BodyDesc {
    BodyDesc {
        body_type: ph2d_physics::RigidBodyType::Fixed,
        x,
        y,
        rotation: rot,
        density: 1.0,
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
        offset,
    }
}

/// Uma caixa rodada 90° com offset: o offset roda com o corpo, e a caixa troca de eixos.
#[test]
fn a_caixa_rodada_com_offset_cai_onde_o_solver_a_poe() {
    let d = desc(
        ShapeDesc::Cuboid {
            half_x: 2.0,
            half_y: 0.5,
        },
        10.0,
        0.0,
        std::f32::consts::FRAC_PI_2,
        [1.0, 0.0],
    );
    let Shape::Convex(pts) = forma(&d) else {
        panic!("uma caixa é convexa");
    };
    // Centro em (10, 1) — o offset `[1, 0]` rodado 90° é `[0, 1]`; meias-medidas trocadas.
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for p in &pts {
        for k in 0..2 {
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    }
    for (a, b) in [(lo[0], 9.5), (hi[0], 10.5), (lo[1], -1.0), (hi[1], 3.0)] {
        assert!((a - b).abs() < 1e-5, "{lo:?}..{hi:?}");
    }
}

/// O raio derivado envolve o colisor inteiro, contado do centro do corpo.
#[test]
fn o_raio_derivado_envolve_o_colisor_e_o_offset() {
    let bola = desc(ShapeDesc::Ball { radius: 0.4 }, 0.0, 0.0, 0.0, [0.3, 0.4]);
    assert!((raio_que_envolve(&bola) - 0.9).abs() < 1e-6);
    let caixa = desc(
        ShapeDesc::Cuboid {
            half_x: 0.3,
            half_y: 0.4,
        },
        0.0,
        0.0,
        0.0,
        [0.0, 0.0],
    );
    assert!((raio_que_envolve(&caixa) - 0.5).abs() < 1e-6);
}
