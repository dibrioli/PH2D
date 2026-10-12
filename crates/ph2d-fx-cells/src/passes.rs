//! Os passes pequenos das células e os auxiliares de placa (pipelines, buffers, a relocação, os
//! quads da volta e do mapa). Saíram do `celulas.rs` pelo tecto de LOC: aquele é a ORQUESTRAÇÃO de
//! uma corrida (as caixas, o plano, a página); isto é o que cada passo despacha.

use ph2d_gpu::GpuContext;
use ph2d_render::GpuTexRun;

use crate::plano::Celula;

pub(crate) fn compute(gpu: &GpuContext, src: &str, rotulo: &str) -> wgpu::ComputePipeline {
    let module = gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(rotulo),
            source: wgpu::ShaderSource::Wgsl(src.into()),
        });
    gpu.device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(rotulo),
            layout: None,
            module: &module,
            entry_point: Some("cs"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        })
}

pub(crate) fn render(
    gpu: &GpuContext,
    module: &wgpu::ShaderModule,
    vs: &str,
    fs: &str,
    format: wgpu::TextureFormat,
    blend: Option<wgpu::BlendState>,
) -> wgpu::RenderPipeline {
    gpu.device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fx_cells"),
            layout: None,
            vertex: wgpu::VertexState {
                module,
                entry_point: Some(vs),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module,
                entry_point: Some(fs),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        })
}

pub(crate) fn buffer(
    gpu: &GpuContext,
    rotulo: &str,
    bytes: &[u8],
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    let b = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(rotulo),
        size: (bytes.len() as u64).max(16).next_multiple_of(4),
        usage: usage | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    if !bytes.is_empty() {
        gpu.queue.write_buffer(&b, 0, bytes);
    }
    b
}

pub(crate) fn vazio(
    gpu: &GpuContext,
    rotulo: &str,
    bytes: u64,
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(rotulo),
        size: bytes.max(16),
        usage,
        mapped_at_creation: false,
    })
}

pub(crate) fn liga<'a>(
    gpu: &GpuContext,
    pipeline_layout: &wgpu::BindGroupLayout,
    entradas: &[wgpu::BindingResource<'a>],
) -> wgpu::BindGroup {
    let entries: Vec<wgpu::BindGroupEntry<'_>> = entradas
        .iter()
        .enumerate()
        .map(|(i, r)| wgpu::BindGroupEntry {
            #[expect(clippy::cast_possible_truncation, reason = "poucas ligações")]
            binding: i as u32,
            resource: r.clone(),
        })
        .collect();
    gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("fx_cells"),
        layout: pipeline_layout,
        entries: &entries,
    })
}

pub(crate) fn limpa(enc: &mut wgpu::CommandEncoder, alvo: &wgpu::TextureView) {
    drop(enc.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("fx_cells limpa"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: alvo,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    }));
}

/// O registo de uma célula para o `COMPOE_WGSL` (`Cel`).
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct CelGpu {
    pub(crate) saida: [i32; 4],
    pub(crate) t: [i32; 2],
    pub(crate) org: [f32; 2],
}

impl From<&Celula> for CelGpu {
    fn from(c: &Celula) -> Self {
        Self {
            saida: c.saida,
            t: c.t,
            org: c.org,
        }
    }
}

/// As faixas de desenho das sprites de uma página: as da lista inteira, seguidas pelas instâncias
/// `idx` (já compactadas na ordem). Vazias continuam vazias («o átlas, em Mix»).
pub(crate) fn faixas_da_pagina(runs: &[GpuTexRun], idx: &[u32]) -> Vec<GpuTexRun> {
    if runs.is_empty() {
        return Vec::new();
    }
    let mut out: Vec<GpuTexRun> = Vec::new();
    for (j, &i) in idx.iter().enumerate() {
        let Some(r) = runs.iter().find(|r| r.start <= i && i < r.end) else {
            continue;
        };
        #[expect(clippy::cast_possible_truncation, reason = "uma página cabe num u32")]
        let j = j as u32;
        match out.last_mut() {
            Some(u)
                if u.end == j
                    && u.texture_id == r.texture_id
                    && u.blend == r.blend
                    && u.sampling == r.sampling =>
            {
                u.end = j + 1;
            }
            _ => out.push(GpuTexRun {
                start: j,
                end: j + 1,
                ..*r
            }),
        }
    }
    out
}

