//! ⭐⭐⭐ **O CAMPO IMPLÍCITO NO DISPOSITIVO.**
//!
//! # A medição que escolheu esta rota (`docs/Render3d/05` §32–§33)
//!
//! A mesma peça, a mesma lei de marcha, o mesmo orçamento de passos:
//!
//! | `1920×1080` | CPU | **GPU** | ganho |
//! |---|---:|---:|---:|
//! | só o traçado | `29,18 ms` | `0,67 ms` | `43,6×` |
//! | traçado + oclusão inteira | `1 998 ms` | **`5,00 ms`** | **`399,5×`** |
//!
//! ⇒ o que na CPU custava **dois segundos em dezasseis etapas visíveis** cabe em `5 ms` de um
//! quadro de `16,7`.
//!
//! # ⭐ O catálogo inteiro por VINTE E OITO opcodes
//!
//! O gerador não conhece nenhuma das `62` primitivas: ele traduz a **fita** que o documento já
//! produz ([`ph2d_field_eval::wgsl`]), onde uma rosca e um filete já são `min`, `max` e `sqrt`.
//!
//! # ⚠️ Um pipeline por ESTRUTURA, e as constantes num buffer
//!
//! Arrastar um slider muda um número e não a árvore. Se a constante fosse escrita no shader, cada
//! quadro do arrasto recompilaria — medido, `6` a `49 ms`. ⇒ a chave do cache é o **texto**, que
//! não muda, e os números viajam num buffer que se reescreve de graça.

// ⚠️ **`BTreeMap` e não `HashMap`** — HR-5/ADR-0022. Aqui ele também é o certo por outra razão:
// a chave é o TEXTO do shader, e uma ordem estável faz um censo de pipelines compilados ser
// reprodutível entre corridas.
use std::collections::BTreeMap;

pub mod parity;

/// O molde do shader: a fita do documento, mais o que o chamador quiser à volta.
///
/// ⚠️ `{FIELD}` é substituído pelo corpo gerado e `{CONSTS}` pelo binding das constantes — e é por
/// isso que um consumidor (o traçado, a oclusão, a malha) escreve só a **sua** parte.
pub const FIELD_SLOT: &str = "{FIELD}";

/// Um cache de pipelines **por estrutura**, com as constantes de fora.
pub struct FieldPipelines {
    por_texto: BTreeMap<String, wgpu::ComputePipeline>,
}

impl Default for FieldPipelines {
    fn default() -> Self {
        Self::new()
    }
}

impl FieldPipelines {
    #[must_use]
    pub fn new() -> Self {
        Self {
            por_texto: BTreeMap::new(),
        }
    }

    /// Quantos pipelines estão compilados — o número que um gate de *«um arrasto não recompila»*
    /// observa.
    #[must_use]
    pub fn compiled(&self) -> usize {
        self.por_texto.len()
    }

    /// ⭐ **O pipeline desta estrutura**, compilando-o na primeira vez que ela aparece.
    ///
    /// `molde` é o WGSL do consumidor com [`FIELD_SLOT`] algures dentro.
    pub fn get(
        &mut self,
        device: &wgpu::Device,
        molde: &str,
        field: &ph2d_field_eval::wgsl::TapeWgsl,
    ) -> &wgpu::ComputePipeline {
        let src = molde.replace(FIELD_SLOT, &field.source);
        self.por_texto.entry(src.clone()).or_insert_with(|| {
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("campo"),
                source: wgpu::ShaderSource::Wgsl(src.as_str().into()),
            });
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("campo"),
                layout: None,
                module: &module,
                entry_point: Some("main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            })
        })
    }
}
