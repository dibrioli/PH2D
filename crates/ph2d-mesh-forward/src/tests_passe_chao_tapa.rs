//! ⭐ **O chão que tapa num passe de ecrã cheio** ([`crate::chao_tapa`]) — o arnês [`Passe`]: a lei
//! WGSL sobre um céu do chão do lado do desenhista, sem o resto do quadro. Dois usos:
//!
//! - o INSTRUMENTO do custo — o que o `forward.wgsl` paga por pixel de peça com chão: a difusa
//!   ([`crate::chao_tapa::RAIOS_CHAO`] leituras) e o reflexo (dois anéis de
//!   [`crate::chao_tapa::TAPS_ANEL`]), `K` vezes por envio a `1920 × 1080`, MENOS o passe sem a lei;
//! - o GATE da memória por pixel — a mesma pergunta responde uma vez, e uma pergunta DIFERENTE não
//!   herda a resposta da anterior.
//!
//! Corra (placa de exclusão):
//! `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test --release -p ph2d-mesh-forward --lib
//!  instrumento_custo_chao_tapa -- --ignored --nocapture` (o gate: `a_memoria_do_pixel_nao_troca_respostas`)

use std::time::Instant;

const K: u32 = 50;
const ENVIOS: usize = 9;
const FMT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// O cabeçalho dos dois fragmentos: o uniforme (o `quadro` que a lei lê e o modo), a textura do céu
/// do chão e o triângulo de ecrã cheio. `u` = o pixel em `[0, 1]²`, `p` um ponto baixo que anda pelo
/// chão, `n` uma normal que roda para baixo.
const CABECA: &str = r"
struct Q { chao: vec4<f32>, sombra: vec4<f32>, ceu_vp: mat4x4<f32>, modo: vec4<f32> };
@group(0) @binding(0) var<uniform> quadro: Q;
@group(0) @binding(1) var ceu_chao: texture_2d<f32>;
@group(0) @binding(2) var liso: sampler;

@vertex
fn vs(@builtin(vertex_index) k: u32) -> @builtin(position) vec4<f32> {
    let p = vec2<f32>(f32((k << 1u) & 2u) * 2.0 - 1.0, f32(k & 2u) * 2.0 - 1.0);
    return vec4<f32>(p, 0.5, 1.0);
}

fn ponto(u: vec2<f32>) -> vec3<f32> {
    return vec3<f32>(u.x * 2.0 - 1.0, 0.05 + 0.3 * u.y, u.y * 2.0 - 1.0);
}

fn normal(u: vec2<f32>) -> vec3<f32> {
    return normalize(vec3<f32>(0.6 * u.x - 0.3, -0.6, 0.4 - 0.5 * u.y));
}
";

const CUSTO: &str = r"
@fragment
fn fs(@builtin(position) q: vec4<f32>) -> @location(0) vec4<f32> {
    let u = q.xy / quadro.modo.zw;
    let p = ponto(u);
    let n = normal(u);
    var c = 0.0;
    if (quadro.modo.x > 0.5) {
        c = c + chao_tapa(p, n);
    }
    if (quadro.modo.y > 0.5) {
        c = c + chao_reflexo(p, reflect(vec3<f32>(0.0, 0.0, -1.0), n), 0.25);
    }
    return vec4<f32>(c, p.xz * 1.0e-6, 1.0);
}
";

/// `r`: o 1.º reflexo memorizado = o directo · `g`: o 2.º, com OUTRA reflectida e outra rugosidade (o
/// verniz por cima do metal áspero), = o directo · `b`: a difusa, duas normais seguidas · `a`: o
/// CONTROLO — as duas perguntas do reflexo têm respostas diferentes (senão o gate é cego).
const MEMORIA: &str = r"
fn igual(a: f32, b: f32) -> f32 {
    return select(0.0, 1.0, abs(a - b) < 1.0e-6);
}

@fragment
fn fs(@builtin(position) q: vec4<f32>) -> @location(0) vec4<f32> {
    let u = q.xy / quadro.modo.zw;
    let p = ponto(u);
    let n = normal(u);
    let r1 = reflect(vec3<f32>(0.0, 0.0, -1.0), n);
    let r2 = reflect(normalize(vec3<f32>(0.3, -0.2, -1.0)), n);
    let a = igual(chao_reflexo_no_pixel(p, r1, 0.25), chao_reflexo(p, r1, 0.25));
    let b = igual(chao_reflexo_no_pixel(p, r2, 0.0), chao_reflexo(p, r2, 0.0));
    let m = vec3<f32>(n.x, -n.y, n.z);
    let d = igual(chao_tapa_no_pixel(p, n), chao_tapa(p, n)) * igual(chao_tapa_no_pixel(p, m), chao_tapa(p, m));
    let difere = select(0.0, 1.0, abs(chao_reflexo(p, r1, 0.25) - chao_reflexo(p, r2, 0.0)) > 1.0e-3);
    return vec4<f32>(a, b, d, difere);
}
";

/// O arnês: o passe de ecrã cheio, o céu do chão com manchas (as leituras não são uniformes) e o
/// alvo que se lê de volta.
struct Passe {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    grupo: wgpu::BindGroup,
    ub: wgpu::Buffer,
    alvo: wgpu::Texture,
    tamanho: (u32, u32),
}

