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
    fino_px: 1.0,
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

/// A fluência rgb de um solver, lida de volta (`Rgba16Float`), com a grade ONDE ela vive (a fina
/// da luz direta: amostrá-la por outra grade lê o texel errado sem erro nenhum).
pub struct Fluencia {
    pub v: Vec<[f32; 3]>,
    pub grade: ph2d_light2d_gpu::Grade,
}

pub fn fluencia_lida(gpu: &GpuContext, s: &ph2d_light2d_gpu::Solver) -> Fluencia {
    let v = ler(gpu, s.fluencia(), 8)
        .as_chunks::<8>()
        .0
        .iter()
        .map(|c| {
            let h = |k: usize| half::f16::from_le_bytes([c[2 * k], c[2 * k + 1]]).to_f32();
            [h(0), h(1), h(2)]
        })
        .collect();
    Fluencia {
        v,
        grade: s.grade_da_fluencia(),
    }
}

/// A fração VISÍVEL do disco (`centro`, `raio`) vista de `p`, com a parede abaixo da reta `p → canto`
/// (a geometria, sem traçar raio nenhum): no plano a fluência de um disco uniforme é a radiância
/// vezes o ÂNGULO que ele subtende, logo a fração é a do ângulo.
pub fn visivel(p: [f64; 2], centro: [f64; 2], raio: f64, canto: [f64; 2]) -> f64 {
    let d = [centro[0] - p[0], centro[1] - p[1]];
    let fc = d[1].atan2(d[0]);
    let alfa = (raio / d[0].hypot(d[1])).asin();
    let f_canto = (canto[1] - p[1]).atan2(canto[0] - p[0]);
    ((f_canto - (fc - alfa)) / (2.0 * alfa)).clamp(0.0, 1.0)
}

/// A largura entre 10 % e 90 % de uma curva crescente amostrada em `ys`.
pub fn largura_10_90(ys: &[f64], f: &[f64]) -> Option<f64> {
    let cruza = |nivel: f64| {
        let i = f.windows(2).position(|w| w[0] < nivel && w[1] >= nivel)?;
        let t = (nivel - f[i]) / (f[i + 1] - f[i]);
        Some(ys[i] + t * (ys[i + 1] - ys[i]))
    };
    Some((cruza(0.9)? - cruza(0.1)?).abs())
}

impl Fluencia {
    /// A luz recebida em `p` pela CPU (a conta bilinear do `receber.wgsl`).
    pub fn amostra(&self, p: [f64; 2]) -> f64 {
        amostra(&self.v, self.grade, p)
    }
}

/// A luz recebida em `p` pela CPU sobre a fluência lida da placa (a conta do `receber.wgsl`).
fn amostra(f: &[[f32; 3]], g: ph2d_light2d_gpu::Grade, p: [f64; 2]) -> f64 {
    let tx = (p[0] - g.origem_m[0]) / g.texel_m - 0.5;
    let ty = (p[1] - g.origem_m[1]) / g.texel_m - 0.5;
    let (i0, j0) = (tx.floor(), ty.floor());
    let (fx, fy) = (tx - i0, ty - j0);
    let em = |i: f64, j: f64| {
        let i = (i.max(0.0) as u32).min(g.largura - 1);
        let j = (j.max(0.0) as u32).min(g.altura - 1);
        f64::from(f[(j * g.largura + i) as usize][1])
    };
    let a = em(i0, j0) * (1.0 - fx) + em(i0 + 1.0, j0) * fx;
    let b = em(i0, j0 + 1.0) * (1.0 - fx) + em(i0 + 1.0, j0 + 1.0) * fx;
    a * (1.0 - fy) + b * fy
}
