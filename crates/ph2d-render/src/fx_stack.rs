//! **`FxStackPass`** — a PILHA de filtros raster do módulo vetorial (plano 24), 100% na GPU.
//!
//! Recebe a forma isolada já rasterizada numa textura (sRGB, alfa **RETO** — é o que o
//! `render_to_intermediate` do Vello escreve, MEDIDO) e devolve a imagem final numa textura de
//! saída na mesma convenção (o que o `register_texture` do Vello espera), **sem readback e sem uma
//! linha de blur na CPU**. É o que torna o FX viável em RUNTIME
//! de jogo: a forma pode animar todo frame que o custo é um render + alguns passes na placa,
//! nunca um roundtrip GPU→CPU→GPU.
//!
//! Molde: [`crate::impasto_light::ImpastoLightPass`] (passe bespoke textura→textura).
//!
//! # O fold, e o invariante que o torna possível
//!
//! ```text
//! forma rasterizada (sRGB) → ingest → [op₁] → [op₂] → … → [opₙ] → resolve → saída (sRGB, reta)
//!                                     └────────── LINEAR premultiplicado ──────────┘
//! ```
//!
//! **O miolo fala LINEAR PREMULTIPLICADO, e só as duas pontas falam sRGB reto.** As duas
//! conversões vivem no `cs_ingest` e no `cs_resolve`, e mais em lugar nenhum.
//!
//! Linear porque é a convenção de toda composição séria (o `linearRGB` default dos filtros SVG, o
//! *1.0 Gamma* do AE, OpenEXR/ACES): um borrão feito sobre bytes codificados produz a franja escura
//! clássica. Premultiplicado porque é o que Porter-Duff exige — o halo por baixo, a soma pesada do
//! borrão e o `inner_tint` só fecham com alfa associado.
//!
//! ⚠️ **Este módulo afirmou por muito tempo que a FONTE já era premultiplicada, e era falso** — o
//! censo do rasterizador real dá 1696 de 1696 texels parciais com a cor cheia. O sintoma era o
//! contorno tracejado do feather. Detalhe no `cs_ingest` ([`crate::fx_stack_shader`]).
//!
//! **Todo op é imagem → imagem, LINEAR premultiplicada, do MESMO tamanho** — é por isso que a pilha
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
//! # Uma tabela, e o número de tipos não se escreve aqui
//!
//! Os códigos e o que cada tipo É vivem no [`ph2d_ecs::FxOp`] (`SPECS`): o painel lê a tabela para
//! saber que controles oferecer, este passe lê para saber quanto espalhar e quantos dispatches
//! gastar, e o **WGSL recebe os códigos GERADOS** ([`kind_consts_wgsl`]). (Este título dizia *"Sete
//! tipos"* e envelheceu quatro vezes — um número numa prosa é uma cópia que ninguém atualiza.)
//!
//! # Os intermediários são `Rgba16Float`, e isso não é luxo
//!
//! Entre ops a imagem é linear premultiplicada. Guardá-la em `Rgba8Unorm` e des-premultiplicar depois
//! **quantiza justamente a borda macia** que o borrão existe para produzir (alfa baixo ⇒ a divisão
//! amplifica o erro). `rgba16float` é formato de storage do baseline do WebGPU, então isto não
//! custa nem uma feature: paga-se largura de banda em texturas temporárias, que são do tamanho da
//! forma.

use ph2d_gpu::GpuContext;

use crate::fx_stack_field::FIELD_WGSL;
use crate::fx_stack_noise::{NOISE_WGSL, WARP_WGSL};
use crate::fx_stack_res::{Globals, Tex, make_tex, write_at};
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
///
/// ⚠️ **512 e não 256, e a diferença é MEDIDA:** os oito stops da rampa levam o `Globals` a 320
/// bytes, e o offset dinâmico tem de ser múltiplo do alinhamento do device (256). Dobrar o stride
/// custa **nada** — o mesmo frame de 32 formas dá **2,345 ms contra 2,332**, dentro do ruído (duas
/// corridas cada). ⚠️ A primeira leitura desta medição disse **3× pior** e era **carga da
/// máquina**; o teto de stops teria nascido errado se eu não a tivesse repetido.
pub(crate) const UNIFORM_STRIDE: u64 = 512;

