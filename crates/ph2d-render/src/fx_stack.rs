//! **`FxStackPass`** — a PILHA de filtros raster do módulo vetorial (plano 24), 100% na GPU.
//!
//! Recebe a forma isolada já rasterizada numa textura (premultiplicada — é o que o Vello escreve)
//! e devolve a imagem final numa textura de saída (alfa RETO — o que o `register_texture` do Vello
//! espera), **sem readback e sem uma linha de blur na CPU**. É o que torna o FX viável em RUNTIME
//! de jogo: a forma pode animar todo frame que o custo é um render + alguns passes na placa,
//! nunca um roundtrip GPU→CPU→GPU.
//!
//! Molde: [`crate::impasto_light::ImpastoLightPass`] (passe bespoke textura→textura).
//!
//! # O fold, e o invariante que o torna possível
//!
//! ```text
//! forma rasterizada → [op₁] → [op₂] → … → [opₙ] → resolve → textura de saída (reta)
//! ```
//!
//! **Todo op é imagem → imagem, premultiplicada, do MESMO tamanho** — é por isso que a pilha
//! compõe, e é a mesma frase que governa a pilha de geometria (`ph2d_vec_scene::effect`). A
//! consequência que decide o desenho: **Glow e Drop Shadow compõem o halo POR BAIXO da entrada
//! DENTRO do próprio op** (`src_over(entrada, halo)`), em vez de pedir ao compositor que desenhe
//! algo atrás da forma. Um op que devolvesse *duas* camadas não poderia alimentar o seguinte.
//!
//! Um op que borra custa **dois** dispatches (Gaussiana separável: H, depois V+finalize+composite);
//! um op **pontual** (o Color Overlay) custa **um**. Mais **um** `resolve` no fim, para a pilha
//! inteira. Quem responde *"quantos passes?"* é [`passes_of`] — uma porta, dois consumidores (quem
//! escreve os globals e quem despacha), porque as duas varreduras andam em lockstep sobre a mesma
//! lista e um `if` duplicado as descasaria em silêncio.
//!
//! # Sete tipos, uma tabela
//!
//! Os códigos e o que cada tipo É vivem no [`ph2d_ecs::FxOp`] (`SPECS`): o painel lê a tabela para
//! saber que controles oferecer, este passe lê para saber quanto espalhar e quantos dispatches
//! gastar, e o **WGSL recebe os códigos GERADOS** ([`kind_consts_wgsl`]) em vez de os repetir do
//! outro lado da fronteira de linguagem.
//!
//! # Os intermediários são `Rgba16Float`, e isso não é luxo
//!
//! Entre ops a imagem é premultiplicada. Guardá-la em `Rgba8Unorm` e des-premultiplicar depois
//! **quantiza justamente a borda macia** que o borrão existe para produzir (alfa baixo ⇒ a divisão
//! amplifica o erro). `rgba16float` é formato de storage do baseline do WebGPU, então isto não
//! custa nem uma feature: paga-se largura de banda em texturas temporárias, que são do tamanho da
//! forma.

use ph2d_ecs::FxOp;
use ph2d_gpu::GpuContext;

use crate::fx_stack_shader::{
    FX_STACK_MID_WGSL, FX_STACK_OUT_WGSL, FX_STACK_WGSL, kind_consts_wgsl,
};

/// Meia-largura MÁXIMA do kernel (o laço do shader é limitado por ela). `96` cobre `sigma ≈ 32`
/// px com suporte de 3σ — um borrão bem forte na tela. Acima disso o kernel satura no cap (o
/// borrão para de crescer), o que é um limite de CUSTO honesto do passe, não um teto de produto.
pub const MAX_HALF: u32 = 96;

/// O alinhamento mínimo de offset dinâmico de uniform buffer no WebGPU. A pilha escreve os
/// globals de TODOS os passes de uma vez e indexa por offset — senão um `write_buffer` por passe
/// antes de um único `submit` deixaria o último a valer para todos.
const UNIFORM_STRIDE: u64 = 256;

