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

use std::collections::BTreeMap;

use ph2d_gpu::GpuContext;
use ph2d_mesh::Mesh;
use ph2d_mesh_render::{AchataDaTinta, MeshRenderer};

use crate::objects::ObjectId;
use ph2d_render::layer_compositor::{
    LayerCompositeError, LayerCompositor, LayerOp, LayerPixelProvider, LayerPixels, Region,
};
use ph2d_tool_painter::{LayerId, flatten_for_gpu};

use crate::pilha_da_peca::{PilhaDaPeca, dobra};

/// Por que a placa não compôs a pilha — o chamador compõe na CPU.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum NaoCompos {
    /// A pilha tem o que o compositor de GPU não exprime (um dos seis ajustes
    /// sem código de placa — `flatten_for_gpu` é a porta que decide).
    NaoRepresentavel,
    /// O compositor recusou (tecto de camadas, dobra maior que a placa…).
    Compositor(LayerCompositeError),
    /// O slot da peça não tem o plano armado com estas amostras.
    SemPlano,
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
                w: p.largura(),
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
/// dela (a chave é o `LayerId`, que só é único dentro de uma pilha) — e o
/// FUNDO dela na placa, amostra a amostra, em luz.
pub(crate) struct CompostoNaPlaca {
    compositor: LayerCompositor,
    /// O fundo na placa e a chave dele (o fundo por vértice e o degrau).
    fundo: Option<(wgpu::Buffer, Vec<[f32; 3]>, u8)>,
}

/// ⭐ **A composição na placa de uma cena**: o pipeline do achatamento e o
/// compositor de cada peça.
#[derive(Default)]
pub(crate) struct CompostosDaCena {
    achata: Option<AchataDaTinta>,
    por_peca: BTreeMap<ObjectId, CompostoNaPlaca>,
}

impl CompostosDaCena {
    /// ⭐⭐⭐⭐ **A pilha da peça composta na placa e achatada no plano do slot
    /// `k`** — o plano da placa fica com a cor da peça sem subir da CPU.
    #[allow(clippy::too_many_arguments)] // gpu+renderer+slot+peça+pilha+malha+degrau
    pub(crate) fn compoe_e_achata(
        &mut self,
        gpu: &GpuContext,
        renderer: &MeshRenderer,
        k: usize,
        peca: ObjectId,
        pilha: &mut PilhaDaPeca,
        mesh: &Mesh,
        nivel: u8,
    ) -> Result<(), NaoCompos> {
        let achata = self
            .achata
            .get_or_insert_with(|| AchataDaTinta::new(&gpu.device));
        let n = pilha.amostras();
        let dobra = dobra(n);
        let c = self
            .por_peca
            .entry(peca)
            .or_insert_with(|| CompostoNaPlaca::novo(gpu));
        c.compoe(gpu, pilha)?;
        c.garante_fundo(gpu, pilha, mesh, nivel);
        let fundo = &c.fundo.as_ref().ok_or(NaoCompos::SemPlano)?.0;
        let vista = c
            .composto()
            .ok_or(NaoCompos::SemPlano)?
            .create_view(&wgpu::TextureViewDescriptor::default());
        renderer
            .achata_tinta_at(
                &gpu.device,
                &gpu.queue,
                k,
                achata,
                &vista,
                dobra.0,
                fundo,
                n,
            )
            .then_some(())
            .ok_or(NaoCompos::SemPlano)
    }

    /// Esquece as peças que já não estão na cena.
    pub(crate) fn so_estas(&mut self, vivas: impl Fn(ObjectId) -> bool) {
        self.por_peca.retain(|id, _| vivas(*id));
    }
}

impl CompostoNaPlaca {
    /// ⚠️ Uma dobra de outro tamanho (outro degrau) não pede outro compositor:
    /// ele reconstrói as fatias e esquece o cache (`ensure_array`).
    pub(crate) fn novo(gpu: &GpuContext) -> Self {
        Self {
            compositor: LayerCompositor::new(gpu),
            fundo: None,
        }
    }

    /// ⭐ **O fundo da pilha na placa** — sobe UMA vez por pilha e degrau: o
    /// fundo é fixado quando a pilha nasce (`pilha_da_peca_fundo`).
    fn garante_fundo(&mut self, gpu: &GpuContext, pilha: &PilhaDaPeca, mesh: &Mesh, nivel: u8) {
        let serve = self
            .fundo
            .as_ref()
            .is_some_and(|(_, v, k)| *k == nivel && v.as_slice() == pilha.fundo());
        if !serve {
            let luz: Vec<[f32; 3]> = pilha
                .fundo_semeado(mesh, nivel)
                .iter()
                .map(|c| c.map(ph2d_color::srgb::srgb_to_linear_unit))
                .collect();
            let buf = AchataDaTinta::fundo(&gpu.device, &luz);
            self.fundo = Some((buf, pilha.fundo().to_vec(), nivel));
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

#[cfg(test)]
#[path = "composto_na_placa_tests.rs"]
mod tests;
