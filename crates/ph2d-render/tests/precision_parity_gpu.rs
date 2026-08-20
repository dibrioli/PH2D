//! **As duas precisões entregam a MESMA cor ao shader** — o gate que abre a W2 do plano
//! [`docs/Sprite_projeto/18`](../../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md).
//!
//! # O defeito que este gate existe para impedir
//!
//! ⚠️ **Não existe variante sRGB de formato de 16 bits algum.** Uma textura `Rgba8UnormSrgb` é
//! decodificada sRGB→linear **pelo hardware** na amostragem; uma `Rgba16Float` **não é**. Logo:
//!
//! - guardar valores **lineares** no `Rgba16Float` ⇒ o shader recebe o mesmo que recebia. ✅
//! - guardar os **bytes sRGB** promovidos a 16 bits ⇒ compila, sobe para a GPU sem uma queixa, e a
//!   sprite renderiza **visivelmente mais clara** que a gémea de 8 bits.
//!
//! O segundo caso é o modo de falha natural de quem escreve `x as u16` — não há erro, não há
//! validação, não há aviso. Só a cor muda. *Um bug que não tem sintoma no código tem de ter um
//! gate na saída.*
//!
//! # Por que AMOSTRAR, e por que ler os bytes crus não serve
//!
//! Comparar o conteúdo das duas texturas byte-a-byte compararia **representações**, que são
//! diferentes por construção (uma é sRGB de 8 bits, a outra meio-float linear) — e passaria verde
//! sobre o bug, porque o bug está exactamente na tradução que a **amostragem** faz. Por isso este
//! teste desenha: um triângulo cobre o alvo, amostra a textura ligada, e o que se lê é o que o
//! shader de sprite leria.
//!
//! # A barra, e a MEDIÇÃO que a mudou
//!
//! ⚠️ A primeira versão comparava os dois **floats** com barra `2⁻¹⁰` (dois passos do meio-float).
//! Ela **reprovou sobre código correto**, e o dump que a diagnosticou vale mais que o gate:
//!
//! ```text
//! x   byte  esperado   guardado16   gpu8       gpu16
//! 5   182   0.467784   0.467773     0.468750   0.467773
//! 6   218   0.701102   0.701172     0.699219   0.701172
//! ```
//!
//! `esperado` é a curva sRGB exacta. O caminho de **16 bits** acerta-a a menos de meio ULP; quem se
//! afasta — até `0,00195` — é o de **8 bits**. Ou seja: **a decodificação sRGB feita pelo hardware
//! é aproximada**, e a barra estava a exigir que o caminho exacto concordasse com o aproximado.
//! *Quando um gate reprova, a primeira pergunta é qual dos dois lados é o oráculo.*
//!
//! A barra passa a ser **a imagem**: as duas leituras voltam a byte sRGB e têm de dar **o mesmo
//! byte**. É o que o produto de facto vê (tudo a jusante é de 8 bits), é imune à precisão da tabela
//! do hardware, e continua a apanhar o defeito do espaço — que afasta os bytes **dezenas** de
//! códigos, não um. A magnitude linear medida fica registada num segundo `assert` para que um
//! crescimento silencioso apareça.
//!
//! ⚠️ Este teste precisa de adapter. Sem GPU ele **salta**, e *saltar não é verde* — quem o corre
//! numa máquina sem placa não recebeu resposta nenhuma.

use ph2d_gpu::GpuContext;
use ph2d_render::IndividualTextureStore;

/// Tecto da divergência linear entre os dois caminhos, **medido** em 2026-08-20 (RTX, wgpu 28):
/// pior caso `0,001953`, que é ~27% de um código sRGB de 8 bits naquele brilho. Não é um limite de
/// precisão do formato — é o erro da tabela sRGB do hardware, e por isso mora aqui como sentinela
/// de crescimento, não como a barra principal.
const WORST_LINEAR_BAR: f32 = 0.003;

const W: u32 = 8;
const H: u32 = 8;

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

/// O mesmo `material_bgl` do `SpritePipeline` — textura filtrável + sampler.
///
/// ⚠️ **Que os dois formatos partilhem ESTE layout é metade da tese da wave.** `Rgba16Float` é
/// `Float { filterable: true }` no WebGPU core, tal como `Rgba8UnormSrgb`; é por isso que a
/// precisão alta não precisa de um segundo pipeline nem de um ramo no shader.
fn material_bgl(gpu: &GpuContext) -> wgpu::BindGroupLayout {
    gpu.device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("precision parity material bgl"),
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
        })
}

