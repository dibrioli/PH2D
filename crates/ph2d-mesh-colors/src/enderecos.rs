//! ⭐⭐⭐ **O ENDEREÇO DE UMA AMOSTRA** — a aritmética que faz a fronteira ser
//! PARTILHADA.
//!
//! A disposição é **três blocos contíguos**, e a ordem não é arrumação:
//!
//! ```text
//!   [ 0 .. V )                         uma amostra por VÉRTICE
//!   [ V .. V + Σ(Lₑ−1) )               Lₑ−1 por ARESTA, do vértice MENOR para o maior
//!   [ V + Σ(Lₑ−1) .. )                 o INTERIOR de cada face, ao nível DELA
//! ```
//!
//! ⚠️ **Os dois blocos de baixo são SOMAS e não produtos desde a P2:** cada face
//! tem o nível dela e cada aresta leva o **máximo** dos dois vizinhos. Com um
//! nível uniforme as somas valem `A·(L−1)` e `F·interior(L)` **exactamente**, e
//! é isso que faz o plano graduado ser um superconjunto do uniforme em vez de
//! um formato novo.
//!
//! ⭐⭐⭐ **O bloco dos vértices vem PRIMEIRO e na numeração da malha, e é isso que
//! torna o nível base byte-idêntico à cor por-vértice de hoje:** a `L = 1` os
//! outros dois blocos têm comprimento zero e `amostras[v]` **é** `colors[v]`.
//! *Uma família nova cujo caso base é o produto que já shipa não precisa de uma
//! migração.*
//!
//! # A retícula de um triângulo
//!
//! Os pontos são `(i, j, k)` inteiros com `i + j + k = L`, e a posição é
//! `(i·A + j·B + k·C) / L` — **afim**, logo a densidade de amostras por área é
//! exactamente uniforme dentro da face (é essa a frase da §5 do doc 27).
//!
//! | onde | condição | quem guarda |
//! |---|---|---|
//! | canto | duas coordenadas a zero | o **vértice** |
//! | aresta | exactamente uma a zero | a **aresta**, partilhada com a face do outro lado |
//! | interior | nenhuma a zero | a **face** |
//!
//! # ⛔ E a de um quad é `(i, j)` em `0..=L`, com `j = 0` a ser a aresta `a→b`
//!
//! O percurso de uma face é `a, b, c, d`, logo o lado `2` vai de `c` para `d` e
//! o lado `3` de `d` para `a` — **as duas contam `t` ao contrário do eixo**, e é
//! aí que uma leitura distraída espelha meia peça.

use crate::topo::Topologia;

/// Onde uma amostra da retícula mora.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sitio {
    /// O canto `c` da face (índice **local**: `0..cantos`).
    Canto(usize),
    /// O lado `s` da face, a `t` passos do início dele (`1 <= t < lado`).
    Aresta { lado_da_face: usize, t: u32 },
    /// O interior, com o índice já resolvido dentro da face.
    Interior(u32),
}

/// ⭐ **Classifica um ponto `(i, j, k)` da retícula de um TRIÂNGULO.**
///
/// ⚠️ **A ordem dos ramos é a lei:** um canto satisfaz *duas* condições de
/// aresta ao mesmo tempo, então perguntar «está numa aresta?» antes de «é um
/// canto?» dá à amostra do canto **um endereço de aresta** — e aí dois cantos
/// diferentes da mesma aresta colidem no mesmo `t`. *Uma classificação por
/// exclusão tem de começar pelo caso mais específico.*
#[must_use]
pub fn sitio_tri(lado: u32, i: u32, j: u32, k: u32) -> Sitio {
    debug_assert_eq!(i + j + k, lado, "um ponto da retícula soma `lado`");
    match (i == 0, j == 0, k == 0) {
        (false, true, true) => Sitio::Canto(0),
        (true, false, true) => Sitio::Canto(1),
        (true, true, false) => Sitio::Canto(2),
        // lado 0 = a→b, com `t` a contar de `a`: em `k = 0` andar é subir `j`.
        (_, _, true) => Sitio::Aresta {
            lado_da_face: 0,
            t: j,
        },
        // lado 1 = b→c, `t` conta de `b`: em `i = 0` andar é subir `k`.
        (true, _, _) => Sitio::Aresta {
            lado_da_face: 1,
            t: k,
        },
        // lado 2 = c→a, `t` conta de `c`: em `j = 0` andar é subir `i`.
        (_, true, _) => Sitio::Aresta {
            lado_da_face: 2,
            t: i,
        },
        _ => Sitio::Interior(interior_tri(lado, i, j)),
    }
}

