//! ⭐⭐⭐ **A FORMA de um botão que tem VIZINHOS** — a lei de agrupamento do Blender.
//!
//! Irmão do [`super`] pelo tecto de 500 LOC dos primitivos, e o corte é por RESPONSABILIDADE: ali
//! mora *que cor tem o fundo de um botão desenhado à mão*; aqui, *que forma ele tem quando não
//! está sozinho*.

use crate::zones::Rect;

/// ⭐⭐⭐ **Onde uma peça está numa FILEIRA de botões vizinhos.**
///
/// Enio, 2026-09-06, apontando o `None | Vertices | Faces` do Blender: *«uma coisa muito legal que
/// o Blender tem: se 2 ou mais botões estão lado a lado, só as bordas externas dos botões das
/// extremidades recebem arredondamento»*.
///
/// ⭐⭐ **E é a resposta à outra queixa da mesma mensagem** (*«espaços demais entre botões»*): num
/// grupo as peças **ENCOSTAM**, porque o que separa duas peças de um mesmo controlo é a **QUINA**,
/// não o espaço. Um vão entre elas diria que são coisas diferentes — e elas não são: são uma
/// escolha entre irmãos, que o HIG do Blender manda expandir a toda a largura (`layouts.md`,
/// «Mode toggling buttons»).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GroupPos {
    /// Sozinho — arredonda os quatro cantos, como sempre.
    Only,
    /// A primeira de uma fileira: arredonda só à esquerda.
    First,
    /// Do meio: **nenhum** canto arredondado.
    Middle,
    /// A última: arredonda só à direita.
    Last,
}

impl GroupPos {
    /// A posição da peça `i` numa fileira de `n`.
    #[must_use]
    pub fn of(i: usize, n: usize) -> Self {
        match (i, n) {
            (_, 0 | 1) => Self::Only,
            (0, _) => Self::First,
            (i, n) if i + 1 == n => Self::Last,
            _ => Self::Middle,
        }
    }

    /// Os quatro raios de uma peça que só tem vizinhos aos LADOS, na ordem do kurbo:
    /// `(cima-esq, cima-dir, baixo-dir, baixo-esq)`.
    #[must_use]
    pub fn radii(self, radius: f32) -> (f32, f32, f32, f32) {
        GroupCell {
            col: self,
            row: Self::Only,
        }
        .radii(radius)
    }

    /// Esta peça toca a borda de INÍCIO do grupo (a esquerda numa fileira, o topo numa coluna)?
    #[must_use]
    pub fn starts(self) -> bool {
        matches!(self, Self::Only | Self::First)
    }

    /// E a de FIM (a direita, ou o fundo)?
    #[must_use]
    pub fn ends(self) -> bool {
        matches!(self, Self::Only | Self::Last)
    }
}

/// ⭐⭐⭐ **Onde uma peça está numa GRELHA de vizinhos — as duas dimensões.**
///
/// Enio, 2026-09-06, depois de ver a fileira agrupada: *«na horizontal ficou bom. Na vertical
/// ainda tem muito espaço ainda.»* — e ele tem razão porque a lei que ele apontou **nunca foi só
/// horizontal**: no cartão *Transform* do Blender que ele fotografou, o `Location X / Y / Z`
/// é uma COLUNA de linhas que encostam, com o arredondamento só no topo da primeira e no fundo
/// da última. *Eu tinha aplicado metade da lei.*
///
/// ⚠️ **Um canto só arredonda se estiver na borda das DUAS:** o canto de cima-à-esquerda pertence
/// ao grupo se esta peça começa a coluna **e** começa a linha. É isto que faz um bloco de 4×2
/// botões ler-se como **um** corpo com quatro cantos, e não como oito cantos.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GroupCell {
    /// Onde a peça está na FILEIRA (o eixo x).
    pub col: GroupPos,
    /// Onde a peça está na COLUNA (o eixo y).
    pub row: GroupPos,
}

