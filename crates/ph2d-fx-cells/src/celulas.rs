//! **As células na placa** — o `fx.filter` em CADA forma (doc 129 §3.4): as caixas no ecrã, o
//! plano, a relocação para o átlas, o desenho, a pilha UMA vez sobre o átlas e a volta à camada.

use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_render::{
    BandBlit, BandSource, Camera2d, FxOpGpu, FxStackPass, GpuTexRun, Origem, RenderInstance,
    SpriteRenderer, sort_render_order, stack_reach, stack_read,
};
use ph2d_shape_gpu::{Copias, ShapeInstance, ShapePass, ShapeView};

use crate::plano::{Caixa, Celula, Pagina, planeia};
use crate::shaders::{COMPOE_WGSL, RELOCAR_WGSL, limites_wgsl};

/// O formato da camada e das texturas do átlas (o da camada do Motion).
pub const FORMATO: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
const FORMATO_MAPA: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;

/// **O lado máximo de uma página do átlas.** Recurso: MEMÓRIA da placa — cada página paga sete
/// texturas do tamanho dela (quatro de trabalho da pilha, `Rgba16Float`; o alvo das sprites, o das
/// formas e a camada, `Rgba16Float`; a saída, `Rgba8Unorm`; o mapa da origem, `Rgba32Float`):
///
/// | lado | por textura `Rgba16Float` | as sete |
/// |---:|---:|---:|
/// | 1024 | 8 MB | ~76 MB |
/// | **2048** | **32 MB** | **~304 MB** |
/// | 4096 | 128 MB | ~1,2 GB |
///
/// Uma cena que não cabe numa página corre em várias (o relógio por página está no doc 129 §5.6).
pub const LADO_MAX: u32 = 2048;

/// As sprites do Motion: o buffer do cozimento na placa (com as faixas de textura) ou a fatia da CPU.
#[derive(Clone, Copy)]
pub enum Sprites<'a> {
    Placa {
        buffer: &'a wgpu::Buffer,
        n: u32,
        runs: &'a [GpuTexRun],
    },
    Cpu(&'a [RenderInstance]),
}

/// As formas da placa deste quadro: o buffer de [`ShapeInstance`] que a placa desenhou, quantas, e a
/// vista (o afim mundo → pixel da camada).
#[derive(Clone, Copy)]
pub struct Formas<'a> {
    pub buffer: &'a wgpu::Buffer,
    pub n: u32,
    pub vista: ShapeView,
}

/// O que uma corrida pede.
pub struct Entrada<'a> {
    pub gpu: &'a GpuContext,
    pub renderer: &'a mut SpriteRenderer,
    pub camera: &'a Camera2d,
    pub window: WindowSize,
    pub scene_viewport: Option<[f32; 4]>,
    pub sprites: Sprites<'a>,
    pub formas: Option<Formas<'a>>,
    /// Os degraus `Each Shape` já resolvidos em pixels.
    pub ops: &'a [FxOpGpu],
    pub pilha: &'a mut FxStackPass,
    /// A camada (`FORMATO`, pré-multiplicada no espaço do desenhista) — LIMPA aqui e cheia com as
    /// células na ordem de z.
    pub camada: &'a wgpu::TextureView,
    pub tamanho: (u32, u32),
}

/// **O afim mundo → pixel da camada das SPRITES** (`[a, b, c, d, e, f]`, o do kurbo) — porta única: a
/// projecção da câmera no sub-rect da cena, sem a matriz de modelo (o passe de sprites nunca a põe),
/// mais o canto do sub-rect.
#[must_use]
pub fn afim_das_sprites(
    camera: &Camera2d,
    window: WindowSize,
    scene_viewport: Option<[f32; 4]>,
) -> [f64; 6] {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "o split dá pixels inteiros e não negativos"
    )]
    let (canto, alvo) = scene_viewport.map_or(([0.0, 0.0], window), |v| {
        (
            [v[0], v[1]],
            WindowSize {
                width: v[2].max(1.0) as u32,
                height: v[3].max(1.0) as u32,
            },
        )
    });
    let mut cam = *camera;
    cam.model = Camera2d::MODEL_IDENTITY;
    let [a, b, c, d, e, f] = cam.world_to_screen_affine(alvo).as_coeffs();
    [a, b, c, d, f64::from(canto[0]) + e, f64::from(canto[1]) + f]
}

