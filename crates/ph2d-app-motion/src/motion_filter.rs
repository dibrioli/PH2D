//! **OS FILTROS DA CAMADA do Motion no quadro** (ciclo 20 W3, [doc 129](../../../docs/Motion%20Nodes/129_ciclo_20_o_acabamento.md)
//! §3.3) — os `fx.filter` do fio, a camada do Motion desenhada À PARTE, e a pilha raster do Vector
//! (`ph2d_render::FxStackPass`) sobre ela.
//!
//! Três tempos, três portas (a forma da [`crate::motion_lens`]):
//! 1. [`MotionFiltro::prepara`] — no passo do Motion: a pilha, na ordem do fio;
//! 2. [`MotionFiltro::liga`] — a shell, ao planear o quadro: com pilha, o Motion SAI do passe fundido
//!    ([`MotionState::sprites_da_camada`]) e o quadro vai em faixas (o acumulador do mundo);
//! 3. [`MotionState::filtra_a_camada`] — a shell, no lugar da colagem da placa e antes da lente:
//!    as sprites do Motion num alvo linear, coladas com a placa das formas numa camada no espaço do
//!    DESENHISTA (o do acumulador), a pilha sobre ela e o resultado colado no mundo.
//!
//! ⚠️ **A camada filtrada vai POR CIMA do mundo** (a camada do After Effects): fora do passe fundido
//! o Motion deixa de se intercalar por z com as sprites da cena. Onion, partículas e barras ficam lá.
//! ⚠️ **Sem tonemap:** o passe fundido o atravessa, e ele está em BYPASS (`clamp`, gate
//! `the_layer_skips_a_tonemap_that_is_a_bypass`) — se o AgX ligar, esta camada tem de o seguir.
//! ⚠️ Formas da rota da CPU (o Vello, `formas_pedem_o_vello`) não estão na placa nem na camada.

use crate::motion_state::MotionState;
use ph2d_node_fx_filter::NoDoFiltro;
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_render::{BandBlit, BandSource, FxOpGpu, FxStackPass, RenderInstance};

/// As sprites do Motion no DISPOSITIVO: o buffer do cozimento, quantas, e os runs de textura.
pub type SpritesDaPlaca<'a> = (&'a wgpu::Buffer, u32, &'a [ph2d_render::GpuTexRun]);

/// O que a camada filtrada pede da shell.
pub struct Engrenagem<'a> {
    pub gpu: &'a ph2d_gpu::GpuContext,
    pub renderer: &'a mut ph2d_render::SpriteRenderer,
    pub camera: &'a ph2d_render::Camera2d,
    pub window: ph2d_host::WindowSize,
    pub scene_viewport: Option<[f32; 4]>,
    /// A camada da placa das formas deste quadro (`PlacaDeFormas::camada`).
    pub formas: Option<&'a wgpu::TextureView>,
    pub mundo: &'a ph2d_render::WorldRt,
    /// A colagem do mundo (alvo `WorldRt::FORMAT`).
    pub band_blit: &'a BandBlit,
}

struct Alvos {
    tamanho: (u32, u32),
    /// As sprites, em luz LINEAR pré-multiplicada (o `game_rt` do passe fundido).
    linear: wgpu::TextureView,
    /// A camada no espaço do DESENHISTA, pré-multiplicada (sprites + formas).
    camada: wgpu::Texture,
    camada_view: wgpu::TextureView,
    /// A saída da pilha: sRGB reto, `Rgba8Unorm`.
    saida: wgpu::Texture,
    saida_view: wgpu::TextureView,
}

struct Placa {
    pilha: FxStackPass,
    /// A colagem sobre a CAMADA (meio-float).
    colagem: BandBlit,
    alvos: Option<Alvos>,
}

/// O estado dos filtros entre quadros.
#[derive(Default)]
pub struct MotionFiltro {
    pilha: Vec<NoDoFiltro>,
    fora: Vec<NodeId>,
    desenha: bool,
    ligada: bool,
    placa: Option<Placa>,
}

