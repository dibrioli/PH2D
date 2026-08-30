//! `WorldRt` — **o acumulador do MUNDO**, onde as faixas de desenho se empilham (ADR-0154 Fase 2).
//!
//! Até 2026-08-30 o mundo era duas texturas coladas por um `over` **fixo**: o `game_rt` (sprites,
//! HDR, tonemapeado) por baixo, o intermediário do Vello (formas + chrome) por cima. ⇒ uma forma
//! vetorial ficava acima de toda imagem, **sempre**, e nenhum valor de Z podia mudá-lo.
//!
//! Este RT é o sítio onde as duas famílias se empilham **na ordem que o ordenador decidiu**: cada
//! faixa é rasterizada pelo motor dela, e colada aqui por cima do que já cá está.
//!
//! # ⭐⭐ Por que `Bgra8Unorm` — e por que ele tem DUAS vistas
//!
//! O compositor deste app faz o `over` **no espaço do DESENHISTA** (valores já codificados em
//! sRGB), e não em luz linear: é a convenção do Figma/Illustrator, e é o que toda a arte vetorial
//! já existente assume. Misturar em linear mudaria **todas** as bordas de vetor sobre imagem.
//!
//! ⇒ A textura é **`Bgra8Unorm` (sem sRGB)** e guarda valores **já codificados**. A mistura de
//! hardware opera sobre os bytes crus, o que É a mistura no espaço do desenhista — exactamente o
//! que o shader do compositor emula à mão.
//!
//! ⚠️ **E a vista de LEITURA é `Bgra8UnormSrgb`.** O compositor amostra o `game_rt` a contar com a
//! descodificação automática de um formato sRGB e re-codifica com `linear_to_srgb`. Com uma vista
//! crua ele codificaria duas vezes. Duas vistas da MESMA textura fazem os dois lados lerem o que
//! esperam — **e o compositor não muda uma linha**.

use ph2d_gpu::GpuContext;

pub struct WorldRt {
    texture: wgpu::Texture,
    /// Alvo de desenho — formato CRU, para a mistura de hardware acontecer no espaço do desenhista.
    blend_view: wgpu::TextureView,
    /// Vista de amostragem — formato sRGB, para o compositor descodificar como faz com o `game_rt`.
    sample_view: wgpu::TextureView,
    size: (u32, u32),
}

impl WorldRt {
    /// O formato do ALVO (cru).
    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;
    /// O formato da vista de amostragem.
    pub const SAMPLE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

    pub fn new(gpu: &GpuContext, size: (u32, u32)) -> Self {
        let (texture, blend_view, sample_view) = create(&gpu.device, size);
        Self {
            texture,
            blend_view,
            sample_view,
            size,
        }
    }

    /// Recria a textura se as dimensões mudaram. No-op quando o tamanho bate.
    pub fn ensure_size(&mut self, gpu: &GpuContext, size: (u32, u32)) {
        if size == self.size || size.0 == 0 || size.1 == 0 {
            return;
        }
        let (t, b, s) = create(&gpu.device, size);
        self.texture = t;
        self.blend_view = b;
        self.sample_view = s;
        self.size = size;
    }

    /// A vista onde as faixas se desenham.
    pub fn blend_view(&self) -> &wgpu::TextureView {
        &self.blend_view
    }

    /// A vista que o compositor amostra.
    pub fn sample_view(&self) -> &wgpu::TextureView {
        &self.sample_view
    }