/// `L⁻¹ · t`: a translação no MUNDO que move a projecção `t` pixels.
fn no_mundo(lin: [f64; 4], t: [i32; 2]) -> [f32; 2] {
    let [a, b, c, d] = lin;
    let det = a * d - b * c;
    let (x, y) = (f64::from(t[0]), f64::from(t[1]));
    #[expect(clippy::cast_possible_truncation, reason = "o mundo lê f32")]
    [((d * x - c * y) / det) as f32, ((a * y - b * x) / det) as f32]
}

/// A câmera que projecta no ÁTLAS (`alvo`) com a MESMA escala e translação que a camada.
fn camera_do_atlas(camera: &Camera2d, afim: [f64; 6], alvo: (u32, u32)) -> Camera2d {
    let k = afim[0];
    let (w, h) = (f64::from(alvo.0), f64::from(alvo.1));
    let mut c = *camera;
    c.model = Camera2d::MODEL_IDENTITY;
    #[expect(clippy::cast_possible_truncation, reason = "a câmera lê f32")]
    {
        c.height_world = (h / k) as f32;
        c.center = [((w * 0.5 - afim[4]) / k) as f32, ((afim[5] - h * 0.5) / k) as f32];
    }
    c
}

struct Atlas {
    tamanho: (u32, u32),
    linear: wgpu::TextureView,
    formas: wgpu::TextureView,
    camada: wgpu::Texture,
    camada_view: wgpu::TextureView,
    saida: wgpu::Texture,
    saida_view: wgpu::TextureView,
    mapa: wgpu::TextureView,
}

