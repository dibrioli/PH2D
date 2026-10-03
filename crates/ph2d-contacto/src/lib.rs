//! ⭐⭐⭐ **O CONTACTO ENTRE PEÇAS** — quanto do céu cada peça tapa às vizinhas.
//!
//! Uma peça rígida tapa o céu dos pontos à volta dela da mesma maneira seja qual for a pose: a
//! resposta vive no referencial DELA. Guarda-se uma vez, ao extrair, numa [`Grade`] de
//! [`LADO`]`³` pontos à volta da peça: em cada ponto, a visibilidade da peça em todas as direcções
//! projectada em harmónicos esféricos até à ordem `2` ([`COEFS`]). Ao desenhar, um ponto de outra
//! peça com a normal `n` lê a grelha (`3` leituras na placa) e convolve com o cosseno: a fracção do
//! céu, ponderada pelo cosseno, que aquela peça lhe tapa. É a *neighborhood transfer* do PRT (Sloan
//! et al. 2002) e os *ambient occlusion fields* (Kontkanen & Laine 2005).
//!
//! Medido contra o Cycles (`docs/3DModeling/ferramentas/oraculo_contacto_blender.py`): ver o gate
//! `tests::a_oclusao_entre_pecas_e_a_do_cycles`.
//!
//! A peça entra como um [`Volume`] de distância (o campo amostrado): os raios marcham nele, não no
//! campo — `~1000×` mais barato que avaliar o campo por passo.

mod grade;
mod propria;
mod volume;
pub mod wgsl;

pub use grade::{A_CONVOLUCAO, Grade, base_sh, direcoes};
pub use propria::{ALFA, CONES, visibilidade_propria};
pub use volume::Volume;

/// Os pontos por aresta da grelha.
pub const LADO: usize = 16;
/// A margem da grelha à volta da bola da peça, em raios dela: a grelha é o cubo de meia-aresta
/// `raio · (1 + MARGEM)`. Fora dele a oclusão cai como `1/D²` a partir da da borda.
pub const MARGEM: f32 = 1.0;
/// Os coeficientes por ponto (harmónicos reais até à ordem `2`).
pub const COEFS: usize = 9;
/// As direcções que cada ponto da grelha marcha.
pub const RAIOS: usize = 64;
/// Até onde (em meias-arestas da grelha, a partir do centro) a cauda `1/D²` ainda conta.
pub const ALCANCE: f32 = 8.0;

#[cfg(test)]
mod tests;
