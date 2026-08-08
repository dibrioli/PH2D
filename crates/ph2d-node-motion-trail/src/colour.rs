//! **O que o rastro faz com a COR de um eco** — a metade do padrão-ouro que faltava
//! (doc 88 §B3, ordem do Enio de 2026-08-08: *"todos os features de outros apps e mais
//! alguns"*).
//!
//! O catálogo de referência dá ao rastro `hueShift` e `satMin/Max` (*"partícula com cauda
//! colorida"* é o caso de uso que ele próprio nomeia) e nós tínhamos só a alfa. Aqui
//! estão os dois, como **operadores por TICK** — a mesma forma geométrica do `fade` e do
//! `shrink`, então um eco de `n` ticks recebeu o operador exatamente `n` vezes e a
//! semântica *"por eco"* cai de graça.
//!
//! ## ⚠️ Por que NÃO é OKLCH aqui, e a recusa anterior estava na camada errada
//!
//! Uma nota anterior deste módulo recusava o `hueShift` dizendo que *"a cor neste app
//! passa por OKLCH"*. Isso é verdade da **AUTORIA** — o picker, o editor de gradiente, a
//! paleta — e **falso do COOK**: a coluna `tint`, o `motion.color_ramp` (*"the ramp is
//! evaluated in linear RGB — the same space the tint column and the compositor use"*) e a
//! lowering inteira falam **linear RGB**. Girar matiz em OKLCH aqui introduziria um
//! segundo espaço de cor no meio do cozimento, e o preço não é teórico: a ida polar é uma
//! `cbrt` + `atan2` **por linha e por tick**, num laço por-elemento.
//!
//! O operador certo nesta camada é a **rotação de matiz que PRESERVA a luma**, uma matriz
//! 3×3 sobre linear RGB — exatamente o `feColorMatrix type="hueRotate"` do SVG, que é
//! especificado em linearRGB pelo mesmo motivo. Os pesos são os do **Rec.709 linear**
//! (0.213 / 0.715 / 0.072), e o `sincos` roda **uma vez por tick**, nunca por linha.

/// Pesos de luma do Rec.709 em luz linear — os mesmos que o `feColorMatrix` do SVG usa.
const LUMA: [f32; 3] = [0.213, 0.715, 0.072];

/// Uma matriz 3×3 linha-a-linha sobre RGB linear.
pub(crate) type Mat3 = [[f32; 3]; 3];

/// A identidade — o que um ângulo de zero e uma saturação de um produzem, e o que faz do
/// caminho novo um no-op **bit a bit** quando os knobs estão nos defaults.
pub(crate) const IDENTITY: Mat3 = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// **A rotação de matiz que preserva a luma**, por `deg` graus (o `hueRotate` do SVG).
///
/// Construída UMA vez por tick: o `sincos` é de `libm` — porte puro-Rust do MUSL, o mesmo
/// pin `=0.2.16` que cinco crates deste repo já usam — para que o mesmo documento dê a
/// mesma cor em Linux, macOS e Windows. Um `f32::sin` da plataforma não é especificado ao
/// ulp e faria a arte divergir entre máquinas.
#[must_use]
pub(crate) fn hue_rotation(deg: f32) -> Mat3 {
    if deg == 0.0 || !deg.is_finite() {
        return IDENTITY;
    }
    let (s, c) = libm::sincosf(deg.to_radians());
    let [lr, lg, lb] = LUMA;
    [
        [
            lr + c * (1.0 - lr) - s * lr,
            lg - c * lg - s * lg,
            lb - c * lb + s * (1.0 - lb),
        ],
        [
            lr - c * lr + s * 0.143,
            lg + c * (1.0 - lg) + s * 0.140,
            lb - c * lb - s * 0.283,
        ],
        [
            lr - c * lr - s * (1.0 - lr),
            lg - c * lg + s * lg,
            lb + c * (1.0 - lb) + s * lb,
        ],
    ]
}

/// **A saturação como matriz**, para compor com a matiz numa multiplicação só.
///
/// `k = 1` é a identidade, `k = 0` colapsa na luma (cinza), `k > 1` satura. É a mesma
/// forma do `feColorMatrix type="saturate"`.
#[must_use]
pub(crate) fn saturation(k: f32) -> Mat3 {
    if k == 1.0 || !k.is_finite() {
        return IDENTITY;
    }
    let [lr, lg, lb] = LUMA;
    [
        [lr + k * (1.0 - lr), lg - k * lg, lb - k * lb],
        [lr - k * lr, lg + k * (1.0 - lg), lb - k * lb],
        [lr - k * lr, lg - k * lg, lb + k * (1.0 - lb)],
    ]
}

