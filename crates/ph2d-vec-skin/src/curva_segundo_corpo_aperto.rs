//! A13 — o `fecha` do ajuste recusa a cúbica que aperta mais que a fonte. Irmão de [`super`].

use kurbo::Point;

/// ⭐ **A cúbica não aperta mais que a fonte** (A13): o menor raio de curvatura dela no alcance tem
/// de ficar acima de `1 / APERTO_DA_CUBICA` do menor da fonte (o assado denso) — ou acima do
/// comprimento do pedaço. Medido na barra em C
/// a `45°`/`60°` com o bake refinado: o `fit_to_cubic` aceitava cúbicas `12×`–`224×` mais apertadas
/// que a fonte (raio `0,0174`/`0,0008` contra `0,21`/`0,19`), dentro da tolerância de distância — e
/// a bola arredondava esse bico fora do contacto. As cúbicas sãs leem `1,0`–`1,1`.
pub(super) fn curva_como_a_fonte(fonte: &[Point], c: kurbo::CubicBez) -> bool {
    use kurbo::ParamCurveCurvature;
    const APERTO_DA_CUBICA: f64 = 2.0;
    let raio3 = |a: Point, b: Point, d: Point| {
        let cr = (b - a).cross(d - a).abs();
        if cr < 1e-300 {
            f64::INFINITY
        } else {
            (b - a).hypot() * (d - b).hypot() * (d - a).hypot() / (2.0 * cr)
        }
    };
    let da_fonte = fonte
        .windows(3)
        .map(|w| raio3(w[0], w[1], w[2]))
        .fold(f64::INFINITY, f64::min);
    // `64` passos, e mais junto das pontas (o braço curto do ajuste aperta a ponta).
    let pontas = [1.0 / 512.0, 1.0 / 256.0, 1.0 / 128.0];
    let ts = (0..=64)
        .map(|i| f64::from(i) / 64.0)
        .chain(pontas.iter().flat_map(|t| [*t, 1.0 - t]));
    let da_cubica = ts
        .map(|t| 1.0 / c.curvature(t).abs().max(1e-300))
        .fold(f64::INFINITY, f64::min);
    // ⚠️ Só um bico CURTO conta: um raio maior que o próprio pedaço é uma cúbica quase recta, e numa
    // fonte recta (raio infinito) toda cúbica real seria recusada até ao fundo da recursão.
    let comprimento: f64 = fonte.windows(2).map(|w| (w[1] - w[0]).hypot()).sum();
    da_cubica * APERTO_DA_CUBICA >= da_fonte || da_cubica >= comprimento
}

#[cfg(test)]
mod tests {
    use super::super::Assado;

    /// ⭐ GATE — **uma cúbica com um BICO na ponta não passa** (A13): sobre um arco suave (raio `10`),
    /// o arco como uma cúbica passa e a MESMA com o braço de partida encolhido a `2 %` (o braço curto que
    /// o `fit_to_cubic` deixa) — dentro da tolerância de distância — não passa.
    #[test]
    fn uma_cubica_com_bico_na_ponta_nao_passa() {
        use kurbo::{ParamCurve, Point};
        let pts: Vec<Point> = (0..=12)
            .map(|i| {
                let a = 0.2 * f64::from(i) / 12.0;
                Point::new(10.0 * a.sin(), 10.0 * (1.0 - a.cos()))
            })
            .collect();
        let src = Assado(&pts, None);
        let fonte = src.densos(&(0.0..1.0));
        // O arco como UMA cúbica (alças `4/3·tan(θ/4)·R` nas tangentes do círculo).
        let (r, th) = (10.0_f64, 0.2_f64);
        let k = 4.0 / 3.0 * (th / 4.0).tan() * r;
        let fim = Point::new(r * th.sin(), r * (1.0 - th.cos()));
        let sa = kurbo::CubicBez::new(
            Point::ZERO,
            Point::new(k, 0.0),
            fim - kurbo::Vec2::new(th.cos(), th.sin()) * k,
            fim,
        );
        assert!(
            super::curva_como_a_fonte(&fonte, sa),
            "a cúbica sã foi recusada"
        );
        let bico = kurbo::CubicBez::new(sa.p0, sa.p0 + (sa.p1 - sa.p0) * 0.02, sa.p2, sa.p3);
        let longe = (0..=64)
            .map(|i| {
                let p = bico.eval(f64::from(i) / 64.0);
                fonte
                    .iter()
                    .map(|q| (*q - p).hypot())
                    .fold(f64::INFINITY, f64::min)
            })
            .fold(0.0, f64::max);
        assert!(
            longe < 0.05,
            "a fixtura saiu da tolerância ({longe}) — já não mede o bico"
        );
        assert!(
            !super::curva_como_a_fonte(&fonte, bico),
            "a cúbica com o bico na ponta passou"
        );
    }
}
