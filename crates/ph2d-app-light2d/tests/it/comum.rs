//! A cadeia do quadro do editor, em miniatura e sem janela: sprites → `game_rt` (HDR) → tonemap →
//! acumulador do mundo (faixa de sprites) ← faixa do vetor (Vello de verdade). As MESMAS peças e a
//! MESMA ordem do `present_bands` da shell, com a luz ligada pelas três portas da família.

use ph2d_ecs::PresentWorld;
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_render::tonemap::Tonemap;
use ph2d_render::{
    BandBlit, BandSource, Camera2d, GameRt, LiftedInstances, RenderInstance, SpriteRenderer,
    TextureAtlas, VelloPass, WorldRt,
};
use vello::kurbo::{Affine, Rect};
use vello::peniko::{Color, Fill};

/// ⭐ O degrau FINO das leis (texel de 2 px — o «Alto» do G3): os gates de LEI (a captura por texel
/// e a do vetor contra o polígono) medem-se onde a discretização não mascara a lei. A qualidade do
/// degrau PADRÃO (4 px) mede-se à parte (a paridade com o gabarito e a rodada).
pub const ALTO: ph2d_app_light2d::Degrau = ph2d_app_light2d::Degrau {
    c0_px: 2.0,
    margem: 0.25,
    vetor_px_por_texel: 1,
};

pub const W: u32 = 64;
pub const H: u32 = 32;
pub const FUNDO: wgpu::Color = wgpu::Color {
    r: 0.02,
    g: 0.02,
    b: 0.02,
    a: 1.0,
};

pub fn gpu() -> Option<GpuContext> {
    GpuContext::new(GpuContext::default_instance(), None).ok()
}

pub fn janela() -> WindowSize {
    WindowSize::new(W, H)
}

/// 16 px/m, a vista `x ∈ [−2, 2]`, `y ∈ [−1, 1]`.
pub fn camera() -> Camera2d {
    Camera2d::new([0.0, 0.0], 2.0)
}

pub fn afim() -> Affine {
    camera().world_to_screen_affine(janela())
}

/// Um renderer de sprites sobre `Rgba16Float` (o `game_rt`).
pub fn renderer(gpu: &GpuContext) -> SpriteRenderer {
    SpriteRenderer::new(
        gpu.clone(),
        wgpu::TextureFormat::Rgba16Float,
        TextureAtlas::new(gpu, 256),
        16,
    )
}

pub struct Cadeia {
    pub renderer: SpriteRenderer,
    pub game: GameRt,
    pub tonemap: Tonemap,
    pub mundo: WorldRt,
    pub colagem: BandBlit,
    pub vello: VelloPass,
    /// Uma textura 1×1 opaca de cada cor pedida.
    pub texturas: Vec<([u8; 4], u32)>,
    pub tamanho: (u32, u32),
}

impl Cadeia {
    pub fn nova(gpu: &GpuContext) -> Self {
        Self::nova_com(gpu, (W, H))
    }

    pub fn nova_com(gpu: &GpuContext, tamanho: (u32, u32)) -> Self {
        let game = GameRt::new(gpu, tamanho);
        let tonemap = Tonemap::new(
            gpu,
            game.texture().create_view(&Default::default()),
            tamanho,
        );
        Self {
            renderer: renderer(gpu),
            game,
            tonemap,
            mundo: WorldRt::new(gpu, tamanho),
            colagem: BandBlit::new(gpu, WorldRt::FORMAT),
            vello: VelloPass::new(gpu, wgpu::TextureFormat::Bgra8UnormSrgb, tamanho)
                .expect("vello"),
            texturas: Vec::new(),
            tamanho,
        }
    }

    pub fn textura(&mut self, cor: [u8; 4]) -> u32 {
        if let Some((_, id)) = self.texturas.iter().find(|(c, _)| *c == cor) {
            return *id;
        }
        let id = self
            .renderer
            .acquire_individual(1, 1, &cor)
            .expect("textura");
        self.texturas.push((cor, id));
        id
    }

    /// Um quadro inteiro; devolve os bytes BGRA crus do acumulador.
    pub fn quadro(
        &mut self,
        gpu: &GpuContext,
        present: &mut PresentWorld,
        extra: &LiftedInstances,
        vetor: &vello::Scene,
    ) -> Vec<u8> {
        self.desenhar(gpu, present, extra, vetor, &camera());
        ler_bgra(gpu, self.mundo.texture())
    }

