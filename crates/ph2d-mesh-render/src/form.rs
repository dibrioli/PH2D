//! **A DOAÇÃO, empacotada para quem consome** — o G-buffer de normais lido de volta como o plano
//! `[nx, ny, nz, cobertura]` que o passe de luz do Painter espera.
//!
//! O [`MeshRenderer::render_gbuffer`] responde *"que forma há em cada pixel?"* numa textura. Este
//! módulo responde a pergunta seguinte, que é a de quem **paga**: *"como isso vira um plano que o
//! Painter instala?"* — e a resposta é uma leitura de volta, com todo o preço que ela tem.
//!
//! ## Por que a volta pela CPU, se o consumidor de GPU já teria a textura
//!
//! São **DOIS** consumidores: o passe de luz da CPU (que lê `ReliefFields::form`, um slice) e o da
//! GPU (que lê uma textura). Dois produtores para o mesmo número é a falha que esta linha já pagou
//! duas vezes nesta wave — *uma doação que aparecesse numa rota e não na outra seria uma escultura
//! que ilumina a tinta até o artista redimensionar a janela*. O plano é produzido **UMA** vez, e as
//! duas rotas leem o mesmo.
//!
//! ⚠️ **O preço fica NOMEADO, não escondido:** para a rota de GPU isto é uma ida e volta ao device
//! que ela não precisaria — a textura já estava lá. Removê-lo é uma wave própria (a rota de GPU
//! consumindo a textura direto) que só se justifica **depois** de medir; e ela nasce devendo o gate
//! de paridade que hoje é grátis.
//!
//! ⚠️ **Isto BLOQUEIA** (`map_async` + `poll`). É por isso que a porta não é chamada por frame: o
//! chamador só a invoca quando a forma de fato mudou — a malha, a câmera ou o tamanho do canvas.

use crate::{Camera3d, MeshRenderer};

impl MeshRenderer {
    /// Rasteriza a forma no tamanho pedido e devolve o plano `[nx, ny, nz, cobertura]` por texel, em
    /// ordem de LINHA (o mesmo layout de todo plano canvas-shaped do Painter).
    ///
    /// `None` sem geometria ou com extensão vazia — o mesmo silêncio do `render_gbuffer`, porque
    /// *"não há forma"* e *"a forma é toda transparente"* são a mesma resposta para quem consome.
    ///
    /// ⚠️ **O `f16` sobe para `f32` sem perda**, e não é conveniência: `f16` tem 10 bits de mantissa
    /// e expoente menor, então todo valor representável nele é exatamente representável em `f32`. O
    /// que o dispositivo escreveu é o que o Painter lê — é isso que torna a paridade CPU×GPU do
    /// plano uma propriedade, e não uma tolerância.
    #[must_use]
    pub fn form_plane(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: &Camera3d,
        size: (u32, u32),
    ) -> Option<Vec<f32>> {
        if !self.has_mesh() || size.0 == 0 || size.1 == 0 {
            return None;
        }
        let (w, h) = size;
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ph2d-mesh form plane"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::GBUFFER_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = target.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        // ⚠️ Sem pré-passe de limpeza, de propósito: o `render_gbuffer` LIMPA o alvo ele mesmo, e é
        // isso que faz a cobertura sair certa sem o chamador combinar nada.
        self.render_gbuffer(device, queue, &mut encoder, &view, camera, size);

        let bpr = (w * 8).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-mesh form readback"),
            size: u64::from(bpr) * u64::from(h),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &target,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bpr),
                    rows_per_image: Some(h),
                },
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        queue.submit([encoder.finish()]);

        let slice = buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
        let mapped = slice.get_mapped_range();
        let (w, h) = (w as usize, h as usize);
        let bpr = bpr as usize;
        let mut out = vec![0f32; w * h * 4];
        for row in 0..h {
            let src = row * bpr;
            let dst = row * w * 4;
            for x in 0..w {
                let o = src + x * 8;
                for k in 0..4 {
                    let bits = u16::from_le_bytes([mapped[o + k * 2], mapped[o + k * 2 + 1]]);
                    out[dst + x * 4 + k] = half::f16::from_bits(bits).to_f32();
                }
            }
        }
        drop(mapped);
        buffer.unmap();
        Some(out)
    }
}