impl GroupCell {
    /// Os quatro raios, na ordem do kurbo: `(cima-esq, cima-dir, baixo-dir, baixo-esq)`.
    #[must_use]
    pub fn radii(self, radius: f32) -> (f32, f32, f32, f32) {
        let pick = |on: bool| if on { radius } else { 0.0 };
        (
            pick(self.col.starts() && self.row.starts()),
            pick(self.col.ends() && self.row.starts()),
            pick(self.col.ends() && self.row.ends()),
            pick(self.col.starts() && self.row.ends()),
        )
    }
}

/// O traço entre duas peças de um grupo — **um pixel**.
///
/// ⚠️ **Não é um degrau da escada de espaço, e não devia ser:** ele não é folga, é o traço que
/// deixa ver onde uma peça acaba e a seguinte começa. *Pôr aqui um `Spacing::Xs` diria que as
/// peças são coisas separadas, que é exactamente o que o grupo nega.*
pub const SEGMENT_HAIRLINE: f32 = 1.0; // LITERAL-PX-OK: a costura de um grupo e' 1 px por definicao

/// ⭐⭐ **A fileira: `n` peças que ENCOSTAM, cada uma com a sua posição no grupo.**
///
/// ⚠️ As larguras são **arredondadas ao pixel** e a última peça come o resto: sem isso, `n` peças
/// de largura fraccionária deixam uma costura de sub-pixel entre duas quinas que deviam formar
/// uma linha recta.
///
/// ⚠️ **Devolve uma [`GroupCell`], não um [`GroupPos`]** — uma fileira solta é um bloco de UMA
/// linha, e dizê-lo aqui evita que cada chamador construa a célula à mão (e que metade deles se
/// esqueça da segunda dimensão, que foi o defeito de 2026-09-06).
#[must_use]
pub fn segment_rects(row: Rect, n: usize) -> Vec<(Rect, GroupCell)> {
    if n == 0 {
        return Vec::new();
    }
    let gaps = SEGMENT_HAIRLINE * (n.saturating_sub(1)) as f32;
    let each = ((row.w - gaps) / n as f32).floor().max(1.0);
    (0..n)
        .map(|i| {
            let x = row.x + i as f32 * (each + SEGMENT_HAIRLINE);
            let w = if i + 1 == n {
                (row.x + row.w - x).max(1.0)
            } else {
                each
            };
            (
                Rect::new(x, row.y, w, row.h),
                GroupCell {
                    col: GroupPos::of(i, n),
                    row: GroupPos::Only,
                },
            )
        })
        .collect()
}

/// ⭐⭐ **A GRELHA: `rows` × `cols` peças que encostam nas DUAS direções.**
///
/// Devolve as células por linhas, da esquerda para a direita — `out[r * cols + c]`. É o
/// [`segment_rects`] com a segunda dimensão, e a resposta a *«na vertical ainda tem muito
/// espaço»*: entre duas linhas irmãs há o mesmo traço de um pixel que já havia entre duas colunas.
///
/// ⚠️ **A altura de cada linha é dada, não derivada da caixa** — um bloco de botões tem a altura
/// de linha da casa, e é o CHAMADOR que a conhece.
#[must_use]
pub fn grid_cells(origin: Rect, rows: usize, cols: usize, row_h: f32) -> Vec<(Rect, GroupCell)> {
    block_cells(origin, &vec![cols; rows], row_h)
        .into_iter()
        .flatten()
        .collect()
}

/// ⭐⭐ **O BLOCO com linhas de contagens DIFERENTES** — `cols_per_row[r]` peças na linha `r`.
///
/// ⚠️ **É esta a forma real, e a uniforme é o caso fácil:** o cartão *Transform* do Blender que o
/// dono fotografou tem `Location X/Y/Z` (três linhas), depois um `Mode` de uma peça só — e as
/// quatro encostam na mesma coluna. Uma grelha rectangular não exprimiria a barra de ferramentas
/// deste app (`3 · 3 · 2`), e forçá-la partiria os blocos em três, que é a folga que o dono está
/// a ver.
///
/// Devolve uma linha por entrada de `cols_per_row`.
#[must_use]
pub fn block_cells(
    origin: Rect,
    cols_per_row: &[usize],
    row_h: f32,
) -> Vec<Vec<(Rect, GroupCell)>> {
    let rows = cols_per_row.len();
    cols_per_row
        .iter()
        .enumerate()
        .map(|(r, &cols)| {
            let y = origin.y + r as f32 * (row_h + SEGMENT_HAIRLINE);
            let line = Rect::new(origin.x, y, origin.w, row_h);
            segment_rects(line, cols)
                .into_iter()
                .map(|(rect, cell)| {
                    (
                        rect,
                        GroupCell {
                            col: cell.col,
                            row: GroupPos::of(r, rows),
                        },
                    )
                })
                .collect()
        })
        .collect()
}

