//! ⭐ **A miniatura do que a câmera vê** (A6, `docs/GameEngineFuncional/18_miniatura_da_camera_plano.md`) — a vista
//! da câmera seleccionada TROCA-SE com a do editor à volta do MESMO código que desenha o canvas (`present*.rs`), num
//! alvo do tamanho do canto; o acumulador dela passa a um retrato `Rgba8Unorm` que a cena do chrome desenha como
//! imagem registada (a fase da selecção, por cima das molduras).
//!
//! Por vista: a câmera, os alvos do tamanho dela, as cenas do documento por faixa, a luz 2D, o Flip, a placa e a
//! camada do Motion e as de cada palco de uma macro. Partilhado (lido): o mundo, o renderer, o cozimento do Motion,
//! os palcos (UM estado de reprodução), o documento do Flip.

use crate::AppGfx;
use ph2d_render::{Camera2d, GameRt, MotionFx, Tonemap, VelloPass, WorldRt};

/// O pedido deste quadro (a fase da câmera) e a vista, criada no primeiro pedido.
#[derive(Default)]
pub(crate) struct Miniatura {
    pedido: Option<[f32; 4]>,
    vista: Option<Box<Vista>>,
    /// O CPU da codificação deste quadro (ms), somado ao do desenho no [`Custo`].
    pub(crate) cpu_codifica: f64,
    custo: Custo,
    /// O quadro está a desenhar a vista da câmera (GEF 18 A6): o chrome de autoria fica de fora.
    pub(crate) na_vista: bool,
}

/// ⭐ INSTRUMENTO `PH2D_MINIATURA_CUSTO=1`: o CPU da vista (codificar + desenhar), mínimo e mediana a cada 120
/// quadros. A placa: o `span` `render.miniatura` do perfilador de passes (`PH2D_FLUID_PROFILE=1`).
#[derive(Default)]
struct Custo {
    amostras: Vec<f64>,
}

impl Custo {
    fn conta(&mut self, ms: f64) {
        if std::env::var_os("PH2D_MINIATURA_CUSTO").is_none() {
            return;
        }
        self.amostras.push(ms);
        if self.amostras.len() == 120 {
            self.amostras.sort_by(f64::total_cmp);
            let (min, med) = (self.amostras[0], self.amostras[60]);
            eprintln!("[miniatura] cpu min {min:.2} med {med:.2} ms (codificar + desenhar)");
            self.amostras.clear();
        }
    }
}

/// O que a vista da câmera tem de seu.
pub(crate) struct Vista {
    pub(crate) camera: Camera2d,
    tamanho: (u32, u32),
    game_rt: GameRt,
    world_rt: WorldRt,
    tonemap: Tonemap,
    vello_pass: VelloPass,
    motion_fx: MotionFx,
    pub(crate) band_doc_scenes: Vec<ph2d_vector::VectorScene>,
    pub(crate) luz2d: ph2d_app_light2d::LuzDoQuadro,
    flip_render: ph2d_flip_render::FlipRenderer,
    flip_compose: ph2d_flip_render::FlipCompose,
    pub(crate) placa: ph2d_app_motion::motion_shape_placa::PlacaDeFormas,
    pub(crate) motion: ph2d_app_motion::motion_filter::VistaDoMotion,
    pub(crate) palcos: ph2d_app_motion::macro_track::palcos::VistaDosPalcos,
    /// As formas do Motion que a placa não leva — no editor vão na cena do chrome; aqui colam-se por cima do mundo.
    pub(crate) cena_do_motion: ph2d_vector::VectorScene,
    retrato: Retrato,
}

/// A cópia do acumulador que o Vello lê (o formato que o `register_texture` aceita).
struct Retrato {
    textura: wgpu::Texture,
    copia: wgpu::util::TextureBlitter,
    registo: ph2d_vector::ImageData,
    imagem: ph2d_vector::StableImage,
}

impl Retrato {
    fn new(gpu: &ph2d_gpu::GpuContext, vello: &mut VelloPass, tamanho: (u32, u32)) -> Self {
        let textura = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ph2d.vista_camera.retrato"),
            size: wgpu::Extent3d {
                width: tamanho.0,
                height: tamanho.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let copia = wgpu::util::TextureBlitter::new(&gpu.device, wgpu::TextureFormat::Rgba8Unorm);
        let registo = vello.register_texture(textura.clone());
        let imagem = ph2d_vector::StableImage::from_registered(registo.clone());
        Self {
            textura,
            copia,
            registo,
            imagem,
        }
    }

    /// Copia os bytes do acumulador (sRGB codificado, lido pela vista NÃO-sRGB: os bytes passam como estão).
    fn copia_de(&self, gpu: &ph2d_gpu::GpuContext, mundo: &WorldRt) {
        let alvo = self
            .textura
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d.vista_camera.copia"),
            });
        self.copia
            .copy(&gpu.device, &mut enc, mundo.blend_view(), &alvo);
        gpu.queue.submit([enc.finish()]);
    }
}

