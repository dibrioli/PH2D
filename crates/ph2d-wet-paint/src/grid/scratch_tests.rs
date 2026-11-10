//! B12.4 (missão 4K): o 1.º [`SolverScratch::ensure`] não ESCREVE a tela — todo plano nasce a zeros, a página
//! que o sistema dá preguiçosa (`alloc_zeroed`). O `bt` a [`SolverScratch::BT_NADA`] eram `n` bytes escritos
//! (8,3 MB a 2880²) que nenhum passe lê: o `prepare_rows` escreve a faixa viva que o `outflow_rows` lê. A
//! paridade ao bit do passo é a de `advect_jacobi_bits_are_the_recorded_ones` (constante gravada antes).
use super::SolverScratch;

/// Os elementos que o `ensure` deixou diferentes de zero (bit a bit) — o que ele teve de ESCREVER.
fn escritos(s: &SolverScratch) -> usize {
    let f = |v: &[f32]| v.iter().filter(|x| x.to_bits() != 0).count();
    f(s.uv.as_flattened())
        + f(&s.outflow)
        + s.bt.iter().filter(|&&x| x != 0).count()
        + f(&s.dst_film)
        + f(&s.dst_susp)
        + f(s.dst_rgb.as_flattened())
        + s.edge.iter().filter(|&&x| x != 0).count()
}

#[test]
fn o_primeiro_ensure_nao_escreve_o_rascunho() {
    let n = 4096;
    let mut s = SolverScratch::default();
    s.ensure(n);
    assert_eq!(s.bt.len(), n);
    assert_eq!(
        escritos(&s),
        0,
        "o ensure escreveu planos que nenhum passe lê"
    );
}
