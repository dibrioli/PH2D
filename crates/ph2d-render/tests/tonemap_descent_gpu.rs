//! **A descida final para 8 bits: a invariante que ela cumpre, e a folga que ela NÃO tem** — W6.2
//! do plano [`docs/Sprite_projeto/18`](../../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md).
//!
//! # Duas coisas vivem aqui, e são duas de propósito
//!
//! 1. **Um gate**: uma cor chapada de 8 bits atravessa o passe de tonemap **byte-exacta**. É a
//!    promessa central de um editor 2D — *o que eu pintei é o que está no ecrã* — e ela atravessa
//!    hardware sRGB, meio-float e hardware sRGB outra vez sem nunca ter sido medida até hoje.
//! 2. **Uma sonda**: quanto se pode empurrar esse valor antes de ele cair no byte vizinho. É o
//!    número que **recusou** o dither neste passe, e ele mede-se em qualquer máquina.
//!
//! # A recusa, e por que a sonda fica no repo
//!
//! Dither no tonemap é a ideia óbvia: é o último sítio com mais de 8 bits, e um degradê contínuo
//! quantiza ali em faixas visíveis. Foi construído e medido em 2026-08-21 (RTX, wgpu 28), sobre os
//! 256 bytes × as 64 células:
//!
//! | | |
//! |---|---|
//! | folga máxima que não move byte nenhum | **~0,0283 LSB** (de 0,5 possíveis) |
//! | com o pico que a CPU usa (0,4311 LSB) | **5,98%** dos pixels movidos |
//!
//! Um dither aqui teria de caber em **7%** da amplitude do caminho de software — e a 7% ele não
//! espalha nada. A alternativa seria aceitar que 6% de uma cor chapada vire mosquito.
//!
//! ⚠️ **O mecanismo é o que importa, porque é ele que impede reconstruir isto.** O valor que chega
//! ao tonemap é `hw_decode(byte)`, e a tabela sRGB do hardware **não é a curva ideal** — medido
//! nesta mesma wave ([`precision_parity_gpu`]), ela afasta-se até `0,00195` em linear. O que salva o
//! caminho sem dither é que o hardware se cancela consigo próprio: `hw_encode(hw_decode(N)) == N` é
//! requisito das especificações. Mas **só enquanto ninguém empurra o valor pelo meio**. Um shader
//! que re-codifique com a curva *ideal*, some o viés e volte a descodificar está a medir a distância
//! à fronteira com uma régua que não é a do hardware, e a folga que sobra é uma propriedade **da
//! placa** — não do formato, não da representação, não de nada que se possa escrever numa constante
//! portátil.
//!
//! ⛔ **Encolher a amplitude até o gate passar** trocaria um defeito visível (faixas) por um número
//! ajustado a uma placa só. É por isso que a sonda ficou: quem quiser reabrir isto começa por a
//! correr **na sua** máquina, não por escrever um shader.
//!
//! ✅ O dither que ship*ou* é o da descida que o autor **comanda** — o botão `RGBA8` do Inspector —,
//! onde a conversão é software de ponta a ponta e a amplitude sai de uma deriva medida e portátil
//! ([`ph2d_color::dither`]).
//!
//! # O aparelho
//!
//! A cadeia real, e não uma sua aproximação:
//!
//! ```text
//!   textura Rgba8UnormSrgb   →  descodificação em HARDWARE
//!         ▼ (passe 1, sampler NEAREST, + o viés da sonda)
//!   Rgba16Float               =  o que o `GameRt` de facto guarda
//!         ▼ (passe 2, o Tonemap DE VERDADE)
//!   Bgra8UnormSrgb            →  codificação + quantização em HARDWARE
//! ```
//!
//! ⚠️ **`Nearest` de propósito:** o produto amostra com filtro, mas o que se mede é a *quantização*,
//! não a interpolação. Com bilinear, meio texel de desvio apareceria como um byte trocado e o gate
//! acusaria a descida por um defeito de amostragem.
//!
//! ⚠️ **O viés vive no shader da SONDA, nunca no de produção.** Foi assim que a recusa ficou
//! mensurável sem o código recusado andar por lá à espera de alguém o ligar.
//!
//! ⚠️ **Os 256 bytes × as 64 células**, e não uma amostra: a fixture desloca o padrão um byte a cada
//! oito linhas, por isso cada valor visita todas as posições da matriz.
//!
//! ⚠️ Estes testes precisam de adapter. Sem GPU eles **saltam**, e *saltar não é verde*.