/// Triângulo que cobre o alvo e amostra a textura ligada. O alvo tem o tamanho da fonte e o `uv`
/// cai no **centro** de cada texel, por isso o bilinear devolve o texel exacto: o que se mede é a
/// tradução de espaço, não a interpolação.
const WGSL: &str = r#"
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

/// Desenha a textura do `id` para um alvo `Rgba16Float` e devolve os valores lineares lidos.
fn sample_through_the_shader(
    gpu: &GpuContext,
    store: &IndividualTextureStore,
    id: u32,
    bgl: &wgpu::BindGroupLayout,
) -> Vec<f32> {
    let shader = gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("precision parity shader"),
            source: wgpu::ShaderSource::Wgsl(WGSL.into()),
        });
    let layout = gpu
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("precision parity layout"),
            bind_group_layouts: &[bgl],
            immediate_size: 0,
        });
    let target_format = wgpu::TextureFormat::Rgba16Float;
    let pipeline = gpu
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("precision parity pipeline"),
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
                targets: &[Some(target_format.into())],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

    let target = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("precision parity target"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: target_format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());

    // O readback exige linhas múltiplas de 256 bytes; a fonte tem `W × 8`, logo há padding.
    let unpadded = (W * 8) as usize;
    let padded = unpadded.div_ceil(256) * 256;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("precision parity staging"),
        size: (padded * H as usize) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("precision parity pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target_view,
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
        pass.set_bind_group(0, store.bind_group(id).expect("o id existe"), &[]);
        pass.draw(0..3, 0..1);
    }
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &target,
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
    gpu.queue.submit(Some(encoder.finish()));

    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("a fila devia drenar");
    rx.recv()
        .expect("o map devia responder")
        .expect("o map devia ter sucesso");

    let data = slice.get_mapped_range();
    let mut out = Vec::with_capacity((W * H * 4) as usize);
    for row in 0..H as usize {
        let start = row * padded;
        let bytes = &data[start..start + unpadded];
        for pair in bytes.chunks_exact(2) {
            let bits = u16::from_le_bytes([pair[0], pair[1]]);
            out.push(ph2d_imageio::half_to_f32(bits));
        }
    }
    drop(data);
    staging.unmap();
    out
}

/// Uma rampa que cobre os escuros (onde a curva sRGB é mais íngreme e o erro mais visível), os
/// meios-tons e os claros — mais alfa variável, para apanhar quem aplique a curva ao canal errado.
fn ramp() -> Vec<u8> {
    let mut out = Vec::with_capacity((W * H * 4) as usize);
    for y in 0..H {
        for x in 0..W {
            let v = (x * 255 / (W - 1)) as u8;
            out.extend_from_slice(&[v, 255 - v, (y * 255 / (H - 1)) as u8, 255 - (y as u8) * 8]);
        }
    }
    out
}