pub use crate::fx_stack_op::FxOpGpu;
use crate::fx_stack_plan::plan_of;
pub use crate::fx_stack_plan::{jump_count, kernel_half, op_reach, stack_reach};

/// **Como este degrau é executado.** Porta única: quem ESCREVE os globals e quem os DESPACHA
/// perguntam à mesma — as duas varreduras andam em lockstep sobre a mesma lista, e um `if`
/// duplicado as descasaria em silêncio.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Plan {
    /// Um dispatch, sem vizinho nenhum (Color Overlay).
    Point,
    /// Gaussiana separável: H, depois V+finalize+composite.
    Blur,
    /// Campo de distância: semente + `n` saltos do JFA + finalize. Serve os degraus de dentro em
    /// modo Contour **e o CONTORNO** — os dois pedem uma distância, não um borrão.
    Field { jumps: usize, raster_seed: bool },
    /// Um dispatch que lê a imagem numa posição DESLOCADA (a turbulência). Um passe só, como o
    /// `Point`, mas **não** pontual — o nome do outro mentiria sobre o que ele faz, e é ele que
    /// escolhe o pipeline.
    Warp,
}

impl Plan {
    fn passes(self) -> usize {
        match self {
            Self::Point | Self::Warp => 1,
            Self::Blur => 2,
            // Com geometria não há semente nem saltos: só o finalize.
            Self::Field { jumps, raster_seed } => jumps + usize::from(raster_seed) + 1,
        }
    }
}

/// O passe da pilha. Pipelines build-once; as três texturas de trabalho (ping/pong/mid) são
/// **grow-only** — uma cena com formas de tamanhos diferentes não realoca por forma por frame.
pub struct FxStackPass {
    pipeline_ingest: wgpu::ComputePipeline,
    pipeline_h: wgpu::ComputePipeline,
    pipeline_v: wgpu::ComputePipeline,
    pipeline_point: wgpu::ComputePipeline,
    pipeline_warp: wgpu::ComputePipeline,
    pipeline_seed: wgpu::ComputePipeline,
    pipeline_jump: wgpu::ComputePipeline,
    pipeline_field: wgpu::ComputePipeline,
    pipeline_out: wgpu::ComputePipeline,
    bgl_mid: wgpu::BindGroupLayout,
    bgl_out: wgpu::BindGroupLayout,
    globals: wgpu::Buffer,
    globals_cap: u64,
    /// Os segmentos da silhueta, em espaço de TEXEL do scratch. Grow-only, como as `work`.
    segs: wgpu::Buffer,
    segs_cap: u64,
    work: Option<[Tex; 4]>,
}