use ph2d_gpu::GpuContext;
use ph2d_render::tonemap::Tonemap;

/// Largura: um byte por coluna.
const W: u32 = 256;
/// Altura: oito grupos de oito linhas — o que faz cada byte visitar as 64 células.
const H: u32 = 64;

const TONEMAP_WGSL: &str = include_str!("../src/shaders/tonemap.wgsl");

/// **A validação do WGSL, sem GPU nenhuma.** Sem isto, uma recusa do naga só apareceria ao abrir a
/// janela — e o passe de tonemap está no caminho de **todo** o conteúdo do app.
#[test]
fn tonemap_wgsl_parses_and_validates() {
    let module = naga::front::wgsl::parse_str(TONEMAP_WGSL).unwrap_or_else(|e| {
        panic!(
            "tonemap.wgsl failed naga parse:\n{}",
            e.emit_to_string(TONEMAP_WGSL)
        )
    });
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator
        .validate(&module)
        .unwrap_or_else(|e| panic!("tonemap.wgsl failed naga validation: {e:?}"));
}

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| {
            let instance = GpuContext::default_instance();
            GpuContext::new(instance, None).ok()
        })
        .clone()
}

/// O byte que a fixture põe em `(x, y)`. O deslocamento por grupo de oito linhas é o que faz cada
/// valor visitar as 64 células da matriz de Bayer.
fn fixture_byte(x: u32, y: u32) -> u8 {
    ((x + y / 8) % 256) as u8
}

/// Passe 1 — amostra a textura sRGB de 8 bits para um alvo `Rgba16Float`, que é o que o `GameRt` é.
///
/// ⛔ **Não é aqui que o viés entra, e a primeira versão deste ficheiro errou-o.** Somar o viés
/// antes do meio-float faz a sonda medir outra cadeia: o `f16` é uma quantização, e um viés
/// infinitesimal que mude o lado para que ele arredonda vira um salto de 0,037 LSB. A sonda dizia
/// então «folga = 0,0000», que é a resposta certa à pergunta errada. *Um aparelho que mede uma
/// cadeia diferente da que se vai construir dá um número verdadeiro sobre nada.*
///
/// O viés pertence **depois** do `f16`, no passe de descida — que é onde o dither recusado teria
/// vivido, e é onde a CPU o põe também (`linear_to_srgb_byte_biased` recebe o valor já
/// desempacotado do meio-float).
const SAMPLE_WGSL: &str = r#"
@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs(@builtin(vertex_index) i: u32) -> VsOut {
    var out: VsOut;
    let x = f32((i << 1u) & 2u) * 2.0 - 1.0;
    let y = f32(i & 2u) * 2.0 - 1.0;
    out.pos = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
    return textureSample(src, samp, in.uv);
}
"#;

/// Passe 2 da SONDA — o tonemap recusado: a mesma passagem `clamp` do de produção, mais o viés de
/// Bayer em passos de 8 bits no domínio sRGB.
///
/// ⚠️ **Ele vive aqui e não no shader de produção**, e é isso que torna a recusa mensurável sem o
/// código recusado ficar na árvore à espera de alguém o ligar.
const PROBE_DESCENT_WGSL: &str = r#"
@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

