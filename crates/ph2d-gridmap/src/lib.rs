//! ⭐⭐⭐ **O MAPA DE GRADE INTEIRA** — uma parametrização para a peça inteira, em vez
//! de um achatamento por patch.
//!
//! # Por que esta fase existe, e a razão é MEDIDA
//!
//! O F5 resolve **cada patch em separado** contra um domínio plano, e por isso a
//! marcação de cada arco — onde caem os pontos de subdivisão — é escolhida
//! **localmente**. ⛔ Mas sobre um arco pesam **duas** exigências ao mesmo tempo:
//!
//! 1. servir o **lado oposto do próprio patch** (o ponto `k` do lado 0 contra o ponto
//!    `k` do lado 2, senão a linha de grade que os une nasce torta);
//! 2. servir **o patch do outro lado da costura**, que tem os seus próprios opostos.
//!
//! Cada arco está preso nas duas, e a cadeia de dependências **atravessa a peça**.
//!
//! ⭐⭐ **Seis curas locais foram construídas e medidas em 2026-08-23**, e a lista é a
//! justificação desta crate — não uma citação da referência:
//!
//! | cura | onde parou |
//! |---|---|
//! | achatamento de valor médio (Tutte) | é o que shipa; mascara a discordância |
//! | cotangente | `18° → 18°`, `0/16` recuos |
//! | quadrilátero extremal | conforme, e **recusa-se** em patch grande |
//! | LSCM (conforme a `1,01`) | ⛔ **piora**: `18° → 28°`, dobras `0 → 68` |
//! | poda de patches | cura a topologia, **colapsa** a geometria |
//! | ponto fixo sobre o layout | contrai a `½`/ronda e **não move o número** |
//!
//! ⇒ *o constrangimento não é como um patch é preenchido; é que a marcação é escolhida
//! num sítio onde a informação necessária não está.*
//!
//! # ⭐ O que muda de espécie
//!
//! Num mapa de grade inteira as marcações **não são escolhidas**: elas são onde as
//! isolinhas inteiras de `(u, v)` cruzam cada arco. Os dois patches que partilham um
//! arco leem a **mesma** função, logo concordam **por construção** — não por acordo
//! negociado depois.
//!
//! ⚠️ E os inteiros já existem: o **F4** ([`ph2d_trace`] → `ph2d_quantize`) decide
//! quantos segmentos leva cada arco. ⇒ *o que falta desta fase é LINEAR.*
//!
//! # A ordem das peças
//!
//! | passo | o que faz | onde |
//! |---|---|---|
//! | **G1** | a **malha cortada**: cada patch fica um disco próprio, com a tabela de costuras | [`cut`] |
//! | G2 | pentear o campo dentro de cada patch (`ph2d_crossfield::comb`) | — |
//! | G3 | resolver `(u, v)` alinhado ao campo, com as costuras acopladas | — |
//! | G4 | ler as marcações onde as isolinhas inteiras cruzam cada arco | — |
//!
//! ⚠️ **Cada passo entra com o seu controlo.** *Uma fase grande construída de uma vez
//! é uma fase grande sem nenhum ponto onde a medição possa entrar* — foi assim que
//! quatro «byte-idêntico ao controlo» custaram meia jornada nesta linha.

pub mod comb;
pub mod cut;

pub use comb::{CombReport, Combed, comb_patches, jumps_only};
pub use cut::{CutMesh, CutReport, Seam, cut_along_patches};
