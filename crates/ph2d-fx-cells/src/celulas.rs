//! **As células na placa** — o `fx.filter` em CADA forma (doc 129 §3.4): as caixas no ecrã, o
//! plano, a relocação para o átlas, o desenho, a pilha UMA vez sobre o átlas e a volta à camada.

use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_render::{
    BandBlit, BandSource, Camera2d, FxOpGpu, FxStackPass, GpuTexRun, Origem, RenderInstance,
    SpriteRenderer, sort_render_order, stack_reach, stack_read,
};
use ph2d_shape_gpu::{Copias, Corrida, Mistura, ShapeInstance, ShapePass, ShapeView};

use crate::passes::{
    CelGpu, abre, buffer, compute, faixas_da_pagina, fecha, limpa, quads, reloca, render,
};
use crate::plano::{Caixa, Pagina, Vala, planeia_com_par};
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
/// Uma cena que não cabe numa página corre em várias, EM SÉRIE no mesmo átlas: a pilha custa pela
/// ÁREA das páginas (`0,244 ms/Mpx` a 1, 2, 4 e 8 páginas — doc 131 §7 C.10), logo uma página maior
/// ou um `texture_2d_array` com um despacho por fase pagariam memória e não ganhariam nada.
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
    /// As corridas de mistura das cópias (B5, [`crate::volta`]) — vazias, ou uma só `Normal` sem
    /// grupo, é a volta de sempre em `Normal`.
    pub corridas: &'a [Corrida],
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
    [
        ((d * x - c * y) / det) as f32,
        ((a * y - b * x) / det) as f32,
    ]
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
        c.center = [
            ((w * 0.5 - afim[4]) / k) as f32,
            ((afim[5] - h * 0.5) / k) as f32,
        ];
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
    /// B5: a volta com mistura — nasce no primeiro quadro que a pede.
    voltas: Option<crate::volta::Voltas>,
    variante: Variante,
}

/// As variantes da rodada C.10 (doc 131 §7), escolhíveis em EXECUÇÃO — a régua intercalada e os
/// gates de paridade correm as duas no mesmo processo. O `Default` é o do produto.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Variante {
    /// A vala `A + L + 1` ([`Vala`]); `false` = a `2·max(A, L) + 1` do W3b.
    pub vala_justa: bool,
    /// Uma página sem sprites (ou sem formas) não desenha nem cola o alvo vazio.
    pub salta_vazias: bool,
}

/// ⭐ As duas ligadas: o mesmo pixel (`the_c10_variants_give_the_same_pixels`) e, a 4 096 formas,
/// `5,14 → 2,22 ms` de placa (doc 131 §7 C.10).
impl Default for Variante {
    fn default() -> Self {
        Self {
            vala_justa: true,
            salta_vazias: true,
        }
    }
}

