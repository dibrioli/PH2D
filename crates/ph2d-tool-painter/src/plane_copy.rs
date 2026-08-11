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

/// **Materializa um plano de `n` elementos no valor `value`** — e o caso do ZERO é de graça.
///
/// ⚠️ **`Vec::resize` num vetor VAZIO escreve elemento a elemento**, mesmo quando o valor é zero;
/// `vec![zero; n]` pede ao SO páginas **já zeradas** (a especialização `IsZero` da `std`) e não toca um
/// byte. A diferença é medida e não é pequena: o primeiro commit de um documento a 4096² paga **2,05 ms**
/// só para materializar o plano de COBERTURA, que são 16,8 MB de zeros escritos à mão.
///
/// ⚠️ **E o valor NÃO-zero continua a ser escrito** (o material começa em `NEUTRAL`, não em zero — zero
/// é `roughness = 0`, que é ESPELHO): aí não há atalho de alocador, e o número fica NOMEADO em vez de
/// escondido — **19,50 ms** no mesmo commit, o maior item isolado de um pen-up.
///
/// ⛔ **MEDIDO E REJEITADO, não refaça: preencher por DUPLICAÇÃO** (escrever um elemento e copiar o
/// prefixo sobre o dobro do espaço, fazendo de cada passo um `memcpy`). Foi construída e medida: **17,5
/// contra 18,7 ms** para os 117 MB do material a 4096², 6% — porque os dois estão no MESMO teto. 117 MB
/// em 17 ms são **6,9 GB/s**: o custo é o *first-touch* das páginas, não o laço que as escreve, e
/// nenhuma esperteza sobre COMO escrever muda quantas páginas há para tocar. Quinze linhas por 6% num
/// custo de uma vez por documento não se pagam.
///
/// ⚠️ **O que resta é um piso, e ele tem nome:** a única forma de não pagar os 18 ms é o plano de
/// material não ser canvas-shaped (esparso/por tile) ou ser pago noutro momento — a mesma decisão de
/// produto que o `prewarm` da luz já tem em aberto, e pelo mesmo preço (memória em TODO bind, para quem
/// nunca deposita material).
pub(crate) fn size_to<T>(dst: &mut Vec<T>, n: usize, value: T)
where
    T: Copy + Default + PartialEq,
{
    if dst.len() == n {
        return;
    }
    if !dst.is_empty() {
        dst.resize(n, value); // tela que mudou de tamanho: o conteúdo existente manda
        return;
    }
    if value == T::default() {
        // `alloc_zeroed`: o SO entrega as páginas prontas e ninguém escreve um byte.
        *dst = vec![T::default(); n];
        return;
    }
    dst.resize(n, value);
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

    /// **A porta devolve o mesmo plano que o `resize` devolvia** — nos dois ramos, e num vetor que já
    /// tem conteúdo (o caso da tela que mudou de tamanho, onde o atalho do zero não pode disparar).
    #[test]
    fn sizing_a_plane_gives_what_resize_gave() {
        for n in [4usize, 1_000] {
            let (mut a, mut b) = (Vec::<u8>::new(), Vec::<u8>::new());
            size_to(&mut a, n, 0);
            b.resize(n, 0);
            assert_eq!(a, b, "o ramo do zero, n = {n}");

            let neutral = [3u8, 1, 4, 1, 5, 9, 2];
            let (mut c, mut d) = (Vec::<[u8; 7]>::new(), Vec::<[u8; 7]>::new());
            size_to(&mut c, n, neutral);
            d.resize(n, neutral);
            assert_eq!(c, d, "o ramo do valor, n = {n}");

            // Já com conteúdo: o atalho do zero NÃO pode disparar, senão apagaria o que lá está.
            let mut e = vec![7u8; 3];
            size_to(&mut e, n.max(3), 0);
            assert_eq!(
                &e[..3],
                &[7, 7, 7],
                "o atalho do zero apagou conteudo existente"
            );
        }
    }
}
