//! ⭐⭐⭐ **A PARTIÇÃO EM CÉLULAS e a ORDEM DE VISITA da malha** (espec §3.1-bis).
//!
//! ⛔⛔ **Esta não é uma optimização espacial — é METADE DA LEI.** A ordem em que
//! os vértices são visitados fixa a ordem em que as restrições entram na lista, a
//! lista é resolvida em Gauss–Seidel, e Gauss–Seidel não comuta. Medido em
//! 2026-09-06 sobre as 65 fixtures do oráculo: varrer por índice crescente em vez
//! de por célula deixa `30` traços acima da barra de paridade; pela célula ficam
//! `15`, e `17` saem praticamente ao bit.
//!
//! ⚠️ **A partição é da MALHA, não do traço:** calcula-se uma vez, sobre as
//! posições de repouso, e não muda enquanto o traço corre. O cursor entra só no
//! passo que escolhe QUAIS células ficam activas (§2.1).

use crate::V3;

/// O tecto de faces de uma folha (espec §3.1-bis).
pub const FACES_POR_FOLHA: usize = 2500;
/// A profundidade máxima da bissecção.
pub const PROFUNDIDADE_MAX: u32 = 99;

/// O **centro de uma face**: o ponto médio da CAIXA dos vértices dela.
///
/// ⛔ **Não é o centróide** — num quad alongado as duas coisas não coincidem, e é
/// o ponto médio da caixa que decide de que lado da bissecção a face cai.
fn centro_da_face(posicoes: &[V3], face: &[u32]) -> V3 {
    let mut lo = [f64::INFINITY; 3];
    let mut hi = [f64::NEG_INFINITY; 3];
    for &v in face {
        let p = posicoes[v as usize];
        for c in 0..3 {
            lo[c] = lo[c].min(p[c]);
            hi[c] = hi[c].max(p[c]);
        }
    }
    [
        (lo[0] + hi[0]) * 0.5,
        (lo[1] + hi[1]) * 0.5,
        (lo[2] + hi[2]) * 0.5,
    ]
}

/// A caixa de um conjunto de pontos.
fn caixa(pontos: impl Iterator<Item = V3>) -> ([f64; 3], [f64; 3]) {
    let mut lo = [f64::INFINITY; 3];
    let mut hi = [f64::NEG_INFINITY; 3];
    for p in pontos {
        for c in 0..3 {
            lo[c] = lo[c].min(p[c]);
            hi[c] = hi[c].max(p[c]);
        }
    }
    (lo, hi)
}

/// O **eixo de maior extensão**, com o desempate da referência.
///
/// ⚠️ **`X` só ganha se for ESTRITAMENTE maior que os outros dois**, e entre `Y` e
/// `Z` empatados ganha `Z` — o empate vai sempre para o eixo de índice mais alto.
/// *Numa folha plana em `z = 0` os extremos de `x` e `y` empatam, e é este
/// desempate que manda a bissecção pelo `y`.*
fn eixo_maior(lo: [f64; 3], hi: [f64; 3]) -> usize {
    let e = [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]];
    if e[0] > e[1] && e[0] > e[2] {
        0
    } else if e[1] > e[2] {
        1
    } else {
        2
    }
}

/// Um nó da bissecção: as faces que lhe cabem e a caixa que ele parte.
struct No {
    faces: Vec<u32>,
    lo: [f64; 3],
    hi: [f64; 3],
}

/// **A ORDEM DE VISITA da malha** (espec §3.1-bis): a concatenação, por folha em
/// índice crescente, dos **vértices próprios** de cada uma em índice crescente.
///
/// ⭐ **Os vértices próprios PARTICIONAM a malha**: percorrendo as folhas por
/// índice crescente, cada uma **reclama** os vértices das faces dela que ainda
/// ninguém reclamou. ⇒ a soma dos próprios bate `N` — descontados os vértices que
/// **nenhuma face usa**, que não pertencem a célula nenhuma e por isso ⚠️ **nunca
/// são simulados nem integrados**, sob área nenhuma.
///
/// ⚠️ **A recursão desce PRIMEIRO no filho do lado `≥`**, e os dois filhos são
/// reservados em par no fim do vector ⇒ os índices são atribuídos em profundidade
/// primeiro, e é essa numeração que a ordem das folhas usa.
#[must_use]
pub fn ordem_de_visita<F: AsRef<[u32]>>(posicoes: &[V3], faces: &[F]) -> Vec<u32> {
    ordem_de_visita_com_eixo_raiz(posicoes, faces, None)
}

