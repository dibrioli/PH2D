//! Gates da TROCA DE DIAGONAL — as cercas que o corpus do corte **não
//! alcança**, em fixturas sintéticas.
//!
//! ⛔⛔ **Elas existem porque uma medição as nomeou.** Instrumentando as seis
//! posições do corte mais a lâmina mínima, as cercas disparam assim:
//!
//! | cerca | disparos |
//! |---|---|
//! | 1a (a face é toda da PEÇA) | `9 494` |
//! | 1b (o TAMANHO da malha) | só na lâmina mínima |
//! | 3 (planura) | `47` |
//! | 4 (melhora estrita) | `1` |
//! | **2 (a aresta nova já existe)** | **`0`** |
//! | **5 (inversão)** | **`0`** |
//!
//! ⇒ as duas últimas são, no corpus real, *comentário com sintaxe de código* —
//! e a lei da casa é que uma linha que a mutação não mata não é lei. ⭐ A saída
//! **não** é apagá-las: as duas descrevem modos de falha clássicos de uma troca
//! de diagonal sobre malha de artista (aresta repetida ⇒ **não-manifold**;
//! quadrilátero não convexo ⇒ **face do avesso**), e é a mesma decisão que a
//! linha tomou com a almofada — *fixtura sintética com as duas metades*.

use super::*;

/// Corre a troca sobre um par sintético: **todos** os vértices contam como
/// novos (não há peça nenhuma aqui) e o alvo é `1,0`, para as cercas 1a e 1b
/// ficarem fora do caminho e a fixtura medir só a cerca que nomeia.
fn corre(pos: &[[f32; 3]], t: &mut [[u32; 3]]) -> usize {
    endireita_as_lascas(pos, t, &|_| true, 1.0)
}

/// O par são: `A`–`B` partilhada, `P` a `0,02` acima (aspecto `50`) e `Q` bem
/// abaixo. É a fixtura de que as outras três são variações.
fn par_sao() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    let pos = vec![
        [0.0, 0.0, 0.0],  // 0 = A
        [1.0, 0.0, 0.0],  // 1 = B
        [0.5, 0.02, 0.0], // 2 = P  (a lasca)
        [0.5, -0.5, 0.0], // 3 = Q
    ];
    (pos, vec![[0, 1, 2], [1, 0, 3]])
}

/// ⭐ **O CONTROLO: num par são a troca ACONTECE.**
///
/// Sem ele as três recusas abaixo passariam sobre uma lei que nunca troca nada.
#[test]
fn o_controlo_num_par_sao_a_troca_acontece() {
    let (pos, mut t) = par_sao();
    assert!(
        aspecto(pos[0], pos[1], pos[2]) > ASPECTO_DA_LASCA,
        "a fixtura deixou de conter uma lasca: aspecto {}",
        aspecto(pos[0], pos[1], pos[2])
    );
    assert_eq!(corre(&pos, &mut t), 1, "a troca não aconteceu num par são");
    // A diagonal passou a ser `P`–`Q`: nenhuma das faces contém `A` e `B` juntos.
    assert!(
        !t.iter().any(|x| x.contains(&0) && x.contains(&1)),
        "a aresta A-B sobreviveu à troca: {t:?}"
    );
}

/// ⛔ **CERCA 2 — a troca recusa quando a aresta NOVA já existe.**
///
/// Sem ela a aresta `P`–`Q` ficaria com **três** faces, que é o não-manifold que
/// a porta do corte existe para não emitir.
#[test]
fn a_troca_recusa_quando_a_aresta_nova_ja_existe() {
    let (mut pos, mut t) = par_sao();
    pos.push([1.5, -0.25, 0.0]); // 4 = R
    t.push([2, 3, 4]); // a face que já liga P a Q
    let antes = t.clone();
    assert_eq!(
        corre(&pos, &mut t),
        0,
        "a troca aconteceu e a aresta P-Q passou a ter três faces"
    );
    assert_eq!(t, antes, "a lei mexeu na lista com a troca recusada");
}

/// ⛔ **CERCA 5 — a troca recusa quando INVERTERIA uma face.**
///
/// ⚠️ **A fixtura precisa de uma face de área ZERO**, e isso não é conveniência:
/// num par PLANO e coerentemente orientado o quadrilátero é sempre convexo
/// (`P` acima e `Q` abaixo da recta `A`–`B`), logo a troca **nunca** inverte —
/// e a cerca 3 obriga a quase-planura. *A inversão só é alcançável pelo braço
/// que trata a face degenerada como plana*, que é exactamente a junta em T que
/// o motor sela com uma face sem área.
///
/// Aqui `Q` é **colinear com `A`–`B` e para lá de `B`**: a face `(B, A, Q)` tem
/// área zero, e a troca produziria `(B, P, Q)` virada ao contrário.
#[test]
fn a_troca_recusa_quando_inverteria_uma_face() {
    let (mut pos, mut t) = par_sao();
    pos[3] = [1.3, 0.0, 0.0]; // Q colinear, para lá de B
    let antes = t.clone();
    // A metade que torna a outra uma afirmação: a troca de facto INVERTERIA.
    let z = |a: [f32; 3], b: [f32; 3], c: [f32; 3]| normal_crua(a, b, c)[2];
    assert!(
        z(pos[1], pos[2], pos[3]) * z(pos[0], pos[1], pos[2]) < 0.0,
        "a fixtura deixou de conter a inversão"
    );
    assert_eq!(corre(&pos, &mut t), 0, "a troca emitiu uma face do avesso");
    assert_eq!(t, antes, "a lei mexeu na lista com a troca recusada");
}
