//! **A QUADRANGULAÇÃO POR PATCH** — a fase **F5**, e a primeira que devolve malha.
//!
//! Clean-room a partir de Pietroni et al., *QuadWild* (SIGGRAPH 2021), **§7 e §8**.
//! ⚠️ Nenhuma linha traduzida de fonte GPL — ver **ADR-0162**, Trilha A.
//!
//! # O que ela recebe, e por que já está tudo decidido
//!
//! | de onde | o quê |
//! |---|---|
//! | **F3** ([`ph2d_trace`]) | os patches, os lados, os arcos e a cadeia de vértices de cada arco |
//! | **F4** ([`ph2d_quantize`]) | quantos quads cada arco leva, e o **leque** `e_j` de cada patch |
//!
//! ⭐ **Nada aqui escolhe uma contagem.** Os inteiros vieram do F4 com ótimo
//! demonstrado; esta fase só tem de os **realizar** em geometria. É por isso que
//! ela não pode falhar por combinatória: se falhar, é geometria.
//!
//! # O ladrilho de um patch: o LEQUE
//!
//! Um patch de valência `n` vira `n` quads em volta de **um** vértice central.
//! Sobre o lado `i` há um ponto de corte que o parte em `u_i = e_{i-1}` e
//! `v_i = e_{i+1}`; do centro sai um raio de comprimento `e_j` até o corte do
//! lado `j`. O quad entre os lados `i` e `i+1` é uma grade `e_{i+1} × e_i`.
//!
//! ⚠️ **É exatamente a mesma lei que o F4 resolveu** (`L_i = e_{i-1} + e_{i+1}`),
//! agora lida ao contrário: lá ela decidia os inteiros, aqui ela diz onde pôr os
//! pontos. *Duas leis diferentes para a mesma coisa seria a costura que não fecha.*
//!
//! # ⚠️ O que faz a malha não RASGAR nas divisas
//!
//! Os pontos de um arco são amostrados **uma vez**, na ordem canónica do arco, e
//! os dois patches que o partilham reusam os mesmos índices — um deles ao
//! contrário. Amostrar por patch produziria dois conjuntos de pontos quase iguais
//! sobre a mesma curva, e a malha sairia aberta ao longo de **toda** fronteira de
//! patch, com um erro pequeno demais para se ver e grande demais para se ignorar.

#![forbid(unsafe_code)]

/// ⭐⭐ **O INTERIOR DE UM PATCH SEGUE O CAMPO** — ver [`aligned`].
pub mod aligned;
/// **O PREENCHIMENTO de um patch** — o leque e a grade — ver [`fan`].
pub mod fan;
/// **O PATCH ACHATADO** — a grade nasce na superfície, não no espaço — ver [`param`].
mod param;
/// **UM PATCH VIRA QUADS** — o domínio, os bordos e a grade — ver [`patch`].
mod patch;
/// ⭐ **A RELAXAÇÃO QUE OLHA PARA O ÂNGULO** — o ajuste de quadrado — ver [`relax`].
mod relax;
/// **O QUE A MONTAGEM DIZ** — recusa, relatório e proveniência — ver [`report`].
pub mod report;
/// ⭐ **A FORMA DE CADA QUAD** — a régua por-face — ver [`shape`].
pub mod shape;
/// **A COSTURA** — amostragem partilhada e montagem — ver [`stitch`].
pub mod stitch;

pub use aligned::{INTERIOR, Interior};
pub use relax::SQUARE_ROUNDS;
pub use report::{
    FillError, FillReport, Provenance, detail_lost, folded_against, folded_by_neighbours,
    follows_relief,
};
pub use shape::{QuadShape, quad_shape, skew_by_fan, skew_by_provenance};
pub use stitch::{SMOOTHING_ROUNDS, fill, fill_with};
