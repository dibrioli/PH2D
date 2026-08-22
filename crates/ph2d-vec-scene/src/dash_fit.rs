//! **O AJUSTE do tracejado ao caminho** — o período é esticado o mínimo necessário para
//! caber um número INTEIRO de vezes no contorno, e a emenda deixa de se ver.
//!
//! # O defeito
//!
//! Um tracejado é um padrão de comprimento fixo percorrido ao longo da curva. Num contorno
//! FECHADO o percurso volta ao início, e o padrão quase nunca acaba ali: sobra um pedaço, e o
//! que se vê é **um traço curto encostado a um longo** exactamente na emenda — sempre no mesmo
//! sítio, sempre o mesmo tamanho, e a olho parece um erro de desenho (Enio, 2026-08-22, com a
//! seta a apontar para a quina inferior esquerda de um retângulo picotado).
//!
//! # A cura, e de onde ela vem
//!
//! É a do Illustrator (*"aligns dashes to corners and path ends, adjusting lengths to fit"*) e
//! a do Figma: **não se muda o número de traços, muda-se o período**. Com `L` o comprimento do
//! contorno e `p` o período pedido, toma-se `n = round(L / p)` períodos e escala-se tudo por
//! `k = L / (n·p)`. O traço e o vão andam JUNTOS (a razão que o artista autorou sobrevive), e
//! o erro que se corrige é no máximo meio período — ou seja `k ∈ [2/3, 2]` para `n = 1` e
//! aperta depressa: `1 ± 1/(2n)`.
//!
//! ⚠️ **Num contorno ABERTO a lei é outra e a razão é a mesma:** ali não há emenda, há duas
//! PONTAS, e o que se vê de errado é uma ponta a acabar a meio de um vão. Então cabe-se
//! `n` períodos **mais um traço**, e o caminho começa e acaba com traço inteiro.
//!
//! ⚠️ **A escala é do PERÍODO, nunca da largura.** Mexer na largura para fechar a conta
//! engrossaria a linha em função do perímetro — dois retângulos do mesmo desenho e tamanhos
//! diferentes sairiam com traços de espessuras diferentes.

use crate::VecPath;
use crate::arc_path::ArcPath;
use crate::compound::Contour;
use crate::stroke_style::StrokeSpec;

/// **A lei, pura.** `raw` é `[traço, vão]` em COMPRIMENTO (o que a
/// [`StrokeSpec::dash_lengths`] devolve), `total` é o comprimento do contorno.
///
/// Devolve `raw` intocado quando não há o que ajustar (contorno degenerado, período nulo) —
/// *não ajustar* é sempre melhor que dividir por quase-zero.
#[must_use]
pub fn fit(raw: [f64; 2], total: f64, closed: bool) -> [f64; 2] {
    let period = raw[0] + raw[1];
    // ⚠️ `<=` e não `!(_ > _)`: os dois recusam o NaN, e o clippy prefere a forma que não
    // esconde a incomparabilidade atrás de uma negação.
    if total <= 0.0 || period <= 0.0 || total.is_nan() || period.is_nan() {
        return raw;
    }
    // Quantos períodos cabem, e o factor que os faz caber EXACTAMENTE.
    //
    // ⚠️ `round`, e não `floor`/`ceil`: arredondar para baixo alonga o período até 2× no pior
    // caso, e para cima encolhe-o a metade — as duas fazem o tracejado mudar de carácter
    // quando o caminho cresce um pixel. O mais próximo erra no máximo meio período.
    let denom = if closed {
        (total / period).round().max(1.0) * period
    } else {
        // Um traço a mais: o caminho começa e acaba com traço inteiro.
        ((total - raw[0]) / period).round().max(0.0) * period + raw[0]
    };
    if denom <= 0.0 || denom.is_nan() {
        return raw;
    }
    let k = total / denom;
    if !k.is_finite() || k <= 0.0 {
        return raw;
    }
    [raw[0] * k, raw[1] * k]
}

/// O contorno mais LONGO do caminho: `(comprimento, fechado)`.
///
/// ⚠️ **O mais longo, e é uma escolha com preço nomeado.** Um caminho COMPOSTO (o furo da
/// engrenagem, um anel) tem vários contornos com comprimentos diferentes, e o desenho carrega
/// **um** padrão de tracejado para todos — então só um deles pode fechar exactamente. O mais
/// longo é o que se vê primeiro e o que tem mais traços (logo o menor erro relativo nos
/// outros). Fechar todos exigiria picotar a geometria nós próprios, contorno a contorno, em
/// vez de pedir o padrão ao traçador: é outra wave, e está nomeada aqui para não ser
/// redescoberta.
#[must_use]
pub fn longest_contour(path: &VecPath) -> Option<(f64, bool)> {
    let mut best: Option<(f64, bool)> = None;
    let mut consider = |verts: &[crate::VecVertex], closed: bool| {
        if let Some(a) = ArcPath::from_contour(verts, closed) {
            let t = a.total();
            if best.is_none_or(|(b, _)| t > b) {
                best = Some((t, closed));
            }
        }
    };
    consider(&path.verts, path.closed);
    for c in &path.subpaths {
        let Contour { verts, closed } = c;
        consider(verts, *closed);
    }
    best
}

/// **A PORTA ÚNICA** — o `[traço, vão]` que um traçador deve usar para este caminho.
///
/// O renderer e o *Outline Stroke* constroem cada um o seu `Stroke` (versões diferentes da
/// kurbo), e os dois têm de concordar sobre quanto mede um traço: se um ajustasse e o outro
/// não, a forma assada sairia com outra cadência que a desenhada.
///
/// ⚠️ **Recebe o caminho COZIDO.** Um Trim ou um efeito da pilha mudam o comprimento, e ajustar
/// ao caminho de origem poria a emenda de volta — no sítio errado. Quem tem a FONTE em mão
/// chama [`crate::dash_for`], que coze e delega aqui — é a mesma lei, e é a única.
///
/// ⚠️ **Sem contorno que se meça, devolve o padrão AUTORADO, não `None`**: `None` é «sólido»,
/// e um tracejado pedido sobre geometria degenerada continua a ser um tracejado pedido.
#[must_use]
pub fn dash_lengths_for(path: &VecPath, s: &StrokeSpec) -> Option<[f64; 2]> {
    let raw = s.dash_lengths()?;
    Some(match longest_contour(path) {
        Some((total, closed)) => fit(raw, total, closed),
        None => raw,
    })
}

#[cfg(test)]
#[path = "dash_fit_tests.rs"]
mod tests;