const FORMATO_DA_CAMADA: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

impl MotionFiltro {
    /// **Por quadro, no passo do Motion**: a pilha do fio e os filtros fora dele.
    pub fn prepara(&mut self, graph: &Graph) {
        self.pilha = ph2d_node_fx_filter::from_graph(graph);
        self.fora = ph2d_node_fx_filter::fora_do_fio(graph, &self.pilha);
    }

    /// A pilha do quadro, na ordem do fio.
    #[must_use]
    pub fn pilha(&self) -> &[NoDoFiltro] {
        &self.pilha
    }

    /// **A shell, ao planear o quadro**: os filtros correm? `desenha` = o Motion está na cena.
    /// Sem `fx.filter` no fio ⇒ `false` e o quadro é o de sempre, byte a byte.
    pub fn liga(&mut self, desenha: bool) -> bool {
        self.desenha = desenha;
        self.ligada = desenha && !self.pilha.is_empty();
        self.ligada
    }

    /// Os filtros correm neste quadro (a última resposta do [`Self::liga`]).
    #[must_use]
    pub fn ligada(&self) -> bool {
        self.ligada
    }

    /// **O que o cartão do nó `no` diz**: um `fx.filter` fora do fio não filtra nada.
    #[must_use]
    pub fn queixa(&self, no: NodeId) -> Option<String> {
        self.fora
            .contains(&no)
            .then(|| ph2d_i18n::tr("app.motion.motion_filter.off_the_wire").to_string())
    }

    fn placa(&mut self, gpu: &ph2d_gpu::GpuContext, tamanho: (u32, u32)) -> &mut Placa {
        let placa = self.placa.get_or_insert_with(|| Placa {
            pilha: FxStackPass::new(gpu),
            colagem: BandBlit::new(gpu, FORMATO_DA_CAMADA),
            alvos: None,
        });
        if placa.alvos.as_ref().is_none_or(|a| a.tamanho != tamanho) {
            placa.alvos = Some(Alvos::new(gpu, tamanho));
        }
        placa
    }
}

impl Alvos {
    fn new(gpu: &ph2d_gpu::GpuContext, (w, h): (u32, u32)) -> Self {
        let alvo = |rotulo| {
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
                format: FORMATO_DA_CAMADA,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
        };
        let vista = |t: &wgpu::Texture| t.create_view(&wgpu::TextureViewDescriptor::default());
        let linear = vista(&alvo("motion_filtro_linear"));
        let camada = alvo("motion_filtro_camada");
        let saida = ph2d_render::make_output_texture(gpu, w, h);
        Self {
            tamanho: (w, h),
            linear,
            camada_view: vista(&camada),
            camada,
            saida_view: vista(&saida),
            saida,
        }
    }
}

/// Limpa `alvo` a transparente.
fn limpa(gpu: &ph2d_gpu::GpuContext, alvo: &wgpu::TextureView) {
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("motion_filtro_limpa"),
        });
    drop(enc.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("motion_filtro_limpa"),
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
    gpu.queue.submit(Some(enc.finish()));
}

/// **A parte linear de mundo → pixel da cena** — a projecção da câmera (`world_to_screen_affine`,
/// porta única) no tamanho em que o passe de sprites a usa: o sub-rect da cena, ou a janela.
/// Sem a matriz de modelo: o passe de sprites nunca a põe.
#[must_use]
pub fn mundo_para_pixel(
    camera: &ph2d_render::Camera2d,
    window: ph2d_host::WindowSize,
    scene_viewport: Option<[f32; 4]>,
) -> [f64; 4] {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "o split dá pixels inteiros e não negativos"
    )]
    let alvo = scene_viewport.map_or(window, |v| ph2d_host::WindowSize {
        width: v[2].max(1.0) as u32,
        height: v[3].max(1.0) as u32,
    });
    let mut cam = *camera;
    cam.model = ph2d_render::Camera2d::MODEL_IDENTITY;
    let [a, b, c, d, _, _] = cam.world_to_screen_affine(alvo).as_coeffs();
    [a, b, c, d]
}

