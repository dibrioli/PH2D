//! **O relógio de fases do quadro do Motion** — a última passagem de cada fase do
//! [`super::quadro::o_quadro`], para a sonda do custo fixo da rota híbrida (doc 120 §8.3.1). Vive no
//! `MotionState` porque é o único estado que as três portas (o quadro, o `coze`, o `cook_gpu`)
//! partilham; o que é de DENTRO do cozimento na placa vem do `GpuCook`
//! ([`ph2d_gpu_cook::FasesDoCozimento`]), somado por cozimento. Só `Instant`: sempre ligado.

use std::time::Duration;

/// A última passagem de cada fase de um quadro. Uma fase que o quadro não percorreu fica a zero.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FasesDoQuadro {
    /// O `o_quadro` antes de cozer: sinks, tomadas, membranas da macro.
    pub(crate) tomadas: Duration,
    /// O `coze` antes da placa: externos, escopos, leques, preguiça, lente, filtro.
    pub(crate) preparo: Duration,
    /// O `cook_gpu` inteiro (também quando ele recusa e a bomba desenha).
    pub(crate) cook_gpu: Duration,
    /// O `cook_gpu` antes do plano: as cercas (formas, colisor, mistura, objecto).
    pub(crate) cercas: Duration,
    /// O `plano_do_produto` (com os `valores_dirigidos` que ele próprio pede).
    pub(crate) plano: Duration,
    /// A marcha na CPU: o prefixo até às fronteiras (híbrida) ou a bomba inteira (CPU).
    pub(crate) marcha: Duration,
    /// Os `valores_dirigidos` por tique, fora do plano.
    pub(crate) dirigidos: Duration,
    /// O `cook_many` inteiro, somado pelos cozimentos do quadro.
    pub(crate) cook_many: Duration,
    /// O que o `GpuCook` mediu dentro do `cook_many`, somado.
    pub(crate) cozimento: ph2d_gpu_cook::FasesDoCozimento,
    /// Depois do cozimento: origem, separação das imagens, LOD.
    pub(crate) depois: Duration,
    /// O `o_quadro` inteiro.
    pub(crate) total: Duration,
}
