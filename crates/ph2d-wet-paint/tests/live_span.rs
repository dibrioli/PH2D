//! **A RÉGUA DA POÇA** — `Grid::live_span_cells`, o divisor que o log do
//! produto publica ao lado do custo de cada passo.
//!
//! ⚠️ **Ela existe porque um custo sem tamanho não é atribuível.** O smoke de
//! 2026-07-31 mediu o passo a **45,52 ms pintando** contra **20,11 ms só
//! assistindo**, com o `busy` do worker praticamente igual — e duas leituras
//! cabem nisso, com curas OPOSTAS: *a poça ficou maior* (nada a consertar) ou
//! *a máquina ficou disputada* (o solver é core-limited, doc 28 §5.47). Com o
//! tamanho ao lado do custo a pergunta some.
//!
//! Os dois gates aqui afirmam as duas propriedades que a tornam publicável a
//! cada passo: ela conta o que os passes **de fato percorrem** (a faixa, não a
//! bbox — as duas divergem, e a bbox mente para cima), e é `O(linhas)`.

use ph2d_wet_paint::grid::Grid;

/// A faixa viva é o que os passes ANDAM, e ela NÃO é a bbox.
///
/// Mutação que sangra: devolver `(bx1-bx0+1) * linhas` (a bbox) — numa poça em
/// diagonal, que é a forma que um traço de verdade tem, a bbox é um múltiplo da
/// faixa e o `ns/célula` do log passa a mentir para BAIXO exatamente quando a
/// poça é grande, que é quando alguém o lê.
#[test]
fn the_ruler_counts_the_band_the_passes_walk_not_the_bounding_box() {
    let mut g = Grid::new(256, 256);
    // Uma diagonal: cada linha tem uma faixa CURTA, e a bbox as contém todas.
    g.spans_enabled = true;
    g.by0 = 1;
    g.by1 = 200;
    g.bx0 = 1;
    g.bx1 = 208;
    for y in 1..=200i32 {
        g.note_live(y, y, (y + 8).min(208), y);
    }
    let band = g.live_span_cells();
    let rows = u64::try_from(g.by1 - g.by0 + 1).expect("bbox nao vazia");
    let bbox = rows * u64::try_from(g.bx1 - g.bx0 + 1).expect("bbox nao vazia");
    assert!(band > 0, "uma poca marcada tem faixa viva");
    assert!(
        band * 4 < bbox,
        "a faixa ({band}) tem de ser MUITO menor que a bbox ({bbox}) numa diagonal — \
         se elas coincidem, a regua esta contando a caixa e o ns/celula do log mente"
    );
}

/// Uma grade sem vida mede zero — e é o `checked_div` do log que depende disso
/// (dividir por uma poça inventada daria um `ns/célula` de um passo que ninguém deu).
#[test]
fn an_empty_grid_measures_zero() {
    let g = Grid::new(64, 64);
    assert_eq!(g.live_span_cells(), 0);
}
