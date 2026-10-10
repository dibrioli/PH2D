//! **A VINHETA** — uma superelipse com a borda borrada por uma gaussiana, calculada EXACTAMENTE.
//!
//! O oráculo (ImageMagick `-vignette`, doc 129 §5.1) ensinou a lei: a máscara é o indicador da
//! elipse convolvido com uma gaussiana `σ`. A aproximação `Φ(distância/σ)` erra até `0,23` com a
//! borda larga (medido); a convolução exacta é uma integral 1D por pixel — em cada linha `y'` a
//! forma ocupa `[-h(y'), h(y')]`, e o borrão em `x` dessa faixa é uma diferença de `Φ`.

/// Os nós da quadratura do ponto médio sobre `[y-5σ, y+5σ]`: 16 e 64 dão o MESMO erro contra o
/// oráculo (medido, as quatro configurações), logo 16.
pub const NOS: u32 = 16;

/// Os params do artista (os quatro da vinheta do colorista, mais o centro e a cor).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vinheta {
    /// `-1..1`: positivo escurece para [`Self::cor`], negativo clareia para o branco. `0` desliga.
    pub amount: f32,
    /// O tamanho da forma: `1` inscreve-a no quadro.
    pub midpoint: f32,
    /// `-1..1`: `0` segue o quadro, `1` é um círculo, `-1` é quase um retângulo (expoente 8).
    pub roundness: f32,
    /// `0..1`: o `σ` da borda é `feather × ¼` do lado menor.
    pub feather: f32,
    /// O deslocamento do centro, em metades do quadro (`0` = o meio).
    pub center: [f32; 2],
    /// A cor para onde escurece (RGB linear).
    pub cor: [f32; 3],
}

impl Vinheta {
    /// Desligada: o passe salta-a.
    #[must_use]
    pub fn neutra(&self) -> bool {
        self.amount == 0.0
    }
}

/// A forma em pixels, derivada uma vez por quadro.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Geometria {
    pub centro: [f32; 2],
    pub semi: [f32; 2],
    pub expoente: f32,
    pub sigma: f32,
}

/// O `σ` mínimo: `feather = 0` é uma aresta dura, e a quadratura continua definida.
pub const SIGMA_MIN: f32 = 1e-3;

#[must_use]
pub fn geometria(v: &Vinheta, quadro: [f32; 2]) -> Geometria {
    let [w, h] = quadro;
    let m = v.midpoint.max(0.0);
    let r = v.roundness.clamp(-1.0, 1.0);
    let base = [m * w * 0.5, m * h * 0.5];
    let raio = m * (w * h).sqrt() * 0.5;
    let semi = if r > 0.0 {
        [
            base[0] + (raio - base[0]) * r,
            base[1] + (raio - base[1]) * r,
        ]
    } else {
        base
    };
    Geometria {
        centro: [w * 0.5 * (1.0 + v.center[0]), h * 0.5 * (1.0 + v.center[1])],
        semi,
        expoente: 2.0 + 6.0 * (-r).max(0.0),
        sigma: (v.feather.clamp(0.0, 1.0) * 0.25 * w.min(h)).max(SIGMA_MIN),
    }
}

/// `Φ` pela fórmula 7.1.26 de Abramowitz & Stegun (`|erro| ≤ 1,5e-7`) — a MESMA nas duas línguas;
/// o `erf` da libm não existe no WGSL.
#[must_use]
pub fn phi(t: f32) -> f32 {
    let z = t.abs() * std::f32::consts::FRAC_1_SQRT_2;
    let k = 1.0 / (1.0 + 0.327_591_1 * z);
    let poli = k
        * (0.254_829_6
            + k * (-0.284_496_74 + k * (1.421_413_7 + k * (-1.453_152_1 + k * 1.061_405_4))));
    let erf = 1.0 - poli * (-z * z).exp();
    if t >= 0.0 {
        0.5 + 0.5 * erf
    } else {
        0.5 - 0.5 * erf
    }
}

/// A meia-largura da superelipse na linha `y` (relativa ao centro).
#[must_use]
pub fn meia_largura(y: f32, semi: [f32; 2], expoente: f32) -> f32 {
    let t = (y / semi[1]).abs();
    if t >= 1.0 {
        return 0.0;
    }
    semi[0] * (1.0 - t.powf(expoente)).powf(1.0 / expoente)
}

/// **A máscara** em `q` (relativo ao centro, em px): `1` dentro, `0` longe fora.
#[must_use]
pub fn mascara(q: [f32; 2], semi: [f32; 2], expoente: f32, sigma: f32) -> f32 {
    if semi[0] <= 0.0 || semi[1] <= 0.0 {
        return 0.0;
    }
    let [x, y] = q;
    let lo = (-semi[1]).max(y - 5.0 * sigma);
    let hi = semi[1].min(y + 5.0 * sigma);
    if hi <= lo {
        return 0.0;
    }
    let passo = (hi - lo) / NOS as f32;
    let mut soma = 0.0;
    for k in 0..NOS {
        let yl = lo + (k as f32 + 0.5) * passo;
        let h = meia_largura(yl, semi, expoente);
        let d = (y - yl) / sigma;
        soma += (-0.5 * d * d).exp() * (phi((h - x) / sigma) - phi((-h - x) / sigma));
    }
    soma * passo / (sigma * (2.0 * std::f32::consts::PI).sqrt())
}

/// A vinheta aplicada a uma cor LINEAR no pixel de centro `p`.
#[must_use]
pub fn aplica(c: [f32; 3], p: [f32; 2], v: &Vinheta, g: &Geometria) -> [f32; 3] {
    let q = [p[0] - g.centro[0], p[1] - g.centro[1]];
    let w = mascara(q, g.semi, g.expoente, g.sigma);
    let alvo = if v.amount >= 0.0 { v.cor } else { [1.0; 3] };
    let t = v.amount.abs().min(1.0) * (1.0 - w);
    [0, 1, 2].map(|i| c[i] + (alvo[i] - c[i]) * t)
}