/// **Os dois módulos WGSL que a pilha compila** — porta ÚNICA, com dois consumidores: o
/// [`FxStackPass::new`] (que os manda ao dispositivo) e o gate de naga (que os valida sem GPU).
///
/// ⚠️ **É porta única por um motivo que já mordeu este repositório:** um gate que montasse a sua
/// própria concatenação ficaria VERDE sobre um produto que deixou de prefixar o bloco
/// compartilhado — validar-se-ia a si mesmo. Aqui o gate valida exactamente a string que o
/// dispositivo recebe.
///
/// ⚠️ **As leis de mistura vêm do MESMO arquivo que o compositor de camadas compila** — o
/// `blend_modes.wgsl` foi extraído dele quando ganhou este segundo consumidor. *Como duas cores se
/// combinam* é pergunta já respondida e pinada bit a bit contra o Rust; uma cópia aqui divergiria
/// no único lugar onde ninguém lê um número: uma captura de tela.
fn module_sources() -> [(&'static str, String); 2] {
    let kinds = kind_consts_wgsl();
    let blend = crate::layer_compositor::BLEND_MODES_WGSL;
    // ⚠️ **O ajuste de cor vem do arquivo COMPARTILHADO**, o mesmo movimento das leis de
    // mistura: a lei do Color Adjust É a do `AdjustmentKind::HueSaturationBrightness` do
    // Painter, e uma cópia local seria a segunda resposta que este prefixo existe para não
    // ter (há gate).
    let adjust = crate::layer_compositor::COLOUR_ADJUST_WGSL;
    [
        (
            "mid",
            // ⚠️ O RUÍDO entra ANTES do fold (o `cs_op_warp` do `WARP_WGSL` chama o `fbm`) e o
            // WARP depois dele (ele chama o `tap_img`). A ordem é a das dependências, não gosto.
            format!(
                "{blend}\n{adjust}\n{kinds}{NOISE_WGSL}{FX_STACK_WGSL}{FX_STACK_MID_WGSL}{FIELD_WGSL}{WARP_WGSL}"
            ),
        ),
        (
            "out",
            // O módulo de SAÍDA escreve noutro formato de storage e não tem o `dst` que o warp
            // usa — ele não recebe o passe, só o campo de que o resto do fold não depende.
            format!("{blend}\n{adjust}\n{kinds}{FX_STACK_WGSL}{FX_STACK_OUT_WGSL}"),
        ),
    ]
}

impl FxStackPass {
    /// Constrói os três pipelines. Barato — nenhuma textura até o 1º [`Self::run`].
    #[must_use]
    pub fn new(gpu: &GpuContext) -> Self {
        let [(_, mid_src), (_, out_src)] = module_sources();
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
        let pipeline_ingest = make_pipe(&layout_mid, &shader_mid, "cs_ingest");
        let pipeline_h = make_pipe(&layout_mid, &shader_mid, "cs_blur_h");
        let pipeline_v = make_pipe(&layout_mid, &shader_mid, "cs_op_v");
        let pipeline_point = make_pipe(&layout_mid, &shader_mid, "cs_op_point");
        let pipeline_warp = make_pipe(&layout_mid, &shader_mid, "cs_op_warp");
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
        let segs_cap = (std::mem::size_of::<[f32; 4]>() * 64) as u64;
        let segs = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-render fx_stack segments"),
            size: segs_cap,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            pipeline_ingest,
            pipeline_h,
            pipeline_v,
            pipeline_point,
            pipeline_warp,
            pipeline_seed,
            pipeline_jump,
            pipeline_field,
            pipeline_out,
            bgl_mid,
            bgl_out,
            globals,
            globals_cap,
            segs,
            segs_cap,
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
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            })
    }

    /// Roda a pilha: `src` (sRGB, alfa RETO, `w×h`) → `ops` em sequência → `dst` (mesma
    /// convenção, `w×h`).
    ///
    /// Uma pilha VAZIA ainda faz o par ingest+resolve — o chamador é quem decide não produzir
    /// imagem nenhuma para uma forma sem filtro; aqui a operação vazia é a identidade, não um caso
    /// especial. ⚠️ E é identidade **de facto**: a viagem sRGB→linear f16→sRGB devolve o byte de
    /// entrada (há gate), porque meia-precisão em ponto flutuante tem folga de sobra sobre 8 bits.
    // ⚠️ Oito argumentos, e o oitavo é a GEOMETRIA — o que separa um campo de distância exato de
    // um estimado da cobertura. Agrupá-los num struct de opções esconderia justamente o parâmetro
    // cuja presença muda o número de passes despachados.
    #[allow(clippy::too_many_arguments)]
    pub fn run(
        &mut self,
        gpu: &GpuContext,
        src: &wgpu::Texture,
        dst: &wgpu::Texture,
        w: u32,
        h: u32,
        ops: &[FxOpGpu],
        geom: &[[f32; 4]],
    ) {
        self.run_from(gpu, src, [0, 0], dst, w, h, ops, geom);
    }

    /// A mesma pilha, lendo a fonte a partir de `src_org` — a forma vive numa **CÉLULA** de uma
    /// textura partilhada por todas as formas filtradas do frame.
    ///
    /// ⚠️ **Só o ingest desloca.** Tudo a jusante (as work textures, os segmentos de silhueta, a
    /// origem do ruído, o `dst`) fala em coordenadas LOCAIS da forma, exactamente como antes —
    /// então esta porta não é um segundo sistema de coordenadas a correr em paralelo, é uma
    /// tradução feita uma vez, na fronteira.
    #[allow(clippy::too_many_arguments)]
    pub fn run_from(
        &mut self,
        gpu: &GpuContext,
        src: &wgpu::Texture,
        src_org: [i32; 2],
        dst: &wgpu::Texture,
        w: u32,
        h: u32,
        ops: &[FxOpGpu],
        geom: &[[f32; 4]],
    ) {
        if w == 0 || h == 0 {
            return;
        }
        self.ensure_work(gpu, w, h);
        self.ensure_segments(gpu, geom);
        // Slot 0 é o INGEST (sempre) · depois os ops · o resolve é o último.
        let op_passes: usize = ops
            .iter()
            .map(|o| plan_of(o, geom.is_empty()).passes())
            .sum();
        let total_slots = op_passes + 2;
        self.ensure_globals(gpu, total_slots);

        // Os globals de TODOS os passes, escritos de uma vez e indexados por offset dinâmico.
        let mut blob = vec![0u8; total_slots * UNIFORM_STRIDE as usize];
        // O ingest e o resolve só precisam saber as dimensões — nenhum lê tint, sigma ou banda.
        let edges = Globals {
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
            n_segs: 0,
            blend: 0,
            noise_scale: 1.0,
            octaves: 1,
            seed: 0,
            mode: 0,
            org: [0.0, 0.0],
            grow_px: 0.0,
            _pad: [0.0],
            hue: 0.0,
            sat: 0.0,
            bright: 0.0,
            _pad2: [0.0],
            tint_b: [0.0; 4],
            src_org: [0, 0],
            stop_count: 0,
            _pad3: 0,
            stops: [[0.0; 4]; 8],
            stop_pos: [[0.0; 4]; 2],
        };
        // ⚠️ O INGEST leva a origem da célula; o RESOLVE, não. Os dois compartilham este `edges`
        // porque nenhum lê tint/sigma/banda — mas o resolve lê `work[cur]`, que já é local, e
        // dar-lhe a origem seria escrever um número que ninguém lê hoje e que o próximo leitor
        // interpretaria como verdade.
        write_at(&mut blob, 0, &Globals { src_org, ..edges });
        write_at(&mut blob, total_slots - 1, &edges);
        // **A ORIGEM da grade de ruído é a MARGEM da pilha** — a mesma `stack_reach` que
        // dimensionou o scratch, perguntada aqui em vez de recebida por parâmetro: quem reserva a
        // margem e quem ancora o padrão têm de dar a MESMA resposta, e um argumento a mais seria
        // uma segunda oportunidade de discordar. Assim `(pixel − org)` é a posição relativa à
        // caixa da FORMA, e o padrão não anda quando outro degrau muda de raio.
        let (ml, mt, _, _) = stack_reach(ops);
        #[allow(clippy::cast_precision_loss)]
        let org = [ml as f32, mt as f32];
        let mut slot = 1usize;
        for op in ops {
            let sigma = op.sigma_px.max(1e-4);
            let plan = plan_of(op, geom.is_empty());
            // ⚠️ **A pergunta *"contra o QUE este degrau mede?"* tem UMA resposta, e ela é o PLANO.**
            // O finalize do campo escolhe entre o pé exato dos segmentos e a textura semeada
            // olhando `n_segs`, então semear o raster e deixar `n_segs` a apontar para a geometria
            // construiria um campo que ninguém lê e responderia pela FORMA a um degrau que
            // perguntou pela IMAGEM — sem erro nenhum, e com todo gate de unidade verde.
            //
            // Isto **preserva** o que já se fazia: `raster_seed` era `geom.is_empty()`, logo os dois
            // já coincidiam. O que ele impede é o quinto tipo do campo divergir em silêncio.
            let n_segs = match plan {
                Plan::Field {
                    raster_seed: true, ..
                } => 0,
                _ => u32::try_from(geom.len()).unwrap_or(u32::MAX),
            };
            let g = Globals::for_op(op, [w, h], org, n_segs);
            match plan {
                Plan::Point | Plan::Warp => {
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
                Plan::Field { jumps, raster_seed } => {
                    // Semente (não lê deslocamento nenhum) · os saltos `K, K/2, …, 1` · o finalize,
                    // que é o único que amostra o campo DESLOCADO pela luz.
                    let head = usize::from(raster_seed);
                    if raster_seed {
                        let mut seed = g;
                        seed.off_x = 0;
                        seed.off_y = 0;
                        write_at(&mut blob, slot, &seed);
                        for i in 0..jumps {
                            let mut gj = seed;
                            gj.jump = 1 << (jumps - 1 - i);
                            write_at(&mut blob, slot + 1 + i, &gj);
                        }
                    }
                    write_at(&mut blob, slot + head + jumps, &g);
                    slot += jumps + head + 1;
                }
            }
        }
        debug_assert_eq!(
            slot,
            total_slots - 1,
            "as duas varreduras andam em lockstep"
        );
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
        //
        // ⚠️ O INGEST é o que põe a fonte no espaço de trabalho, então depois dele **nenhum op
        // volta a ler `src`**: `cur` nasce em `Some(0)` e nunca mais é `None`. É isso que garante
        // que a convenção linear não tem furo — não sobra um caminho que leia sRGB por engano.
        let bg_in = self.bind(gpu, &self.bgl_mid, &src_view, &src_view, &work[0].view);
        dispatch(&mut encoder, &self.pipeline_ingest, &bg_in, 0, gx, gy);
        let mut cur: usize = 0;
        let mut slot = 1u32;
        for op in ops {
            let input = &work[cur].view;
            let next = 1 - cur;
            match plan_of(op, geom.is_empty()) {
                Plan::Point => {
                    // Pontual: lê a entrada e escreve o resultado — sem intermediário.
                    let bg = self.bind(gpu, &self.bgl_mid, input, input, &work[next].view);
                    dispatch(&mut encoder, &self.pipeline_point, &bg, slot, gx, gy);
                    slot += 1;
                }
                Plan::Warp => {
                    // Deforma: lê a entrada numa posição deslocada pelo ruído. Um dispatch, como o
                    // pontual, mas cada texel lê os VIZINHOS — daí o pipeline próprio.
                    let bg = self.bind(gpu, &self.bgl_mid, input, input, &work[next].view);
                    dispatch(&mut encoder, &self.pipeline_warp, &bg, slot, gx, gy);
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
                Plan::Field { jumps, raster_seed } => {
                    let mut field = 2usize;
                    if raster_seed {
                        let bg_seed = self.bind(gpu, &self.bgl_mid, input, input, &work[2].view);
                        dispatch(&mut encoder, &self.pipeline_seed, &bg_seed, slot, gx, gy);
                        slot += 1;
                    }
                    // Ping/pong 2 <-> 3: o salto `i` lê de onde o anterior escreveu.
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
            cur = next;
        }
        let last = &work[cur].view;
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
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: self.segs.as_entire_binding(),
                },
            ],
        })
    }

    /// Sobe os segmentos da silhueta. **Grow-only**, como as `work` — uma cena com formas de
    /// complexidades diferentes não paga uma realocação por forma por frame.
    ///
    /// ⚠️ O buffer NUNCA encolhe a zero: um `storage` de tamanho 0 é binding inválido, e o layout
    /// exige a entrada mesmo nos passes que não a leem. Sem geometria o shader olha `n_segs == 0`
    /// e cai no caminho da cobertura — o buffer fica lá, intocado.
    fn ensure_segments(&mut self, gpu: &GpuContext, geom: &[[f32; 4]]) {
        if geom.is_empty() {
            return;
        }
        let need = std::mem::size_of_val(geom) as u64;
        if need > self.segs_cap {
            self.segs_cap = need.next_power_of_two();
            self.segs = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ph2d-render fx_stack segments"),
                size: self.segs_cap,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        gpu.queue
            .write_buffer(&self.segs, 0, bytemuck::cast_slice(geom));
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

    fn ensure_globals(&mut self, gpu: &GpuContext, total_slots: usize) {
        let need = (total_slots as u64) * UNIFORM_STRIDE;
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

#[cfg(test)]
#[path = "fx_stack_tests.rs"]
mod tests;
