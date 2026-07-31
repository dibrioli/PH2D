//! **O que subir depois de um dab** — o plano, separado do ato.
//!
//! Um dab move alguns milhares de vértices de uma malha de milhões. Reenviar a
//! malha inteira a cada movimento do mouse é o mesmo defeito que a
//! `line/Painter` mediu no fold do relevo em 2026-07-26: o custo do gesto vira
//! função do DOCUMENTO em vez da PEGADA, e o sintoma é um traço que engasga
//! quanto maior a peça.
//!
//! Aqui mora só a **decisão** (que faixas de vértice copiar), como função pura —
//! o padrão `hit_plan` do repo. Quem executa é o [`crate::MeshRenderer`], que
//! precisa de um device; a decisão precisa de um vetor de índices, e é por isso
//! que ela tem gate próprio rodando em qualquer máquina.

/// Índices vizinhos separados por menos que isto entram na MESMA faixa.
///
/// O trade é literal: emendar um vão de `g` vértices copia `g × 12` bytes à toa
/// e poupa uma chamada de cópia. A 64 são 768 B — bem menos que o custo fixo de
/// uma escrita de buffer, que envolve reservar espaço no *staging belt* e
/// enfileirar um comando. ⚠️ **É um número de SMOKE**, não teto de recurso: ele
/// não protege nada, só troca bytes por comandos.
pub const GAP_VERTS: u32 = 64;

/// Acima disto, o plano desiste de faixas e manda **uma** que cobre de ponta a
/// ponta.
///
/// ⚠️ Existe porque a ordem dos vértices é do ARQUIVO, não nossa: numa esfera UV
/// a pegada de um dab é quase contígua, e num OBJ de terceiro ela pode estar
/// espalhada pelo buffer inteiro. Sem o teto, um dab nessa malha viraria
/// milhares de escritas minúsculas — pior que a cópia cheia que ele evita.
pub const MAX_RUNS: usize = 64;

/// As faixas `[início, fim)` de vértice a copiar.
///
/// `sorted` tem de estar **ordenado e sem repetição**; o chamador o ordena uma
/// vez por dab. Devolve vazio para entrada vazia.
///
/// A saída é escrita em `out` em vez de alocada: isto roda por movimento do
/// mouse.
pub fn plan_runs(sorted: &[u32], out: &mut Vec<(u32, u32)>) {
    out.clear();
    let Some(&first) = sorted.first() else {
        return;
    };
    let mut start = first;
    let mut end = first + 1;
    for &v in &sorted[1..] {
        if v <= end + GAP_VERTS {
            end = end.max(v + 1);
        } else {
            out.push((start, end));
            start = v;
            end = v + 1;
        }
    }
    out.push((start, end));

    if out.len() > MAX_RUNS {
        let span = (out[0].0, out[out.len() - 1].1);
        out.clear();
        out.push(span);
    }
}

/// Quantos vértices as faixas cobrem — o que de fato viaja para o device.
#[must_use]
pub fn covered(runs: &[(u32, u32)]) -> u32 {
    runs.iter().map(|(a, b)| b - a).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_contiguous_block_is_one_run() {
        let mut out = Vec::new();
        plan_runs(&[10, 11, 12, 13], &mut out);
        assert_eq!(out, vec![(10, 14)]);
        assert_eq!(covered(&out), 4);
    }

    #[test]
    fn a_small_gap_is_bridged_and_a_big_one_is_not() {
        let mut out = Vec::new();
        // Vão de exatamente `GAP_VERTS`: emenda.
        plan_runs(&[0, 1 + GAP_VERTS], &mut out);
        assert_eq!(out, vec![(0, 2 + GAP_VERTS)]);
        // Um a mais: separa. O gate pina os DOIS lados da fronteira — um que só
        // testasse "emenda vãos pequenos" ficaria verde com a emenda infinita,
        // que é a cópia cheia disfarçada.
        plan_runs(&[0, 2 + GAP_VERTS], &mut out);
        assert_eq!(out, vec![(0, 1), (2 + GAP_VERTS, 3 + GAP_VERTS)]);
    }

    #[test]
    fn a_scattered_footprint_collapses_to_one_span_instead_of_a_thousand_writes() {
        // A ordem dos vértices é do ARQUIVO: num OBJ de terceiro a pegada de um
        // dab pode estar espalhada, e aí mil escritas minúsculas custam mais que
        // a cópia cheia que elas evitam.
        let scattered: Vec<u32> = (0..500).map(|k| k * (GAP_VERTS + 10)).collect();
        let mut out = Vec::new();
        plan_runs(&scattered, &mut out);
        assert_eq!(out.len(), 1, "o teto de faixas não segurou");
        assert_eq!(out[0].0, 0);
        assert_eq!(out[0].1, *scattered.last().unwrap() + 1);
    }

    #[test]
    fn a_local_dab_moves_far_less_than_the_whole_mesh() {
        // A razão que é a razão de existir deste módulo. Uma pegada de 2 mil
        // vértices contíguos numa malha de um milhão tem de viajar como 2 mil,
        // não como um milhão.
        let footprint: Vec<u32> = (400_000..402_000).collect();
        let mut out = Vec::new();
        plan_runs(&footprint, &mut out);
        let ratio = f64::from(covered(&out)) / 1_000_000.0;
        assert!(ratio < 0.005, "a pegada viajou como {ratio:.4} da malha");
    }

    #[test]
    fn an_empty_footprint_plans_nothing() {
        let mut out = vec![(1, 2)];
        plan_runs(&[], &mut out);
        assert!(out.is_empty());
        assert_eq!(covered(&out), 0);
    }

    #[test]
    fn a_repeated_index_does_not_open_a_backwards_run() {
        // O chamador promete ordenado e sem repetição, mas a promessa é barata
        // de honrar e cara de depurar: uma faixa com `fim < início` vira uma
        // fatia invertida e um pânico dentro do laço de ponteiro.
        let mut out = Vec::new();
        plan_runs(&[5, 5, 6, 6], &mut out);
        assert_eq!(out, vec![(5, 7)]);
        for (a, b) in out {
            assert!(a < b);
        }
    }
}
