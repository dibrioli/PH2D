//! ⭐⭐⭐ **SOLDAR** (plano 39) — linhas cruzadas passam a partilhar o nó.
//!
//! Ideia do Enio (2026-08-31): *"e se pudéssemos soldar linhas cruzadas? Ou seja: linhas cruzadas
//! compartilham o mesmo nó de modo que criem várias áreas fechadas interligadas?"*
//!
//! # A lei
//!
//! > **Cada contorno parte-se em ARCOS nos pontos onde encontra os outros** — e as pontas dos arcos
//! > vizinhos caem exactamente no mesmo sítio, porque saem do mesmo cruzamento.
//!
//! O grafo não é uma estrutura de dados: ele é **implícito nas coordenadas coincidentes**. É o
//! modelo do desenho de CAD (um esboço do Fusion é uma rede desde sempre, e é por isso que o Trim
//! de lá parece natural) e a metade barata do *vector network* da Figma.
//!
//! # ⛔ O que isto NÃO é
//!
//! **Não é o `Pathfinder > Divide` do Illustrator.** Aquele *"perde as partes de caminhos abertos
//! que ficam de fora"* — é a queixa documentada dele. Aqui **todo arco sobrevive**, incluindo o toco
//! que sobra para fora: o que sai é a rede inteira, não só as faces.
//!
//! **Não é a rede da Figma.** Lá um ponto é um nó de um multigrafo e sobrevive à edição; aqui a
//! soldadura é um **acto**, e arrastar um nó depois separa outra vez as duas pontas. ⚠️ Manter a
//! rede soldada durante a edição é modelo novo — a Figma conta que refazer o tipo *caminho* teve
//! *"becos sem saída"* e que quase desistiram. **Decisão do Enio: soldar CONSOME os traços.**
//!
//! # ⛔ E não é automático
//!
//! Se cruzar duas linhas as colasse sozinho, seria impossível apenas **sobrepor** dois traços. O
//! gesto é um verbo explícito sobre a selecção.

use crate::VecVertex;

/// Duas fracções mais próximas que isto são o mesmo corte.
const EPS: f64 = 1e-9;

/// **OS ARCOS em que este contorno se parte** nos `cruzamentos` dados (fracções de arco `0..=1`,
/// em qualquer ordem).
///
/// Sem cruzamento nenhum devolve o contorno **intacto** — e intacto quer dizer os mesmos vértices,
/// não uma reconstrução: um caminho que não encontra ninguém não pode pagar o custo de ser cortado
/// e recosturado.
///
/// | o contorno | os cruzamentos | o que sai |
/// |---|---|---|
/// | qualquer | nenhum | ele próprio, intacto |
/// | aberto | `n` | `n + 1` arcos abertos |
/// | fechado | `n` | `n` arcos abertos (o último dá a volta pela emenda) |
///
/// ⚠️ **Um fechado com UM cruzamento vira UM arco aberto** — o anel é cortado num ponto e passa a
/// ter duas pontas, que caem no mesmo sítio. Não é degenerado: é um anel aberto.
#[must_use]
pub fn split_at(
    verts: &[VecVertex],
    closed: bool,
    cruzamentos: &[f64],
) -> Vec<(Vec<VecVertex>, bool)> {
    let mut cortes: Vec<f64> = cruzamentos
        .iter()
        .copied()
        .filter(|f| f.is_finite() && *f > EPS && *f < 1.0 - EPS)
        .collect();
    cortes.sort_by(f64::total_cmp);
    cortes.dedup_by(|a, b| (*a - *b).abs() < EPS);
    if cortes.is_empty() {
        return vec![(verts.to_vec(), closed)];
    }
    // As FRONTEIRAS dos arcos. Num aberto as duas pontas entram; num fechado a lista fecha-se
    // sobre si mesma e o último arco dá a volta pela emenda.
    let mut arcos = Vec::with_capacity(cortes.len() + 1);
    let mut emitir = |de: f64, ate: f64| {
        if let Some(v) = crate::trim_tool::piece_geometry(verts, closed, de, ate) {
            arcos.push((v, false));
        }
    };
    if closed {
        for w in 0..cortes.len() {
            emitir(cortes[w], cortes[(w + 1) % cortes.len()]);
        }
    } else {
        let mut anterior = 0.0;
        for &c in &cortes {
            emitir(anterior, c);
            anterior = c;
        }
        emitir(anterior, 1.0);
    }
    arcos
}

#[cfg(test)]
#[path = "weld_tests.rs"]
mod tests;
