//! A opacidade parcial é POR ENTRADA (`1 − α` cada vez que o raio entra no objeto) e o alcance
//! finito aplica a `queda` à distância do ACERTO — conferidos contra formas fechadas e contra uma
//! quadratura independente.

use ph2d_light2d_ecs::Oclusor;
use ph2d_light2d_ref::fluencia;
use ph2d_light2d_ref::queda::queda;

use crate::comum::{cena, disco, dist, f64x2};

const TOL: f64 = 1e-12;

fn barra(x0: f32, x1: f32) -> Vec<[f32; 2]> {
    vec![[x0, -3.0], [x1, -3.0], [x1, 3.0], [x0, 3.0]]
}

#[test]
fn uma_entrada_num_oclusor_parcial_multiplica_por_1_menos_alfa() {
    let fonte = disco([4.0, 0.3], 0.2, [2.0; 3], f32::INFINITY);
    let p = [0.0, 0.0];
    let vazio = fluencia(&cena(vec![fonte.clone()], vec![]), p)[0];
    assert!(vazio > 0.0, "controlo: sem oclusor a fonte ilumina");
    for alfa in [0.0_f32, 0.25, 0.5, 1.0] {
        let f = fluencia(
            &cena(
                vec![fonte.clone()],
                vec![Oclusor {
                    vertices: barra(1.0, 1.4),
                    opacidade: alfa,
                    albedo: [0.0; 3],
                }],
            ),
            p,
        )[0];
        let esperado = (1.0 - f64::from(alfa)) * vazio;
        assert!(
            (f - esperado).abs() <= TOL * vazio,
            "α={alfa}: {f} × {esperado}"
        );
    }
}

/// Um «U» côncavo: os raios entram DUAS vezes no MESMO objeto ⇒ `(1 − α)²`.
#[test]
fn duas_entradas_no_mesmo_oclusor_concavo_multiplicam_duas_vezes() {
    let fonte = disco([4.0, 0.3], 0.2, [2.0; 3], f32::INFINITY);
    let u = vec![
        [1.0, -3.0],
        [1.2, -3.0],
        [1.2, 3.0],
        [2.0, 3.0],
        [2.0, -3.0],
        [2.2, -3.0],
        [2.2, 3.5],
        [1.0, 3.5],
    ];
    let p = [0.0, 0.0];
    let vazio = fluencia(&cena(vec![fonte.clone()], vec![]), p)[0];
    assert!(vazio > 0.0, "controlo: sem oclusor a fonte ilumina");
    let alfa = 0.5_f32;
    let f = fluencia(
        &cena(
            vec![fonte],
            vec![Oclusor {
                vertices: u,
                opacidade: alfa,
                albedo: [0.0; 3],
            }],
        ),
        p,
    )[0];
    let esperado = (1.0 - f64::from(alfa)).powi(2) * vazio;
    assert!((f - esperado).abs() <= TOL * vazio, "{f} × {esperado}");
}

/// Quadratura independente: `θ = δ·sin u` remove a singularidade de `√` nas tangentes, e Simpson
/// com 20 000 painéis sobre `u` converge muito abaixo de `TOL`.
fn por_quadratura(a: f64, r: f64, l: f64, alcance: f64) -> f64 {
    let delta = (a / r).asin();
    let integrando = |u: f64| {
        let th = delta * u.sin();
        let disc = (a * a - r * r * th.sin() * th.sin()).max(0.0);
        let t = r * th.cos() - disc.sqrt();
        l * queda(t, alcance) * delta * u.cos()
    };
    let n = 20_000;
    let h = std::f64::consts::PI / f64::from(n);
    let mut soma =
        integrando(-std::f64::consts::FRAC_PI_2) + integrando(std::f64::consts::FRAC_PI_2);
    for k in 1..n {
        let u = -std::f64::consts::FRAC_PI_2 + h * f64::from(k);
        soma += if k % 2 == 1 { 4.0 } else { 2.0 } * integrando(u);
    }
    soma * h / 3.0
}

#[test]
fn o_alcance_finito_aplica_a_queda_a_distancia_do_acerto() {
    let c = [0.4_f32, -0.2];
    let l = 1.5_f32;
    for (a, r, alcance) in [
        (0.3_f32, 1.0, 1.5_f32),
        (0.5, 2.0, 2.2),
        (0.1, 0.5, 3.0),
        (0.8, 1.2, 0.9),
    ] {
        let p = [f64::from(c[0]) + r * 0.6, f64::from(c[1]) + r * 0.8];
        let r = dist(p, f64x2(c));
        let f = fluencia(&cena(vec![disco(c, a, [l; 3], alcance)], vec![]), p)[0];
        let esperado = por_quadratura(f64::from(a), r, f64::from(l), f64::from(alcance));
        assert!(
            esperado > 0.0,
            "controlo: a fixture tem luz dentro do alcance"
        );
        assert!(
            (f - esperado).abs() <= 1e-10 * esperado.max(1e-300),
            "a={a} r={r} R={alcance}: {f} × {esperado}"
        );
    }
    // Todo o disco além do alcance ⇒ zero exato.
    let f = fluencia(&cena(vec![disco(c, 0.2, [l; 3], 1.0)], vec![]), [3.0, 3.0])[0];
    assert_eq!(f, 0.0);
}
