//! **ONDE DOIS PLANOS DIFEREM** — filho de [`super`] (`#[path]`), partido pelo tecto de LOC (R4): o pai
//! guarda o passo, aqui mora a régua exacta que acha a janela dele.

use super::{PAR_MIN, PlaneWindow, fits};
use rayon::prelude::*;

/// O bbox **EXATO** em que dois planos do mesmo tamanho diferem, ou `None` se são idênticos.
///
/// ⚠️ **Ele é calculado, não recebido, e isso é deliberado.** O ADR-0124 diz que quem está a jusante tem
/// de ser *informado* do intervalo em vez de obrigado a redescobri-lo — e aqui o chamador de fato sabe
/// (o traço tem um bbox). Mas há **75 sítios** que commitam uma entrada estrutural, quase nenhum é um
/// traço, e uma janela informada errado não falha: ela deixa fora exatamente os texels que o undo depois
/// não restaura. Derivá-la do conteúdo **estabelece** o invariante de que a materialização depende, em
/// vez de assumi-lo do chamador — a mesma escolha que o motor de blend do vetor teve de fazer quando o
/// invariante *"a origem está na lista"* era esperado de quem chamava.
///
/// ⚠️ **O custo dessa escolha foi MEDIDO e a estimativa que morava aqui estava errada.** Esta doc dizia
/// *"uma varredura por linha, uma vez por commit (user-paced)"*, como se user-paced quisesse dizer
/// barato. Um commit acontece no **pen-up de todo traço**, e a 4096² com impasto são quatro planos a
/// varrer (~256 MB): medido, **91% do pen-up era este scan** — 40,20 ms de pen-up contra 3,49 sem o
/// commit, com o resto plano na tela (`measure_stroke_owners`). A varredura ficou (a decisão acima
/// continua certa: uma janela informada errado não falha, some com texels) e virou **paralela**
/// ([`diff_rows`] / [`diff_cols`]) — 25,03 → 10,96 ms no `record_structural`. Só nos planos que os
/// `Arc`s já não deram como idênticos, como sempre foi.
///
/// ⚠️ **Só é chamada com um stride que divide o plano** ([`fits`]) — porque `None` aqui significa
/// *idênticos*, e devolver `None` para *"não sei medir"* faria o `split` gravar `Unchanged` sobre dois
/// buffers que de fato diferem: o undo perderia a edição, sem erro e sem warning. Os dois casos têm de
/// ser perguntas separadas, e são.
pub(super) fn diff_window<T: PartialEq + Sync>(
    a: &[T],
    b: &[T],
    stride: usize,
) -> Option<PlaneWindow> {
    debug_assert_eq!(
        a.len(),
        b.len(),
        "diff_window quer dois planos do mesmo tamanho"
    );
    debug_assert!(
        fits(a.len(), stride),
        "diff_window quer um stride que divide o plano"
    );
    // As LINHAS primeiro: uma comparação de slice por linha (memcmp vetorizado em `u8`), e a maioria
    // das linhas de um traço não difere.
    let (row, last) = diff_rows(a, b, stride)?;
    // …e só então as COLUNAS, dentro das linhas que já sabemos que diferem.
    let (col, end) = diff_cols(a, b, stride, row, last);
    Some(PlaneWindow {
        row,
        rows: last - row + 1,
        col,
        cols: end.saturating_sub(col),
        stride,
        plane_len: a.len(),
    })
}

/// **A varredura de LINHAS, paralela acima de [`PAR_MIN`]** — a primeira e a última linha em que os dois
/// planos diferem.
///
/// Uma linha é comparada contra a linha de MESMO índice do outro plano e não olha nenhuma outra: as
/// linhas são **disjuntas e a leitura é pura**, que é a forma que o [ADR-0109] sanciona e que esta crate
/// já usa no `sculpt_offset`, no `sculpt_close`, no campo da aquarela e no fold da luz. **Byte-idêntica
/// por construção**: muda qual thread avalia qual linha, nunca o que a linha responde — `min`/`max` sobre
/// índices é associativo e comutativo, então a ordem da redução não entra no resultado.
///
/// [ADR-0109]: ../../../docs/architecture/decisions/0109-rayon-exception-watercolor-composite.md
fn diff_rows<T: PartialEq + Sync>(a: &[T], b: &[T], stride: usize) -> Option<(usize, usize)> {
    let rows = a.len() / stride;
    if a.len() < PAR_MIN {
        let mut first = None;
        let mut last = 0usize;
        for r in 0..rows {
            let s = r * stride;
            if a[s..s + stride] != b[s..s + stride] {
                if first.is_none() {
                    first = Some(r);
                }
                last = r;
            }
        }
        return first.map(|f| (f, last));
    }
    a.par_chunks(stride)
        .zip(b.par_chunks(stride))
        .enumerate()
        .filter(|(_, (x, y))| x != y)
        .map(|(r, _)| (r, r))
        .reduce_with(|(f1, l1), (f2, l2)| (f1.min(f2), l1.max(l2)))
}

/// **A varredura de COLUNAS, dentro da faixa que já se sabe diferente** — paralela pela mesma razão e
/// com a mesma redução associativa.
///
/// ⚠️ Ela é element-a-element (não há memcmp que devolva *onde*), então num traço VERTICAL a faixa é a
/// tela inteira em linhas e esta metade custa tanto quanto a outra. É por isso que ela também é
/// paralela, e não só a de linhas.
fn diff_cols<T: PartialEq + Sync>(
    a: &[T],
    b: &[T],
    stride: usize,
    row: usize,
    last: usize,
) -> (usize, usize) {
    let band = (last - row + 1) * stride;
    let (from, to) = (row * stride, (last + 1) * stride);
    let scan = |x: &[T], y: &[T]| -> (usize, usize) {
        let mut col = stride;
        let mut end = 0usize;
        for c in 0..stride {
            if x[c] != y[c] {
                if c < col {
                    col = c;
                }
                if c + 1 > end {
                    end = c + 1;
                }
            }
        }
        (col, end)
    };
    if band < PAR_MIN {
        let mut col = stride;
        let mut end = 0usize;
        for r in row..=last {
            let s = r * stride;
            let (c, e) = scan(&a[s..s + stride], &b[s..s + stride]);
            col = col.min(c);
            end = end.max(e);
        }
        return (col, end);
    }
    a[from..to]
        .par_chunks(stride)
        .zip(b[from..to].par_chunks(stride))
        .map(|(x, y)| scan(x, y))
        .reduce(
            || (stride, 0),
            |(c1, e1), (c2, e2)| (c1.min(c2), e1.max(e2)),
        )
}