const PALAVRAS_SPRITE: u32 = (std::mem::size_of::<RenderInstance>() / 4) as u32;
const PALAVRAS_FORMA: u32 = (std::mem::size_of::<ShapeInstance>() / 4) as u32;
#[expect(
    clippy::cast_possible_truncation,
    reason = "um campo no início da struct"
)]
const POS_SPRITE: u32 = (std::mem::offset_of!(RenderInstance, world_pos) / 4) as u32;
#[expect(
    clippy::cast_possible_truncation,
    reason = "um campo no início da struct"
)]
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
            voltas: None,
            variante: Variante::default(),
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

    /// A variante das próximas corridas (régua e gates da rodada C.10).
    #[doc(hidden)]
    pub fn escolhe(&mut self, v: Variante) {
        self.variante = v;
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

        let relogio = abre(gpu, "fx_cells.caixas");
        self.ultimas_caixas = self.caixas(gpu, s_buf, n_s, afim, e.formas);
        fecha(gpu, relogio);
        // A vala cobre o que a pilha ESPALHA para fora de uma forma E o que ela LÊ em volta de cada
        // pixel que escreve ([`Vala`]).
        let (l, t, r, b) = stack_reach(e.ops);
        let vala = Vala {
            alcance: [l, t, r, b],
            leitura: stack_read(e.ops),
        };
        let vala = if self.variante.vala_justa {
            vala
        } else {
            vala.dupla()
        };
        let paginas = planeia_com_par(
            &self.ultimas_caixas,
            [e.tamanho.0, e.tamanho.1],
            vala,
            LADO_MAX,
            n_s,
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
        let usa_mapa = e.ops.iter().any(|o| o.kind == ph2d_fx_op::FxOp::TURBULENCE);
        let corridas = e.formas.as_ref().map_or(&[][..], |f| f.corridas);
        let simples = match corridas {
            [] => true,
            [c] => c.mistura == Mistura::Normal && c.grupo == 0,
            _ => false,
        };
        // B5: com mistura, as formas voltam à camada PRÓPRIA delas, colada no fim ([`crate::volta`]).
        let mut voltas = (!simples).then(|| {
            let mut v = self
                .voltas
                .take()
                .unwrap_or_else(|| crate::volta::Voltas::new(gpu, e.tamanho));
            v.garante(gpu, e.tamanho);
            v
        });
        let mut mistas = voltas.as_mut().map(|v| {
            let sessao = v.compositor.sessao(
                gpu,
                &v.formas.1,
                e.tamanho,
                wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            );
            (&v.pipes, sessao)
        });
        for pagina in &paginas {
            self.pagina(
                &mut e,
                pagina,
                (s_buf, n_s, runs),
                &ordenadas,
                afim,
                usa_mapa,
                mistas.as_mut().map(|(p, s)| (*p, s, corridas)),
            );
        }
        if let Some((_, sessao)) = mistas {
            let mut enc = gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            sessao.fecha(&mut enc);
            gpu.queue.submit([enc.finish()]);
        }
        if let Some(v) = voltas.take() {
            let mut enc = gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            v.compositor.cola(gpu, &mut enc, &v.formas.1, e.camada);
            gpu.queue.submit([enc.finish()]);
            self.voltas = Some(v);
        }
        self.ultimas_paginas = paginas;
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "a página de sempre mais a volta com mistura (B5)"
    )]
    fn pagina(
        &mut self,
        e: &mut Entrada<'_>,
        pagina: &Pagina,
        (s_buf, n_s, runs): (&wgpu::Buffer, u32, &[GpuTexRun]),
        ordenadas: &[RenderInstance],
        afim: [f64; 6],
        usa_mapa: bool,
        mistas: Option<(
            &crate::volta::Pipes,
            &mut ph2d_shape_gpu::Sessao<'_>,
            &[Corrida],
        )>,
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
        let relogio = abre(gpu, "fx_cells.reloca");
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
        fecha(gpu, relogio);

        let atlas = self.atlas.as_ref().expect("garantido pelo corre");
        let alvo = atlas.tamanho;
        let cam = camera_do_atlas(e.camera, afim, alvo);
        let faixas = faixas_da_pagina(runs, &idx_s);
        #[expect(clippy::cast_possible_truncation, reason = "uma página cabe num u32")]
        let extra = reloc_s
            .as_ref()
            .map(|b| (b, idx_s.len() as u32, faixas.as_slice()));
        // Um alvo VAZIO colado sobre a camada limpa é a identidade exacta (`0 + d·1`): saltá-lo
        // não muda um bit.
        let salta = self.variante.salta_vazias;
        let com_s = !(salta && idx_s.is_empty());
        let com_f = !(salta && reloc_f.is_none());
        let relogio = abre(gpu, "fx_cells.sprites");
        if com_s {
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
        }
        fecha(gpu, relogio);

        let relogio = abre(gpu, "fx_cells.formas");
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
            _ if com_f => limpa(&mut enc, &atlas.formas),
            _ => {}
        }
        limpa(&mut enc, &atlas.camada_view);
        gpu.queue.submit([enc.finish()]);
        fecha(gpu, relogio);
        let relogio = abre(gpu, "fx_cells.colagem");
        if com_s {
            self.colagem
                .blit(gpu, &atlas.camada_view, &atlas.linear, BandSource::Sprites);
        }
        if com_f {
            self.colagem
                .blit(gpu, &atlas.camada_view, &atlas.formas, BandSource::Formas);
        }
        fecha(gpu, relogio);

        let cels: Vec<CelGpu> = pagina.celulas.iter().map(CelGpu::from).collect();
        let cels = buffer(
            gpu,
            "fx_cells celulas",
            bytemuck::cast_slice(&cels),
            wgpu::BufferUsages::STORAGE,
        );
        #[expect(clippy::cast_possible_truncation, reason = "uma página cabe num u32")]
        let n_cel = pagina.celulas.len() as u32;
        let relogio = abre(gpu, "fx_cells.pilha");
        if usa_mapa {
            quads(
                gpu,
                &self.mapa,
                &atlas.mapa,
                &cels,
                n_cel,
                &atlas.saida_view,
                alvo,
                true,
            );
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
        fecha(gpu, relogio);
        let relogio = abre(gpu, "fx_cells.volta");
        let Some((pipes, sessao, corridas)) = mistas else {
            quads(
                gpu,
                &self.volta,
                e.camada,
                &cels,
                n_cel,
                &atlas.saida_view,
                e.tamanho,
                false,
            );
            fecha(gpu, relogio);
            return;
        };
        // As células vêm EM ORDEM (sprites primeiro): as das sprites voltam à camada, as das formas
        // à sessão.
        #[expect(clippy::cast_possible_truncation, reason = "uma página cabe num u32")]
        let k = pagina
            .celulas
            .iter()
            .take_while(|c| c.instancia < n_s)
            .count() as u32;
        quads(
            gpu,
            &self.volta,
            e.camada,
            &cels,
            k,
            &atlas.saida_view,
            e.tamanho,
            false,
        );
        pipes.volta(
            gpu,
            sessao,
            &pagina.celulas,
            k..n_cel,
            n_s,
            corridas,
            &cels,
            &atlas.saida_view,
            e.tamanho,
        );
        fecha(gpu, relogio);
    }
}

#[path = "celulas_caixas.rs"]
mod caixas;

#[cfg(test)]
#[path = "celulas_tests.rs"]
mod tests;
