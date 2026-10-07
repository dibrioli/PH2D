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
    let ph2d_navmesh::Shape::Convex(pts) = malha::forma(&d, libm::sincosf(d.rotation)) else {
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

/// ⭐ (W11, plano 30 §19) **As paredes da ponte, por mosaicos, são as da malha inteira, ao bit** — pela
/// porta que o desvio usa ([`desvio::ParedesDaMalha::monta`]), com a mesma `ParedesDaMalha` a VIVER
/// entre mudanças: obstáculos que andam, que entram e saem, e a região que encolhe (mosaicos que saem).
#[test]
fn as_paredes_da_ponte_por_mosaicos_sao_as_da_malha_inteira() {
    use ph2d_navmesh::{Params, Shape, TiledMesh};
    let mut s = 0x0011_5EEDu64;
    let mut r = move || {
        s = s
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (s >> 11) as f64 / (1u64 << 53) as f64
    };
    let mut obs: Vec<Shape> = (0..60)
        .map(|_| Shape::Circle {
            center: [r() * 45.0, r() * 30.0],
            radius: 0.3 + r() * 1.5,
        })
        .collect();
    let mut reg = vec![[0.0, 0.0], [45.0, 0.0], [45.0, 30.0], [0.0, 30.0]];
    let mut t = TiledMesh::new(Params::default(), 10.0);
    let mut pm = desvio::ParedesDaMalha::default();
    let (mut atravessam, mut sairam) = (0usize, 0usize);
    for passo in 0..24 {
        if passo > 0 {
            let i = (r() * obs.len() as f64) as usize % obs.len();
            obs[i] = Shape::Circle {
                center: [r() * 45.0, r() * 30.0],
                radius: 0.3 + r() * 1.5,
            };
        }
        if passo == 16 {
            reg = vec![[0.0, 0.0], [28.0, 0.0], [28.0, 19.0], [0.0, 19.0]];
            sairam += 1;
        }
        t.update(&reg, &obs);
        let w = pm.paredes(&t).clone();
        let o = ph2d_orca::Walls::from_walkable_walls(t.mesh().verts(), t.mesh().walls());
        let b = |p: ph2d_nav::V2| [p[0].to_bits(), p[1].to_bits()];
        let faixas = t.paredes_por_mosaico();
        let mosaico_de = |i: usize| faixas.partition_point(|f| f.paredes.end <= i);
        assert_eq!(w.len(), o.len(), "passo {passo}");
        for i in 0..o.len() {
            assert_eq!(b(w.point(i)), b(o.point(i)), "passo {passo}: point {i}");
            assert_eq!(w.next(i), o.next(i), "passo {passo}: next {i}");
            assert_eq!(w.prev(i), o.prev(i), "passo {passo}: prev {i}");
            assert_eq!(b(w.dir(i)), b(o.dir(i)), "passo {passo}: dir {i}");
            assert_eq!(w.convex(i), o.convex(i), "passo {passo}: convex {i}");
            atravessam += usize::from(mosaico_de(i) != mosaico_de(o.next(i)));
        }
        let (mut x, mut y) = (Vec::new(), Vec::new());
        for _ in 0..200 {
            let q = [r() * 46.0 - 0.5, r() * 31.0 - 0.5];
            let a = r() * 5.0;
            w.near(q, a, &mut x);
            o.near(q, a, &mut y);
            assert_eq!(x, y, "passo {passo}: near {q:?} {a}");
        }
    }
    // CONTROLOS de população: cadeias que atravessam costuras e mosaicos que saíram.
    assert!(
        atravessam >= 900,
        "só {atravessam} entradas seguem para outro mosaico (medido: 1 116)"
    );
    assert_eq!(sairam, 1);
}

/// (report do dono, 05/10) A distância à forma de um corpo que não é bola (a saída de um teletransporte
/// tem de a respeitar): zero dentro, a perpendicular ao lado, e a do canto fora dele.
#[test]
fn a_distancia_a_uma_caixa_e_zero_dentro_e_a_do_canto_fora() {
    let caixa = [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]];
    let d = |p| super::desvio::distancia_ao_poligono(&caixa, p);
    assert_eq!(d([0.2, -0.3]), 0.0, "dentro");
    assert!((d([3.0, 0.5]) - 2.0).abs() < 1e-12, "ao lado");
    assert!((d([4.0, 5.0]) - 5.0).abs() < 1e-12, "do canto (3, 4, 5)");
}

/// ⭐ (plano 30 §25, B) **A tangente do caminho a um corpo que anda**: a barreira `[2,9; 3,1] × [−1,5;
/// 1,5]` à frente de quem vai de `(−6, y)` para `+x`. No eixo (o empate) contorna pela DIREITA (`y < 0`,
/// o lado do peso do desvio); acima do eixo, pela esquerda (o lado mais curto); a tangente passa a `r`
/// da quina. CONTROLOS: o canto antes da barreira e a barreira fora do troço não mudam nada.
#[test]
fn o_caminho_contorna_um_corpo_pela_tangente_do_lado_mais_curto() {
    let b = [[2.9, -1.5], [3.1, -1.5], [3.1, 1.5], [2.9, 1.5]];
    let r = 0.6;
    let d = desvio::contorna([-6.0, 0.0], [1.0, 0.0], 12.0, r, &b).expect("no caminho");
    assert!(d[1] < 0.0, "o empate vai pela direita: {d:?}");
    // A tangente passa a `r` da quina mais extrema do lado escolhido: `(2,9; −1,5)`, a de MAIOR ângulo
    // vista de `(−6, 0)` (a de trás, `(3,1; −1,5)`, fica dentro dela).
    let quina = [2.9_f64, -1.5];
    let (wx, wy) = (quina[0] + 6.0, quina[1]);
    let afast = (wx * d[1] - wy * d[0]).abs();
    assert!(
        (afast - r).abs() < 1e-9,
        "a tangente passa a {afast} da quina"
    );
    assert!(((d[0] * d[0] + d[1] * d[1]).sqrt() - 1.0).abs() < 1e-12);
    let d = desvio::contorna([-6.0, 0.4], [1.0, 0.0], 12.0, r, &b).expect("no caminho");
    assert!(d[1] > 0.0, "acima do eixo, pela esquerda: {d:?}");
    // CONTROLOS: o canto antes da barreira (`8 m`, a `2,3` dela) e o troço que passa por cima dela.
    assert_eq!(desvio::contorna([-6.0, 0.0], [1.0, 0.0], 8.0, r, &b), None);
    assert_eq!(desvio::contorna([-6.0, 2.2], [1.0, 0.0], 12.0, r, &b), None);
}
