//! **Re-partir um plano do topo SEM materializar a tela** — o caminho barato da absorção do escorrido
//! ([`crate::undo::absorb`]).
//!
//! Filho de [`super`] por `#[path]` pelo mesmo motivo dos irmãos: ele lê os campos privados de
//! [`PlaneWindow`] e usa o [`diff_window`] do pai, que é a régua EXATA — a resposta tem de sair da
//! mesma função que o caminho caro usa, senão as duas rotas divergem no dia em que a régua mudar.
//!
//! # Porquê existe (medido 2026-09-24, `examples/mede_o_wet_paint pousos`, tela 4096², release)
//!
//! Todo traço de Wet Paint que começa com a água ainda a correr paga a absorção no pen-down, e ela
//! eram **três passes de tela inteira**: o detector (`~3,4 ms`), a materialização do `before` do topo
//! (uma cópia de 67 MB, `~4 ms`) e o re-split que volta a varrer os 67 MB para achar a janela que o
//! detector e o topo já conheciam (`~5–7 ms`). Os dois últimos são o que este módulo tira.
//!
//! # Porquê é a MESMA resposta (e não uma parecida)
//!
//! O caminho caro calcula `split(side(cursor, before=true), after)`. Seja `W` a janela do topo e `D` a
//! janela EXACTA que o detector achou entre o cursor e o `after`:
//!
//! - fora de `W`, o `before` materializado **é** o cursor (a materialização copia o cursor e escreve
//!   `W` por cima);
//! - fora de `D`, o cursor **é** o `after` (o detector usa o [`diff_window`], que é exacto).
//!
//! Logo, fora da caixa `U = W ∪ D` os dois lados do re-split são iguais byte a byte, e a janela exacta
//! dele está DENTRO de `U`. Varrer só `U` com a mesma régua dá a mesma caixa — a varredura por linhas e
//! colunas do [`diff_window`] devolve a caixa envolvente das diferenças, e nenhuma está fora de `U`. E
//! como `U` é menor que meio plano, a janela exacta também é, logo o `from_window` do caminho caro teria
//! escolhido `Patch` — que é o que sai daqui, com os mesmos bytes nos dois lados.
//!
//! Onde a igualdade não é demonstrável barato — um lado `Whole`, planos de tamanhos diferentes, um
//! stride que não mede o plano, ou `U` com metade do plano ou mais — a resposta é `None`, e quem chama
//! corre o caminho caro inteiro. *Menos esperto nunca, errado jamais.*

use super::{PlaneWindow, StoredPlane, diff_window, fits};

/// A caixa envolvente de duas janelas do MESMO plano (mesmo stride, mesmo comprimento).
fn envolve(a: PlaneWindow, b: PlaneWindow) -> PlaneWindow {
    let (row, col) = (a.row.min(b.row), a.col.min(b.col));
    let fim_linha = (a.row + a.rows).max(b.row + b.rows);
    let fim_coluna = (a.col + a.cols).max(b.col + b.cols);
    PlaneWindow {
        row,
        rows: fim_linha - row,
        col,
        cols: fim_coluna - col,
        stride: a.stride,
        plane_len: a.plane_len,
    }
}

impl<T: Copy + PartialEq + Send + Sync> StoredPlane<T> {
    /// **O [`Self::split`] sem janela, respondido varrendo SÓ a janela DECLARADA** — o detector da
    /// absorção, para o canvas.
    ///
    /// `split(before, after, stride, None)` varre o plano inteiro para achar a janela EXACTA. Quando
    /// quem escreveu declarou onde (`crate::undo::window` — a janela é um superconjunto de tudo o que
    /// mudou desde o snapshot), a janela exacta está dentro da declarada, e o mesmo [`diff_window`] sobre
    /// o recorte dá a mesma caixa. O resultado é **o mesmo que o `split` sem janela** — e não o `split`
    /// COM janela, que guardaria a declarada tal como veio: o detector decide se a absorção dispara, e
    /// uma janela declarada sobre bytes iguais a faria disparar onde o caminho de sempre não dispara.
    ///
    /// Sem janela, com uma que não serve a este plano, ou com meio plano ou mais (aí o `split` poderia
    /// guardar `Whole`), é o `split` de sempre. Em DEBUG a janela verdadeira é derivada e tem de caber
    /// na declarada — a mesma rede do commit declarado. O `bool` diz se a resposta saiu de DENTRO da
    /// janela (é o que o gate conta para saber que este caminho correu).
    pub(crate) fn split_exact_within(
        before: &mut std::sync::Arc<Vec<T>>,
        after: &mut std::sync::Arc<Vec<T>>,
        stride: usize,
        declared: Option<PlaneWindow>,
    ) -> (Self, bool) {
        let len = before.len();
        let dentro = declared
            .filter(|_| after.len() == len && fits(len, stride))
            .and_then(|d| d.fit_to(len))
            .filter(|d| d.stride == stride && 2 * d.elems() < len);
        let Some(d) = dentro else {
            return (Self::split(before, after, stride, None), false);
        };
        if std::sync::Arc::ptr_eq(before, after) {
            // O `split` responde isto sem ler um byte; a janela não muda nada aqui.
            return (Self::split(before, after, stride, None), false);
        }
        #[cfg(debug_assertions)]
        if let Some(real) = diff_window(before, after, stride) {
            debug_assert!(
                d.contains(&real),
                "a janela declarada nao contem a verdadeira no detector da absorcao: declarada \
                 {d:?}, real {real:?} — o escorrido fora dela ficaria sem dono"
            );
        }
        let (antes, depois) = (d.extract(before), d.extract(after));
        let resposta = match diff_window(&antes, &depois, d.cols) {
            None => Self::Unchanged,
            Some(local) => Self::Patch {
                win: PlaneWindow {
                    row: d.row + local.row,
                    col: d.col + local.col,
                    stride,
                    plane_len: len,
                    ..local
                },
                before: local.extract(&antes),
                after: local.extract(&depois),
            },
        };
        *before = super::drained();
        *after = super::drained();
        (resposta, true)
    }

