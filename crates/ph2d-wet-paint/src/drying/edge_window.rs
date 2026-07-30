//! **A JANELA DESLIZANTE do fator de borda** (doc 28 §5.43).
//!
//! O `drying_pass` pergunta, por célula, *quantos dos 9 vizinhos carregam
//! pigmento* (`susp > 10`). Escrito direto, isso são **nove cargas por
//! célula** — e num traço largo o laço percorre 1,3 M células, o que medido dá
//! **11,6 ms de um passe de 33,8** (34%).
//!
//! Mas o laço anda **em x**, e as três colunas de uma célula são duas colunas
//! da seguinte: só a coluna mais à direita é nova. Guardando uma **soma por
//! coluna** a conta vira `c[x-1] + c[x] + c[x+1]` com **uma** coluna carregada
//! por passo — três cargas em vez de nove.
//!
//! ⚠️ **A metade que torna isto byte-idêntico é a linha do MEIO.** O `susp` da
//! linha `y` é escrito **por este mesmo laço**, então a célula `x+1` lê em
//! `x` o valor **depois** da escrita de `x` (Gauss-Seidel — é o mecanismo que o
//! ADR-0134 nomeia). Por isso a soma de uma coluna é guardada em **duas
//! partes**: `ud` (as linhas de cima e de baixo, que este laço não toca e são
//! estáveis) e `m` (a linha do meio), e quem escreve avisa a janela
//! ([`EdgeWindow::note_write`]). Fundir as duas num único inteiro tornaria a
//! correção impossível de exprimir.
//!
//! ⚠️ E a janela avança em **TODA** célula, inclusive nas que o early-out pula:
//! uma célula pulada não escreve `susp`, mas ela **é** vizinha das próximas, e
//! uma janela que só anda quando há trabalho descreveria outra vizinhança.

/// O limiar de "esta célula carrega pigmento" (SPEC §6.2).
const PIGMENT: f32 = 10.0;

/// As três colunas vivas da vizinhança 3×3, com a linha do meio separável.
///
/// Índices: `0` é a coluna à ESQUERDA da célula, `1` a dela, `2` a da direita.
pub(crate) struct EdgeWindow {
    /// Linhas `y-1` e `y+1` de cada coluna (0..=2 cada) — estáveis na varredura.
    ud: [u32; 3],
    /// Linha `y` de cada coluna (0 ou 1) — a que este laço reescreve.
    m: [u32; 3],
}

impl EdgeWindow {
    /// Semeia a janela para a primeira célula da faixa (`x = bx0`, índice `i`).
    ///
    /// Carrega as colunas `bx0-1`, `bx0` e `bx0+1` — os mesmos nove texels que
    /// a forma direta lia naquela célula, e nenhum a mais.
    #[inline]
    pub(crate) fn seed(susp: &[f32], i: usize, s: usize) -> Self {
        let mut w = Self {
            ud: [0; 3],
            m: [0; 3],
        };
        for (k, slot) in (0..3).zip(0usize..) {
            let j = i + k - 1;
            w.ud[slot] = u32::from(susp[j - s] > PIGMENT) + u32::from(susp[j + s] > PIGMENT);
            w.m[slot] = u32::from(susp[j] > PIGMENT);
        }
        w
    }

    /// Quantos dos nove vizinhos carregam pigmento.
    #[inline]
    pub(crate) fn count(&self) -> u32 {
        self.ud[0] + self.ud[1] + self.ud[2] + self.m[0] + self.m[1] + self.m[2]
    }

    /// A célula corrente acabou de escrever `susp` — a coluna do MEIO muda.
    ///
    /// Chamado só por quem escreve: uma célula pulada pelo early-out deixa o
    /// `susp` onde estava, e a janela já o carrega.
    #[inline]
    pub(crate) fn note_write(&mut self, susp_c: f32) {
        self.m[1] = u32::from(susp_c > PIGMENT);
    }

