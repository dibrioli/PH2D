//! ⭐⭐⭐⭐ **A PEÇA COMPOSTA NA PLACA** (`docs/3D/30` §13 — etapa 4, W1b).
//!
//! As camadas da peça (os planos DOBRADOS `1024 × ⌈N/1024⌉` da
//! [`PilhaDaPeca`]) sobem à placa como fatias do compositor de GPU do Painter
//! (`ph2d_render::layer_compositor`), e ele compõe-nas com a MESMA lista de
//! operações que o Painter 2D lhe dá (`flatten_for_gpu`). Uma camada só volta a
//! subir quando a versão dos píxeis dela mudou, e só as linhas sujas
//! ([`crate::pilha_da_peca::NaPlaca`]): arrastar a opacidade não sobe píxel
//! nenhum.
//!
//! ⛔ **A CPU continua a REFERÊNCIA** (`PilhaDaPeca::compor`): a placa tem de dar
//! o mesmo composto, byte a byte (gate
//! `tinta_no_produto_tests::placa::a_placa_compoe_a_pilha_rica_como_a_cpu`).

use ph2d_gpu::GpuContext;
use ph2d_painter_layer_ops::flatten_for_gpu;
use ph2d_render::layer_compositor::{
    LayerCompositeError, LayerCompositor, LayerOp, LayerPixelProvider, LayerPixels, Region,
};
use ph2d_tool_painter::LayerId;

use crate::pilha_da_peca::{LARGURA_DA_DOBRA, PilhaDaPeca, dobra};

/// Por que a placa não compôs a pilha — o chamador compõe na CPU.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum NaoCompos {
    /// A pilha tem o que o compositor de GPU não exprime (um dos seis ajustes
    /// sem código de placa — `flatten_for_gpu` é a porta que decide).
    NaoRepresentavel,
    /// O compositor recusou (tecto de camadas, dobra maior que a placa…).
    Compositor(LayerCompositeError),
}

/// Os píxeis das camadas da pilha, pela versão de cada uma.
struct Fonte<'a>(&'a PilhaDaPeca);

impl LayerPixelProvider for Fonte<'_> {
    fn layer_pixels(&self, key: u64) -> Option<LayerPixels<'_>> {
        let p = self.0.plano(LayerId(key))?;
        Some(LayerPixels {
            version: p.na_placa.versao,
            rgba8: p.dobrado(),
            dirty: p.na_placa.linhas.map(|(a, b)| Region {
                x: 0,
                y: a,
                w: LARGURA_DA_DOBRA,
                h: b - a + 1,
            }),
        })
    }
}

/// As camadas (e máscaras) que uma lista de operações lê.
fn chaves(ops: &[LayerOp]) -> impl Iterator<Item = u64> + '_ {
    ops.iter()
        .flat_map(|op| match op {
            LayerOp::Layer { key, mask, .. } => [Some(*key), mask.map(|m| m.key)],
            LayerOp::Adjustment { mask, .. } => [mask.map(|m| m.key), None],
            _ => [None, None],
        })
        .flatten()
}

/// ⭐⭐⭐ **O compositor de GPU de UMA peça** — as fatias dele são as camadas
/// dela (a chave é o `LayerId`, que só é único dentro de uma pilha).
pub(crate) struct CompostoNaPlaca {
    compositor: LayerCompositor,
}

impl CompostoNaPlaca {
    pub(crate) fn novo(gpu: &GpuContext) -> Self {
        Self {
            compositor: LayerCompositor::new(gpu),
        }
    }

    /// ⭐⭐⭐⭐ **Compõe a pilha inteira na placa** — o composto fica na textura
    /// [`Self::composto`] (`Rgba8Unorm`, sRGB direito, a dobra inteira). Sobe
    /// só as camadas cuja versão mudou, e delas só as linhas sujas.
    pub(crate) fn compoe(
        &mut self,
        gpu: &GpuContext,
        pilha: &mut PilhaDaPeca,
    ) -> Result<(), NaoCompos> {
        let (ops, luts) = flatten_for_gpu(pilha.pilha()).ok_or(NaoCompos::NaoRepresentavel)?;
        let (l, h) = dobra(pilha.amostras());
        self.compositor
            .composite_with_luts(gpu, &ops, &luts, &Fonte(pilha), l, h, Region::full(l, h))
            .map_err(NaoCompos::Compositor)?;
        pilha.subiu_a_placa(chaves(&ops).map(LayerId));
        Ok(())
    }

    /// O composto da última [`Self::compoe`].
    pub(crate) fn composto(&self) -> Option<&wgpu::Texture> {
        self.compositor.output_texture()
    }

    /// O composto lido de volta (BLOQUEIA — gates e sondas).
    #[cfg(test)]
    pub(crate) fn le(&self, gpu: &GpuContext) -> Option<Vec<u8>> {
        self.compositor.read_output(gpu)
    }
}