const PROBE_PEAK_LSB: f32 = __BIAS__;
const BAYER_8X8 = array<u32, 64>(__BAYER__);

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs(@builtin(vertex_index) i: u32) -> VsOut {
    var out: VsOut;
    let x = f32((i << 1u) & 2u) * 2.0 - 1.0;
    let y = f32(i & 2u) * 2.0 - 1.0;
    out.pos = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

fn srgb_encode(c: vec3<f32>) -> vec3<f32> {
    let safe = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
    let lo = safe * 12.92;
    let hi = 1.055 * pow(safe, vec3<f32>(1.0 / 2.4)) - vec3<f32>(0.055);
    return mix(lo, hi, step(vec3<f32>(0.0031308), safe));
}

fn srgb_decode(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    return mix(lo, hi, step(vec3<f32>(0.04045), c));
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
    let hdr = textureSample(src, samp, in.uv);
    let px = vec2<u32>(floor(in.pos.xy));
    let cell = BAYER_8X8[(px.y % 8u) * 8u + (px.x % 8u)];
    let unit = (f32(cell) + 0.5) / 64.0;
    let bias = (unit - 0.5) * 2.0 * PROBE_PEAK_LSB / 255.0;
    let moved = srgb_decode(clamp(
        srgb_encode(clamp(hdr.rgb, vec3<f32>(0.0), vec3<f32>(1.0))) + vec3<f32>(bias),
        vec3<f32>(0.0),
        vec3<f32>(1.0),
    ));
    return vec4<f32>(moved, hdr.a);
}
"#;

/// O WGSL da descida da sonda com o pico de viés (em LSB) e a matriz derivada lá dentro.
///
/// ⚠️ **A matriz vem de [`ph2d_color::BAYER_8X8`]**, que a deriva da recorrência. Digitá-la aqui
/// faria a sonda medir um padrão que não é o do produto, e o número que ela produzisse seria sobre
/// outra coisa.
fn probe_descent_wgsl(peak_bias_lsb: f32) -> String {
    let bayer = ph2d_color::BAYER_8X8
        .iter()
        .map(|v| format!("{v}u"))
        .collect::<Vec<_>>()
        .join(", ");
    PROBE_DESCENT_WGSL
        .replace("__BIAS__", &format!("{peak_bias_lsb:?}"))
        .replace("__BAYER__", &bayer)
}

/// Constrói o `GameRt`: uma `Rgba16Float` com o que o hardware devolveu ao descodificar a fixture de
/// 8 bits.
fn game_rt_from_srgb_fixture(gpu: &GpuContext) -> wgpu::Texture {
    let pixels: Vec<u8> = (0..H)
        .flat_map(|y| {
            (0..W).flat_map(move |x| {
                let b = fixture_byte(x, y);
                [b, b, b, 255]
            })
        })
        .collect();

    let src = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("tonemap descent fixture"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &src,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(W * 4),
            rows_per_image: Some(H),
        },
        wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
    );
    let src_view = src.create_view(&wgpu::TextureViewDescriptor::default());
    // ⚠️ `Nearest`: mede-se a quantização, não a interpolação.
    let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("tonemap descent sampler"),
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let bgl = gpu
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("tonemap descent bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
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
            ],
        });
    let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("tonemap descent bind group"),
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&src_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });

    let shader = gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("tonemap descent sample shader"),
            source: wgpu::ShaderSource::Wgsl(SAMPLE_WGSL.into()),
        });
    let layout = gpu
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("tonemap descent sample layout"),
            bind_group_layouts: &[&bgl],
            immediate_size: 0,
        });
    let pipeline = gpu
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("tonemap descent sample pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                targets: &[Some(wgpu::TextureFormat::Rgba16Float.into())],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

    let game_rt = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("tonemap descent game rt"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let game_view = game_rt.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("tonemap descent probe pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &game_view,
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
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
    gpu.queue.submit([encoder.finish()]);
    game_rt
}

/// Lê a saída do tonemap. ⚠️ O formato é **B**GRA — a ordem dos canais está trocada, e a fixture é
/// cinzenta precisamente para que isso não mude a resposta.
fn read_output(gpu: &GpuContext, texture: &wgpu::Texture) -> Vec<u8> {
    let unpadded = (W * 4) as usize;
    let padded = unpadded.div_ceil(256) * 256;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("tonemap descent staging"),
        size: (padded * H as usize) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded as u32),
                rows_per_image: Some(H),
            },
        },
        wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([encoder.finish()]);

    let slice = staging.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    let data = slice.get_mapped_range();
    let mut out = Vec::with_capacity((W * H * 4) as usize);
    for row in 0..H as usize {
        let start = row * padded;
        out.extend_from_slice(&data[start..start + unpadded]);
    }
    drop(data);
    staging.unmap();
    out
}