/// **A LEI DA W2.** A mesma imagem, pelos dois caminhos, chega ao shader com a mesma cor.
#[test]
fn the_two_precisions_deliver_the_same_colour() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("SEM ADAPTER — este teste NAO correu. Saltar nao e' verde.");
        return;
    };
    let bgl = material_bgl(&gpu);
    let mut store = IndividualTextureStore::new(&gpu);
    let source = ramp();

    let id8 = store
        .acquire(&gpu, &bgl, W, H, &source)
        .expect("o caminho de 8 bits devia aceitar a rampa");
    let halves = ph2d_imageio::rgba8_to_rgba16(&source);
    let id16 = store
        .acquire_16(&gpu, &bgl, W, H, &halves)
        .expect("o caminho de 16 bits devia aceitar a rampa");

    assert_eq!(
        store.format(id8),
        Some(wgpu::TextureFormat::Rgba8UnormSrgb),
        "o caminho normal deixou de ser sRGB de 8 bits"
    );
    assert_eq!(
        store.format(id16),
        Some(IndividualTextureStore::FORMAT_16),
        "o caminho de precisao alta nao ficou no formato de 16 bits"
    );

    let got8 = sample_through_the_shader(&gpu, &store, id8, &bgl);
    let got16 = sample_through_the_shader(&gpu, &store, id16, &bgl);
    assert_eq!(got8.len(), got16.len());

    // A barra é a IMAGEM, não o float: as duas leituras voltam a byte sRGB e têm de dar o MESMO
    // byte. Ver o cabeçalho — comparar os floats faria o caminho exacto reprovar por o aproximado
    // ser aproximado.
    let mut broken = Vec::new();
    let mut worst = 0.0_f32;
    for (i, (a, b)) in got8.iter().zip(got16.iter()).enumerate() {
        worst = worst.max((a - b).abs());
        let (byte8, byte16) = (to_byte(i, *a), to_byte(i, *b));
        if byte8 != byte16 {
            broken.push(format!(
                "  canal {} do pixel {} (fonte {}): 8 bits -> {byte8}, 16 bits -> {byte16} \
                 (linear {a:.6} vs {b:.6})",
                i % 4,
                i / 4,
                source[i]
            ));
        }
    }
    assert!(
        broken.is_empty(),
        "as duas precisoes produzem imagens DIFERENTES (divergencia linear maxima {worst:.6}, que \
         sozinha nao e' o defeito -- ver o cabecalho):\n{}\n\
         A causa provavel e' o ESPACO: nao existe variante sRGB de formato de 16 bits, entao o \
         `Rgba16Float` tem de guardar LINEAR. Se alguem escreveu os bytes sRGB promovidos a 16 \
         bits, isto compila, sobe para a GPU sem queixa, e a sprite fica mais clara -- e aqui os \
         bytes afastam-se dezenas de codigos, nao um.",
        broken.join("\n")
    );
    // ⚠️ Documenta a MAGNITUDE medida, e falha se ela crescer uma ordem: a paridade por byte
    // sozinha não distinguiria "exacto" de "quase a virar o byte".
    assert!(
        worst <= WORST_LINEAR_BAR,
        "a divergencia linear maxima subiu para {worst:.6} (medido em 2026-08-20: 0,001953, que e' \
         ~27% de um codigo sRGB de 8 bits neste brilho). Os bytes ainda coincidem, mas alguma coisa \
         mudou no caminho -- meca antes de subir esta barra."
    );
}

/// Volta de linear a byte sRGB. O alfa (canal 3) **não** atravessa a curva — é linear por
/// definição, e passá-lo pela curva é o erro clássico desta conversão.
fn to_byte(index: usize, linear: f32) -> u8 {
    if index % 4 == 3 {
        (linear.clamp(0.0, 1.0) * 255.0).round() as u8
    } else {
        ph2d_color::srgb::linear_to_srgb_byte(linear)
    }
}

/// **Controle positivo — sem ele o gate acima passaria por não medir nada.**
///
/// Se o caminho de 8 bits e o de 16 bits devolvessem ambos zeros (textura vazia, bind group
/// errado, pass que não desenha), a diferença seria `0` e o teste ficaria verde sobre um aparelho
/// morto. Este teste exige que o que foi lido seja **mesmo a rampa**.
#[test]
fn the_apparatus_actually_reads_the_image_it_uploaded() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("SEM ADAPTER — este teste NAO correu. Saltar nao e' verde.");
        return;
    };
    let bgl = material_bgl(&gpu);
    let mut store = IndividualTextureStore::new(&gpu);
    let source = ramp();
    let id8 = store.acquire(&gpu, &bgl, W, H, &source).expect("acquire");
    let got = sample_through_the_shader(&gpu, &store, id8, &bgl);

    // O primeiro pixel da linha 0 é R=0 e G=255; o último da linha é R=255 e G=0. Em linear isso
    // é 0.0 e 1.0 — uma excursão que uma leitura morta não consegue inventar.
    let first_r = got[0];
    let last_r = got[((W - 1) * 4) as usize];
    assert!(
        first_r < 0.01 && last_r > 0.99,
        "o aparelho nao leu a rampa que subiu (R do primeiro pixel = {first_r:.4}, do ultimo = \
         {last_r:.4}). O gate irmao estaria a comparar dois nadas."
    );
    let first_g = got[1];
    assert!(
        first_g > 0.99,
        "o canal G nao veio (deu {first_g:.4}); os canais podem estar trocados ou por preencher"
    );
}