/// A altura total que uma grelha de `rows` linhas ocupa — incluindo os traços entre elas.
#[must_use]
pub fn grid_height(rows: usize, row_h: f32) -> f32 {
    if rows == 0 {
        return 0.0;
    }
    rows as f32 * row_h + (rows - 1) as f32 * SEGMENT_HAIRLINE
}

#[cfg(test)]
mod group_tests {
    use super::*;
    use Rect;

    /// ⭐⭐⭐ **A lei que o dono apontou**: só as bordas de FORA das peças das extremidades
    /// arredondam.
    #[test]
    fn only_the_outer_corners_of_a_row_are_rounded() {
        let r = 3.0;
        assert_eq!(GroupPos::Only.radii(r), (r, r, r, r));
        assert_eq!(GroupPos::First.radii(r), (r, 0.0, 0.0, r), "só à esquerda");
        assert_eq!(
            GroupPos::Middle.radii(r),
            (0.0, 0.0, 0.0, 0.0),
            "uma peça do meio não arredonda canto nenhum"
        );
        assert_eq!(GroupPos::Last.radii(r), (0.0, r, r, 0.0), "só à direita");
    }

    /// ⚠️ **Uma peça sozinha continua a ser um botão** — a lei do grupo não pode achatar quem não
    /// tem vizinhos.
    #[test]
    fn a_row_of_one_is_still_a_plain_button() {
        assert_eq!(GroupPos::of(0, 1), GroupPos::Only);
        assert_eq!(GroupPos::of(0, 0), GroupPos::Only);
    }

    /// ⭐⭐ **As peças ENCOSTAM** — a resposta a *«espaços demais entre botões»*: entre duas peças
    /// de um grupo há um traço de 1 px, não um degrau da escada de espaço.
    #[test]
    fn the_pieces_touch_and_fill_the_row_exactly() {
        let row = Rect::new(10.0, 5.0, 100.0, 22.0);
        for n in 1..=5 {
            let seg = segment_rects(row, n);
            assert_eq!(seg.len(), n);
            assert!(
                (seg[0].0.x - row.x).abs() < 1e-3,
                "a fileira começa no sítio"
            );
            let last = seg[n - 1].0;
            assert!(
                ((last.x + last.w) - (row.x + row.w)).abs() < 1e-3,
                "a fileira acaba no sítio (n = {n})"
            );
            for w in seg.windows(2) {
                let gap = w[1].0.x - (w[0].0.x + w[0].0.w);
                assert!(
                    (gap - SEGMENT_HAIRLINE).abs() < 1e-3,
                    "entre duas peças há UM traço, não uma folga (n = {n}, medido {gap})"
                );
            }
        }
    }

