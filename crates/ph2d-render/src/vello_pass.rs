//! Vello compute-rasterizer pass (M11 widget paint).
//!
//! Renders a [`vello::Scene`] (built upstream by `ph2d-editor` from the
//! current widget tree) into an intermediate `Rgba8Unorm` storage
//! texture, then blits that texture onto the surface frame view via
//! [`wgpu::util::TextureBlitter`]. This compositing pattern is the
//! one Vello docs recommend (`render_to_texture` doc §1) and avoids
//! the GPU vendor pessimization of compute-writes-to-surface.
//!
//! Pass ordering on the desktop shell:
//!     [sprite_pass] writes the surface view (loads clear, draws sprites)
//!     [vello_pass] reads its own intermediate, writes the surface view
//! The blitter overdraws the sprite output where the Vello scene has
//! non-transparent pixels — widgets sit on top of game content.
//!
//! Resize handling: the intermediate texture is sized to the surface
//! and rebuilt only when surface dimensions change. The `vello::Renderer`
//! (shaders + its fixed ~166 MiB resource pool) is ONE per device, shared
//! by every pass ([`crate::vello_partilhado`], F37 §2).

use crate::vello_partilhado::{Porta, trava};
use ph2d_gpu::GpuContext;
use vello::peniko::Color;
use vello::{AaConfig, RenderParams, Scene};
use wgpu::util::{TextureBlitter, TextureBlitterBuilder};

pub struct VelloPass {
    /// O renderer da placa, partilhado com os outros passes ([`crate::vello_partilhado`]).
    pub(crate) partilhado: Porta,
    /// Intermediate Rgba8Unorm storage texture; Vello compute writes
    /// here. Sized at the surface dimensions. Recreated on resize.
    intermediate: wgpu::Texture,
    intermediate_view: wgpu::TextureView,
    /// Blitter samples `intermediate` and draws into the surface view.
    blitter: TextureBlitter,
    last_size: (u32, u32),
    surface_format: wgpu::TextureFormat,
    /// ⛔⛔⛔ **O que impede um quadro sem recurso tardio de apagar o atlas do Vello** — e com ele
    /// toda imagem ESTÁVEL, para sempre. Ver [`crate::vello_keepalive`].
    keepalive: crate::vello_keepalive::KeepAlive,
    /// ⭐ O MUNDO por baixo da cena — criado no 1.º quadro que o pede. Ver [`crate::vello_fundo`].
    fundo: Option<crate::vello_fundo::Fundo>,
    /// Os ids que ESTE pass registou no renderer partilhado — largados no `Drop`.
    meus: std::collections::BTreeSet<u64>,
    /// Este episódio de falta já foi dito ao chamador (um `Err` por episódio, não um por quadro).
    falta_dita: bool,
}

/// O intervalo entre tentativas de refazer o renderer: refazê-lo custa ~6–10 ms
/// (`fx_memoria_sonda_do_preco`).
pub const ESPERA_SEM_MEMORIA: std::time::Duration = std::time::Duration::from_millis(500);

impl VelloPass {
    /// Build the pass. `surface_format` is the format of the wgpu
    /// surface frame view we'll blit into (e.g. Bgra8UnormSrgb on
    /// desktop, Rgba8UnormSrgb on web).
    pub fn new(
        gpu: &GpuContext,
        surface_format: wgpu::TextureFormat,
        initial_size: (u32, u32),
    ) -> Result<Self, String> {
        let partilhado = crate::vello_partilhado::da_placa(&gpu.device)?;

        let (intermediate, intermediate_view) = create_intermediate(&gpu.device, initial_size);
        // Pre-multiplied alpha blending: the intermediate texture has
        // a transparent background (Color::TRANSPARENT in render()) so
        // sprites underneath stay visible where the editor scene is
        // empty. Default `TextureBlitter::new` uses a no-blend
        // pipeline (overdraw) which paints transparent pixels as
        // black — hiding everything.
        let blitter = TextureBlitterBuilder::new(&gpu.device, surface_format)
            .blend_state(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING)
            .build();

        Ok(Self {
            partilhado,
            intermediate,
            intermediate_view,
            blitter,
            last_size: initial_size,
            surface_format,
            keepalive: crate::vello_keepalive::KeepAlive::new(),
            fundo: None,
            meus: std::collections::BTreeSet::new(),
            falta_dita: false,
        })
    }