/// A mesma partição, com o **eixo da bissecção da raiz** dado em vez de derivado.
///
/// ⚠️⚠️ **Esta costura existe por uma razão MEDIDA, e não por conveniência de
/// teste** (2026-09-06). Na esfera das fixtures o desempate do eixo é
/// **irresolúvel com a precisão que temos**: as posições de repouso vêm a seis
/// casas, e nelas as extensões de `x` e de `y` são **exactamente iguais**
/// (`2,000001` as duas) enquanto a de `z` é `2,000000`. Pela regra da §3.1-bis o
/// empate entre `x` e `y` dá `Y`; o alvo, medido, parte por `X` — ⇒ *nos dados
/// dele `x` era estritamente maior, e as seis casas apagaram os bits que o
/// decidiam.*
///
/// ⛔ **Não é uma lei alternativa nem uma afinação:** com `None` o produto corre a
/// regra da espec, e a costura serve para o gate poder afirmar **qual** é a
/// ambiguidade — que na esfera resolver o empate por `X` reproduz o alvo
/// elemento a elemento e por `Y` não. ⚠️ **No plano não há ambiguidade nenhuma**
/// (`x = y = 3` e `z = 0`, e o alvo parte por `Y`, que é o que a regra dá), e é
/// por isso que ele é o lado que o gate afirma sem costura.
#[must_use]
pub fn ordem_de_visita_com_eixo_raiz<F: AsRef<[u32]>>(
    posicoes: &[V3],
    faces: &[F],
    eixo_raiz: Option<usize>,
) -> Vec<u32> {
    if faces.is_empty() {
        return Vec::new();
    }
    let centros: Vec<V3> = faces
        .iter()
        .map(|f| centro_da_face(posicoes, f.as_ref()))
        .collect();
    // A raiz recebe a caixa da MALHA INTEIRA (a fusão das caixas das faces); um
    // nó interior recalcula a sua como a caixa dos CENTROS das faces dele.
    let (lo, hi) = caixa(faces.iter().flat_map(|f| {
        let (a, b) = caixa(f.as_ref().iter().map(|&v| posicoes[v as usize]));
        [a, b]
    }));
    let mut nos = vec![No {
        faces: (0..u32::try_from(faces.len()).unwrap_or(u32::MAX)).collect(),
        lo,
        hi,
    }];
    let mut folhas: Vec<usize> = Vec::new();
    // Pilha de (nó, profundidade); a ordem de POP tem de descer primeiro no `≥`.
    let mut pilha = vec![(0usize, 0u32)];
    while let Some((n, prof)) = pilha.pop() {
        if nos[n].faces.len() <= FACES_POR_FOLHA || prof >= PROFUNDIDADE_MAX {
            folhas.push(n);
            continue;
        }
        let eixo = match eixo_raiz {
            Some(e) if n == 0 => e,
            _ => eixo_maior(nos[n].lo, nos[n].hi),
        };
        let limiar = (nos[n].lo[eixo] + nos[n].hi[eixo]) * 0.5;
        let (mut maior, mut menor) = (Vec::new(), Vec::new());
        for &f in &nos[n].faces {
            if centros[f as usize][eixo] >= limiar {
                maior.push(f);
            } else {
                menor.push(f);
            }
        }
        // ⛔ Uma partição que não separa nada faria a recursão não terminar.
        if maior.is_empty() || menor.is_empty() {
            folhas.push(n);
            continue;
        }
        let base = nos.len();
        for lado in [maior, menor] {
            let (a, b) = caixa(lado.iter().map(|&f| centros[f as usize]));
            nos.push(No {
                faces: lado,
                lo: a,
                hi: b,
            });
        }
        // Empilha o `<` primeiro para que o `≥` seja o próximo a sair.
        pilha.push((base + 1, prof + 1));
        pilha.push((base, prof + 1));
    }
    folhas.sort_unstable();
    let mut reclamado = vec![false; posicoes.len()];
    let mut ordem = Vec::with_capacity(posicoes.len());
    for n in folhas {
        let mut proprios: Vec<u32> = Vec::new();
        for &f in &nos[n].faces {
            for &v in faces[f as usize].as_ref() {
                if !reclamado[v as usize] {
                    reclamado[v as usize] = true;
                    proprios.push(v);
                }
            }
        }
        proprios.sort_unstable();
        ordem.extend(proprios);
    }
    ordem
}

