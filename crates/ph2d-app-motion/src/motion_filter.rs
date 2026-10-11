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
use ph2d_render::{BandBlit, BandSource, FxOpGpu, FxStackPass, Origem, RenderInstance};

/// As sprites do Motion no DISPOSITIVO: o buffer do cozimento, quantas, e os runs de textura.
pub type SpritesDaPlaca<'a> = (&'a wgpu::Buffer, u32, &'a [ph2d_render::GpuTexRun]);

/// O que a camada filtrada pede da shell.
pub struct Engrenagem<'a> {
    pub gpu: &'a ph2d_gpu::GpuContext,
    pub renderer: &'a mut ph2d_render::SpriteRenderer,
    pub camera: &'a ph2d_render::Camera2d,
    pub window: ph2d_host::WindowSize,
    pub scene_viewport: Option<[f32; 4]>,
    /// A placa das formas deste quadro: a camada dela (a camada inteira) e as cópias que desenhou
    /// (cada forma, doc 129 §3.4).
    pub formas: Option<&'a crate::motion_shape_placa::PlacaDeFormas>,
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
    /// A saída da pilha: sRGB reto, `Rgba8Unorm`, do tamanho do RETÂNGULO em que ela corre
    /// ([`ph2d_render::retangulo_da_camada`]).
    saida: wgpu::Texture,
    saida_view: wgpu::TextureView,
}

struct Placa {
    pilha: FxStackPass,
    /// A colagem sobre a CAMADA (meio-float).
    colagem: BandBlit,
    alvos: Option<Alvos>,
    /// As células dos degraus `Each Shape` — nascem no primeiro quadro que as pede.
    celulas: Option<ph2d_fx_cells::Celulas>,
}

/// O estado dos filtros entre quadros.
#[derive(Default)]
pub struct MotionFiltro {
    pilha: Vec<NoDoFiltro>,
    fora: Vec<NodeId>,
    /// Os `Each Shape` depois de um `Whole Layer` (correm na camada).
    na_camada: Vec<NodeId>,
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
        self.na_camada = ph2d_node_fx_filter::na_camada_apesar_do_scope(&self.pilha);
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

    /// **O que o cartão do nó `no` diz**: um `fx.filter` fora do fio não filtra nada; um `Each Shape`
    /// depois de um `Whole Layer` corre na camada.
    #[must_use]
    pub fn queixa(&self, no: NodeId) -> Option<String> {
        let chave = if self.fora.contains(&no) {
            "app.motion.motion_filter.off_the_wire"
        } else if self.na_camada.contains(&no) {
            "app.motion.motion_filter.after_whole_layer"
        } else {
            return None;
        };
        Some(ph2d_i18n::tr(chave).to_string())
    }