impl Passe {
    fn novo(fragmento: &str, tamanho: (u32, u32)) -> Option<Self> {
        let (device, queue, adapter) = crate::gpu_alvo::aparelho_em(wgpu::Backends::all())?;
        eprintln!("placa: {:?}", adapter.get_info().name);
        let fonte = format!("{CABECA}\n{fragmento}\n{}", crate::chao_tapa::wgsl());
        let modulo = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("chão tapa"),
            source: wgpu::ShaderSource::Wgsl(fonte.into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("chão tapa"),
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
                    format: FMT,
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
        let textura = |(w, h): (u32, u32), uso: wgpu::TextureUsages| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some("chão tapa"),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: FMT,
                usage: uso,
                view_formats: &[],
            })
        };
        let alvo = textura(
            tamanho,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        );
        let lado = crate::gpu_ceu_chao::CEU_CHAO_LADO;
        let ceu = textura(
            (lado, lado),
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
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
            label: Some("chão tapa"),
            size: 112,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let grupo = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("chão tapa"),
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
        Some(Self {
            device,
            queue,
            pipeline,
            grupo,
            ub,
            alvo,
            tamanho,
        })
    }

    /// O chão em `y = 0`, o céu do chão em `x, z ∈ [−2, 2]` visto de cima, e o modo `(x, y)`.
    fn uniforme(&self, modo: [f32; 2]) {
        let mut u = vec![0.0f32, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0];
        let vp = [
            [0.5, 0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, -0.5, 0.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        u.extend(vp.iter().flatten());
        u.extend([
            modo[0],
            modo[1],
            self.tamanho.0 as f32,
            self.tamanho.1 as f32,
        ]);
        self.queue
            .write_buffer(&self.ub, 0, bytemuck::cast_slice(&u));
    }

    /// `k` passes num envio; devolve os ms por passe.
    fn desenha(&self, k: u32) -> f64 {
        let vista = self
            .alvo
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("chão tapa"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &vista,
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
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.grupo, &[]);
            for _ in 0..k {
                pass.draw(0..3, 0..1);
            }
        }
        let t0 = Instant::now();
        self.queue.submit([enc.finish()]);
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("poll");
        t0.elapsed().as_secs_f64() * 1000.0 / f64::from(k)
    }

    /// O alvo, lido de volta (RGBA8, linha a linha).
    fn le(&self) -> Vec<u8> {
        let (w, h) = self.tamanho;
        let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("chão tapa leitura"),
            size: u64::from(w * h * 4),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        enc.copy_texture_to_buffer(
            self.alvo.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buf,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(w * 4),
                    rows_per_image: Some(h),
                },
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([enc.finish()]);
        buf.slice(..)
            .map_async(wgpu::MapMode::Read, |r| r.expect("map"));
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("poll");
        buf.slice(..).get_mapped_range().to_vec()
    }
}

#[test]
#[ignore = "instrumento: precisa de aparelho"]
fn instrumento_custo_chao_tapa() {
    let Some(passe) = Passe::novo(CUSTO, (1920, 1080)) else {
        eprintln!("sem aparelho");
        return;
    };
    let mede = |difusa: bool, reflexo: bool| {
        passe.uniforme([f32::from(u8::from(difusa)), f32::from(u8::from(reflexo))]);
        for _ in 0..3 {
            passe.desenha(K);
        }
        let mut v: Vec<f64> = (0..ENVIOS).map(|_| passe.desenha(K)).collect();
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

/// ⭐⭐ **A memória do pixel não troca respostas** — o material pergunta o reflexo do chão duas vezes
/// por pixel (o lobo dielétrico e o metálico, com a MESMA reflectida e rugosidade: responde-se uma
/// vez), e o verniz pergunta outra coisa a seguir: essa tem de ser calculada, não herdada. Contra o
/// Cycles a memória partida passava (o verniz pesa pouco ao lado do metal: `0,0147 → 0,0154`) — daí
/// este gate, pergunta a pergunta. Controlo: as duas perguntas do reflexo dão respostas diferentes.
#[test]
#[ignore = "precisa de aparelho"]
fn a_memoria_do_pixel_nao_troca_respostas() {
    let Some(passe) = Passe::novo(MEMORIA, (256, 256)) else {
        eprintln!("sem aparelho — o gate não corre aqui");
        return;
    };
    passe.uniforme([0.0, 0.0]);
    passe.desenha(1);
    let px = passe.le();
    let conta = |c: usize| px.as_chunks::<4>().0.iter().filter(|p| p[c] == 255).count();
    let total = px.len() / 4;
    let (r1, r2, n, difere) = (conta(0), conta(1), conta(2), conta(3));
    eprintln!(
        "{total} px · 1.º reflexo {r1} · 2.º (outra pergunta) {r2} · difusa {n} · as perguntas diferem em {difere}"
    );
    assert!(
        difere > total / 2,
        "CONTROLO: as duas perguntas têm de dar respostas diferentes"
    );
    assert_eq!(r1, total, "a 1.ª resposta memorizada não é a directa");
    assert_eq!(r2, total, "a 2.ª pergunta herdou a resposta da 1.ª");
    assert_eq!(n, total, "a difusa herdou a resposta de outra normal");
}
