//! ⭐ P14 — as pipelines `fs_main_n` (o sprite com normal map), irmão de `pipeline.rs` por tecto de
//! LOC. Corte mecânico: saíram inteiras, verbatim, de lá; `PipelinesComNormal` é re-exportada no
//! mesmo endereço (`crate::pipeline::PipelinesComNormal`).

use super::*;

impl SpritePipeline {
    /// ⭐ P14 — uma pipeline `fs_main_n` por mistura, com o material de 3 entradas `material_n_bgl`
    /// (`crate::sprite_normal`). Só nasce quando a 1.ª normal é registada.
    pub(crate) fn pipelines_com_normal(
        &self,
        gpu: &GpuContext,
        material_n_bgl: &wgpu::BindGroupLayout,
        color_format: wgpu::TextureFormat,
    ) -> PipelinesComNormal {
        let layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("ph2d-render sprite com normal layout"),
                bind_group_layouts: &[
                    Some(&self.frame_bgl),
                    Some(material_n_bgl),
                    Some(&self.skin_bgl),
                    Some(&self.luz_bgl),
                ],
                immediate_size: 0,
            });
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ph2d-render sprite shader (normal)"),
                source: wgpu::ShaderSource::Wgsl(SPRITE_WGSL.into()),
            });
        let buffers = [
            Some(QuadVertex::buffer_layout()),
            Some(RenderInstance::buffer_layout()),
        ];
        let variante = |label, blend, depth_stencil| {
            build_variant(
                &gpu.device,
                &layout,
                &shader,
                color_format,
                Variant {
                    label,
                    vs_entry: "vs_main",
                    fs_entry: "fs_main_n",
                    buffers: &buffers,
                    color_write: wgpu::ColorWrites::ALL,
                    blend: Some(blend),
                    depth_stencil,
                },
            )
        };
        let premul = wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING;
        PipelinesComNormal {
            mistura: std::array::from_fn(|tag| {
                variante(
                    "ph2d-render sprite com normal pipeline",
                    blend_state_for(tag as u8),
                    None,
                )
            }),
            // Os passes de recorte e de máscara (`clip_pass`): os MESMOS testes de stencil do
            // `test_pipeline`/`test_outside_pipeline`, com o `fs_main_n`.
            teste: variante(
                "ph2d-render clip stencil-test com normal pipeline",
                premul,
                Some(ds_de_teste(wgpu::CompareFunction::Equal)),
            ),
            teste_fora: variante(
                "ph2d-render mask stencil-test-outside com normal pipeline",
                premul,
                Some(ds_de_teste(wgpu::CompareFunction::NotEqual)),
            ),
        }
    }
}

/// ⭐ P14 — as pipelines `fs_main_n`: uma por mistura (passe normal) e as duas de teste de stencil.
pub(crate) struct PipelinesComNormal {
    pub(crate) mistura: [wgpu::RenderPipeline; BLEND_PIPELINE_COUNT],
    pub(crate) teste: wgpu::RenderPipeline,
    pub(crate) teste_fora: wgpu::RenderPipeline,
}

/// O teste de stencil dos passes de recorte/máscara (`compare` contra o `ref`; nunca escreve).
fn ds_de_teste(compare: wgpu::CompareFunction) -> wgpu::DepthStencilState {
    let face = wgpu::StencilFaceState {
        compare,
        fail_op: wgpu::StencilOperation::Keep,
        depth_fail_op: wgpu::StencilOperation::Keep,
        pass_op: wgpu::StencilOperation::Keep,
    };
    wgpu::DepthStencilState {
        format: STENCIL_FORMAT,
        depth_write_enabled: Some(false),
        depth_compare: Some(wgpu::CompareFunction::Always),
        stencil: wgpu::StencilState {
            front: face,
            back: face,
            read_mask: 0xff,
            write_mask: 0x00,
        },
        bias: wgpu::DepthBiasState::default(),
    }
}