/// A descida da **SONDA**: o tonemap recusado, com o viés de Bayer. Devolve a textura
/// `Bgra8UnormSrgb` — o mesmo formato e a mesma lei `clamp` do de produção, mais o viés.
fn descend_with_probe(
    gpu: &GpuContext,
    game_rt: &wgpu::Texture,
    peak_bias_lsb: f32,
) -> wgpu::Texture {
    let src_view = game_rt.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("tonemap descent probe sampler"),
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });
    let bgl = gpu
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("tonemap descent probe bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
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
            ],
        });
    let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("tonemap descent probe bind group"),
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&src_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    let shader = gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("tonemap descent probe shader"),
            source: wgpu::ShaderSource::Wgsl(probe_descent_wgsl(peak_bias_lsb).into()),
        });
    let layout = gpu
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("tonemap descent probe layout"),
            bind_group_layouts: &[&bgl],
            immediate_size: 0,
        });
    let pipeline = gpu
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("tonemap descent probe pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                // ⚠️ O MESMO formato do passe de produção: é o hardware que codifica e quantiza, e
                // trocar isto mediria outra coisa.
                targets: &[Some(Tonemap::OUTPUT_FORMAT.into())],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

    let out = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("tonemap descent probe out"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: Tonemap::OUTPUT_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let out_view = out.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("tonemap descent probe pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &out_view,
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
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
    gpu.queue.submit([encoder.finish()]);
    out
}

/// Quantos pixels saíram com um byte diferente do que entraram, e um exemplo dos primeiros.
fn count_moved(out: &[u8]) -> (usize, Vec<String>) {
    let mut count = 0usize;
    let mut examples = Vec::new();
    for y in 0..H {
        for x in 0..W {
            let expected = fixture_byte(x, y);
            let i = ((y * W + x) * 4) as usize;
            // Bgra8UnormSrgb: [B, G, R, A]. A fixture é cinzenta, logo os três dão o mesmo valor.
            let got = out[i + 2];
            if got != expected {
                count += 1;
                if examples.len() < 8 {
                    examples.push(format!(
                        "  byte {expected} -> {got} (celula de Bayer x={} y={})",
                        x % 8,
                        y % 8
                    ));
                }
            }
        }
    }
    (count, examples)
}

/// **A INVARIANTE: uma cor chapada de 8 bits atravessa o passe final byte-exacta.**
///
/// Ela é a promessa central de uma ferramenta de arte 2D — *o que eu pintei é o que está no ecrã* —
/// e atravessa três traduções que ninguém escolheu: a descodificação sRGB do hardware, o meio-float
/// do `GameRt`, e a codificação sRGB de volta.
///
/// ⚠️ **Ela é verdadeira por CANCELAMENTO, não por exactidão.** Nenhuma das três metades acerta a
/// curva ideal (a do hardware afasta-se até `0,00195` em linear, medido em `precision_parity_gpu`);
/// o que as salva é serem inversas **uma da outra**. É exactamente por isso que este gate existe: um
/// dia alguém vai querer intercalar alguma coisa no meio — um dither, uma curva, um ajuste — e a
/// primeira coisa a saber é que não há folga nenhuma escondida ali.
#[test]
#[ignore = "precisa de adapter"]
fn a_flat_eight_bit_colour_survives_the_descent() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adapter — SALTADO, e saltar nao e' verde");
        return;
    };
    let game_rt = game_rt_from_srgb_fixture(&gpu);
    let view = game_rt.create_view(&wgpu::TextureViewDescriptor::default());
    // ⚠️ O `Tonemap` DE VERDADE, e não uma cópia dele: o que se mede é o passe que ship*a*.
    let tonemap = Tonemap::new(&gpu, view, (W, H));
    tonemap.run(&gpu);
    let (moved, examples) = count_moved(&read_output(&gpu, tonemap.output_texture()));
    assert_eq!(
        moved,
        0,
        "a descida final deixou de ser byte-exacta para conteudo de 8 bits ({moved} pixels):\n{}\n\n\
         O que mudou nao foi o dither (nao ha' nenhum aqui): foi o passe, o formato do alvo, ou o \
         caminho de amostragem. Uma cor chapada TEM de sair como entrou.",
        examples.join("\n")
    );
}

