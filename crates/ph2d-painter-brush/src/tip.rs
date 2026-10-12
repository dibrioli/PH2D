//! **O quadrado unitário do carimbo** — a porta ÚNICA que responde *onde, dentro do stamp cacheado,
//! está esta coordenada de dab, e ela está DENTRO?*
//!
//! ⚠️ **Ela existe porque a resposta estava escrita CINCO vezes.** O motor tem cinco amostradores do
//! mesmo stamp — o de cobertura ([`crate::stamp::sample_mask`]), o de cor
//! ([`crate::stamp_color`]), o do acumulador e os DOIS do acumulador em lote — e cada um repetia a
//! mesma aritmética inline (as duas divisões, o `floor`, o clamp). Em 2026-08-09 o corte *"fora do
//! quadrado unitário não há dab"* foi acrescentado a **um** deles, e os outros quatro continuaram
//! estendendo a borda do carimbo para fora da ponta: o Enio reportou o mesmo defeito de novo, na rota
//! que o checkbox **Use Texture Colors** escolhe (*"o carimbo invadindo o grid à direita e abaixo"*).
//!
//! Uma lista de sítios que precisam lembrar de uma regra apodrece; **uma porta que PRODUZ os texels
//! não**. Aqui não há como calcular os taps sem responder à pergunta — o `None` é o próprio resultado,
//! e um amostrador novo nasce com o corte porque não tem de onde tirar os índices sem ele.
//!
//! ⚠️ **Por EIXO, e não por ponto**, para o inline dos amostradores quentes sobreviver: `v` é
//! invariante na linha (os dois amostradores em lote o resolvem uma vez por linha e caminham em `x`),
//! e uma linha inteira fora da ponta é descartada de uma vez — estritamente MENOS trabalho do que a
//! versão que clampava e depois multiplicava por um peso.

/// ⭐⭐ **Até onde chega o dab**, em raios por eixo: a caixa `base` (a da pegada, ou `[1, 1]`) alargada
/// até à ponta de imagem do Shape quando há uma ([`crate::texture::image_tip_extent`], Bug #52). Todo
/// núcleo que percorre um dab com Shape pergunta AQUI — a ponta rodada ou de Size < 1 sai da caixa
/// `± raio`, e a caixa que a cortava era o defeito. Sem ponta de imagem devolve `base` ao bit.
#[inline]
#[must_use]
pub(crate) fn dab_extent(
    base: [f32; 2],
    shape: &crate::TextureSettings,
    basis: Option<&crate::texture::TexDabBasis>,
) -> [f32; 2] {
    #[cfg(test)]
    if CAIXA_DO_RAIO.with(std::cell::Cell::get) {
        return base; // a caixa de ANTES da cura, para o A/B de custo ser no mesmo processo
    }
    match basis.and_then(|b| crate::texture::image_tip_extent(shape, b)) {
        Some(t) => [base[0].max(t[0]), base[1].max(t[1])],
        None => base,
    }
}