impl Vista {
    fn new(
        gpu: &ph2d_gpu::GpuContext,
        editor: &mut VelloPass,
        tamanho: (u32, u32),
    ) -> Option<Self> {
        let game_rt = GameRt::new(gpu, tamanho);
        let tonemap = Tonemap::new(
            gpu,
            game_rt
                .texture()
                .create_view(&wgpu::TextureViewDescriptor::default()),
            tamanho,
        );
        let vello_pass = VelloPass::new(gpu, editor.surface_format(), tamanho).ok()?;
        Some(Self {
            camera: Camera2d::default(),
            tamanho,
            game_rt,
            world_rt: WorldRt::new(gpu, tamanho),
            tonemap,
            vello_pass,
            motion_fx: MotionFx::new(gpu, tamanho),
            band_doc_scenes: Vec::new(),
            luz2d: ph2d_app_light2d::LuzDoQuadro::da_vista(),
            flip_render: ph2d_flip_render::FlipRenderer::new(&gpu.device, GameRt::FORMAT),
            flip_compose: ph2d_flip_render::FlipCompose::new(&gpu.device, GameRt::FORMAT),
            placa: Default::default(),
            motion: Default::default(),
            palcos: Default::default(),
            cena_do_motion: ph2d_vector::VectorScene::new(),
            retrato: Retrato::new(gpu, editor, tamanho),
        })
    }

    /// A caixa mudou de tamanho: os alvos seguem-na e o retrato renasce (o id registado leva as dimensões).
    fn redimensiona(
        &mut self,
        gpu: &ph2d_gpu::GpuContext,
        editor: &mut VelloPass,
        tamanho: (u32, u32),
    ) {
        self.game_rt.ensure_size(gpu, tamanho);
        self.world_rt.ensure_size(gpu, tamanho);
        self.motion_fx.ensure_size(gpu, tamanho);
        self.tonemap.ensure_size(gpu, tamanho);
        self.tonemap.rebind_game_view(
            gpu,
            self.game_rt
                .texture()
                .create_view(&wgpu::TextureViewDescriptor::default()),
        );
        self.vello_pass.ensure_size(gpu, tamanho);
        let velho = std::mem::replace(&mut self.retrato, Retrato::new(gpu, editor, tamanho));
        editor.unregister_texture(velho.registo);
        self.tamanho = tamanho;
    }

    /// O afim mundo → píxel do alvo da vista (o que a codificação do documento e a luz recebem).
    pub(crate) fn afim(&self) -> ph2d_vector::Affine {
        self.camera.world_to_screen_affine(self.janela())
    }

    pub(crate) fn janela(&self) -> ph2d_host::WindowSize {
        ph2d_host::WindowSize {
            width: self.tamanho.0,
            height: self.tamanho.1,
        }
    }

    /// Troca o que é por vista com o `AppGfx` (e as placas do Motion, que moram no estado da família).
    fn troca(
        &mut self,
        gfx: &mut AppGfx,
        shell: &mut ph2d_app_motion::motion_shell_state::MotionShellState,
    ) {
        std::mem::swap(&mut self.camera, &mut gfx.camera);
        std::mem::swap(&mut self.game_rt, &mut gfx.game_rt);
        std::mem::swap(&mut self.world_rt, &mut gfx.world_rt);
        std::mem::swap(&mut self.tonemap, &mut gfx.tonemap);
        std::mem::swap(&mut self.vello_pass, &mut gfx.vello_pass);
        std::mem::swap(&mut self.motion_fx, &mut gfx.motion_fx);
        std::mem::swap(&mut self.band_doc_scenes, &mut gfx.band_doc_scenes);
        std::mem::swap(&mut self.luz2d, &mut gfx.luz2d);
        std::mem::swap(&mut self.flip_render, &mut gfx.flip_render);
        std::mem::swap(&mut self.flip_compose, &mut gfx.flip_compose);
        std::mem::swap(&mut self.placa, &mut shell.placa);
        gfx.motion.troca_a_vista(&mut self.motion);
        shell.palcos.troca_a_vista(&mut self.palcos);
    }