/// **Um degrau da pilha, já resolvido em PIXELS DE TELA.**
///
/// A conversão mundo→pixel (o zoom da câmera) é da shell: este passe não sabe o que é uma câmera,
/// e um segundo lugar a fazer a conta seria um segundo lugar a errá-la.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FxOpGpu {
    /// `0` Blur · `1` Glow · `2` Drop Shadow (os códigos do `ph2d_ecs::FxOp`).
    pub kind: u8,
    /// O desvio do gaussiano, em pixels de tela.
    pub sigma_px: f32,
    /// O deslocamento do halo, em pixels de tela INTEIROS.
    ///
    /// ⚠️ **Inteiros de propósito.** O halo é amostrado por `textureLoad` (sem sampler), então um
    /// deslocamento fracionário custaria interpolação dentro do laço do borrão. Uma sombra não
    /// precisa de posição sub-pixel — e a textura inteira já é alinhada ao pixel da tela.
    pub offset_px: [i32; 2],
    /// A cor RETA do halo, `[0,1]`.
    pub tint: [f32; 4],
    /// A intensidade deste degrau, `[0,1]`.
    pub opacity: f32,
    /// O MODO (o índice em `FxKindSpec::modes`). Só os degraus de DENTRO o leem hoje.
    pub mode: u8,
}

/// A meia-largura do kernel que o shader de facto percorre para um dado sigma.
///
/// **Porta única**: quem calcula a MARGEM da textura ([`stack_reach`]) e quem a preenche (o
/// shader) têm de concordar, senão o borrão é recortado na borda por uma margem que mentiu.
#[must_use]
pub fn kernel_half(sigma_px: f32) -> u32 {
    let sigma = sigma_px.max(1e-4);
    ((3.0 * sigma).ceil() as u32).clamp(1, MAX_HALF)
}

/// **Como este degrau é executado.** Porta única: quem ESCREVE os globals e quem os DESPACHA
/// perguntam à mesma — as duas varreduras andam em lockstep sobre a mesma lista, e um `if`
/// duplicado as descasaria em silêncio.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Plan {
    /// Um dispatch, sem vizinho nenhum (Color Overlay).
    Point,
    /// Gaussiana separável: H, depois V+finalize+composite.
    Blur,
    /// Campo de distância: semente + `n` saltos do JFA + finalize. Serve os degraus de dentro em
    /// modo Contour **e o CONTORNO** — os dois pedem uma distância, não um borrão.
    Field { jumps: usize },
}

impl Plan {
    fn passes(self) -> usize {
        match self {
            Self::Point => 1,
            Self::Blur => 2,
            Self::Field { jumps } => jumps + 2,
        }
    }
}

/// **Quantos saltos o JFA precisa para uma banda de `band_px`.** Os saltos são `K, K/2, …, 1` com
/// `K = 2^(n-1)`, e o alcance do JFA é a SOMA deles (`2K-1`), logo `n = bits(w)` cobre `w`.
fn jump_count(band_px: f32) -> usize {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let w = band_px.max(1.0).ceil() as u32;
    (u32::BITS - w.leading_zeros()) as usize
}

fn plan_of(op: &FxOpGpu) -> Plan {
    let spec = FxOp::spec(op.kind);
    let by_distance = matches!(op.kind, FxOp::OUTLINE | FxOp::FEATHER | FxOp::BEVEL)
        || (spec.inner && op.mode == FxOp::MODE_CONTOUR);
    if by_distance {
        return Plan::Field {
            jumps: jump_count(op.sigma_px),
        };
    }
    if spec.radius_label.is_none() {
        return Plan::Point;
    }
    Plan::Blur
}