    /// Recreate the intermediate texture if the surface was resized.
    /// Cheap when no-op (just a tuple compare).
    pub fn ensure_size(&mut self, gpu: &GpuContext, size: (u32, u32)) {
        if size == self.last_size || size.0 == 0 || size.1 == 0 {
            return;
        }
        // Sob recusa de memória o tamanho fica por cumprir (ver `GameRt::ensure_size`, F35 §2).
        let ((tex, view), falta) = ph2d_gpu::memoria::sob_escopo_de_memoria(&gpu.device, || {
            create_intermediate(&gpu.device, size)
        });
        self.intermediate = tex;
        self.intermediate_view = view;
        self.last_size = if falta.is_some() { (0, 0) } else { size };
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_format
    }

    /// Intermediate texture view. Bound by the
    /// [`Compositor`](crate::Compositor) as its "UI layer" input.
    pub fn intermediate_view(&self) -> &wgpu::TextureView {
        &self.intermediate_view
    }

    /// Underlying intermediate texture. Exposed so the
    /// [`Compositor`](crate::Compositor) can create extra views —
    /// `wgpu::TextureView` is not `Clone` in wgpu 28.
    pub fn intermediate_texture(&self) -> &wgpu::Texture {
        &self.intermediate
    }

    /// Register a GPU `texture` as a Vello image and return its stable [`ImageData`] handle
    /// (the vector FX, plano 24). Drawing that handle in a `Scene` samples the texture DIRECTLY —
    /// no CPU upload, no re-upload per frame. The texture must be `Rgba8Unorm` + `COPY_SRC`
    /// (Vello copies it into its atlas each render). Keep the texture alive; re-blur into it in
    /// place and Vello sees the fresh pixels behind the SAME id (zero atlas-id churn).
    pub fn register_texture(&mut self, texture: wgpu::Texture) -> vello::peniko::ImageData {
        let mut p = trava(&self.partilhado);
        let imagem = p.renderer.register_texture(texture.clone());
        p.registos
            .insert(imagem.data.id(), (imagem.clone(), texture));
        self.meus.insert(imagem.data.id());
        imagem
    }

    /// **Diz ao Vello que os pixels desta textura MUDARAM desde o último render.**
    ///
    /// ⛔⛔ **Sem isto, um re-cozimento na MESMA textura é invisível.** Até à `vello` 0.8 o atlas de
    /// imagens era **limpo a cada render** (`Resolver::resolve` chamava `image_cache.clear()`),
    /// então toda textura registada era re-copiada de graça e ninguém precisava de avisar ninguém.
    /// A **0.10 tornou o atlas PERSISTENTE**: o `get_or_insert` de uma imagem residente só empurra
    /// o upload **se ela estiver marcada suja**, e o `register_texture` marca-a suja **uma vez**,
    /// no registo. ⇒ escrever pixels novos na mesma textura deixa o atlas a servir os **velhos**,
    /// e o doc do upstream di-lo pelo nome: *«stale image data from the atlas may be used»*.
    ///
    /// ⚠️ *A afirmação «re-cozinhar escreve na mesma textura, então o Vello reusa o slot» era
    /// verdadeira por construção na 0.8 e passou a ser um defeito na 0.10 — sem uma linha nossa
    /// mudar.* É a família das compensações que a biblioteca corrigiu por baixo, ao contrário: uma
    /// **ausência** que a biblioteca passou a exigir.
    pub fn mark_texture_dirty(&mut self, image: &vello::peniko::ImageData) {
        trava(&self.partilhado)
            .renderer
            .mark_override_image_dirty(image);
    }

    /// Unregister a texture previously registered via [`Self::register_texture`] (the filter was
    /// removed / the shape deleted) — frees the atlas slot.
    pub fn unregister_texture(&mut self, image: vello::peniko::ImageData) {
        self.meus.remove(&image.data.id());
        let mut p = trava(&self.partilhado);
        p.registos.remove(&image.data.id());
        p.renderer.unregister_texture(image);
    }