    /// As formas do Motion que a placa não leva, por cima do mundo da vista (o lugar da cena do chrome no editor).
    fn cola_o_motion(&self, gfx: &mut AppGfx) {
        if self.cena_do_motion.inner().encoding().is_empty() {
            return;
        }
        let gpu = gfx.surface.gpu();
        let pintou = gfx.vello_pass.render_to_intermediate(
            gpu,
            self.cena_do_motion.inner(),
            self.tamanho,
            ph2d_vector::Color::TRANSPARENT,
        );
        match pintou {
            Ok(()) => gfx.band_blit.blit(
                gpu,
                gfx.world_rt.blend_view(),
                gfx.vello_pass.intermediate_view(),
                ph2d_render::BandSource::Vector,
            ),
            Err(e) => eprintln!("[miniatura] as formas do Motion: {e}"),
        }
    }
}

impl Miniatura {
    /// O pedido deste quadro: a vista da câmera escolhida na caixa do canto (`None` = sem miniatura). A vista nasce
    /// ou segue o tamanho da caixa aqui, antes de a codificação e a cena do chrome a lerem.
    pub(crate) fn pede(
        &mut self,
        gpu: &ph2d_gpu::GpuContext,
        editor: &mut VelloPass,
        pedido: Option<(Camera2d, [f32; 4])>,
    ) {
        self.pedido = None;
        let Some((camera, caixa)) = pedido else {
            return;
        };
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let tamanho = (caixa[2].round() as u32, caixa[3].round() as u32);
        if tamanho.0 == 0 || tamanho.1 == 0 {
            return;
        }
        match self.vista.as_deref_mut() {
            Some(v) if v.tamanho != tamanho => v.redimensiona(gpu, editor, tamanho),
            Some(_) => {}
            None => self.vista = Vista::new(gpu, editor, tamanho).map(Box::new),
        }
        if let Some(v) = self.vista.as_deref_mut() {
            v.camera = camera;
            self.pedido = Some(caixa);
        }
    }

    /// A vista pedida neste quadro.
    pub(crate) fn pedida(&mut self) -> Option<&mut Vista> {
        self.pedido?;
        self.vista.as_deref_mut()
    }

    /// O retrato e a caixa do canto, para a cena do chrome.
    pub(crate) fn retrato(&self) -> Option<(&ph2d_vector::StableImage, [f32; 4])> {
        let caixa = self.pedido?;
        self.vista.as_ref().map(|v| (&v.retrato.imagem, caixa))
    }
}

impl crate::App {
    /// Desenha a vista da câmera pedida neste quadro — ANTES da do editor (cada passe faz o seu `submit`; o `ligar`
    /// da luz do editor reescreve os receptores depois). Sem pedido, nada.
    pub(super) fn present_vista_da_camera(&mut self, r: f64, g: f64, b: f64, motion_draws: bool) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        if gfx.miniatura.pedida().is_none() {
            return;
        }
        let Some(mut vista) = gfx.miniatura.vista.take() else {
            return;
        };
        let t0 = std::time::Instant::now();
        let span = {
            let g = gfx.surface.gpu();
            ph2d_gpu::pass_profiler::span_begin(&g.device, &g.queue, "render.miniatura")
        };
        vista.troca(gfx, &mut self.motion_shell);
        let janela = vista.janela();
        gfx.miniatura.na_vista = true;
        let vazio = ph2d_render::LiftedInstances::default();
        if let Some((plan, banded)) =
            self.present_world_sprites(r, g, b, janela, None, motion_draws, false, &vazio, &vazio)
        {
            self.present_flip_fx(janela, None, motion_draws, &plan);
            self.present_mundo_por_cima(janela, None, &plan, banded);
        }
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.miniatura.na_vista = false;
            vista.cola_o_motion(gfx);
            vista.troca(gfx, &mut self.motion_shell);
            vista.retrato.copia_de(gfx.surface.gpu(), &vista.world_rt);
            // O atlas do Vello é persistente: sem a marca ele serve o retrato do 1.º quadro (`mark_texture_dirty`).
            gfx.vello_pass.mark_texture_dirty(&vista.retrato.registo);
            if let Some(t) = span {
                let g = gfx.surface.gpu();
                ph2d_gpu::pass_profiler::span_end(&g.device, &g.queue, t);
            }
            let ms = gfx.miniatura.cpu_codifica + t0.elapsed().as_secs_f64() * 1000.0;
            gfx.miniatura.custo.conta(ms);
            gfx.miniatura.vista = Some(vista);
        }
    }
}
