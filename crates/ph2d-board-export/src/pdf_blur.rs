//! O desfoque das sombras do PDF (módulo filho de [`super`]): a máscara de uma sombra desfocada, pela
//! função de erro (a gaussiana integrada).

/// `erf` (Abramowitz & Stegun 7.1.26, erro < 1,5e-7).
fn erf(x: f64) -> f64 {
    let s = x.signum();
    let x = x.abs();
    let t = 1.0 / (1.0 + 0.327_591_1 * x);
    let y = 1.0
        - (((((1.061_405_429 * t - 1.453_152_027) * t) + 1.421_413_741) * t - 0.284_496_736) * t
            + 0.254_829_592)
            * t
            * (-x * x).exp();
    s * y
}

/// A máscara (cinzentos = alfa) do rectângulo `[w, h]` desfocado por `σ`, sobre a caixa alargada de
/// `3σ` de cada lado: o desfoque gaussiano de uma caixa é separável — o produto de dois `erf`.
pub(super) fn blur_mask(w: f64, h: f64, sigma: f64) -> (u32, u32, Vec<u8>) {
    let k = 3.0 * sigma;
    let (bw, bh) = (w + 2.0 * k, h + 2.0 * k);
    // Dois píxeis por σ chegam (é um desfoque), e o lado maior não passa de 256.
    let per = (2.0 / sigma.max(1e-9)).min(256.0 / bw.max(bh));
    let (n, m) = (
        ((bw * per).ceil() as u32).max(1),
        ((bh * per).ceil() as u32).max(1),
    );
    let s2 = sigma.max(1e-9) * std::f64::consts::SQRT_2;
    let axis = |v: f64, len: f64| 0.5 * (erf((v - k) / s2) - erf((v - k - len) / s2));
    let mut out = Vec::with_capacity((n * m) as usize);
    for j in 0..m {
        let ay = axis((f64::from(j) + 0.5) / per, h);
        for i in 0..n {
            let ax = axis((f64::from(i) + 0.5) / per, w);
            out.push((ax * ay * 255.0).round().clamp(0.0, 255.0) as u8); // CLAMP-OK: alfa em [0, 255]
        }
    }
    (n, m, out)
}
