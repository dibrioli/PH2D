//! Gauss–Kronrod 7/15 adaptativo, determinístico (a mesma árvore de bissecção em toda corrida).
//! Nós e pesos: QUADPACK (Piessens et al., 1983), `qk15`.

const XGK: [f64; 8] = [
    0.991_455_371_120_812_6,
    0.949_107_912_342_758_5,
    0.864_864_423_359_769_1,
    0.741_531_185_599_394_4,
    0.586_087_235_467_691_1,
    0.405_845_151_377_397_2,
    0.207_784_955_007_898_5,
    0.0,
];
const WGK: [f64; 8] = [
    0.022_935_322_010_529_22,
    0.063_092_092_629_978_55,
    0.104_790_010_322_250_2,
    0.140_653_259_715_525_9,
    0.169_004_726_639_267_9,
    0.190_350_578_064_785_4,
    0.204_432_940_075_298_9,
    0.209_482_141_084_727_8,
];
/// Pesos de Gauss-7 nos nós ímpares de `XGK` (1, 3, 5) e no centro.
const WG: [f64; 4] = [
    0.129_484_966_168_869_7,
    0.279_705_391_489_276_7,
    0.381_830_050_505_118_9,
    0.417_959_183_673_469_4,
];

/// Profundidade máxima da bissecção: `2⁻⁴⁰` da largura inicial está abaixo de um ulp de ângulo.
const PROFUNDIDADE: u32 = 40;

fn qk15(a: f64, b: f64, f: &mut dyn FnMut(f64) -> f64) -> (f64, f64) {
    let c = 0.5 * (a + b);
    let h = 0.5 * (b - a);
    let fc = f(c);
    let mut k = WGK[7] * fc;
    let mut g = WG[3] * fc;
    for j in 0..7 {
        let x = h * XGK[j];
        let s = f(c - x) + f(c + x);
        k += WGK[j] * s;
        if j % 2 == 1 {
            g += WG[j / 2] * s;
        }
    }
    (k * h, g * h)
}

/// `∫ₐᵇ f` para `|f| ≤ 1`, refinando cada pedaço até `|K − G| ≤ 1e-13·(b − a)` da largura
/// INICIAL. ⚠️ A tolerância é absoluta e fixa de propósito: na tangente a um disco o integrando
/// tem singularidade `√`, e uma tolerância proporcional à largura do PEDAÇO nunca é satisfeita
/// por ele nem pelos vizinhos — a árvore explodia (medido: 31 s num ponto, G10 sem fim). Fixa,
/// ela dá uma malha graduada de profundidade linear junto à singularidade.
pub(crate) fn gauss_kronrod(a: f64, b: f64, f: &mut dyn FnMut(f64) -> f64) -> f64 {
    fn passo(a: f64, b: f64, f: &mut dyn FnMut(f64) -> f64, tol: f64, nivel: u32) -> f64 {
        let (k, g) = qk15(a, b, f);
        if nivel == PROFUNDIDADE || (k - g).abs() <= tol {
            return k;
        }
        let m = 0.5 * (a + b);
        passo(a, m, f, tol, nivel + 1) + passo(m, b, f, tol, nivel + 1)
    }
    passo(a, b, f, 1e-13 * (b - a), 0)
}
