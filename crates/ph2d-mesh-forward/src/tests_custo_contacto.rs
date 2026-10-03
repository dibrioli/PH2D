//! ⭐ **O instrumento do CUSTO do contacto** — o que o `contacto(..)` do `forward.wgsl` paga por pixel
//! com `k` vizinhas: por vizinha, a afim, a `ct_borda`, `3` leituras trilineares de `3D` `Rgba16Float`
//! e a `ct_oclusao`. Um passe de ecrã cheio a `1920 × 1080`, `K` vezes por envio, com o aparelho do
//! desenhista; o custo é o tempo MENOS o do mesmo passe com `k = 0`.
//!
//! Corra (placa de exclusão):
//! `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test --release -p ph2d-mesh-forward --lib
//!  instrumento_custo_contacto -- --ignored --nocapture`

use std::time::Instant;

const W: u32 = 1920;
const H: u32 = 1080;
const K: u32 = 50;
const ENVIOS: usize = 9;
const ATLAS: u32 = 64;

fn shader() -> String {
    format!(
        "{}\n{}",
        ph2d_contacto::wgsl::fonte(),
        r"
struct P { k: u32, _a: u32, _b: u32, _c: u32 };
@group(0) @binding(0) var c0t: texture_3d<f32>;
@group(0) @binding(1) var c1t: texture_3d<f32>;
@group(0) @binding(2) var c2t: texture_3d<f32>;
@group(0) @binding(3) var s: sampler;
@group(0) @binding(4) var<uniform> par: P;

@vertex
fn vs(@builtin(vertex_index) k: u32) -> @builtin(position) vec4<f32> {
    let p = vec2<f32>(f32((k << 1u) & 2u) * 2.0 - 1.0, f32(k & 2u) * 2.0 - 1.0);
    return vec4<f32>(p, 0.5, 1.0);
}

@fragment
fn fs(@builtin(position) q: vec4<f32>) -> @location(0) vec4<f32> {
    // Um ponto que anda devagar pela grelha (como o de uma peça vizinha) e uma normal que roda.
    let p = vec3<f32>(q.xy / 1080.0, 0.37);
    let n = normalize(vec3<f32>(0.3 + 0.2 * p.x, 0.5, 0.8 - 0.3 * p.y));
    var vis = 1.0;
    for (var j = 0u; j < par.k; j = j + 1u) {
        let u = fract(p * (1.0 + 0.13 * f32(j)) + vec3<f32>(0.21 * f32(j)));
        let b = ct_borda(u * 1.2 - vec3<f32>(0.1));
        let t = vec2<f32>(f32(j % 4u), f32(j / 4u)) * 16.0;
        let tc = (vec3<f32>(t, 0.0) + b.xyz * 15.0 + vec3<f32>(0.5)) / vec3<f32>(64.0, 64.0, 16.0);
        let c0 = textureSampleLevel(c0t, s, tc, 0.0);
        let c1 = textureSampleLevel(c1t, s, tc, 0.0);
        let c2 = textureSampleLevel(c2t, s, tc, 0.0);
        vis = vis * (1.0 - b.w * ct_oclusao(c0, c1, c2, n));
    }
    return vec4<f32>(vis, p.xy * 1.0e-6, 1.0);
}
"
    )
}

#[test]
#[ignore = "instrumento: precisa de aparelho"]
fn instrumento_custo_contacto() {
    let Some((device, queue, adapter)) = crate::gpu_alvo::aparelho_em(wgpu::Backends::all()) else {
        eprintln!("sem aparelho");
        return;
    };
    eprintln!("placa: {:?}", adapter.get_info().name);
    let modulo = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("custo contacto"),
        source: wgpu::ShaderSource::Wgsl(shader().into()),
    });
    let mut entradas: Vec<wgpu::BindGroupLayoutEntry> = crate::gpu_contacto::entradas(0).to_vec();
    entradas.push(wgpu::BindGroupLayoutEntry {
        binding: 3,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    });
    entradas.push(wgpu::BindGroupLayoutEntry {
        binding: 4,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    });
    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("custo contacto"),
        entries: &entradas,
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("custo contacto"),
        bind_group_layouts: &[Some(&bgl)],
        immediate_size: 0,
    });
    let fmt = wgpu::TextureFormat::Rgba8Unorm;
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("custo contacto"),
        layout: Some(&layout),
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
    let alvo = device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("custo alvo"),
            size: wgpu::Extent3d {
                width: W,
                height: H,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: fmt,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
        .create_view(&wgpu::TextureViewDescriptor::default());
    let vistas: Vec<wgpu::TextureView> = (0..3)
        .map(|k| {
            let tex = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("custo atlas"),
                size: wgpu::Extent3d {
                    width: ATLAS,
                    height: ATLAS,
                    depth_or_array_layers: 16,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D3,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let dados: Vec<half::f16> = (0..ATLAS * ATLAS * 16 * 4)
                .map(|i| half::f16::from_f32(((i * 7 + k) % 13) as f32 / 26.0))
                .collect();
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                bytemuck::cast_slice(&dados),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(ATLAS * 8),
                    rows_per_image: Some(ATLAS),
                },
                wgpu::Extent3d {
                    width: ATLAS,
                    height: ATLAS,
                    depth_or_array_layers: 16,
                },
            );
            tex.create_view(&wgpu::TextureViewDescriptor::default())
        })
        .collect();
    let s = device.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let ub = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("custo par"),
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let grupo = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("custo contacto"),
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&vistas[0]),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&vistas[1]),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&vistas[2]),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(&s),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: ub.as_entire_binding(),
            },
        ],
    });
    let mede = |k: u32| {
        queue.write_buffer(&ub, 0, bytemuck::cast_slice(&[k, 0, 0, 0]));
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
    let base = mede(0);
    let mut linha = format!("1920×1080 · base {base:.3} ms");
    for k in [1u32, 2, 4, 8, 16] {
        linha += &format!(" · {k} vizinhas +{:.3}", mede(k) - base);
    }
    eprintln!("{linha}");
}
