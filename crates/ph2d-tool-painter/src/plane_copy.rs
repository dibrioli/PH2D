//! **Copiar um plano canvas-shaped, em paralelo** — o primitivo, num lugar só.
//!
//! Duas partes do Painter copiam planos inteiros e as duas custam milissegundos a 4096²: a **porta de
//! fork** ([`crate::tool::paint`], que dá acesso exclusivo a um `Arc` compartilhado) e o **motor de
//! delta** do histórico ([`crate::undo_delta`], cuja materialização de um `Patch` começa clonando o plano
//! do cursor). São perguntas diferentes — *"posso escrever nisto?"* e *"me dê este estado"* — mas a
//! operação embaixo das duas é a mesma, e o limiar embaixo dela também.
//!
//! ⚠️ **É uma CÓPIA, e uma cópia tem uma resposta certa só.** Paralelizar muda qual thread copia qual
//! pedaço e nada mais — byte-idêntico por construção, do mesmo jeito que o fold da luz e os dois passes
//! de sculpt (ADR-0109: linhas disjuntas, leitura pura).

use rayon::prelude::*;

/// **O limiar é em BYTES, não em elementos** — e a diferença foi medida, não escolhida.
///
/// Um plano de `u8` e um de `[u8; 7]` com a mesma CONTAGEM carregam sete vezes a memória, e o que decide
/// se vale espalhar a cópia por threads é quanta memória ela move. Com o limiar em elementos a mesma
/// tela mandava o canvas para o caminho paralelo e o de material junto, e a 1024² isso **dobrou** o
/// custo de um Ctrl+Z (0,42 -> 0,86 ms) porque o fork do rayon passou a dominar quatro cópias pequenas.
///
/// Medido, um Ctrl+Z (que copia os quatro planos do cursor):
///
/// ```text
///   1024²  (planos de 1 a 7 MB)     serial  0,42 ms   paralelo  0,86 ms
///   2048²  (planos de 4 a 29 MB)    serial  3,12 ms   paralelo  3,47 ms
///   4096²  (planos de 17 a 117 MB)  serial 46,56 ms   paralelo 21,86 ms   2,1x
/// ```
///
/// A virada está entre 29 e 67 MB, então o limiar fica em **32 MB**: a 2048² tudo segue serial (onde o
/// serial ganha) e a 4096² os três planos grandes vão para o paralelo (onde ele ganha 2×). ⚠️ O número
/// grande do serial a 4096² **não é largura de banda** (5,8 GB/s é lento demais para isso): é o
/// *first-touch* de 67-117 MB recém-alocados, uma falha de página por vez — e é exatamente isso que
/// espalhar por threads conserta.
pub(crate) const PAR_MIN_BYTES: usize = 32 << 20;

/// Vale paralelizar uma cópia de `len` elementos de `T`? A pergunta é feita aqui e só aqui — a porta de
/// fork DECIDE por ela e o primitivo abaixo EXECUTA por ela, então as duas não podem divergir.
pub(crate) const fn worth_parallel<T>(len: usize) -> bool {
    len.saturating_mul(size_of::<T>()) >= PAR_MIN_BYTES
}

/// Clona um plano, em paralelo quando [`worth_parallel`] diz que vale.
pub(crate) fn par_clone<T>(src: &[T]) -> Vec<T>
where
    T: Copy + Send + Sync,
{
    if !worth_parallel::<T>(src.len()) {
        return src.to_vec();
    }
    src.par_iter().copied().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A cópia paralela é a cópia serial.** Sobre um comprimento que cruza o [`PAR_MIN_BYTES`] (senão o
    /// caminho paralelo não roda — a armadilha do ADR-0120: uma otimização que ninguém exercita é código
    /// verde que nunca executa) e outro que não.
    #[test]
    fn a_parallel_clone_is_the_serial_one() {
        let big = PAR_MIN_BYTES / size_of::<f32>() + 1_000;
        for n in [big, 64] {
            let src: Vec<f32> = (0..n).map(|i| (i as f32) * 0.25 - 3.0).collect();
            assert_eq!(par_clone(&src), src, "n = {n}");
        }
        // O tipo mais largo que o histórico guarda — 7 bytes por elemento, sem `memcpy` de fatia.
        let n7 = PAR_MIN_BYTES / 7 + 7;
        let m: Vec<[u8; 7]> = (0..n7).map(|i| [(i % 251) as u8; 7]).collect();
        assert_eq!(par_clone(&m), m);
    }

    /// **O limiar é em BYTES, e isso é o gate** — porque em ELEMENTOS ele mandava um plano de `u8` de
    /// 1 MB para o caminho paralelo junto com um de material de 7 MB, e a 1024² isso DOBROU o custo de um
    /// Ctrl+Z. A mesma CONTAGEM tem de decidir diferente conforme o tamanho do elemento.
    #[test]
    fn the_threshold_counts_bytes_not_elements() {
        let n = PAR_MIN_BYTES / 4; // 4 bytes por elemento se for `f32`, 1 se for `u8`
        assert!(!worth_parallel::<u8>(n), "u8: {n} elementos sao {n} bytes");
        assert!(worth_parallel::<f32>(n), "f32: {n} elementos sao 4x isso");
        // E o tipo mais largo do histórico cruza antes de todos.
        assert!(worth_parallel::<[u8; 7]>(PAR_MIN_BYTES / 7 + 1));
        assert!(!worth_parallel::<[u8; 7]>(PAR_MIN_BYTES / 7 - 1_000));
    }
}