    fn placa(&mut self, gpu: &ph2d_gpu::GpuContext, tamanho: (u32, u32)) -> &mut Placa {
        let placa = self.placa.get_or_insert_with(|| Placa {
            pilha: FxStackPass::new(gpu),
            colagem: BandBlit::new(gpu, FORMATO_DA_CAMADA),
            alvos: None,
            celulas: None,
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
        let saida = ph2d_render::make_output_texture(gpu, w.max(1), h.max(1));
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

impl Alvos {
    /// A saída da pilha com `w × h` (o retângulo da camada muda com os painéis, raramente).
    fn saida_para(&mut self, gpu: &ph2d_gpu::GpuContext, (w, h): (u32, u32)) {
        if (self.saida.width(), self.saida.height()) != (w, h) {
            self.saida = ph2d_render::make_output_texture(gpu, w, h);
            self.saida_view = self
                .saida
                .create_view(&wgpu::TextureViewDescriptor::default());
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

/// **A parte linear de mundo → pixel da cena** — a projecção da câmera no sub-rect da cena, sem a
/// matriz de modelo (porta única: [`ph2d_fx_cells::afim_das_sprites`], a mesma das células).
#[must_use]
pub fn mundo_para_pixel(
    camera: &ph2d_render::Camera2d,
    window: ph2d_host::WindowSize,
    scene_viewport: Option<[f32; 4]>,
) -> [f64; 4] {
    let [a, b, c, d, _, _] = ph2d_fx_cells::afim_das_sprites(camera, window, scene_viewport);
    [a, b, c, d]
}

/// **O pixel da ORIGEM DO MUNDO na camada** — a âncora do ruído da camada inteira (doc 129 §5.5
/// A6): o padrão anda com o mundo (um pan da câmera leva-o), nunca fica preso à janela.
#[must_use]
pub fn origem_do_mundo(
    camera: &ph2d_render::Camera2d,
    window: ph2d_host::WindowSize,
    scene_viewport: Option<[f32; 4]>,
) -> [f32; 2] {
    let [_, _, _, _, e, f] = ph2d_fx_cells::afim_das_sprites(camera, window, scene_viewport);
    #[expect(
        clippy::cast_possible_truncation,
        reason = "pixels de ecrã cabem num f32"
    )]
    [e as f32, f as f32]
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
    ///
    /// A pilha parte-se pela ordem do fio ([`ph2d_node_fx_filter::partition`]): os degraus `Each
    /// Shape` correm nas CÉLULAS (cada instância sozinha, devolvida no seu lugar), e a camada que
    /// elas devolvem recebe os `Whole Layer` (vazios = só a conversão para o mundo).
    pub fn filtra_a_camada(&mut self, e: Engrenagem<'_>) -> bool {
        if !self.filtro.ligada {
            return false;
        }
        let arte = crate::lei_da_aparencia::a_arte_desenha(self, self.pump.a_lei());
        let linear = mundo_para_pixel(e.camera, e.window, e.scene_viewport);
        let resolve = |pilha: &[NoDoFiltro]| -> Vec<FxOpGpu> {
            pilha
                .iter()
                .filter(|d| d.op.is_active())
                .map(|d| FxOpGpu::resolve(&d.op, linear))
                .collect()
        };
        let (por_forma, da_camada) = ph2d_node_fx_filter::partition(&self.filtro.pilha);
        let (ops_forma, ops_camada) = (resolve(por_forma), resolve(da_camada));
        let tamanho = e.mundo.size();
        let (gpu_sprites, cpu) =
            sprites_de(&self.gpu_cook, &self.pump.instances, self.gpu_live, arte);
        let geometrias = &self.placa_geometrias;
        let Placa {
            pilha,
            colagem,
            alvos,
            celulas,
        } = self.filtro.placa(e.gpu, tamanho);
        let Some(alvos) = alvos.as_mut() else {
            return false;
        };
        if ops_forma.is_empty() {
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
            colagem.blit(
                e.gpu,
                &alvos.camada_view,
                &alvos.linear,
                BandSource::Sprites,
            );
            if let Some(formas) = e.formas.and_then(|p| p.camada()) {
                colagem.blit(e.gpu, &alvos.camada_view, formas, BandSource::Formas);
            }
        } else {
            let celulas = celulas.get_or_insert_with(|| {
                let passe = e.formas.map_or_else(
                    || ph2d_shape_gpu::ShapePass::new(e.gpu, FORMATO_DA_CAMADA),
                    |p| p.passe_irmao(e.gpu),
                );
                ph2d_fx_cells::Celulas::new(e.gpu, passe)
            });
            celulas
                .formas_mut()
                .set_geometries(e.gpu, geometrias.prontas());
            if let Some(p) = e.formas {
                p.afina(celulas.formas_mut());
            }
            let formas = e
                .formas
                .and_then(|p| p.copias_do_quadro())
                .map(|(buffer, n, vista)| ph2d_fx_cells::Formas { buffer, n, vista });
            let sprites = match gpu_sprites {
                Some((buffer, n, runs)) => ph2d_fx_cells::Sprites::Placa { buffer, n, runs },
                None => ph2d_fx_cells::Sprites::Cpu(cpu),
            };
            celulas.corre(ph2d_fx_cells::Entrada {
                gpu: e.gpu,
                renderer: e.renderer,
                camera: e.camera,
                window: e.window,
                scene_viewport: e.scene_viewport,
                sprites,
                formas,
                ops: &ops_forma,
                pilha,
                camada: &alvos.camada_view,
                tamanho,
            });
        }
        // A pilha só no que se VÊ: o retângulo da cena, alargado pelo que ela lê e espalha (doc
        // 129 §5.7 — a 4K, 2,01 → 0,77 ms).
        let (w, h) = tamanho;
        let cena = e
            .scene_viewport
            .map_or([0, 0, w, h], crate::motion_lens::rect_em_pixels);
        let rect = ph2d_render::retangulo_da_camada(cena, tamanho, &ops_camada);
        if rect[2] == 0 || rect[3] == 0 {
            return true;
        }
        alvos.saida_para(e.gpu, (rect[2], rect[3]));
        let origem = Origem::Ponto(origem_do_mundo(e.camera, e.window, e.scene_viewport));
        pilha.run_layer_rect(
            e.gpu,
            &alvos.camada,
            rect,
            &alvos.saida,
            &ops_camada,
            origem,
        );
        e.band_blit.blit_rect(
            e.gpu,
            e.mundo.blend_view(),
            &alvos.saida_view,
            BandSource::Vector,
            Some(rect),
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
