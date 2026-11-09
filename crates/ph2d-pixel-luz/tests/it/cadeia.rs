//! A cadeia do quadro do editor em miniatura, sem janela (cópia mínima do `comum` dos testes da
//! `ph2d-app-light2d`): sprites → tonemap → acumulador ← faixa do Vello, e a fluência lida de volta.

#![allow(dead_code)]

use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_render::tonemap::Tonemap;
use ph2d_render::{BandBlit, Camera2d, GameRt, SpriteRenderer, TextureAtlas, VelloPass, WorldRt};
use vello::kurbo::Affine;

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
    let m = fatia.get_mapped_range().expect("mapa lido");
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

impl Fluencia {
    /// A luz recebida em `p` pela CPU (a conta bilinear do `receber.wgsl`).
    pub fn amostra(&self, p: [f64; 2]) -> f64 {
        amostra(&self.v, self.grade, p, 0)
    }
}

fn amostra(f: &[[f32; 3]], g: ph2d_light2d_gpu::Grade, p: [f64; 2], c: usize) -> f64 {
    let tx = (p[0] - g.origem_m[0]) / g.texel_m - 0.5;
    let ty = (p[1] - g.origem_m[1]) / g.texel_m - 0.5;
    let (i0, j0) = (tx.floor(), ty.floor());
    let (fx, fy) = (tx - i0, ty - j0);
    let em = |i: f64, j: f64| {
        let i = (i.max(0.0) as u32).min(g.largura - 1);
        let j = (j.max(0.0) as u32).min(g.altura - 1);
        f64::from(f[(j * g.largura + i) as usize][c])
    };
    let a = em(i0, j0) * (1.0 - fx) + em(i0 + 1.0, j0) * fx;
    let b = em(i0, j0 + 1.0) * (1.0 - fx) + em(i0 + 1.0, j0 + 1.0) * fx;
    a * (1.0 - fy) + b * fy
}