impl Atlas {
    fn new(gpu: &GpuContext, (w, h): (u32, u32)) -> Self {
        let tex = |rotulo, format| {
            gpu.device.create_texture(&wgpu::TextureDescriptor {
                label: Some(rotulo),
                size: wgpu::Extent3d {
                    width: w.max(1),
                    height: h.max(1),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            })
        };
        let vista = |t: &wgpu::Texture| t.create_view(&wgpu::TextureViewDescriptor::default());
        let camada = tex("fx_cells camada", FORMATO);
        let saida = ph2d_render::make_output_texture(gpu, w, h);
        Self {
            tamanho: (w, h),
            linear: vista(&tex("fx_cells linear", FORMATO)),
            formas: vista(&tex("fx_cells formas", FORMATO)),
            camada_view: vista(&camada),
            camada,
            saida_view: vista(&saida),
            saida,
            mapa: vista(&tex("fx_cells mapa", FORMATO_MAPA)),
        }
    }
}

/// Os passes e o átlas (que só cresce).
pub struct Celulas {
    limites: wgpu::ComputePipeline,
    relocar: wgpu::ComputePipeline,
    volta: wgpu::RenderPipeline,
    mapa: wgpu::RenderPipeline,
    formas: ShapePass,
    colagem: BandBlit,
    atlas: Option<Atlas>,
    /// As caixas do último [`Self::corre`] (sprites ++ formas) — instrumento de gates.
    ultimas_caixas: Vec<Caixa>,
    /// As páginas do último [`Self::corre`] — instrumento de gates e do relógio.
    ultimas_paginas: Vec<Pagina>,
}

fn compute(gpu: &GpuContext, src: &str, rotulo: &str) -> wgpu::ComputePipeline {
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

fn render(
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

fn buffer(gpu: &GpuContext, rotulo: &str, bytes: &[u8], usage: wgpu::BufferUsages) -> wgpu::Buffer {
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

fn vazio(gpu: &GpuContext, rotulo: &str, bytes: u64, usage: wgpu::BufferUsages) -> wgpu::Buffer {
    gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(rotulo),
        size: bytes.max(16),
        usage,
        mapped_at_creation: false,
    })
}

fn liga<'a>(
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

fn limpa(enc: &mut wgpu::CommandEncoder, alvo: &wgpu::TextureView) {
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
struct CelGpu {
    saida: [i32; 4],
    t: [i32; 2],
    org: [f32; 2],
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
fn faixas_da_pagina(runs: &[GpuTexRun], idx: &[u32]) -> Vec<GpuTexRun> {
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

const PALAVRAS_SPRITE: u32 = (std::mem::size_of::<RenderInstance>() / 4) as u32;
const PALAVRAS_FORMA: u32 = (std::mem::size_of::<ShapeInstance>() / 4) as u32;
#[expect(clippy::cast_possible_truncation, reason = "um campo no início da struct")]
const POS_SPRITE: u32 = (std::mem::offset_of!(RenderInstance, world_pos) / 4) as u32;
#[expect(clippy::cast_possible_truncation, reason = "um campo no início da struct")]
const POS_FORMA: u32 = (std::mem::offset_of!(ShapeInstance, pos) / 4) as u32;

impl Celulas {
    /// `formas` = um passe de formas configurado como o da placa (a variante, o contorno).
    #[must_use]
    pub fn new(gpu: &GpuContext, formas: ShapePass) -> Self {
        let module = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("fx_cells compoe"),
                source: wgpu::ShaderSource::Wgsl(COMPOE_WGSL.into()),
            });
        Self {
            limites: compute(gpu, &limites_wgsl(), "fx_cells limites"),
            relocar: compute(gpu, RELOCAR_WGSL, "fx_cells relocar"),
            volta: render(
                gpu,
                &module,
                "vs_volta",
                "fs_volta",
                FORMATO,
                Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
            ),
            mapa: render(gpu, &module, "vs_mapa", "fs_mapa", FORMATO_MAPA, None),
            formas,
            colagem: BandBlit::new(gpu, FORMATO),
            atlas: None,
            ultimas_caixas: Vec::new(),
            ultimas_paginas: Vec::new(),
        }
    }

    /// O passe das formas (para a shell lhe carregar as geometrias do quadro).
    pub fn formas_mut(&mut self) -> &mut ShapePass {
        &mut self.formas
    }

    /// As caixas do último [`Self::corre`].
    #[must_use]
    pub fn ultimas_caixas(&self) -> &[Caixa] {
        &self.ultimas_caixas
    }

    /// As páginas do último [`Self::corre`].
    #[must_use]
    pub fn ultimas_paginas(&self) -> &[Pagina] {
        &self.ultimas_paginas
    }

    /// ⭐ **Cada instância filtrada sozinha, devolvida no seu lugar, na ordem de z**, sobre a
    /// `camada` (limpa aqui). Sprites primeiro, formas por cima — a ordem da camada inteira.
    pub fn corre(&mut self, mut e: Entrada<'_>) {
        let gpu = e.gpu;
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        limpa(&mut enc, e.camada);
        gpu.queue.submit([enc.finish()]);

        // A ordem de desenho da rota da CPU é a do passe de sprites — a mesma porta.
        let ordenadas: Vec<RenderInstance> = match e.sprites {
            Sprites::Cpu(v) => {
                let mut v = v.to_vec();
                sort_render_order(&mut v);
                v
            }
            Sprites::Placa { .. } => Vec::new(),
        };
        let cpu_buf;
        let (s_buf, n_s, runs): (&wgpu::Buffer, u32, &[GpuTexRun]) = match e.sprites {
            Sprites::Placa { buffer, n, runs } => (buffer, n, runs),
            Sprites::Cpu(_) => {
                cpu_buf = buffer(
                    gpu,
                    "fx_cells sprites da cpu",
                    bytemuck::cast_slice(&ordenadas),
                    wgpu::BufferUsages::STORAGE,
                );
                #[expect(clippy::cast_possible_truncation, reason = "instâncias cabem num u32")]
                (&cpu_buf, ordenadas.len() as u32, &[])
            }
        };
        let afim = afim_das_sprites(e.camera, e.window, e.scene_viewport);

        self.ultimas_caixas = self.caixas(gpu, s_buf, n_s, afim, e.formas);
        // A vala entre conteúdos é `2M + 1`: tem de cobrir o que a pilha ESPALHA para fora de uma
        // forma E o que ela LÊ em volta de cada pixel que escreve (plano.rs).
        let (ml, mt, mr, mb) = stack_reach(e.ops);
        let margem = ml.max(mt).max(mr).max(mb).max(stack_read(e.ops));
        let paginas = planeia(
            &self.ultimas_caixas,
            [e.tamanho.0, e.tamanho.1],
            margem,
            LADO_MAX,
        );
        let precisa = paginas.iter().fold((1u32, 1u32), |(w, h), p| {
            (w.max(p.tamanho[0]), h.max(p.tamanho[1]))
        });
        if self
            .atlas
            .as_ref()
            .is_none_or(|a| a.tamanho.0 < precisa.0 || a.tamanho.1 < precisa.1)
        {
            let (w, h) = self.atlas.as_ref().map_or(precisa, |a| {
                (a.tamanho.0.max(precisa.0), a.tamanho.1.max(precisa.1))
            });
            self.atlas = Some(Atlas::new(gpu, (w, h)));
        }
        let usa_mapa = e
            .ops
            .iter()
            .any(|o| o.kind == ph2d_fx_op::FxOp::TURBULENCE);
        for pagina in &paginas {
            self.pagina(&mut e, pagina, (s_buf, n_s, runs), &ordenadas, afim, usa_mapa);
        }
        self.ultimas_paginas = paginas;
    }

    /// As caixas no ecrã, sprites ++ formas — dois passes e UMA leitura (à espera da própria
    /// submissão).
    fn caixas(
        &self,
        gpu: &GpuContext,
        s_buf: &wgpu::Buffer,
        n_s: u32,
        afim: [f64; 6],
        formas: Option<Formas<'_>>,
    ) -> Vec<Caixa> {
        let n_f = formas.map_or(0, |f| f.n);
        let n = u64::from(n_s + n_f);
        if n == 0 {
            return Vec::new();
        }
        let tam = std::mem::size_of::<Caixa>() as u64;
        let uso = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC;
        let cs = vazio(gpu, "fx_cells caixas sprites", u64::from(n_s) * tam, uso);
        let cf = vazio(gpu, "fx_cells caixas formas", u64::from(n_f) * tam, uso);
        let leitura = vazio(
            gpu,
            "fx_cells caixas leitura",
            n * tam,
            wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        );
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        if n_s > 0 {
            #[expect(clippy::cast_possible_truncation, reason = "a placa lê f32")]
            let p: [f32; 8] = [
                afim[0] as f32,
                afim[1] as f32,
                afim[2] as f32,
                afim[3] as f32,
                afim[4] as f32,
                afim[5] as f32,
                f32::from_bits(n_s),
                0.0,
            ];
            let u = buffer(gpu, "fx_cells limites p", bytemuck::cast_slice(&p), wgpu::BufferUsages::UNIFORM);
            let bg = liga(
                gpu,
                &self.limites.get_bind_group_layout(0),
                &[u.as_entire_binding(), s_buf.as_entire_binding(), cs.as_entire_binding()],
            );
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&self.limites);
            pass.set_bind_group(0, &bg, &[]);
            pass.dispatch_workgroups(n_s.div_ceil(64), 1, 1);
        }
        if let Some(f) = formas.filter(|f| f.n > 0) {
            self.formas.caixas(
                gpu,
                &mut enc,
                f.vista,
                Copias {
                    buffer: f.buffer,
                    count: f.n,
                },
                &cf,
            );
        }
        enc.copy_buffer_to_buffer(&cs, 0, &leitura, 0, u64::from(n_s) * tam);
        enc.copy_buffer_to_buffer(&cf, 0, &leitura, u64::from(n_s) * tam, u64::from(n_f) * tam);
        let sub = gpu.queue.submit([enc.finish()]);
        let fatia = leitura.slice(..n * tam);
        fatia.map_async(wgpu::MapMode::Read, |_| {});
        let _ = gpu.device.poll(wgpu::PollType::Wait {
            submission_index: Some(sub),
            timeout: None,
        });
        let caixas = bytemuck::cast_slice::<u8, Caixa>(&fatia.get_mapped_range()).to_vec();
        leitura.unmap();
        caixas
    }

    #[expect(clippy::too_many_arguments, reason = "o estado de uma página")]
    fn pagina(
        &mut self,
        e: &mut Entrada<'_>,
        pagina: &Pagina,
        (s_buf, n_s, runs): (&wgpu::Buffer, u32, &[GpuTexRun]),
        ordenadas: &[RenderInstance],
        afim: [f64; 6],
        usa_mapa: bool,
    ) {
        let gpu = e.gpu;
        let lin_s = [afim[0], afim[1], afim[2], afim[3]];
        let (mut idx_s, mut d_s, mut idx_f, mut d_f) =
            (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for c in &pagina.celulas {
            if c.instancia < n_s {
                idx_s.push(c.instancia);
                d_s.push(no_mundo(lin_s, c.t));
            } else if let Some(f) = e.formas {
                idx_f.push(c.instancia - n_s);
                d_f.push(no_mundo(f.vista.lin.map(f64::from), c.t));
            }
        }
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        // As sprites da placa relocam-se por compute; as da CPU pela mesma soma, aqui.
        let reloc_s = (!idx_s.is_empty() && ordenadas.is_empty()).then(|| {
            reloca(
                &self.relocar,
                gpu,
                &mut enc,
                s_buf,
                &idx_s,
                &d_s,
                (PALAVRAS_SPRITE, POS_SPRITE),
                wgpu::BufferUsages::VERTEX,
            )
        });
        let cpu_s: Vec<RenderInstance> = if ordenadas.is_empty() {
            Vec::new()
        } else {
            idx_s
                .iter()
                .zip(&d_s)
                .map(|(&i, d)| {
                    let mut r = ordenadas[i as usize];
                    r.world_pos = [r.world_pos[0] + d[0], r.world_pos[1] + d[1]];
                    r
                })
                .collect()
        };
        let reloc_f = match e.formas {
            Some(f) if !idx_f.is_empty() => Some(reloca(
                &self.relocar,
                gpu,
                &mut enc,
                f.buffer,
                &idx_f,
                &d_f,
                (PALAVRAS_FORMA, POS_FORMA),
                wgpu::BufferUsages::empty(),
            )),
            _ => None,
        };
        gpu.queue.submit([enc.finish()]);

        let atlas = self.atlas.as_ref().expect("garantido pelo corre");
        let alvo = atlas.tamanho;
        let cam = camera_do_atlas(e.camera, afim, alvo);
        let faixas = faixas_da_pagina(runs, &idx_s);
        #[expect(clippy::cast_possible_truncation, reason = "uma página cabe num u32")]
        let extra = reloc_s
            .as_ref()
            .map(|b| (b, idx_s.len() as u32, faixas.as_slice()));
        e.renderer.render_instances_only(
            &atlas.linear,
            &cam,
            WindowSize {
                width: alvo.0,
                height: alvo.1,
            },
            wgpu::Color::TRANSPARENT,
            &cpu_s,
            extra,
            None,
        );

        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        #[expect(clippy::cast_precision_loss, reason = "um átlas cabe num f32")]
        let vista = e.formas.map(|f| ShapeView {
            alvo: [alvo.0 as f32, alvo.1 as f32],
            ..f.vista
        });
        match (reloc_f.as_ref(), vista) {
            (Some(b), Some(v)) => {
                #[expect(clippy::cast_possible_truncation, reason = "uma página cabe num u32")]
                let count = idx_f.len() as u32;
                self.formas.draw(
                    gpu,
                    &mut enc,
                    &atlas.formas,
                    wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    v,
                    Copias { buffer: b, count },
                );
            }
            _ => limpa(&mut enc, &atlas.formas),
        }
        limpa(&mut enc, &atlas.camada_view);
        gpu.queue.submit([enc.finish()]);
        self.colagem
            .blit(gpu, &atlas.camada_view, &atlas.linear, BandSource::Sprites);
        self.colagem
            .blit(gpu, &atlas.camada_view, &atlas.formas, BandSource::Formas);

        let cels: Vec<CelGpu> = pagina.celulas.iter().map(CelGpu::from).collect();
        let cels = buffer(
            gpu,
            "fx_cells celulas",
            bytemuck::cast_slice(&cels),
            wgpu::BufferUsages::STORAGE,
        );
        #[expect(clippy::cast_possible_truncation, reason = "uma página cabe num u32")]
        let n_cel = pagina.celulas.len() as u32;
        if usa_mapa {
            quads(gpu, &self.mapa, &atlas.mapa, &cels, n_cel, &atlas.saida_view, alvo, true);
        }
        let origem = if usa_mapa {
            Origem::Mapa(&atlas.mapa)
        } else {
            Origem::Margem
        };
        e.pilha.run_layer(
            gpu,
            &atlas.camada,
            &atlas.saida,
            pagina.tamanho[0],
            pagina.tamanho[1],
            e.ops,
            origem,
        );
        quads(gpu, &self.volta, e.camada, &cels, n_cel, &atlas.saida_view, e.tamanho, false);
    }
}

/// A instância `idx[j]` de `origem` no lugar `j`, transladada de `delta[j]` no mundo.
#[expect(clippy::too_many_arguments, reason = "um despacho")]
fn reloca(
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
    let i = buffer(gpu, "fx_cells idx", bytemuck::cast_slice(idx), wgpu::BufferUsages::STORAGE);
    let d = buffer(gpu, "fx_cells delta", bytemuck::cast_slice(delta), wgpu::BufferUsages::STORAGE);
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
fn quads(
    gpu: &GpuContext,
    pipeline: &wgpu::RenderPipeline,
    alvo: &wgpu::TextureView,
    cels: &wgpu::Buffer,
    n: u32,
    atlas: &wgpu::TextureView,
    tamanho: (u32, u32),
    limpa_antes: bool,
) {
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
    let bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("fx_cells quads"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &entries,
    });
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

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ **A relocação na placa É a soma da CPU**: a instância `idx` copiada palavra a palavra, só
    /// a posição somada do `delta` — lida de volta e comparada campo a campo.
    #[test]
    #[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
    fn the_device_relocation_is_the_cpu_sum() {
        let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
            return;
        };
        let r = RenderInstance {
            world_pos: [0.37, -0.45],
            size: [1.1, 0.6],
            atlas_uv: [0.1, 0.2, 0.3, 0.4],
            tint: [0.3, 0.6, 0.9, 1.0],
            basis: [0.8, 0.5, -0.5, 0.8],
            premultiplied: 0.0,
            anchor: [0.1, 0.0],
            per_corner_tint: [[1.0; 4]; 4],
            opacity: 0.5,
            flip_uv: 7,
            texture_id: 3,
            z_order: 9,
            sampling: RenderInstance::SAMPLING_DEFAULT,
            uv_xform: RenderInstance::IDENTITY_UV_XFORM,
            clip_group: RenderInstance::CLIP_GROUP_NONE,
            clip_meta: 0,
            sub_order: 4,
        };
        let mut outra = r;
        outra.world_pos = [5.0, 6.0];
        let pipeline = compute(&gpu, RELOCAR_WGSL, "fx_cells relocar (gate)");
        let origem = buffer(&gpu, "origem", bytemuck::cast_slice(&[outra, r]), wgpu::BufferUsages::STORAGE);
        let mut enc = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let delta = [-8.7f32, 6.25];
        let tam = std::mem::size_of::<RenderInstance>() as u64;
        let d = reloca(
            &pipeline,
            &gpu,
            &mut enc,
            &origem,
            &[1],
            &[delta],
            (PALAVRAS_SPRITE, POS_SPRITE),
            wgpu::BufferUsages::COPY_SRC,
        );
        let l = vazio(&gpu, "leitura", tam, wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ);
        enc.copy_buffer_to_buffer(&d, 0, &l, 0, tam);
        gpu.queue.submit([enc.finish()]);
        l.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        #[expect(clippy::cast_possible_truncation, reason = "188 B")]
        let lida: RenderInstance = *bytemuck::from_bytes(&l.slice(..).get_mapped_range()[..tam as usize]);
        let mut esperada = r;
        esperada.world_pos = [r.world_pos[0] + delta[0], r.world_pos[1] + delta[1]];
        assert_eq!(bytemuck::bytes_of(&lida), bytemuck::bytes_of(&esperada));
    }
}
