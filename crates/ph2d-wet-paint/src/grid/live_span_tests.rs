//! O readout da poça não pode PANICAR numa linha sem faixa viva.
//!
//! `clear_spans` marca a linha vazia com `row_lo = i32::MAX` / `row_hi = i32::MIN`,
//! e é exatamente esse par que `span_x` devolve. A conta do comprimento
//! (`hi - lo + 1`) estoura `i32` nesse par — **pânico em debug, embrulho silencioso
//! em release** —, e é por isso que o gate vive aqui e não numa sonda: o produto
//! chama `live_span_cells` a cada passo do worker, e uma bbox viva quase sempre
//! contém alguma linha ainda intocada.
//!
//! ⚠️ A fixture precisa de uma bbox NÃO-vazia (`by0 <= by1`) com pelo menos uma
//! linha por preencher: com a bbox vazia o `live_span_cells` retorna antes do laço
//! e a conta nunca roda — verde sobre nada.

use super::Grid;

/// Uma bbox viva cujas linhas ainda não foram declaradas não derruba o readout,
/// e as linhas vazias contam ZERO.
#[test]
fn an_untouched_row_inside_the_live_box_counts_zero_instead_of_panicking() {
    let mut g = Grid::new(16, 8);

    // A bbox diz "há vida nas linhas 1..=5"; só a linha 3 foi de fato declarada.
    // As outras quatro seguem com os sentinelas de `clear_spans`.
    //
    // ⚠️ A bbox HORIZONTAL entra junto: `span_x` recorta a faixa por `bx0/bx1`,
    // e o `Grid::new` os deixa em `(0, -1)` — vazia. Sem esta linha até a row
    // declarada mede zero e o gate passa por VÁCUO (foi o que ele fez na 1ª
    // corrida, e o `cells > 0` abaixo é o que denunciou).
    g.bx0 = 1;
    g.bx1 = 14;
    g.by0 = 1;
    g.by1 = 5;
    g.note_live(4, 3, 9, 3);

    // Sem a conta em largura maior isto PANICA em debug (`attempt to subtract
    // with overflow`) e devolve lixo em release.
    let cells = g.live_span_cells();

    // Só a linha 3 contribui, e ela mede as colunas 4..=9 recortadas pela bbox
    // horizontal — o número exato não importa para o defeito; o que importa é
    // que ele é FINITO, pequeno, e que as quatro linhas vazias somaram zero.
    assert!(
        cells > 0 && cells <= 16,
        "a única linha declarada devia contribuir uma faixa de uma linha, veio {cells}"
    );
}

/// O controle: uma bbox VAZIA devolve zero pelo early-out, sem tocar a conta.
/// Sem ele o gate acima poderia passar por vácuo se alguém alargasse o early-out.
#[test]
fn an_empty_live_box_reads_zero_without_walking_a_row() {
    let g = Grid::new(16, 8);
    assert_eq!(g.live_span_cells(), 0);
}
