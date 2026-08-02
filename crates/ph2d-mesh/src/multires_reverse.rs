//! **A REVERSÃO, do lado da PILHA** — o nível novo entra por BAIXO.
//!
//! Módulo FILHO do [`super`] e não irmão: ele mexe nos `levels`, nos `details` e
//! no `sel`, que são privados da pilha de propósito. Um irmão precisaria que
//! eles virassem `pub(crate)`, e aí a próxima wave escreveria neles sem passar
//! pelas leis que o `lower`/`higher` mantêm.
//!
//! # Todo nível ACIMA é renumerado, e é isso que custa
//!
//! A [`Multires`] se apoia em *o vértice `i` de baixo É o vértice `i` de cima*.
//! Inserir um nível embaixo obriga a base antiga a assumir a numeração que
//! `subdivide(novo)` impõe — e essa renumeração **desce em cascata**: o nível 2
//! é numerado pelo 1, o 3 pelo 2, e assim por diante.
//!
//! ⚠️ **A cascata sai das FACES, não de aritmética de índices.** As faces do
//! nível `k` são as mesmas antes e depois (só os índices dentro delas mudam),
//! então o canto `(f, c)` de antes e o de depois descrevem a MESMA aresta: a
//! permutação dos pontos de aresta é `aresta_nova(f, c) → aresta_velha(f, c)`,
//! lida das duas tabelas. Derivá-la de outro jeito seria uma segunda resposta
//! para uma pergunta que o grafo já responde.
//!
//! # O desfazer guarda PERMUTAÇÕES, não malhas
//!
//! [`Reversal`] carrega um `u32` por vértice por nível — um terço do que custa
//! um plano de posições, e a alternativa (clonar a pilha inteira) seria dobrar o
//! modelo na fila de desfazer de um gesto que o artista faz uma vez.
//!
//! ⚠️ **E o REFAZER não guarda nada**: [`Multires::reverse`] é função pura da
//! malha, e desfazer devolve a malha ao bit (permutar e despermutar move dados,
//! não os computa) — então refazer é chamá-la de novo e receber o mesmo
//! resultado. É o oposto do [`Multires::drop_top`], que TEM de entregar o nível
//! porque quem o refizesse subdividiria uma base que o carimbo já mudou.

use super::{Details, Multires, encode};
use crate::face::Face;
use crate::mesh::Mesh;
use crate::reversion::reverse_subdivision;
use crate::subdivide::predict;

/// O que uma reversão precisa para ser desfeita: a renumeração que cada nível
/// sofreu, de cima para baixo.
///
/// Opaco: quem o segura (a fila de desfazer do editor) não tem nada a perguntar
/// a ele — só a devolvê-lo inteiro, como o [`super::DetachedLevel`].
#[derive(Clone, Debug)]
pub struct Reversal {
    /// `perms[k]` é a permutação (novo → velho) do nível que virou `k + 1`.
    perms: Vec<Vec<u32>>,
}

impl Reversal {
    /// Bytes segurados — o que a fila de desfazer paga por esta entrada.
    #[must_use]
    pub fn bytes(&self) -> usize {
        self.perms.iter().map(|p| p.capacity() * 4).sum()
    }
}

impl Multires {
    /// **Reconstrói o nível de BAIXO** a partir da base, e o insere. Devolve o
    /// que desfazê-la exige, ou `None` se a base não é uma subdivisão.
    ///
    /// ⚠️ **Só a partir do nível 0**, e a recusa é estrutural: quem está no meio
    /// está olhando para uma malha que a inserção vai RENUMERAR debaixo dele, e
    /// não há resposta boa para *"onde estava o vértice que eu tinha em mãos?"*.
    /// Descer primeiro é o gesto que o artista já conhece.
    ///
    /// ⚠️ **Nada é mutado antes de a reconstrução INTEIRA ter sucesso.** A
    /// recusa acontece na parte pura (a etiquetagem e a verificação de
    /// bijeção), então uma malha que não é subdivisão sai daqui com a pilha
    /// exatamente como entrou — o oposto de uma pilha meio-renumerada, que
    /// nenhum gate distinguiria de uma malha corrompida pelo pincel.
    pub fn reverse(&mut self) -> Option<Reversal> {
        if self.sel != 0 {
            return None;
        }
        let (coarse, map) = reverse_subdivision(&self.levels[0])?.into_parts();

        let levels = self.levels.len();
        let mut perms: Vec<Vec<u32>> = Vec::with_capacity(levels);
        let mut p = map;
        for k in 0..levels {
            let old_edges = self.levels[k].edges();
            let vk = self.levels[k].vert_count();
            permute_level(&mut self.levels[k], &mut self.details[k], &p);
            perms.push(p);
            if k + 1 == levels {
                break;
            }
            let new_edges = self.levels[k].edges();
            let mut q = vec![u32::MAX; self.levels[k + 1].vert_count()];
            q[..vk].copy_from_slice(&perms[k]);
            for (f, face) in self.levels[k].faces().iter().enumerate() {
                for c in 0..face.vert_count() {
                    // Os dois grafos descrevem as MESMAS faces, então todo canto
                    // tem aresta nos dois. Um `expect` e não um índice de
                    // reserva: cair num inventado produziria uma pilha que sobe
                    // e desce embaralhada, sem erro nenhum.
                    let en = new_edges.face_edge(f, c).expect("canto tem aresta");
                    let eo = old_edges.face_edge(f, c).expect("canto tem aresta");
                    q[vk + en as usize] = vk as u32 + eo;
                }
            }
            // Os pontos de FACE não se movem: eles são numerados pela ordem das
            // faces, e permutar vértices não reordena face nenhuma.
            for (j, slot) in q.iter_mut().enumerate().skip(vk + new_edges.len()) {
                *slot = u32::try_from(j).unwrap_or(u32::MAX);
            }
            p = q;
        }

        self.levels.insert(0, coarse);
        self.details.insert(0, Details::default());
        // O detalhe do que agora é o nível 1 tem de EXISTIR: ele era o do nível
        // 0, que é vazio por definição. É ele que devolve a forma exata que a
        // malha grossa (posições copiadas, não invertidas) não carrega.
        let predicted = predict(&self.levels[0]);
        self.details[1] = encode(&self.levels[1], &predicted);
        self.sel += 1;
        Some(Reversal { perms })
    }