/// **Quanto ESTE degrau espalha para fora do que recebeu**, em pixels.
///
/// Três respostas, e cada uma é um fato sobre o tipo, não uma margem "por segurança":
/// - quem não cresce ([`FxKindSpec::grows`](ph2d_ecs::FxKindSpec) falso) espalha **zero** — o
///   Inner Shadow / Inner Glow desenham só DENTRO da forma, e o Color Overlay não move um texel de
///   cobertura. Margem para eles seria textura paga a troco de nada;
/// - o **Outline** espalha a LARGURA dele (`σ`), não o suporte do kernel (`3σ`): o corte é duro em
///   `Φ(−1)`, então além de `σ` não sobra nada para recortar. (O kernel ainda percorre `3σ` — o
///   *suporte* e o *alcance* são perguntas diferentes, e é por isso que são duas funções.)
/// - o resto espalha o suporte da Gaussiana.
fn op_reach(op: &FxOpGpu) -> u32 {
    if !FxOp::spec(op.kind).grows {
        return 0;
    }
    if matches!(op.kind, FxOp::OUTLINE | FxOp::FEATHER) {
        // O contorno alcança a LARGURA dele; o feather alcança METADE dela (a rampa é centrada na
        // fronteira). Nenhum dos dois paga o suporte do kernel, que é 3×.
        let span = if op.kind == FxOp::FEATHER {
            op.sigma_px * 0.5
        } else {
            op.sigma_px
        };
        return (span.max(0.0).ceil() as u32 + 1).clamp(1, MAX_HALF);
    }
    kernel_half(op.sigma_px)
}

/// **Quanto a pilha inteira espalha, em pixels, para cada lado.** Devolve
/// `(esquerda, cima, direita, baixo)`.
///
/// Cada degrau espalha o que recebeu — logo as reaches **somam** ao longo da pilha, e a margem é
/// função da pilha, nunca do maior degrau. O borrão espalha para os quatro lados; o deslocamento
/// da sombra só para o lado para onde aponta, e é por isso que a margem é assimétrica (uma sombra
/// longa para a direita não paga textura à esquerda).
///
/// ⚠️ O deslocamento de um op de DENTRO não conta: ele desloca o halo *dentro* da silhueta, e a
/// máscara o corta na borda. Quem decide é a mesma [`FxOp::spec`] que decide as rows do painel.
#[must_use]
pub fn stack_reach(ops: &[FxOpGpu]) -> (u32, u32, u32, u32) {
    let (mut l, mut t, mut r, mut b) = (0u32, 0u32, 0u32, 0u32);
    for op in ops {
        let reach = op_reach(op);
        l += reach;
        t += reach;
        r += reach;
        b += reach;
        let spec = FxOp::spec(op.kind);
        if spec.offset_labels.is_some() && spec.grows {
            let (ox, oy) = (op.offset_px[0], op.offset_px[1]);
            l += ox.min(0).unsigned_abs();
            r += ox.max(0).unsigned_abs();
            t += oy.min(0).unsigned_abs();
            b += oy.max(0).unsigned_abs();
        }
    }
    (l, t, r, b)
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    dims: [u32; 2],
    half: u32,
    kind: u32,
    tint: [f32; 4],
    inv_two_sigma2: f32,
    opacity: f32,
    off_x: i32,
    off_y: i32,
    /// O passo do salto do JFA (só os passes de campo de distância o leem).
    jump: i32,
    /// A largura da banda (modo Contour) / do contorno, em pixels.
    band: f32,
    _pad: [f32; 2],
}

struct Tex {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    w: u32,
    h: u32,
}

fn make_tex(gpu: &GpuContext, w: u32, h: u32, format: wgpu::TextureFormat) -> Tex {
    let (w, h) = (w.max(1), h.max(1));
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-render fx_stack tex"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    Tex {
        texture,
        view,
        w,
        h,
    }
}

/// Cria uma textura de SAÍDA para o FX (o chamador a mantém viva por-forma — o Vello a copia no
/// render, DEPOIS do recook). `Rgba8Unorm` com os usos que o `register_texture` exige.
#[must_use]
pub fn make_output_texture(gpu: &GpuContext, w: u32, h: u32) -> wgpu::Texture {
    make_tex(gpu, w, h, wgpu::TextureFormat::Rgba8Unorm).texture
}