#[cfg(test)]
thread_local! {
    /// A medição de custo (`measure_o_custo_do_alcance`): `true` = a caixa `± raio` de antes do #52.
    pub(crate) static CAIXA_DO_RAIO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// O DOMÍNIO do carimbo assado, em raios (`≥ 1`): o quadrado `[-d, d]²` que contém a ponta estática
/// ([`dab_extent`] sobre `[1, 1]`). `1` sem ponta de imagem — o assado de sempre, ao bit. Quem assa
/// percorre `u·d`, quem amostra divide pela mesma `d` (`radius · d` no lugar de `radius`), então
/// [`axis_taps`] continua a responder no quadrado unitário DO CARIMBO.
#[must_use]
pub(crate) fn stamp_domain(
    shape: &crate::TextureSettings,
    basis: Option<&crate::texture::TexDabBasis>,
) -> f32 {
    let e = dab_extent([1.0, 1.0], shape, basis);
    e[0].max(e[1])
}

/// Os dois texels vizinhos e o peso bilinear de UM eixo, ou `None` fora da ponta.
///
/// `c` é a coordenada do dab em `[-1, 1]` (centro `0`), `size` o lado do stamp cacheado. A convenção é
/// **centro de texel** — o texel `i` mora em `c = (i + 0.5)·2/size − 1`, exactamente onde
/// [`crate::stamp::render_stamp_mask`] o escreveu, então a ida e a volta são a mesma conta.
///
/// ⚠️ **O clamp governa só o INTERIOR.** Ele existe porque o meio texel entre a borda e o aro não tem
/// vizinho de fora; estendê-lo para ALÉM do quadrado unitário é que era o defeito — ali não há dab
/// nenhum.
///
/// ⛔ **O quadrado unitário é o do CARIMBO, não o da ponta** (Bug #52). A ponta de imagem é `|tex| ≤ 1`
/// no referencial DELA ([`crate::texture::shape::shape_value`]) — rodada ou de Size < 1 ela sai de
/// `[-1, 1]²` raios. O carimbo assa o domínio [`stamp_domain`] que a contém, e quem amostra divide por
/// ele (`radius · reach`); só a `0°`, Size `1`, sem Offset, os dois quadrados coincidem.
///
/// ⚠️ **O `NaN` sai por `None`, e é EXPLÍCITO:** `a > 1.0` é falso para `NaN`, então sem o teste ele
/// entraria como se estivesse dentro e os índices sairiam do `as i64` de um `NaN` — zero, mudo.
#[inline]
#[must_use]
pub(crate) fn axis_taps(c: f32, size: u32) -> Option<(usize, usize, f32)> {
    let a = c.abs();
    if a.is_nan() || a > 1.0 {
        return None;
    }
    let m = (c + 1.0) * 0.5 * size as f32 - 0.5;
    let f = m.floor();
    let hi = i64::from(size) - 1;
    let i0 = (f as i64).clamp(0, hi) as usize;
    let i1 = (f as i64 + 1).clamp(0, hi) as usize;
    Some((i0, i1, m - f))
}

#[cfg(test)]
mod tests {
    use super::axis_taps;

    /// **Fora da ponta não há texel**, e é esta metade que os quatro amostradores não tinham.
    ///
    /// **Mutação que tem de sangrar:** trocar o teste por `c.abs() <= 1.0 + 1e-3` (ou removê-lo).
    #[test]
    fn outside_the_unit_square_there_is_no_tap() {
        for c in [-1.001f32, 1.001, -2.0, 2.0, f32::NAN, f32::INFINITY] {
            assert!(
                axis_taps(c, 32).is_none(),
                "{c} está fora da ponta e mesmo assim devolveu texels"
            );
        }
        // O CONTROLE: a fronteira PERTENCE ao dab, senão a ponta perde a última coluna.
        assert!(axis_taps(-1.0, 32).is_some() && axis_taps(1.0, 32).is_some());
    }

    /// **A ida e a volta são a mesma conta:** amostrar no centro do texel `i` devolve o texel `i`.
    ///
    /// É o que mantém um carimbo 1:1 nítido — meio texel de erro aqui borra a imagem inteira e
    /// desloca-a meia coluna, que é indistinguível do defeito que esta porta corrige.
    ///
    /// ⚠️ **O oráculo é o RESULTADO da bilinear, não o peso.** A primeira versão deste gate exigia
    /// `t ≈ 0` e reprovou sobre código correto: no texel `0` o `floor` cai em `−1` por um `f32` de
    /// meio-ulp, os DOIS taps clampam em `0` e o peso deixa de querer dizer alguma coisa — o valor lido
    /// continua exactamente o do texel `0`. Um gate que mede um intermediário mede a implementação;
    /// este mede a lei.
    #[test]
    fn a_texel_centre_reads_that_texel_exactly() {
        let size = 40u32;
        for i in 0..size {
            let c = (i as f32 + 0.5) * (2.0 / size as f32) - 1.0;
            let (i0, i1, t) =
                axis_taps(c, size).expect("o centro de um texel está dentro da ponta");
            // O índice que a bilinear de fato lê, como número: `(1−t)·i0 + t·i1`.
            let read = (1.0 - t) * i0 as f32 + t * i1 as f32;
            assert!(
                (read - i as f32).abs() < 1e-3,
                "o centro do texel {i} lê o texel {read} (taps {i0}/{i1}, peso {t})"
            );
        }
    }

    /// O clamp governa o INTERIOR: no meio texel de cada ponta os dois taps colapsam na borda, em vez
    /// de ler para fora do buffer.
    #[test]
    fn the_clamp_governs_the_inside() {
        let (i0, i1, _) = axis_taps(-1.0, 8).expect("dentro");
        assert_eq!(
            (i0, i1),
            (0, 0),
            "a ponta esquerda tem de colapsar no texel 0"
        );
        let (i0, i1, _) = axis_taps(1.0, 8).expect("dentro");
        assert_eq!(
            (i0, i1),
            (7, 7),
            "a ponta direita tem de colapsar no último texel"
        );
    }
}
