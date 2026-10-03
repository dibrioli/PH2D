//! ⭐ **O instrumento do CUSTO da triplanar** — medido ANTES de fixar lado e formato das texturas.
//!
//! Um passe de ecrã cheio a `1920 × 1080`, `K` vezes por envio (sobredesenho sem mistura nem
//! profundidade: cada vez é um quadro inteiro de fragmentos), com o MESMO aparelho do desenhista
//! (`Features::empty()` + limites do WebGL2). Cada fragmento faz `eixos × mapas` leituras
//! trilineares (`textureSampleLevel` com o nível tirado das derivadas projectadas por eixo — o que
//! a lei vai fazer) de um `texture_2d_array`, à densidade de `1` texel por pixel no nível `0`.
//! O custo é o tempo por quadro MENOS o do mesmo passe sem leituras.
//!
//! Corra (placa de exclusão):
//! `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test --release -p ph2d-mesh-forward --lib
//!  instrumento_custo_triplanar -- --ignored --nocapture`

use std::time::Instant;

const W: u32 = 1920;
const H: u32 = 1080;
const K: u32 = 100;
const ENVIOS: usize = 9;

const SHADER: &str = r#"
struct P { mapas: u32, eixos: u32, lado: f32, _p: f32 };
@group(0) @binding(0) var t: texture_2d_array<f32>;
@group(0) @binding(1) var s: sampler;
@group(0) @binding(2) var<uniform> par: P;

@vertex
fn vs(@builtin(vertex_index) k: u32) -> @builtin(position) vec4<f32> {
    let p = vec2<f32>(f32((k << 1u) & 2u) * 2.0 - 1.0, f32(k & 2u) * 2.0 - 1.0);
    return vec4<f32>(p, 0.5, 1.0);
}

fn plano(p: vec3<f32>, a: u32) -> vec2<f32> {
    if (a == 0u) { return p.yz; }
    if (a == 1u) { return p.xz; }
    return p.xy;
}

@fragment
fn fs(@builtin(position) q: vec4<f32>) -> @location(0) vec4<f32> {
    // Um plano inclinado: as tres projeccoes tem pegada de ~1 texel por pixel no nivel 0.
    let p = vec3<f32>(q.x, q.y, q.x * 0.37 + q.y * 0.61) / par.lado;
    let dx = dpdx(p);
    let dy = dpdy(p);
    let w = vec3<f32>(0.34, 0.33, 0.33);
    var c = vec4<f32>(p * 1.0e-6, 1.0);
    for (var a = 0u; a < par.eixos; a = a + 1u) {
        let gx = plano(dx, a) * par.lado;
        let gy = plano(dy, a) * par.lado;
        let lod = 0.5 * log2(max(max(dot(gx, gx), dot(gy, gy)), 1.0e-8));
        let uv = plano(p, a);
        for (var m = 0u; m < par.mapas; m = m + 1u) {
            c = c + textureSampleLevel(t, s, uv, i32(m), lod) * w[a];
        }
    }
    return c;
}
"#;

fn mede(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &wgpu::RenderPipeline,
    grupo: &wgpu::BindGroup,
    alvo: &wgpu::TextureView,
) -> f64 {
    let um = || {
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("custo"),
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
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, grupo, &[]);
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
}

#[test]
#[ignore = "instrumento: precisa de aparelho"]
fn instrumento_custo_triplanar() {
    let Some((device, queue, adapter)) = crate::gpu_alvo::aparelho_em(wgpu::Backends::all())
    else {
        eprintln!("sem aparelho");
        return;
    };
    eprintln!("placa: {:?}", adapter.get_info().name);
    let modulo = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("custo"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("custo"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("custo"),
        bind_group_layouts: &[Some(&bgl)],
        immediate_size: 0,
    });
    let alvo_fmt = wgpu::TextureFormat::Rgba8Unorm;
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("custo"),
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
                format: alvo_fmt,
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
            format: alvo_fmt,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
        .create_view(&wgpu::TextureViewDescriptor::default());
    let s = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("custo"),
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
    });
    let ub = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("custo par"),
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    // Ruído determinístico: o conteúdo não muda a cache, mas um padrão liso podia comprimir.
    let mut semente = 0x9E37_79B9_u32;
    let mut ruido = |n: usize| -> Vec<u8> {
        (0..n)
            .map(|_| {
                semente ^= semente << 13;
                semente ^= semente >> 17;
                semente ^= semente << 5;
                (semente >> 24) as u8
            })
            .collect()
    };
    eprintln!("formato · lado · eixos×mapas → ms/quadro (custo sobre a base)");
    for (fmt, bpp) in [
        (wgpu::TextureFormat::Rgba8UnormSrgb, 4u32),
        (wgpu::TextureFormat::Rgba16Float, 8),
    ] {
        for lado in [512u32, 1024, 2048] {
            let niveis = 32 - lado.leading_zeros();
            let tex = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("custo tex"),
                size: wgpu::Extent3d {
                    width: lado,
                    height: lado,
                    depth_or_array_layers: 3,
                },
                mip_level_count: niveis,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: fmt,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            for nivel in 0..niveis {
                let l = (lado >> nivel).max(1);
                let mut dados = ruido((l * l * bpp * 3) as usize);
                if bpp == 8 {
                    // meios-floats finitos em [0, 1): expoente baixo.
                    for par in dados.chunks_exact_mut(2) {
                        par[1] &= 0x3B;
                    }
                }
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &tex,
                        mip_level: nivel,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &dados,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(l * bpp),
                        rows_per_image: Some(l),
                    },
                    wgpu::Extent3d {
                        width: l,
                        height: l,
                        depth_or_array_layers: 3,
                    },
                );
            }
            let vista = tex.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                ..Default::default()
            });
            let grupo = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("custo"),
                layout: &bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&vista),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&s),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: ub.as_entire_binding(),
                    },
                ],
            });
            let corre = |eixos: u32, mapas: u32| {
                let mut b = [0u8; 16];
                b[0..4].copy_from_slice(&mapas.to_le_bytes());
                b[4..8].copy_from_slice(&eixos.to_le_bytes());
                b[8..12].copy_from_slice(&(lado as f32).to_le_bytes());
                queue.write_buffer(&ub, 0, &b);
                mede(&device, &queue, &pipeline, &grupo, &alvo)
            };
            let base = corre(0, 0);
            let mut linha = format!("{fmt:?} · {lado} · base {base:.3} ms");
            for (eixos, mapas) in [(1, 0), (3, 0), (1, 2), (1, 3), (3, 2), (3, 3)] {
                let t = corre(eixos, mapas);
                linha += &format!(" · {eixos}×{mapas} +{:.3}", t - base);
            }
            eprintln!("{linha}");
        }
    }
}