/// O passe da pilha. Pipelines build-once; as três texturas de trabalho (ping/pong/mid) são
/// **grow-only** — uma cena com formas de tamanhos diferentes não realoca por forma por frame.
pub struct FxStackPass {
    pipeline_h: wgpu::ComputePipeline,
    pipeline_v: wgpu::ComputePipeline,
    pipeline_point: wgpu::ComputePipeline,
    pipeline_seed: wgpu::ComputePipeline,
    pipeline_jump: wgpu::ComputePipeline,
    pipeline_field: wgpu::ComputePipeline,
    pipeline_out: wgpu::ComputePipeline,
    bgl_mid: wgpu::BindGroupLayout,
    bgl_out: wgpu::BindGroupLayout,
    globals: wgpu::Buffer,
    globals_cap: u64,
    work: Option<[Tex; 4]>,
}

impl FxStackPass {
    /// Constrói os três pipelines. Barato — nenhuma textura até o 1º [`Self::run`].
    #[must_use]
    pub fn new(gpu: &GpuContext) -> Self {
        let kinds = kind_consts_wgsl();
        let mid_src = format!("{kinds}{FX_STACK_WGSL}{FX_STACK_MID_WGSL}");
        let out_src = format!("{kinds}{FX_STACK_WGSL}{FX_STACK_OUT_WGSL}");
        let make_shader = |label: &str, src: &str| {
            gpu.device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some(label),
                    source: wgpu::ShaderSource::Wgsl(src.into()),
                })
        };
        let shader_mid = make_shader("ph2d-render fx_stack mid", &mid_src);
        let shader_out = make_shader("ph2d-render fx_stack out", &out_src);

        let bgl_mid = Self::layout(gpu, wgpu::TextureFormat::Rgba16Float);
        let bgl_out = Self::layout(gpu, wgpu::TextureFormat::Rgba8Unorm);
        let pl = |bgl: &wgpu::BindGroupLayout| {
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("ph2d-render fx_stack layout"),
                    bind_group_layouts: &[bgl],
                    immediate_size: 0,
                })
        };
        let layout_mid = pl(&bgl_mid);
        let layout_out = pl(&bgl_out);
        let make_pipe =
            |layout: &wgpu::PipelineLayout, module: &wgpu::ShaderModule, entry: &str| {
                gpu.device
                    .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                        label: Some("ph2d-render fx_stack pipeline"),
                        layout: Some(layout),
                        module,
                        entry_point: Some(entry),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        cache: None,
                    })
            };
        let pipeline_h = make_pipe(&layout_mid, &shader_mid, "cs_blur_h");
        let pipeline_v = make_pipe(&layout_mid, &shader_mid, "cs_op_v");
        let pipeline_point = make_pipe(&layout_mid, &shader_mid, "cs_op_point");
        let pipeline_seed = make_pipe(&layout_mid, &shader_mid, "cs_sdf_seed");
        let pipeline_jump = make_pipe(&layout_mid, &shader_mid, "cs_sdf_jump");
        let pipeline_field = make_pipe(&layout_mid, &shader_mid, "cs_op_field");
        let pipeline_out = make_pipe(&layout_out, &shader_out, "cs_resolve");

        let globals_cap = UNIFORM_STRIDE * 8;
        let globals = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-render fx_stack globals"),
            size: globals_cap,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            pipeline_h,
            pipeline_v,
            pipeline_point,
            pipeline_seed,
            pipeline_jump,
            pipeline_field,
            pipeline_out,
            bgl_mid,
            bgl_out,
            globals,
            globals_cap,
            work: None,
        }
    }

    fn layout(gpu: &GpuContext, storage: wgpu::TextureFormat) -> wgpu::BindGroupLayout {
        let sampled = |binding: u32| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        gpu.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ph2d-render fx_stack bgl"),
                entries: &[
                    sampled(0),
                    sampled(1),
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: wgpu::BufferSize::new(
                                std::mem::size_of::<Globals>() as u64
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: storage,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            })
    }

    /// Roda a pilha: `src` (premultiplicada, `w×h`) → `ops` em sequência → `dst` (reta, `w×h`).
    ///
    /// Uma pilha VAZIA ainda resolve (`src` des-premultiplicada em `dst`) — o chamador é quem
    /// decide não produzir imagem nenhuma para uma forma sem filtro; aqui a operação vazia é a
    /// identidade, não um caso especial.
    pub fn run(
        &mut self,
        gpu: &GpuContext,
        src: &wgpu::Texture,
        dst: &wgpu::Texture,
        w: u32,
        h: u32,
        ops: &[FxOpGpu],
    ) {
        if w == 0 || h == 0 {
            return;
        }
        self.ensure_work(gpu, w, h);
        let total_passes: usize = ops.iter().map(|o| plan_of(o).passes()).sum();
        self.ensure_globals(gpu, total_passes);

        // Os globals de TODOS os passes, escritos de uma vez e indexados por offset dinâmico.
        let mut blob = vec![0u8; (total_passes + 1) * UNIFORM_STRIDE as usize];
        let mut slot = 0usize;
        for op in ops {
            let sigma = op.sigma_px.max(1e-4);
            let g = Globals {
                dims: [w, h],
                half: kernel_half(op.sigma_px),
                kind: u32::from(op.kind),
                tint: op.tint,
                inv_two_sigma2: 1.0 / (2.0 * sigma * sigma),
                opacity: op.opacity,
                off_x: op.offset_px[0],
                off_y: op.offset_px[1],
                jump: 0,
                band: op.sigma_px.max(0.0),
                _pad: [0.0; 2],
            };
            match plan_of(op) {
                Plan::Point => {
                    write_at(&mut blob, slot, &g);
                    slot += 1;
                }
                Plan::Blur => {
                    // O passe H leva o deslocamento em X; o V leva o de Y. Juntos, amostram o halo
                    // em `(x - off_x, y - off_y)` — a sombra cai deslocada DENTRO da imagem.
                    let mut gh = g;
                    gh.off_y = 0;
                    let mut gv = g;
                    gv.off_x = 0;
                    write_at(&mut blob, slot, &gh);
                    write_at(&mut blob, slot + 1, &gv);
                    slot += 2;
                }
                Plan::Field { jumps } => {
                    // Semente (não lê deslocamento nenhum) · os saltos `K, K/2, …, 1` · o finalize,
                    // que é o único que amostra o campo DESLOCADO pela luz.
                    let mut seed = g;
                    seed.off_x = 0;
                    seed.off_y = 0;
                    write_at(&mut blob, slot, &seed);
                    for i in 0..jumps {
                        let mut gj = seed;
                        gj.jump = 1 << (jumps - 1 - i);
                        write_at(&mut blob, slot + 1 + i, &gj);
                    }
                    write_at(&mut blob, slot + 1 + jumps, &g);
                    slot += jumps + 2;
                }
            }
        }
        let resolve = Globals {
            dims: [w, h],
            half: 1,
            kind: 0,
            tint: [0.0; 4],
            inv_two_sigma2: 1.0,
            opacity: 1.0,
            off_x: 0,
            off_y: 0,
            jump: 0,
            band: 1.0,
            _pad: [0.0; 2],
        };
        write_at(&mut blob, total_passes, &resolve);
        gpu.queue.write_buffer(&self.globals, 0, &blob);

        let work = self.work.as_ref().expect("just ensured");
        let src_view = src.create_view(&wgpu::TextureViewDescriptor::default());
        let dst_view = dst.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render fx_stack encoder"),
            });
        let (gx, gy) = (w.div_ceil(8), h.div_ceil(8));

        // ping/pong da IMAGEM entre work[0] e work[1]; work[2]/work[3] são o temp do passe
        // horizontal e o ping/pong do CAMPO DE DISTÂNCIA (que nunca coexiste com o temp do blur —
        // um op é de um tipo só).
        let mut cur: Option<usize> = None; // `None` = a fonte é `src`
        let mut slot = 0u32;
        for op in ops {
            let input = cur.map_or(&src_view, |k| &work[k].view);
            let next = cur.map_or(0, |k| 1 - k);
            match plan_of(op) {
                Plan::Point => {
                    // Pontual: lê a entrada e escreve o resultado — sem intermediário.
                    let bg = self.bind(gpu, &self.bgl_mid, input, input, &work[next].view);
                    dispatch(&mut encoder, &self.pipeline_point, &bg, slot, gx, gy);
                    slot += 1;
                }
                Plan::Blur => {
                    let bg_h = self.bind(gpu, &self.bgl_mid, input, input, &work[2].view);
                    let bg_v =
                        self.bind(gpu, &self.bgl_mid, input, &work[2].view, &work[next].view);
                    dispatch(&mut encoder, &self.pipeline_h, &bg_h, slot, gx, gy);
                    dispatch(&mut encoder, &self.pipeline_v, &bg_v, slot + 1, gx, gy);
                    slot += 2;
                }
                Plan::Field { jumps } => {
                    let bg_seed = self.bind(gpu, &self.bgl_mid, input, input, &work[2].view);
                    dispatch(&mut encoder, &self.pipeline_seed, &bg_seed, slot, gx, gy);
                    slot += 1;
                    // Ping/pong 2 <-> 3: o salto `i` lê de onde o anterior escreveu.
                    let mut field = 2usize;
                    for _ in 0..jumps {
                        let to = 5 - field; // 2 <-> 3
                        let bg =
                            self.bind(gpu, &self.bgl_mid, input, &work[field].view, &work[to].view);
                        dispatch(&mut encoder, &self.pipeline_jump, &bg, slot, gx, gy);
                        slot += 1;
                        field = to;
                    }
                    let bg = self.bind(
                        gpu,
                        &self.bgl_mid,
                        input,
                        &work[field].view,
                        &work[next].view,
                    );
                    dispatch(&mut encoder, &self.pipeline_field, &bg, slot, gx, gy);
                    slot += 1;
                }
            }
            cur = Some(next);
        }
        let last = cur.map_or(&src_view, |k| &work[k].view);
        let bg_out = self.bind(gpu, &self.bgl_out, last, last, &dst_view);
        dispatch(&mut encoder, &self.pipeline_out, &bg_out, slot, gx, gy);
        gpu.queue.submit([encoder.finish()]);
    }

    fn bind(
        &self,
        gpu: &GpuContext,
        layout: &wgpu::BindGroupLayout,
        t0: &wgpu::TextureView,
        t1: &wgpu::TextureView,
        dst: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render fx_stack bg"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(t0),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(t1),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.globals,
                        offset: 0,
                        size: wgpu::BufferSize::new(std::mem::size_of::<Globals>() as u64),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(dst),
                },
            ],
        })
    }

    /// As texturas de trabalho, **grow-only**: uma cena com formas de tamanhos diferentes não
    /// paga uma realocação por forma por frame. O shader escreve só `dims`, então uma textura
    /// maior que a região é inofensiva.
    fn ensure_work(&mut self, gpu: &GpuContext, w: u32, h: u32) {
        let big = matches!(&self.work, Some(t) if t[0].w >= w && t[0].h >= h);
        if big {
            return;
        }
        let (nw, nh) = match &self.work {
            Some(t) => (t[0].w.max(w), t[0].h.max(h)),
            None => (w, h),
        };
        let f = wgpu::TextureFormat::Rgba16Float;
        self.work = Some([
            make_tex(gpu, nw, nh, f),
            make_tex(gpu, nw, nh, f),
            make_tex(gpu, nw, nh, f),
            make_tex(gpu, nw, nh, f),
        ]);
    }

    fn ensure_globals(&mut self, gpu: &GpuContext, total_passes: usize) {
        let need = ((total_passes + 1) as u64) * UNIFORM_STRIDE;
        if need <= self.globals_cap {
            return;
        }
        self.globals = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-render fx_stack globals"),
            size: need,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.globals_cap = need;
    }
}

fn write_at(blob: &mut [u8], slot: usize, g: &Globals) {
    let off = slot * UNIFORM_STRIDE as usize;
    let bytes = bytemuck::bytes_of(g);
    blob[off..off + bytes.len()].copy_from_slice(bytes);
}

fn dispatch(
    encoder: &mut wgpu::CommandEncoder,
    pipeline: &wgpu::ComputePipeline,
    bg: &wgpu::BindGroup,
    slot: u32,
    gx: u32,
    gy: u32,
) {
    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("ph2d-render fx_stack pass"),
        timestamp_writes: None,
    });
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, bg, &[slot * UNIFORM_STRIDE as u32]);
    pass.dispatch_workgroups(gx, gy, 1);
}
