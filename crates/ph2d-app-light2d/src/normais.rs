//! ⭐ **A banda NORMAL de cada faixa** (P14 §4 L) — a cena que a família desenha (o Pixel:
//! `ph2d_app_pixel::luz::banda_normal`), por faixa, e a textura onde ela pousa para a colagem a ler
//! (`BandBlit::blit_com_normal`). Mora na luz porque a luz decide: com alguma banda, a cena da luz
//! liga `com_normal` (o momento `M`); sem nenhuma, nada é alocado (G6).

use ph2d_vector::VectorScene;

/// A banda normal de cada faixa do quadro (índice = o da cena da faixa) e a textura onde ela pousa,
/// alocada só quando alguma faixa a tem.
#[derive(Default)]
pub struct NormaisDasFaixas {
    pub cenas: Vec<Option<VectorScene>>,
    textura: Option<(wgpu::Texture, wgpu::TextureView, (u32, u32))>,
}

impl NormaisDasFaixas {
    /// Alguma faixa deste quadro tem banda normal (⇒ `CenaDeLuz.com_normal` do que a luz grava)?
    #[must_use]
    pub fn alguma(&self) -> bool {
        self.cenas.iter().any(Option::is_some)
    }

    /// Os bytes da textura da banda (0 = nada alocado: a guarda G6).
    #[must_use]
    pub fn bytes(&self) -> u64 {
        self.textura
            .as_ref()
            .map_or(0, |(_, _, (w, h))| u64::from(*w) * u64::from(*h) * 4)
    }

    /// ⭐ Cola a faixa `i` (`cena`, a cor) sobre `alvo`: com banda normal, ela vai primeiro ao
    /// intermediário do Vello e é copiada para a textura da banda; a cor depois, pela colagem com
    /// a normal. Sem ela, a colagem de sempre.
    pub fn colar(
        &mut self,
        gpu: &ph2d_gpu::GpuContext,
        vello: &mut ph2d_render::VelloPass,
        colagem: &ph2d_render::BandBlit,
        alvo: &wgpu::TextureView,
        (i, cena): (usize, &VectorScene),
        tamanho: (u32, u32),
    ) {
        let transparente = ph2d_vector::Color::TRANSPARENT;
        let normal = self.cenas.get_mut(i).and_then(Option::take);
        let vista = normal.as_ref().map(|n| {
            if let Err(e) = vello.render_to_intermediate(gpu, n.inner(), tamanho, transparente) {
                eprintln!("[zorder] banda normal falhou: {e}");
            }
            let (tex, vista) = self.textura_de(gpu, tamanho, vello.intermediate_texture().format());
            let mut enc = gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("ph2d-app-pixel banda normal"),
                });
            enc.copy_texture_to_texture(
                vello.intermediate_texture().as_image_copy(),
                tex.as_image_copy(),
                wgpu::Extent3d {
                    width: tamanho.0,
                    height: tamanho.1,
                    depth_or_array_layers: 1,
                },
            );
            gpu.queue.submit([enc.finish()]);
            vista
        });
        if let Some(n) = normal {
            self.cenas[i] = Some(n);
        }
        if let Err(e) = vello.render_to_intermediate(gpu, cena.inner(), tamanho, transparente) {
            eprintln!("[zorder] faixa de vetor falhou: {e}");
        }
        let fonte = ph2d_render::BandSource::Vector;
        match vista {
            Some(n) => {
                colagem.blit_com_normal(gpu, alvo, vello.intermediate_view(), &n, fonte, None)
            }
            None => colagem.blit(gpu, alvo, vello.intermediate_view(), fonte),
        }
        if !self.alguma() {
            self.textura = None;
        }
    }

    fn textura_de(
        &mut self,
        gpu: &ph2d_gpu::GpuContext,
        tamanho: (u32, u32),
        formato: wgpu::TextureFormat,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        if self
            .textura
            .as_ref()
            .is_none_or(|(t, _, s)| *s != tamanho || t.format() != formato)
        {
            let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("ph2d-app-pixel banda normal"),
                size: wgpu::Extent3d {
                    width: tamanho.0,
                    height: tamanho.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: formato,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let vista = tex.create_view(&wgpu::TextureViewDescriptor::default());
            self.textura = Some((tex, vista, tamanho));
        }
        let (t, v, _) = self.textura.as_ref().expect("acabada de criar");
        (t.clone(), v.clone())
    }
}
