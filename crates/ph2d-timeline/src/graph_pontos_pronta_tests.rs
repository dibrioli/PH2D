use super::*;
use ph2d_anim::{Easing, EasingFamily as F, EasingMode as M};

/// Cada curva pronta convertida tem a cara dela e deixa lugar ao ponto do artista. Medido 15/11
/// (erro máximo no quadrado unitário): Quad, Cubic, Back e Bounce In/Out exatos; Sine, Expo,
/// Circ e Elastic In/Out ≤ 1,1 %; Elastic e Bounce In-out ~4,5 % — ondas a mais para os
/// `SPLINE_MAX_POINTS` (o tecto de memória da `Interp`).
#[test]
fn a_curva_pronta_convertida_tem_a_mesma_cara_e_deixa_lugar_ao_ponto() {
    for fam in [
        F::Sine,
        F::Quad,
        F::Cubic,
        F::Expo,
        F::Circ,
        F::Back,
        F::Elastic,
        F::Bounce,
    ] {
        for modo in [M::In, M::Out, M::InOut] {
            let interp = Interp::Eased(Easing::new(fam, modo));
            let c = preset_as_curve(interp).expect("curva pronta");
            assert!(
                c.points().len() < SPLINE_MAX_POINTS,
                "{fam:?} {modo:?}: sem lugar"
            );
            let erro = (0..=400)
                .map(|i| f64::from(i) / 400.0)
                .map(|x| (c.eval(x) - interp.remap(x)).abs())
                .fold(0.0, f64::max);
            let teto = if matches!((fam, modo), (F::Elastic | F::Bounce, M::InOut)) {
                0.05
            } else {
                0.012
            };
            assert!(erro < teto, "{fam:?} {modo:?}: erro {erro}");
        }
    }
    let exatas = [
        (F::Quad, M::InOut),
        (F::Back, M::InOut),
        (F::Bounce, M::Out),
    ];
    for (fam, modo) in exatas {
        let interp = Interp::Eased(Easing::new(fam, modo));
        let c = preset_as_curve(interp).unwrap();
        let erro = (0..=400)
            .map(|i| f64::from(i) / 400.0)
            .map(|x| (c.eval(x) - interp.remap(x)).abs())
            .fold(0.0, f64::max);
        assert!(erro < 1.0e-3, "{fam:?} {modo:?} devia ser exata: {erro}");
    }
    assert!(preset_as_curve(Interp::Hold).is_none());
    assert!(preset_as_curve(Interp::Linear).is_none());
}