    /// ⭐⭐⭐ **A lei nas DUAS direções**: num bloco de 3×2, só os quatro cantos do BLOCO
    /// arredondam — as outras 12 quinas são rectas.
    #[test]
    fn only_the_four_corners_of_a_block_are_rounded() {
        let r = 3.0;
        let cells = grid_cells(Rect::new(0.0, 0.0, 100.0, 0.0), 3, 2, 22.0);
        assert_eq!(cells.len(), 6);
        let rounded: Vec<usize> = cells
            .iter()
            .map(|(_, c)| {
                let (a, b, cc, d) = c.radii(r);
                [a, b, cc, d].iter().filter(|v| **v > 0.0).count()
            })
            .collect();
        assert_eq!(
            rounded,
            vec![1, 1, 0, 0, 1, 1],
            "só as quatro peças de canto arredondam, e cada uma UM canto"
        );
        // E o canto certo em cada uma.
        assert_eq!(cells[0].1.radii(r), (r, 0.0, 0.0, 0.0), "topo-esquerda");
        assert_eq!(cells[1].1.radii(r), (0.0, r, 0.0, 0.0), "topo-direita");
        assert_eq!(cells[4].1.radii(r), (0.0, 0.0, 0.0, r), "fundo-esquerda");
        assert_eq!(cells[5].1.radii(r), (0.0, 0.0, r, 0.0), "fundo-direita");
    }

    /// ⭐ **As linhas encostam** — a queixa do dono: *«na vertical ainda tem muito espaço»*.
    #[test]
    fn the_rows_of_a_block_touch_like_its_columns() {
        let row_h = 22.0;
        let cells = grid_cells(Rect::new(0.0, 10.0, 100.0, 0.0), 4, 2, row_h);
        for r in 1..4 {
            let above = cells[(r - 1) * 2].0;
            let here = cells[r * 2].0;
            let gap = here.y - (above.y + above.h);
            assert!(
                (gap - SEGMENT_HAIRLINE).abs() < 1e-3,
                "entre a linha {} e a {r} há {gap} px, e devia haver um traço",
                r - 1
            );
        }
        assert!(
            (grid_height(4, row_h) - (4.0 * row_h + 3.0 * SEGMENT_HAIRLINE)).abs() < 1e-3,
            "a altura declarada tem de contar os traços"
        );
    }

    /// ⚠️ **Uma fileira de UMA linha continua a arredondar os quatro cantos** — a segunda dimensão
    /// não pode achatar quem não tem vizinhos por cima nem por baixo.
    #[test]
    fn a_single_row_block_is_still_a_plain_row() {
        let r = 3.0;
        let cells = grid_cells(Rect::new(0.0, 0.0, 90.0, 0.0), 1, 3, 22.0);
        assert_eq!(
            cells[0].1.radii(r),
            (r, 0.0, 0.0, r),
            "a primeira da fileira"
        );
        assert_eq!(cells[1].1.radii(r), (0.0, 0.0, 0.0, 0.0), "a do meio");
        assert_eq!(cells[2].1.radii(r), (0.0, r, r, 0.0), "a última");
    }

    /// ⭐⭐ **Um bloco de linhas DESIGUAIS ainda é um corpo só** — `3 · 3 · 2`, como a barra de
    /// ferramentas deste app: quatro cantos arredondados e mais nenhum.
    #[test]
    fn a_ragged_block_still_has_four_corners() {
        let r = 3.0;
        let rows = block_cells(Rect::new(0.0, 0.0, 120.0, 0.0), &[3, 3, 2], 22.0);
        assert_eq!(rows.len(), 3);
        assert_eq!((rows[0].len(), rows[1].len(), rows[2].len()), (3, 3, 2));
        let corners: usize = rows
            .iter()
            .flatten()
            .map(|(_, c)| {
                let (a, b, cc, d) = c.radii(r);
                [a, b, cc, d].iter().filter(|v| **v > 0.0).count()
            })
            .sum();
        assert_eq!(
            corners, 4,
            "um bloco tem QUATRO cantos, quantas linhas tiver"
        );
        assert_eq!(
            rows[1][1].1.radii(r),
            (0.0, 0.0, 0.0, 0.0),
            "o miolo é recto"
        );
    }

    /// ⭐ **As pontas são as pontas** — e é isto que uma fileira de 3 tem de dizer.
    #[test]
    fn the_ends_of_a_row_know_they_are_ends() {
        let seg = segment_rects(Rect::new(0.0, 0.0, 90.0, 22.0), 3);
        assert_eq!(seg[0].1.col, GroupPos::First);
        assert_eq!(seg[1].1.col, GroupPos::Middle);
        assert_eq!(seg[2].1.col, GroupPos::Last);
    }
}