    /// Anda uma célula para a direita: as colunas 1 e 2 viram 0 e 1, e a nova
    /// coluna 2 (`x+2`, índice `i + 2` *da célula que acabou*) é carregada.
    #[inline]
    pub(crate) fn advance(&mut self, susp: &[f32], next_right: usize, s: usize) {
        self.ud[0] = self.ud[1];
        self.ud[1] = self.ud[2];
        self.m[0] = self.m[1];
        self.m[1] = self.m[2];
        self.ud[2] =
            u32::from(susp[next_right - s] > PIGMENT) + u32::from(susp[next_right + s] > PIGMENT);
        self.m[2] = u32::from(susp[next_right] > PIGMENT);
    }
}

/// **A FORMA DIRETA, congelada** — as nove cargas que o `drying_pass` fazia
/// antes da janela, verbatim, para o gate de identidade ter um oráculo que não
/// é o código sob teste.
///
/// `#[cfg(test)]` de propósito: um `pub` sem chamador não é código morto
/// silencioso, é uma **segunda resposta** esperando alguém chamá-la.
#[cfg(test)]
pub(crate) fn edge_count_direct(susp: &[f32], i: usize, s: usize) -> u32 {
    let up = i - s;
    let dn = i + s;
    u32::from(susp[up - 1] > PIGMENT)
        + u32::from(susp[up] > PIGMENT)
        + u32::from(susp[up + 1] > PIGMENT)
        + u32::from(susp[i - 1] > PIGMENT)
        + u32::from(susp[i] > PIGMENT)
        + u32::from(susp[i + 1] > PIGMENT)
        + u32::from(susp[dn - 1] > PIGMENT)
        + u32::from(susp[dn] > PIGMENT)
        + u32::from(susp[dn + 1] > PIGMENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Uma grade pequena com estrutura DELIBERADA na vizinhança: um campo
    /// chato faria as duas contagens concordarem por vácuo.
    fn fixture(w: usize, h: usize) -> (Vec<f32>, usize) {
        let s = w;
        let mut susp = vec![0.0f32; w * h];
        for y in 0..h {
            for x in 0..w {
                // Faixas, buracos e valores dos DOIS lados do limiar de 10.
                let v = match (x % 5, y % 4) {
                    (0, _) => 0.0,
                    (_, 0) => 9.999,
                    (2, 2) => 10.0, // exatamente o limiar: NAO conta (`>`)
                    (3, _) => 10.001,
                    _ => 40.0 + (x * 7 + y * 13) as f32,
                };
                susp[x + y * s] = v;
            }
        }
        (susp, s)
    }

    /// **A janela é a forma direta**, célula a célula, incluindo a interação
    /// com a escrita da linha do meio.
    ///
    /// Mutação: tirar o `note_write` faz a contagem divergir na primeira
    /// célula que escreve; trocar `advance` por um shift sem carga nova faz
    /// divergir na segunda.
    #[test]
    fn the_sliding_window_is_the_direct_gather() {
        let (mut susp, s) = fixture(24, 9);
        for y in 1..8 {
            let (bx0, bx1) = (1usize, 22usize);
            let mut i = bx0 + y * s;
            let mut w = EdgeWindow::seed(&susp, i, s);
            for x in bx0..=bx1 {
                // A MESMA condução do produto: avança no topo, carregando a
                // coluna `i + 1` da célula corrente.
                if x > bx0 {
                    w.advance(&susp, i + 1, s);
                }
                assert_eq!(
                    w.count(),
                    edge_count_direct(&susp, i, s),
                    "divergiu em ({x}, {y})"
                );
                // Metade das células escreve, como o produto faz — e o valor
                // atravessa o limiar nos DOIS sentidos.
                if x % 2 == 0 {
                    let novo = if susp[i] > PIGMENT { 0.0 } else { 50.0 };
                    susp[i] = novo;
                    w.note_write(novo);
                }
                i += 1;
            }
        }
    }

    /// A janela semeada no meio de uma faixa concorda com a forma direta sem
    /// nenhuma escrita — o controle do gate acima.
    #[test]
    fn a_freshly_seeded_window_agrees_with_the_direct_gather() {
        let (susp, s) = fixture(24, 9);
        for y in 1..8 {
            for x in 1..23 {
                let i = x + y * s;
                assert_eq!(
                    EdgeWindow::seed(&susp, i, s).count(),
                    edge_count_direct(&susp, i, s),
                    "semeadura divergiu em ({x}, {y})"
                );
            }
        }
    }
}