    /// O quadro SEM ler de volta (a sonda mede o relógio dele).
    pub fn desenhar(
        &mut self,
        gpu: &GpuContext,
        present: &mut PresentWorld,
        extra: &LiftedInstances,
        vetor: &vello::Scene,
        camera: &Camera2d,
    ) {
        let (w, h) = self.tamanho;
        self.renderer.render_with_streams(
            self.game.view(),
            present,
            camera,
            WindowSize::new(w, h),
            wgpu::Color::TRANSPARENT,
            extra,
            None,
            None,
            None,
            None,
        );
        self.tonemap.run(gpu);
        self.mundo.clear_linear(gpu, FUNDO);
        self.colagem.blit(
            gpu,
            self.mundo.blend_view(),
            self.tonemap.output_view(),
            BandSource::Sprites,
        );
        self.vello
            .render_to_intermediate(gpu, vetor, (w, h), Color::TRANSPARENT)
            .expect("vello");
        self.colagem.blit(
            gpu,
            self.mundo.blend_view(),
            self.vello.intermediate_view(),
            BandSource::Vector,
        );
    }
}

/// Um quad de 1 m centrado em `centro`, com a textura `tex`.
pub fn quad(tex: u32, centro: [f32; 2]) -> RenderInstance {
    RenderInstance {
        world_pos: centro,
        size: [1.0, 1.0],
        atlas_uv: [0.0, 0.0, 1.0, 1.0],
        tint: [1.0; 4],
        basis: RenderInstance::IDENTITY_BASIS,
        premultiplied: 0.0,
        anchor: [0.0, 0.0],
        per_corner_tint: [[1.0; 4]; 4],
        opacity: 1.0,
        flip_uv: 0,
        texture_id: tex,
        z_order: 0,
        sampling: RenderInstance::SAMPLING_DEFAULT,
        uv_xform: RenderInstance::IDENTITY_UV_XFORM,
        clip_group: RenderInstance::CLIP_GROUP_NONE,
        clip_meta: 0,
        sub_order: 0,
    }
}

/// Um quadrado vetorial de 1 m centrado em `centro`, pela porta do documento (Vello).
pub fn quadrado_vetorial(cena: &mut vello::Scene, cor: [u8; 4], centro: [f64; 2]) {
    cena.fill(
        Fill::NonZero,
        afim(),
        Color::from_rgba8(cor[0], cor[1], cor[2], cor[3]),
        None,
        &Rect::new(
            centro[0] - 0.5,
            centro[1] - 0.5,
            centro[0] + 0.5,
            centro[1] + 0.5,
        ),
    );
}

pub fn ler_bgra(gpu: &GpuContext, t: &wgpu::Texture) -> Vec<u8> {
    ler(gpu, t, 4)
}

/// Lê uma textura de `bpp` bytes por texel, linhas contíguas.
pub fn ler(gpu: &GpuContext, t: &wgpu::Texture, bpp: u32) -> Vec<u8> {
    let (w, h) = (t.width(), t.height());
    let padded =
        (w * bpp).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("luz2d app leitura"),
        size: u64::from(padded * h),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu.device.create_command_encoder(&Default::default());
    enc.copy_texture_to_buffer(
        t.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(h),
            },
        },
        t.size(),
    );
    gpu.queue.submit([enc.finish()]);
    let fatia = buf.slice(..);
    fatia.map_async(wgpu::MapMode::Read, |_| {});
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    let m = fatia.get_mapped_range();
    let mut out = Vec::new();
    for y in 0..h as usize {
        out.extend_from_slice(&m[y * padded as usize..][..(w * bpp) as usize]);
    }
    out
}

/// A fluência rgb de um solver, lida de volta (`Rgba16Float`).
pub fn fluencia_lida(gpu: &GpuContext, s: &ph2d_light2d_gpu::Solver) -> Vec<[f32; 3]> {
    ler(gpu, s.fluencia(), 8)
        .as_chunks::<8>()
        .0
        .iter()
        .map(|c| {
            let h = |k: usize| half::f16::from_le_bytes([c[2 * k], c[2 * k + 1]]).to_f32();
            [h(0), h(1), h(2)]
        })
        .collect()
}