    /// **O plano que o re-split da absorção guardaria**, sem materializar nem varrer a tela inteira.
    ///
    /// `self` é o plano guardado no TOPO (o lado `after` dele é o `cursor`), `drip` é o que o DETECTOR
    /// guardou entre o `cursor` e o `after`, e `stride` é o que o re-split usaria (o do `before` do
    /// topo). Devolve exactamente o que
    /// `StoredPlane::split(&mut self.side(cursor, true), &mut after, stride, None)` devolveria, ou `None`
    /// onde isso não é demonstrável barato — e aí quem chama corre o caminho caro.
    pub(crate) fn absorbed(
        &self,
        cursor: &[T],
        after: &[T],
        drip: &Self,
        stride: usize,
    ) -> Option<Self> {
        let len = cursor.len();
        if after.len() != len || !fits(len, stride) {
            return None;
        }
        // Uma janela só serve se foi medida NESTE plano e com ESTE stride — senão a caixa `U` não
        // descreveria o mesmo retângulo que o re-split varreria.
        let serve = |w: &PlaneWindow| w.plane_len == len && w.stride == stride;
        let topo = match self {
            Self::Unchanged => None,
            Self::Patch { win, before, .. } if serve(win) => Some((*win, before)),
            Self::Patch { .. } | Self::Whole { .. } => return None,
        };
        let pinga = match drip {
            Self::Unchanged => None,
            Self::Patch { win, .. } if serve(win) => Some(*win),
            Self::Patch { .. } | Self::Whole { .. } => return None,
        };
        let u = match (topo.map(|(w, _)| w), pinga) {
            // Os dois lados são o cursor: o re-split veria o mesmo conteúdo e guardaria nada.
            (None, None) => return Some(Self::Unchanged),
            (Some(w), None) | (None, Some(w)) => w,
            (Some(a), Some(b)) => envolve(a, b),
        };
        if 2 * u.elems() >= len {
            return None;
        }
        // Os dois lados do re-split, só dentro de `U`: o `before` materializado é o cursor com a janela
        // do topo escrita por cima; o `after` é o `after`.
        let n = u.elems();
        let (mut antes, mut depois) = (Vec::with_capacity(n), Vec::with_capacity(n));
        for r in 0..u.rows {
            let s = (u.row + r) * stride + u.col;
            antes.extend_from_slice(&cursor[s..s + u.cols]);
            depois.extend_from_slice(&after[s..s + u.cols]);
        }
        if let Some((w, patch)) = topo {
            for r in 0..w.rows {
                let d = (w.row - u.row + r) * u.cols + (w.col - u.col);
                let p = r * w.cols;
                antes[d..d + w.cols].copy_from_slice(&patch[p..p + w.cols]);
            }
        }
        // A régua EXACTA do caminho caro, sobre o recorte (o stride do recorte é a largura dele).
        let Some(local) = diff_window(&antes, &depois, u.cols) else {
            return Some(Self::Unchanged);
        };
        let win = PlaneWindow {
            row: u.row + local.row,
            col: u.col + local.col,
            stride,
            plane_len: len,
            ..local
        };
        Some(Self::Patch {
            win,
            before: local.extract(&antes),
            after: local.extract(&depois),
        })
    }
}