impl MotionState {
    /// **As sprites do Motion neste quadro** — a MESMA pergunta para o passe fundido e para a camada
    /// filtrada: as do dispositivo (o cozimento da placa) ou a fatia da CPU. `ativo = false` ⇒ nada.
    #[must_use]
    pub fn sprites_da_camada(
        &self,
        ativo: bool,
    ) -> (Option<SpritesDaPlaca<'_>>, &[RenderInstance]) {
        if !ativo {
            return (None, &[]);
        }
        let arte = crate::lei_da_aparencia::a_arte_desenha(self, self.pump.a_lei());
        sprites(self, arte)
    }

    /// **A camada filtrada, colada no mundo** — no lugar da colagem da placa. `false` = os filtros
    /// não correm neste quadro, e a shell cola a placa como sempre.
    pub fn filtra_a_camada(&mut self, e: Engrenagem<'_>) -> bool {
        if !self.filtro.ligada {
            return false;
        }
        let arte = crate::lei_da_aparencia::a_arte_desenha(self, self.pump.a_lei());
        let linear = mundo_para_pixel(e.camera, e.window, e.scene_viewport);
        let ops: Vec<FxOpGpu> = self
            .filtro
            .pilha
            .iter()
            .filter(|d| d.op.is_active())
            .map(|d| FxOpGpu::resolve(&d.op, linear))
            .collect();
        let tamanho = e.mundo.size();
        let (gpu_sprites, cpu) =
            sprites_de(&self.gpu_cook, &self.pump.instances, self.gpu_live, arte);
        let placa = self.filtro.placa(e.gpu, tamanho);
        let Some(alvos) = placa.alvos.as_ref() else {
            return false;
        };
        limpa(e.gpu, &alvos.camada_view);
        e.renderer.render_instances_only(
            &alvos.linear,
            e.camera,
            e.window,
            wgpu::Color::TRANSPARENT,
            cpu,
            gpu_sprites,
            e.scene_viewport,
        );
        placa.colagem.blit(
            e.gpu,
            &alvos.camada_view,
            &alvos.linear,
            BandSource::Sprites,
        );
        if let Some(formas) = e.formas {
            placa
                .colagem
                .blit(e.gpu, &alvos.camada_view, formas, BandSource::Formas);
        }
        let (w, h) = tamanho;
        placa
            .pilha
            .run_premultiplied(e.gpu, &alvos.camada, &alvos.saida, w, h, &ops);
        e.band_blit.blit(
            e.gpu,
            e.mundo.blend_view(),
            &alvos.saida_view,
            BandSource::Vector,
        );
        true
    }
}

fn sprites(m: &MotionState, arte: bool) -> (Option<SpritesDaPlaca<'_>>, &[RenderInstance]) {
    sprites_de(&m.gpu_cook, &m.pump.instances, m.gpu_live, arte)
}

/// Por CAMPO, para o [`MotionState::filtra_a_camada`] emprestar o `filtro` mutável ao lado.
fn sprites_de<'a>(
    cook: &'a ph2d_gpu_cook::GpuCook,
    cpu: &'a [RenderInstance],
    gpu_live: bool,
    arte: bool,
) -> (Option<SpritesDaPlaca<'a>>, &'a [RenderInstance]) {
    let gpu = (arte && gpu_live)
        .then(|| cook.instances())
        .flatten()
        .map(|gi| (gi.buffer(), gi.len(), cook.texture_runs()));
    let cpu = if gpu.is_none() { cpu } else { &[] };
    (gpu, cpu)
}

#[cfg(test)]
#[path = "motion_filter_tests.rs"]
mod tests;
