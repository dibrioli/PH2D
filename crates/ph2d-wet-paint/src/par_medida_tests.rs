//! A [`Rows::Medida`] com uma redução NÃO idempotente: a soma inteira apanha o item do meio perdido ou contado
//! duas vezes, que o max-reduce do advect (o consumidor do produto) não vê.
use super::*;

fn soma(mode: Rows) -> u64 {
    // O pico no item do MEIO: é ele que a rota medida corre à parte.
    let mut linhas: Vec<u64> = (0..64)
        .map(|i| if i == 32 { 1_000_000 } else { i })
        .collect();
    walk_rows_reduce(
        mode,
        &mut linhas,
        1,
        0u64,
        |r, l| l[0] * (r as u64 + 1),
        |a, b| a + b,
    )
}

#[test]
fn a_reducao_medida_conta_cada_linha_uma_vez() {
    let serie = soma(Rows::Serial);
    for piso in [0, u64::MAX] {
        assert_eq!(soma(Rows::Medida(piso)), serie, "Medida({piso})");
    }
    let mut itens: Vec<u64> = (0..64).collect();
    let r = walk_items_reduce(Rows::Medida(0), &mut itens, 0u64, |x| *x, |a, b| a + b);
    assert_eq!(r, (0..64).sum::<u64>());
}