/// **A SONDA que recusou o dither neste passe** — corre-a antes de reabrir a ideia.
///
/// Ela mede, nesta máquina, o maior pico de viés (em passos de 8 bits) que ainda deixa os 256 bytes
/// intactos nas 64 células. Esse número é a folga que o `round` do hardware deixa depois de se
/// cancelar consigo próprio, e é **propriedade da placa**.
///
/// ⚠️ **Não é um gate, e não tem barra**: pôr uma barra aqui seria fixar uma propriedade de hardware
/// numa constante, que é precisamente o erro que a recusa evita. O que ela afirma é só que o
/// aparelho MEDE — que um viés de meio passo move mesmo alguma coisa. Um dia em que essa afirmação
/// falhe é um dia em que a sonda parou de medir, não um dia em que o hardware ficou perfeito.
///
/// Medido em 2026-08-21 (RTX, wgpu 28): folga ~`0,0283` LSB, e com o pico da CPU (`0,4311`)
/// moveram-se 980 dos 16 384 pixels (5,98%).
#[test]
#[ignore = "precisa de adapter"]
fn the_headroom_for_a_dither_here_is_measured_not_assumed() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem adapter — SALTADO, e saltar nao e' verde");
        return;
    };
    let game_rt = game_rt_from_srgb_fixture(&gpu);
    let moved_at = |peak: f32| {
        let out = descend_with_probe(&gpu, &game_rt, peak);
        count_moved(&read_output(&gpu, &out))
    };
    let total = (W * H) as f32;

    // ⚠️ **Controle NEGATIVO primeiro:** com viés zero a sonda tem de dar exactamente o que o
    // `Tonemap` de produção dá. Se ela já mover bytes a zero, o aparelho está torto e todos os
    // números abaixo são sobre a sonda, não sobre a placa.
    let (at_zero, zero_examples) = moved_at(0.0);
    assert_eq!(
        at_zero,
        0,
        "a sonda move bytes com vies ZERO — ela nao reproduz o passe de producao:\n{}",
        zero_examples.join("\n")
    );

    // O pico que a CPU usa, e que o `ph2d_color::dither` prova exacto em software.
    let cpu_peak = 0.5 * ph2d_color::DITHER_SPAN_LSB * 63.0 / 64.0;
    let (at_cpu_peak, examples) = moved_at(cpu_peak);
    eprintln!("— folga da descida final, medida nesta maquina —");
    eprintln!(
        "  pico da CPU ({cpu_peak:.4} LSB): {at_cpu_peak} pixels de {total:.0} movidos ({:.2}%)",
        at_cpu_peak as f32 / total * 100.0
    );
    for e in &examples {
        eprintln!("{e}");
    }

    // Busca binária pela maior folga que ainda deixa TODOS os 256 bytes intactos nas 64 células.
    let (mut lo, mut hi) = (0.0f32, 0.5f32);
    for _ in 0..10 {
        let mid = (lo + hi) * 0.5;
        if moved_at(mid).0 == 0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    eprintln!("  folga maxima sem mover byte nenhum: ~{lo:.4} LSB (de 0,5 possiveis)");
    eprintln!(
        "  => um dither aqui teria de caber em {:.0}% do que a CPU usa",
        lo / cpu_peak * 100.0
    );

    // ⚠️ Controle POSITIVO: meio passo inteiro TEM de mover alguma coisa. Se não mover, o aparelho
    // parou — e um número saído de um aparelho parado é pior que nenhum.
    assert!(
        moved_at(0.5).0 > 0,
        "um vies de meio passo nao moveu byte nenhum — a sonda nao esta' a medir. Verifique se o \
         `__BIAS__` foi mesmo substituido no WGSL da sonda."
    );
}