#[cfg(test)]
mod testes {
    use super::*;

    /// ⭐⭐ **O centro de uma face é o ponto médio da CAIXA, não o CENTRÓIDE**
    /// (espec §3.1-bis).
    ///
    /// ⛔⛔ **Este gate existe porque a mutação que troca os dois SOBREVIVEU ao
    /// corpus inteiro**, e a razão é a terceira das três: *a fixtura não produz o
    /// fenómeno.* Nas duas malhas do oráculo — uma grelha de quads planos e uma
    /// esfera UV — os quads são simétricos o bastante para as duas leituras
    /// coincidirem à escala que decide de que lado da bissecção uma face cai.
    /// ⇒ a lei fica afirmada aqui, sobre um quad **deliberadamente enviesado**,
    /// onde as duas divergem por construção.
    #[test]
    fn o_centro_de_uma_face_e_o_meio_da_caixa_e_nao_o_centroide() {
        // Três cantos amontoados perto da origem e um longe: o centróide é
        // puxado pelos três, o meio da caixa não sabe quantos são.
        let pos = vec![
            [0.0, 0.0, 0.0],
            [0.1, 0.0, 0.0],
            [0.1, 0.1, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let face = [0u32, 1, 2, 3];
        let c = centro_da_face(&pos, &face);
        assert!(
            (c[0] - 0.5).abs() < 1e-12 && (c[1] - 0.5).abs() < 1e-12,
            "o centro nao e' o meio da caixa: {c:?}"
        );
        let centroide = [
            (0.0 + 0.1 + 0.1 + 1.0) / 4.0,
            (0.0 + 0.0 + 0.1 + 1.0) / 4.0,
            0.0,
        ];
        assert!(
            (c[0] - centroide[0]).abs() > 0.15,
            "a fixtura nao separa as duas leituras: {c:?} contra {centroide:?}"
        );
    }

    /// ⭐ **Uma malha com uma folha só visita por índice crescente** — a partição
    /// só morde acima de [`FACES_POR_FOLHA`], e é honesto dizê-lo.
    #[test]
    fn abaixo_do_tecto_da_folha_a_ordem_e_a_crescente() {
        let n = 6usize;
        let mut pos = Vec::new();
        for j in 0..n {
            for i in 0..n {
                pos.push([i as f64, j as f64, 0.0]);
            }
        }
        let idx = |i: usize, j: usize| u32::try_from(j * n + i).expect("u32");
        let mut faces = Vec::new();
        for j in 0..n - 1 {
            for i in 0..n - 1 {
                faces.push(vec![
                    idx(i, j),
                    idx(i + 1, j),
                    idx(i + 1, j + 1),
                    idx(i, j + 1),
                ]);
            }
        }
        assert!(
            faces.len() <= FACES_POR_FOLHA,
            "a fixtura tinha de caber numa folha"
        );
        let ordem = ordem_de_visita(&pos, &faces);
        let crescente: Vec<u32> = (0..u32::try_from(pos.len()).expect("u32")).collect();
        assert_eq!(ordem, crescente, "com uma folha so' a ordem e' a crescente");
    }
}