    /// **A entrega ao Vello sob escopo de memória** (F35 §2). Uma recusa envenena o pool do renderer
    /// (o buffer inválido volta ao pool e é reusado em todo quadro: o editor em branco que «não
    /// recupera até fechar»); daí em diante o renderer é REFEITO de [`ESPERA_SEM_MEMORIA`] em
    /// [`ESPERA_SEM_MEMORIA`], herdando as texturas registadas. Entre tentativas a intermédia
    /// guarda o último quadro. Um `Err` por episódio; a recusa vai ao [`ph2d_gpu::memoria`].
    fn entrega(
        &mut self,
        gpu: &GpuContext,
        scene: &Scene,
        params: &RenderParams,
    ) -> Result<(), String> {
        let mut p = trava(&self.partilhado);
        if let Some(quando) = p.refazer_em {
            if std::time::Instant::now() < quando {
                return Ok(());
            }
            p.refaz()?;
        }
        let cena = self.keepalive.scene_for_vello(scene);
        let (renderer, alvo) = (&mut p.renderer, &self.intermediate_view);
        let (feito, falta) = ph2d_gpu::memoria::sob_escopo_de_memoria(&gpu.device, || {
            renderer.render_to_texture(&gpu.device, &gpu.queue, cena, alvo, params)
        });
        if let Some(e) = falta {
            p.refazer_em = Some(std::time::Instant::now() + ESPERA_SEM_MEMORIA);
            if !std::mem::replace(&mut self.falta_dita, true) {
                return Err(format!("a placa recusou memória ao Vello: {e}"));
            }
            return Ok(());
        }
        p.refazer_em = None;
        self.falta_dita = false;
        feito.map_err(|e| format!("vello render_to_texture: {e}"))
    }

    /// Para testes: finge que a última entrega foi recusada (o renderer será refeito na próxima).
    #[doc(hidden)]
    pub fn finge_falta_de_memoria(&mut self) {
        trava(&self.partilhado).refazer_em = Some(std::time::Instant::now());
    }

    /// `(renderer a refazer?, vezes refeito)` do renderer partilhado — diagnóstico e gate.
    #[doc(hidden)]
    #[must_use]
    pub fn estado_do_renderer(&self) -> (bool, u32) {
        let p = trava(&self.partilhado);
        (p.refazer_em.is_some(), p.refeitos)
    }

    /// M14.5: render the Vello `scene` into the intermediate texture
    /// **without** the final blit onto a surface view. The intermediate
    /// stays available via [`intermediate_view`](Self::intermediate_view)
    /// so a downstream compositor can read it as a texture binding.
    ///
    /// ⛔ **A escolha de anti-aliasing NÃO é um parâmetro desta função,
    /// e não pode voltar a sê-lo.** Ela é `AaConfig::Area`, sempre.
    ///
    /// O `AaConfig` do Vello vive em `RenderParams` e vale para o
    /// **PASSE INTEIRO** — não há caminho de desenho a que se aplique
    /// separadamente. Este passe carrega o chrome do editor **e** a arte
    /// vectorial do documento no mesmo `Scene`, portanto quem escolhe o
    /// AA escolhe-o para os vectores do artista.
    ///
    /// Até 2026-08-30 esta função aceitava `prefer_msaa: bool`, e o
    /// shell lia-o do preset de TEXTO em uso
    /// (`TextRendering::CrispHeavyPlus`) — um preset de tipografia a
    /// decidir a rasterização das formas. O preço já estava medido e
    /// escrito duas linhas abaixo desta assinatura: MSAA16 stippla
    /// traços finos (1–1,5 px) em ângulos quase-axiais. Report do dono
    /// do produto: «manchas animadas parecendo TV antiga» à volta das
    /// formas vectoriais (`docs/Atualizar Stack/04_registro.md` §22.2).
    ///
    /// ⚠️ A alternativa real — chrome e documento em **dois passes**,
    /// cada um com o seu `AaConfig` — é **arquitectura** (segundo alvo,
    /// segundo `Renderer`, mais uma composição), não uma bandeira nesta
    /// assinatura. O `AaSupport::all()` no construtor fica: ele é o que
    /// mantém essa porta aberta sem custo.
    ///
    /// Gate: `the_pass_aa_is_never_chosen_by_a_text_preference`.
    pub fn render_to_intermediate(
        &mut self,
        gpu: &GpuContext,
        scene: &Scene,
        size: (u32, u32),
        bg_color: Color,
    ) -> Result<(), String> {
        self.ensure_size(gpu, size);
        let params = RenderParams {
            base_color: bg_color,
            width: self.last_size.0,
            height: self.last_size.1,
            // `Area` analytical coverage, sem alternativa. MSAA16
            // produced visible stippling on thin (1-1.5 px) vector
            // strokes at near-axis angles — `Area` integrates coverage
            // analytically per pixel, smoother for thin strokes AND
            // cheaper. Ver o doc-comment acima para o porquê de isto
            // não ser um parâmetro.
            antialiasing_method: AaConfig::Area,
        };
        // ⛔⛔⛔ Uma cena sem recurso tardio apagaria o atlas do Vello (`vello_keepalive`) — a
        // `entrega` passa por ele.
        self.entrega(gpu, scene, &params)
    }