    /// Limpa o acumulador para a cor de fundo do canvas, dada em **luz LINEAR** — a mesma que o
    /// passe de sprite usa.
    ///
    /// ⛔⛔ **Ela CODIFICA aqui dentro, e a primeira versão não codificava.** Report do Enio,
    /// 2026-08-30: *«toda vez que a imagem fica sobre o vetor o canvas escurece»*. O alvo é de
    /// formato **cru** de propósito (ver o cabeçalho) ⇒ o `LoadOp::Clear` escreve o valor tal e
    /// qual, e escrever um valor **linear** num buffer que guarda valores **codificados** dá um
    /// fundo visivelmente mais escuro (linear `0,2` lê-se onde deveria estar `0,48`).
    ///
    /// ⚠️ **O doc-comment que aqui estava PREVIA este defeito** — *«passar aqui a cor linear do
    /// passe de sprite daria um fundo visivelmente mais escuro»* — e o chamador passou-a. ⇒ a
    /// porta deixou de aceitar a escolha: **ela recebe linear e converte**, que é a única
    /// convenção que o chamador tem para dar. *Um aviso no doc não é uma cerca.*
    pub fn clear_linear(&self, gpu: &GpuContext, linear: wgpu::Color) {
        self.clear(
            gpu,
            wgpu::Color {
                r: linear_to_srgb(linear.r),
                g: linear_to_srgb(linear.g),
                b: linear_to_srgb(linear.b),
                a: linear.a,
            },
        )
    }

    /// Limpa o acumulador para `color`, **já em espaço do desenhista**.
    fn clear(&self, gpu: &GpuContext, color: wgpu::Color) {
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render world RT clear encoder"),
            });
        let _ = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ph2d-render world RT clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.blend_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        gpu.queue.submit(Some(enc.finish()));
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }
}

fn create(
    device: &wgpu::Device,
    size: (u32, u32),
) -> (wgpu::Texture, wgpu::TextureView, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-render world RT (designer-space accumulator)"),
        size: wgpu::Extent3d {
            width: size.0.max(1),
            height: size.1.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: WorldRt::FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        // ⚠️ **A vista sRGB só é construível se ela for DECLARADA aqui.** Sem esta linha o
        // `create_view` com outro formato é erro de validação em tempo de execução — e o modo de
        // falha seria um painel preto no primeiro quadro com uma forma.
        view_formats: &[WorldRt::SAMPLE_FORMAT],
    });
    let blend_view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("ph2d-render world RT blend view (raw)"),
        format: Some(WorldRt::FORMAT),
        ..Default::default()
    });
    let sample_view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("ph2d-render world RT sample view (sRGB)"),
        format: Some(WorldRt::SAMPLE_FORMAT),
        ..Default::default()
    });
    (texture, blend_view, sample_view)
}

/// sRGB (IEC 61966-2-1) — a mesma curva do `compositor.wgsl` e do `band_blit.wgsl`, em CPU.
///
/// ⚠️ Ela existe em três sítios porque vive em duas linguagens; os gates abaixo prendem-na aos
/// pontos que a definem (`0`, `1`, e o joelho em `0,0031308`).
fn linear_to_srgb(c: f64) -> f64 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

#[cfg(test)]
mod tests {
    use super::linear_to_srgb;

    /// ⛔ **A conversão CLAREIA** — é isto que o report de 2026-08-30 mediu com o olho: sem ela o
    /// fundo do canvas entrava escuro no acumulador.
    #[test]
    fn the_encoding_lightens_every_midtone() {
        for lin in [0.05, 0.1, 0.2, 0.35, 0.5, 0.8] {
            let enc = linear_to_srgb(lin);
            assert!(
                enc > lin,
                "linear {lin} devia codificar para MAIS claro, veio {enc}"
            );
        }
    }

    /// E os extremos são pontos fixos — senão o preto deixa de ser preto.
    #[test]
    fn black_and_white_are_fixed_points() {
        assert!((linear_to_srgb(0.0)).abs() < 1e-12);
        assert!((linear_to_srgb(1.0) - 1.0).abs() < 1e-9);
    }

    /// O joelho da curva bate com o das duas cópias em WGSL.
    #[test]
    fn the_knee_matches_the_shader_copies() {
        let k = 0.003_130_8;
        assert!((linear_to_srgb(k) - k * 12.92).abs() < 1e-9);
        assert!(
            (linear_to_srgb(k + 1e-9) - (1.055 * (k + 1e-9f64).powf(1.0 / 2.4) - 0.055)).abs()
                < 1e-9
        );
    }
}
