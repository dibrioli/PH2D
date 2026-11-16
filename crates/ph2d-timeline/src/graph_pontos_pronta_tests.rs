use super::*;
use ph2d_anim::{Easing, EasingFamily as F, EasingMode as M};

/// Cada curva pronta convertida tem a cara dela, deixa lugar ao ponto do artista e não cola pontos.
/// Medido 16/11 (erro máximo no quadrado unitário, alças afinadas por trecho, nós a ≥ `ESPACO_MIN`):
/// Quad, Cubic, Back e Bounce (os três modos) exatos; Sine, Expo, Circ e Elastic In/Out ≤ 0,8 %;
/// Elastic In-out 2,8 % (antes 4,1 %; o Bounce In-out era 4,7 % — os cantos não cabiam junto com
/// as cristas). Os nós mais colados: 4,5 % (os cantos do Bounce In-out).
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
            let teto = if (fam, modo) == (F::Elastic, M::InOut) {
                0.0285
            } else {
                0.008
            };
            let xs: Vec<f64> = std::iter::once(0.0)
                .chain(c.points().iter().map(|p| f64::from(p.p[0])))
                .chain([1.0])
                .collect();
            let colados = xs.windows(2).map(|w| w[1] - w[0]).fold(1.0, f64::min);
            assert!(colados >= 0.04, "{fam:?} {modo:?}: nós a {colados}");
            assert!(erro < teto, "{fam:?} {modo:?}: erro {erro}");
        }
    }
    let exatas = [
        (F::Quad, M::InOut),
        (F::Back, M::InOut),
        (F::Bounce, M::Out),
        (F::Bounce, M::In),
        (F::Bounce, M::InOut),
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