    /// **Desfaz uma reversão**: despermuta cada nível e tira a base nova.
    /// `false` se a pilha não está onde a reversão a deixou.
    pub fn unreverse(&mut self, r: &Reversal) -> bool {
        if self.sel != 1 || self.levels.len() != r.perms.len() + 1 {
            return false;
        }
        for (k, p) in r.perms.iter().enumerate() {
            let mut inv = vec![0u32; p.len()];
            for (j, &o) in p.iter().enumerate() {
                inv[o as usize] = u32::try_from(j).unwrap_or(u32::MAX);
            }
            permute_level(&mut self.levels[k + 1], &mut self.details[k + 1], &inv);
        }
        self.levels.remove(0);
        self.details.remove(0);
        // O que sobrou em `details[0]` é o detalhe COMPUTADO pela reversão, e o
        // detalhe do nível 0 é vazio por definição — deixá-lo ali seria memória
        // que nada lê e que faz duas pilhas logicamente iguais compararem
        // diferentes.
        self.details[0] = Details::default();
        self.sel = 0;
        true
    }
}

/// Reordena um nível inteiro por `p` (novo → velho): posições, canais, faces e
/// o detalhe dele.
///
/// ⚠️ **Move dados, nunca os computa** — e é isso que torna reverter e desfazer
/// exatos ao bit. A malha é reconstruída (o `rebuild` refaz normais, adjacência
/// e octree) e as normais saem idênticas porque o anel de faces de um vértice é
/// ordenado por ÍNDICE DE FACE, que a permutação de vértices não toca: a soma
/// acontece na mesma ordem, sobre os mesmos números.
fn permute_level(mesh: &mut Mesh, details: &mut Details, p: &[u32]) {
    debug_assert_eq!(p.len(), mesh.vert_count(), "a permutação mede o nível");
    let mut inv = vec![0u32; p.len()];
    for (j, &o) in p.iter().enumerate() {
        inv[o as usize] = u32::try_from(j).unwrap_or(u32::MAX);
    }
    let positions: Vec<[f32; 3]> = p.iter().map(|&o| mesh.positions()[o as usize]).collect();
    let colors: Option<Vec<[f32; 3]>> = mesh
        .colors()
        .map(|c| p.iter().map(|&o| c[o as usize]).collect());
    let masks: Option<Vec<f32>> = mesh
        .masks()
        .map(|m| p.iter().map(|&o| m[o as usize]).collect());
    let faces: Vec<Face> = mesh
        .faces()
        .iter()
        .map(|f| {
            let mut g = *f;
            for k in 0..f.vert_count() {
                g.0[k] = inv[g.0[k] as usize];
            }
            g
        })
        .collect();
    let mut out = Mesh::from_parts(positions, faces).expect("permutar não inventa índice");
    if let Some(c) = colors {
        out.colors_mut().copy_from_slice(&c);
    }
    if let Some(m) = masks {
        out.put_masks(m);
    }
    *mesh = out;

    if !details.xyz.is_empty() {
        let old = std::mem::take(&mut details.xyz);
        details.xyz = p.iter().map(|&o| old[o as usize]).collect();
    }
    if let Some(old) = details.colors.take() {
        details.colors = Some(p.iter().map(|&o| old[o as usize]).collect());
    }
    if let Some(old) = details.masks.take() {
        details.masks = Some(p.iter().map(|&o| old[o as usize]).collect());
    }
}

#[cfg(test)]
#[path = "multires_reverse_tests.rs"]
mod tests;