    /// ⭐⭐⭐ **O mesmo que [`Self::render_to_intermediate`], com o MUNDO por baixo da cena**
    /// (doc 118 do Motion, W2) — para uma cena que [pede o mundo por baixo](
    /// ph2d_vector::VectorScene::quer_o_mundo_por_baixo).
    ///
    /// `mundo` é uma vista **sRGB** da textura que o compositor poria por baixo do intermédio: o
    /// resultado do tonemap no quadro de sempre, o acumulador `WorldRt` quando o compositor o lê.
    /// O intermédio sai OPACO, e o `over` do compositor devolve-o tal e qual.
    ///
    /// ⚠️ **A MESMA porta de render**: a cena composta atravessa o `keepalive` e o `Area` como a
    /// outra — duas entregas ao Vello com regras diferentes seriam o defeito que o `vello_keepalive`
    /// já pagou.
    pub fn render_to_intermediate_over_world(
        &mut self,
        gpu: &GpuContext,
        scene: &Scene,
        mundo: &wgpu::TextureView,
        size: (u32, u32),
    ) -> Result<(), String> {
        self.ensure_size(gpu, size);
        let mut fundo = self
            .fundo
            .take()
            .unwrap_or_else(|| crate::vello_fundo::Fundo::new(gpu));
        let imagem = fundo.copia(gpu, self, mundo, self.last_size);
        crate::vello_fundo::compoe(&mut fundo.composta, imagem, scene);
        let feito = self.render_to_intermediate(gpu, &fundo.composta, size, Color::TRANSPARENT);
        self.fundo = Some(fundo);
        feito
    }

    /// Render `scene` into the intermediate at `size` **on a transparent
    /// background** and read the whole region back to a straight-RGBA8 `Vec`
    /// (`size.0 * size.1 * 4` bytes, sRGB-encoded — ver a nota de espaço de cor na
    /// [`crate::screen_pick`]).
    ///
    /// This is the FX producer's rasterizer (plano 24): the shell renders ONE
    /// isolated vector shape into a scratch [`VelloPass`] sized to the shape's
    /// screen bbox, reads it back, and blurs/tints it on the CPU. It is the
    /// full-region sibling of [`crate::screen_pick::read_texel`] — same `copy_texture_to_buffer`
    /// dance, same synchronous map, but a whole texture instead of one texel.
    ///
    /// wgpu requires `bytes_per_row` to be a multiple of 256, so the staging
    /// buffer is padded per row and the padding is stripped on the way out —
    /// the returned `Vec` is tightly packed (`width * 4` per row).
    ///
    /// Blocks the calling thread until the copy lands and the buffer maps
    /// (a few ms). Do NOT call this per-frame per shape without a budget — the
    /// producer sizes the scratch to the shape, not the surface, for exactly
    /// this reason.
    pub fn render_and_readback(
        &mut self,
        gpu: &GpuContext,
        scene: &Scene,
        size: (u32, u32),
    ) -> Result<Vec<u8>, String> {
        if size.0 == 0 || size.1 == 0 {
            return Ok(Vec::new());
        }
        self.render_to_intermediate(gpu, scene, size, Color::TRANSPARENT)?;
        self.read_intermediate(gpu)
    }