/// A instância `idx[j]` de `origem` no lugar `j`, transladada de `delta[j]` no mundo.
#[expect(clippy::too_many_arguments, reason = "um despacho")]
pub(crate) fn reloca(
    pipeline: &wgpu::ComputePipeline,
    gpu: &GpuContext,
    enc: &mut wgpu::CommandEncoder,
    origem: &wgpu::Buffer,
    idx: &[u32],
    delta: &[[f32; 2]],
    (palavras, pos): (u32, u32),
    uso: wgpu::BufferUsages,
) -> wgpu::Buffer {
    #[expect(clippy::cast_possible_truncation, reason = "uma página cabe num u32")]
    let n = idx.len() as u32;
    let destino = vazio(
        gpu,
        "fx_cells relocadas",
        u64::from(n) * u64::from(palavras) * 4,
        wgpu::BufferUsages::STORAGE | uso,
    );
    let p = buffer(
        gpu,
        "fx_cells relocar p",
        bytemuck::cast_slice(&[n, palavras, pos, 0]),
        wgpu::BufferUsages::UNIFORM,
    );
    let i = buffer(
        gpu,
        "fx_cells idx",
        bytemuck::cast_slice(idx),
        wgpu::BufferUsages::STORAGE,
    );
    let d = buffer(
        gpu,
        "fx_cells delta",
        bytemuck::cast_slice(delta),
        wgpu::BufferUsages::STORAGE,
    );
    let bg = liga(
        gpu,
        &pipeline.get_bind_group_layout(0),
        &[
            p.as_entire_binding(),
            origem.as_entire_binding(),
            i.as_entire_binding(),
            d.as_entire_binding(),
            destino.as_entire_binding(),
        ],
    );
    let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, &bg, &[]);
    pass.dispatch_workgroups(n.div_ceil(64), 1, 1);
    drop(pass);
    destino
}

/// Um quad por célula sobre `alvo` (`limpa` = o mapa, que começa a zero).
#[expect(clippy::too_many_arguments, reason = "um passe")]
pub(crate) fn quads(
    gpu: &GpuContext,
    pipeline: &wgpu::RenderPipeline,
    alvo: &wgpu::TextureView,
    cels: &wgpu::Buffer,
    n: u32,
    atlas: &wgpu::TextureView,
    tamanho: (u32, u32),
    limpa_antes: bool,
) {
    let bg = grupo_dos_quads(gpu, pipeline, cels, atlas, tamanho, limpa_antes);
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("fx_cells quads"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: alvo,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: if limpa_antes {
                        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
                    } else {
                        wgpu::LoadOp::Load
                    },
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bg, &[]);
        pass.draw(0..6, 0..n);
    }
    gpu.queue.submit([enc.finish()]);
}

/// O grupo `0` dos quads de `pipeline` (layout AUTOMÁTICO: o mapa não lê o átlas).
pub(crate) fn grupo_dos_quads(
    gpu: &GpuContext,
    pipeline: &wgpu::RenderPipeline,
    cels: &wgpu::Buffer,
    atlas: &wgpu::TextureView,
    tamanho: (u32, u32),
    limpa_antes: bool,
) -> wgpu::BindGroup {
    #[expect(clippy::cast_precision_loss, reason = "um alvo cabe num f32")]
    let u = buffer(
        gpu,
        "fx_cells alvo",
        bytemuck::cast_slice(&[tamanho.0 as f32, tamanho.1 as f32, 0.0, 0.0]),
        wgpu::BufferUsages::UNIFORM,
    );
    // O layout é o AUTOMÁTICO do par de estágios: o mapa não lê o átlas (ligações 0 e 2).
    let mut entries = vec![wgpu::BindGroupEntry {
        binding: 0,
        resource: cels.as_entire_binding(),
    }];
    if !limpa_antes {
        entries.push(wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::TextureView(atlas),
        });
    }
    entries.push(wgpu::BindGroupEntry {
        binding: 2,
        resource: u.as_entire_binding(),
    });
    gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("fx_cells quads"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &entries,
    })
}