/// O índice de `(i, j)` dentro do interior de um triângulo, `i, j >= 1`.
///
/// A enumeração é por **fileiras de `j`**: a fileira `j` tem `L − 1 − j`
/// pontos (`i` de `1` a `L − 1 − j`), e o prefixo é a soma das anteriores.
#[must_use]
pub fn interior_tri(lado: u32, i: u32, j: u32) -> u32 {
    debug_assert!(i >= 1 && j >= 1 && i + j < lado);
    let l = lado - 1;
    // Σ_{j'=1}^{j-1} (l − j') = (j−1)·l − (j−1)j/2
    let antes = (j - 1) * l - (j - 1) * j / 2;
    antes + (i - 1)
}

/// ⭐ **Classifica um ponto `(i, j)` da retícula de um QUAD**, `0 <= i, j <= L`.
#[must_use]
pub fn sitio_quad(lado: u32, i: u32, j: u32) -> Sitio {
    debug_assert!(i <= lado && j <= lado);
    let (i0, i_max, j0, j_max) = (i == 0, i == lado, j == 0, j == lado);
    match (i0, i_max, j0, j_max) {
        (true, _, true, _) => Sitio::Canto(0),
        (_, true, true, _) => Sitio::Canto(1),
        (_, true, _, true) => Sitio::Canto(2),
        (true, _, _, true) => Sitio::Canto(3),
        // a→b: `t = i`
        (_, _, true, _) => Sitio::Aresta {
            lado_da_face: 0,
            t: i,
        },
        // b→c: `t = j`
        (_, true, _, _) => Sitio::Aresta {
            lado_da_face: 1,
            t: j,
        },
        // ⚠️ c→d ANDA PARA TRÁS no eixo `i`.
        (_, _, _, true) => Sitio::Aresta {
            lado_da_face: 2,
            t: lado - i,
        },
        // ⚠️ d→a ANDA PARA TRÁS no eixo `j`.
        (true, _, _, _) => Sitio::Aresta {
            lado_da_face: 3,
            t: lado - j,
        },
        _ => Sitio::Interior((j - 1) * (lado - 1) + (i - 1)),
    }
}

/// ⭐⭐⭐ **O índice GLOBAL de uma amostra** — a porta por onde todo consumidor
/// passa.
///
/// ⛔ Ela é a única que sabe a disposição dos três blocos e a única que sabe
/// virar o `t` de uma aresta percorrida ao contrário. *Uma segunda cópia desta
/// aritmética é a forma exacta como metade da peça fica espelhada.*
#[must_use]
pub fn indice(topo: &Topologia, face: usize, sitio: Sitio, cantos: &[u32]) -> u32 {
    let v = topo.verts as u32;
    match sitio {
        Sitio::Canto(c) => cantos[c],
        Sitio::Aresta { lado_da_face, t } => {
            let (id, virada) = topo.aresta(face, lado_da_face);
            // ⭐⭐⭐⭐ **A LEI DO SUBCONJUNTO, e ela vive AQUI e em mais lado
            //   nenhum.** A face conta `t` na retícula DELA (`0..lf`) e a aresta
            //   guarda as amostras na retícula DELA (`0..le`, o máximo dos dois
            //   vizinhos). O passo é inteiro porque as duas são potências de dois
            //   e `le >= lf` ⇒ a amostra da face cai **em cima** de uma da aresta.
            //   ⛔ *Uma segunda cópia desta multiplicação é como metade de uma
            //   peça fica com a tinta da vizinha.*
            let lf = topo.lado_de(face);
            let le = topo.aresta_lado(id);
            debug_assert!(
                le >= lf && le.is_multiple_of(lf),
                "a aresta é um múltiplo da face"
            );
            let t = t * (le / lf);
            let t = if virada { le - t } else { t };
            debug_assert!(t >= 1 && t < le, "uma amostra de aresta não é um canto");
            v + topo.aresta_off(id) + (t - 1)
        }
        Sitio::Interior(n) => v + topo.arestas_amostras() + topo.off_interior[face] + n,
    }
}

/// Quantas amostras a malha inteira tem.
#[must_use]
pub fn total(topo: &Topologia) -> usize {
    topo.verts + topo.arestas_amostras() as usize + *topo.off_interior.last().unwrap_or(&0) as usize
}
