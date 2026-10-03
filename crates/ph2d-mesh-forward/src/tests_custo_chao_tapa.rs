//! ⭐ **O instrumento do CUSTO do chão que tapa** ([`crate::chao_tapa`]) — o que o `forward.wgsl` paga
//! por pixel de peça com chão: a difusa ([`crate::chao_tapa::RAIOS_CHAO`] leituras do céu do chão) e
//! o reflexo (dois anéis de [`crate::chao_tapa::TAPS_ANEL`]). Um passe de ecrã cheio a `1920 × 1080`,
//! `K` vezes por envio, sobre um céu do chão do lado do desenhista; o custo é o tempo MENOS o do mesmo
//! passe sem a lei.
//!
//! Corra (placa de exclusão):
//! `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test --release -p ph2d-mesh-forward --lib
//!  instrumento_custo_chao_tapa -- --ignored --nocapture`

use std::time::Instant;

const W: u32 = 1920;
const H: u32 = 1080;
const K: u32 = 50;
const ENVIOS: usize = 9;

fn shader() -> String {
    format!(
        "{}\n{}",
        r"
struct Q { chao: vec4<f32>, sombra: vec4<f32>, ceu_vp: mat4x4<f32>, modo: vec4<f32> };
@group(0) @binding(0) var<uniform> quadro: Q;
@group(0) @binding(1) var ceu_chao: texture_2d<f32>;
@group(0) @binding(2) var liso: sampler;

@vertex
fn vs(@builtin(vertex_index) k: u32) -> @builtin(position) vec4<f32> {
    let p = vec2<f32>(f32((k << 1u) & 2u) * 2.0 - 1.0, f32(k & 2u) * 2.0 - 1.0);
    return vec4<f32>(p, 0.5, 1.0);
}

@fragment
fn fs(@builtin(position) q: vec4<f32>) -> @location(0) vec4<f32> {
    // Um ponto baixo que anda pelo chão da cena e uma normal que roda para baixo.
    let u = q.xy / vec2<f32>(1920.0, 1080.0);
    let p = vec3<f32>(u.x * 2.0 - 1.0, 0.05 + 0.3 * u.y, u.y * 2.0 - 1.0);
    let n = normalize(vec3<f32>(0.6 * u.x - 0.3, -0.6, 0.4 - 0.5 * u.y));
    var c = 0.0;
    if (quadro.modo.x > 0.5) {
        c = c + chao_tapa(p, n);
    }
    if (quadro.modo.y > 0.5) {
        c = c + chao_reflexo(p, reflect(vec3<f32>(0.0, 0.0, -1.0), n), 0.25);
    }
    return vec4<f32>(c, p.xz * 1.0e-6, 1.0);
}
",
        crate::chao_tapa::wgsl()
    )
}

#[test]
#[ignore = "instrumento: precisa de aparelho"]
fn instrumento_custo_chao_tapa() {
    let Some((device, queue, adapter)) = crate::gpu_alvo::aparelho_em(wgpu::Backends::all()) else {
        eprintln!("sem aparelho");
        return;
    };
    eprintln!("placa: {:?}", adapter.get_info().name);
    let modulo = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("custo chão tapa"),
        source: wgpu::ShaderSource::Wgsl(shader().into()),
    });
    let fmt = wgpu::TextureFormat::Rgba8Unorm;
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("custo chão tapa"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &modulo,
            entry_point: Some("vs"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &modulo,
            entry_point: Some("fs"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: fmt,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });
    let textura = |w: u32, h: u32, uso: wgpu::TextureUsages| {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("custo chão tapa"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: fmt,
            usage: uso,
            view_formats: &[],
        })
    };
    let alvo = textura(W, H, wgpu::TextureUsages::RENDER_ATTACHMENT)
        .create_view(&wgpu::TextureViewDescriptor::default());
    let lado = crate::gpu_ceu_chao::CEU_CHAO_LADO;
    let ceu = textura(
        lado,
        lado,
        wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    );
    // Um céu do chão com manchas (a visibilidade e a validade), para as leituras não serem uniformes.
    let dados: Vec<u8> = (0..lado * lado)
        .flat_map(|i| [((i * 37) % 251) as u8, 255, 0, 255])
        .collect();
    queue.write_texture(
        ceu.as_image_copy(),
        &dados,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(lado * 4),
            rows_per_image: Some(lado),
        },
        wgpu::Extent3d {
            width: lado,
            height: lado,
            depth_or_array_layers: 1,
        },
    );
    let ceu = ceu.create_view(&wgpu::TextureViewDescriptor::default());
    let s = device.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let ub = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("custo chão tapa"),
        size: 112,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let grupo = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("custo chão tapa"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: ub.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&ceu),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&s),
            },
        ],
    });
    let mede = |difusa: bool, reflexo: bool| {
        // O chão em `y = 0`; o quadro do céu do chão: `x, z ∈ [−2, 2]` de cima.
        let mut u = vec![0.0f32, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0];
        let vp = [
            [0.5, 0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, -0.5, 0.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        u.extend(vp.iter().flatten());
        u.extend([
            f32::from(u8::from(difusa)),
            f32::from(u8::from(reflexo)),
            0.0,
            0.0,
        ]);
        queue.write_buffer(&ub, 0, bytemuck::cast_slice(&u));
        let um = || {
            let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            {
                let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("custo"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &alvo,
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
                });
                pass.set_pipeline(&pipeline);
                pass.set_bind_group(0, &grupo, &[]);
                for _ in 0..K {
                    pass.draw(0..3, 0..1);
                }
            }
            let t0 = Instant::now();
            queue.submit([enc.finish()]);
            device
                .poll(wgpu::PollType::wait_indefinitely())
                .expect("poll");
            t0.elapsed().as_secs_f64() * 1000.0 / f64::from(K)
        };
        for _ in 0..3 {
            um();
        }
        let mut v: Vec<f64> = (0..ENVIOS).map(|_| um()).collect();
        v.sort_by(f64::total_cmp);
        v[ENVIOS / 2]
    };
    let base = mede(false, false);
    eprintln!(
        "1920×1080 · base {base:.3} ms · a difusa +{:.3} · o reflexo +{:.3} · as duas +{:.3}",
        mede(true, false) - base,
        mede(false, true) - base,
        mede(true, true) - base
    );
}