/// `a · b` — compor os dois operadores num só, para o laço por-linha fazer **nove
/// multiplicações e mais nada**, sejam um, dois ou nenhum knob armados.
#[must_use]
pub(crate) fn compose(a: Mat3, b: Mat3) -> Mat3 {
    let mut m = [[0.0f32; 3]; 3];
    for (r, row) in m.iter_mut().enumerate() {
        for (c, out) in row.iter_mut().enumerate() {
            *out = a[r][0] * b[0][c] + a[r][1] * b[1][c] + a[r][2] * b[2][c];
        }
    }
    m
}

/// Aplica o operador ao RGB de um `tint`, **deixando a alfa intocada** — ela é território
/// do `fade`, e um operador de cor que mexesse nela seria uma segunda porta para o mesmo
/// número.
///
/// ⚠️ **Sem clamp, de propósito:** a coluna `tint` é linear e o compositor deste app é
/// HDR-tolerante; cravar em `[0,1]` aqui aplainaria um realce que a rotação legitimamente
/// produz, e o lugar de decidir gamut é a saída, não o meio da cadeia.
pub(crate) fn apply(m: Mat3, tint: &mut [f32; 4]) {
    let [r, g, b, _] = *tint;
    tint[0] = m[0][0] * r + m[0][1] * g + m[0][2] * b;
    tint[1] = m[1][0] * r + m[1][1] * g + m[1][2] * b;
    tint[2] = m[2][0] * r + m[2][1] * g + m[2][2] * b;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn luma(c: [f32; 4]) -> f32 {
        LUMA[0] * c[0] + LUMA[1] * c[1] + LUMA[2] * c[2]
    }

    /// **A rotação PRESERVA a luma** — é o que separa um giro de matiz de um filtro que
    /// escurece a cauda. O oráculo é a grandeza que o operador promete conservar.
    #[test]
    fn the_hue_rotation_keeps_the_luma() {
        for deg in [10.0, 90.0, 180.0, 270.0, -45.0] {
            let m = hue_rotation(deg);
            for base in [[0.8, 0.2, 0.1, 1.0], [0.1, 0.6, 0.9, 0.5]] {
                let mut c = base;
                apply(m, &mut c);
                assert!(
                    (luma(c) - luma(base)).abs() < 1e-4,
                    "{deg}deg mudou a luma de {} para {}",
                    luma(base),
                    luma(c)
                );
                assert_eq!(c[3], base[3], "a alfa e territorio do Fade");
            }
        }
    }

    /// Ângulo zero e saturação um são a IDENTIDADE **ao bit** — o que torna o caminho novo
    /// invisível nos defaults, e é o que protege toda arte já autorada.
    #[test]
    fn the_neutral_knobs_are_the_identity_to_the_bit() {
        assert_eq!(hue_rotation(0.0), IDENTITY);
        assert_eq!(saturation(1.0), IDENTITY);
        assert_eq!(compose(IDENTITY, IDENTITY), IDENTITY);
        let mut c = [0.3, 0.7, 0.11, 0.4];
        let before = c;
        apply(compose(hue_rotation(0.0), saturation(1.0)), &mut c);
        assert_eq!(c, before);
    }

    /// Saturação zero colapsa na luma (cinza puro) e a luma sobrevive — as duas metades.
    #[test]
    fn zero_saturation_is_grey_at_the_same_luma() {
        let base = [0.9, 0.2, 0.4, 1.0];
        let mut c = base;
        apply(saturation(0.0), &mut c);
        assert!(
            (c[0] - c[1]).abs() < 1e-6 && (c[1] - c[2]).abs() < 1e-6,
            "cinza: {c:?}"
        );
        assert!((luma(c) - luma(base)).abs() < 1e-4);
    }

    /// **Girar 360 graus volta ao mesmo lugar** — a propriedade de GRUPO da rotação, que
    /// um sinal trocado ou um peso errado quebra sem mexer na luma.
    #[test]
    fn a_full_turn_comes_back() {
        let base = [0.8, 0.25, 0.1, 1.0];
        let mut c = base;
        apply(hue_rotation(360.0), &mut c);
        for i in 0..3 {
            assert!(
                (c[i] - base[i]).abs() < 1e-3,
                "volta inteira: {c:?} vs {base:?}"
            );
        }
    }

    /// Compor duas rotações é rotacionar pela SOMA — o que garante que aplicar o operador
    /// uma vez por tick dá, num eco de `n` ticks, exatamente `n·deg`.
    #[test]
    fn composing_two_rotations_adds_their_angles() {
        let base = [0.7, 0.3, 0.15, 1.0];
        let (mut twice, mut once) = (base, base);
        apply(hue_rotation(30.0), &mut twice);
        apply(hue_rotation(30.0), &mut twice);
        apply(hue_rotation(60.0), &mut once);
        for i in 0..3 {
            assert!(
                (twice[i] - once[i]).abs() < 1e-3,
                "30+30 tinha de ser 60: {twice:?} vs {once:?}"
            );
        }
    }
}