    /// **Lê o intermédio inteiro** para um `Vec` RGBA8 compacto — a metade de leitura do
    /// [`Self::render_and_readback`], separada para quem renderiza por outra porta (o gate do
    /// mundo por baixo, [`Self::render_to_intermediate_over_world`]).
    pub(crate) fn read_intermediate(&self, gpu: &GpuContext) -> Result<Vec<u8>, String> {
        let (w, h) = self.last_size;
        // 256-byte row alignment for texture→buffer copies.
        let unpadded = w * 4;
        let padded = unpadded.div_ceil(256) * 256;
        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-render fx readback"),
            size: (padded as u64) * (h as u64),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render fx readback encoder"),
            });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.intermediate,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(h),
                },
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        gpu.queue.submit([encoder.finish()]);

        let slice = buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |res| {
            let _ = tx.send(res);
        });
        gpu.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| format!("fx readback poll: {e}"))?;
        rx.recv()
            .map_err(|e| format!("fx readback recv: {e}"))?
            .map_err(|e| format!("fx readback map: {e}"))?;
        let view = slice.get_mapped_range();
        // Strip the per-row padding into a tightly-packed straight-RGBA8 buffer.
        let mut out = Vec::with_capacity((unpadded as usize) * (h as usize));
        for row in 0..h as usize {
            let start = row * padded as usize;
            out.extend_from_slice(&view[start..start + unpadded as usize]);
        }
        drop(view);
        buffer.unmap();
        Ok(out)
    }

    /// Render `scene` into the intermediate then blit onto `target`.
    /// `target` must match the surface_format passed at construction.
    /// `bg_color` is the Vello clear before drawing; pass transparent
    /// (`Color::TRANSPARENT`) to overlay onto the existing surface
    /// pixels (sprite content stays visible where the scene is empty).
    pub fn render(
        &mut self,
        gpu: &GpuContext,
        scene: &Scene,
        target: &wgpu::TextureView,
        size: (u32, u32),
        bg_color: Color,
    ) -> Result<(), String> {
        self.ensure_size(gpu, size);
        let params = RenderParams {
            base_color: bg_color,
            width: self.last_size.0,
            height: self.last_size.1,
            // M14.5 round 8: `Area` analytical coverage. MSAA16
            // produced visible stippling on thin (1-1.5 px) strokes
            // at near-axis angles — the 16 fixed sample positions
            // hit-or-miss the stroke in patterns that read as
            // pixelation. `Area` integrates coverage analytically
            // per pixel, smoother for thin strokes and cheaper
            // than MSAA16. The earlier "Area looks low-rez" note
            // pre-dated the gamma-correct compositor + glyph-snap
            // fixes; in the current pipeline Area wins.
            antialiasing_method: AaConfig::Area,
        };
        // ⛔⛔⛔ A MESMA porta que o `render_to_intermediate` — as duas entregas ao Vello passam
        // por ela, ou o defeito volta pela que ficou de fora (`vello_keepalive`, F35 §2).
        self.entrega(gpu, scene, &params)?;

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render vello blit"),
            });
        self.blitter
            .copy(&gpu.device, &mut encoder, &self.intermediate_view, target);
        gpu.queue.submit([encoder.finish()]);
        Ok(())
    }
}

impl Drop for VelloPass {
    /// As texturas que este pass registou saem do renderer partilhado com ele (antes, saíam com o
    /// renderer dele).
    fn drop(&mut self) {
        let mut p = trava(&self.partilhado);
        for id in std::mem::take(&mut self.meus) {
            if let Some((imagem, _)) = p.registos.remove(&id) {
                p.renderer.unregister_texture(imagem);
            }
        }
    }
}

fn create_intermediate(
    device: &wgpu::Device,
    size: (u32, u32),
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-render vello intermediate"),
        size: wgpu::Extent3d {
            width: size.0.max(1),
            height: size.1.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        // Vello requires Rgba8Unorm + STORAGE_BINDING; we add
        // TEXTURE_BINDING so the compositor can `textureLoad`
        // individual super-samples, and COPY_SRC so the eyedropper
        // can copy a single pixel back to a CPU-mappable buffer for
        // color readback. No sRGB view sibling — STORAGE_BINDING
        // forbids that combination per wgpu validation.
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

#[cfg(test)]
#[path = "vello_pass_memoria_tests.rs"]
mod memoria_tests;
