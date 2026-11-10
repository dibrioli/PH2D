//! ⭐ **A banda de RAMPA de cada faixa** (a luz 2D a lê: `ph2d_app_light2d::NormaisDasFaixas`) (P13 L, `docs/PixelArt/P13_plano.md` §4.L), irmã da banda
//! normal: a cena (`ph2d_pixel_luz::banda_rampa`, composta na shell) e a LUT da faixa
//! (`LutDaLuz::texeis`). Só existe com «Light: Palette» num sprite à vista; sem nenhuma, nada é
//! alocado (G6).

use ph2d_vector::VectorScene;

/// A banda de rampa de uma faixa: a cena, os texels da LUT (`(L, r, g, b)`) e `(largura, altura)`.
pub struct RampaDaFaixa {
    pub cena: VectorScene,
    pub lut: Vec<[f32; 4]>,
    pub tamanho_lut: (u32, u32),
}

/// As bandas de rampa do quadro (índice = o da faixa) e as texturas onde pousam.
#[derive(Default)]
pub struct RampasDasFaixas {
    pub cenas: Vec<Option<RampaDaFaixa>>,
    textura: Option<(wgpu::Texture, wgpu::TextureView, (u32, u32))>,
}

impl RampasDasFaixas {
    /// Os bytes da textura da banda (0 = nada alocado).
    #[must_use]
    pub fn bytes(&self) -> u64 {
        self.textura
            .as_ref()
            .map_or(0, |(_, _, (w, h))| u64::from(*w) * u64::from(*h) * 4)
    }

    /// A banda da faixa `i` desenhada pelo Vello e copiada para a textura da banda, mais a LUT
    /// carregada; `None` = a faixa não tem banda de rampa.
    pub fn preparar(
        &mut self,
        gpu: &ph2d_gpu::GpuContext,
        vello: &mut crate::VelloPass,
        i: usize,
        tamanho: (u32, u32),
    ) -> Option<(wgpu::TextureView, wgpu::TextureView)> {
        use wgpu::util::DeviceExt as _;
        let r = self.cenas.get(i)?.as_ref()?;
        let transparente = ph2d_vector::Color::TRANSPARENT;
        if let Err(e) = vello.render_to_intermediate(gpu, r.cena.inner(), tamanho, transparente) {
            eprintln!("[zorder] banda de rampa falhou: {e}");
        }
        let formato = vello.intermediate_texture().format();
        if self
            .textura
            .as_ref()
            .is_none_or(|(t, _, s)| *s != tamanho || t.format() != formato)
        {
            let tex = textura_da_banda(gpu, tamanho, formato, "banda de rampa");
            let vista = tex.create_view(&wgpu::TextureViewDescriptor::default());
            self.textura = Some((tex, vista, tamanho));
        }
        let (tex, vista, _) = self.textura.as_ref()?;
        copia_do_intermediario(gpu, vello, tex, tamanho);
        let texeis: Vec<u8> = r
            .lut
            .iter()
            .flatten()
            .flat_map(|f| f.to_le_bytes())
            .collect();
        let lut = gpu.device.create_texture_with_data(
            &gpu.queue,
            &wgpu::TextureDescriptor {
                label: Some("ph2d-app-light2d lut da rampa"),
                size: wgpu::Extent3d {
                    width: r.tamanho_lut.0,
                    height: r.tamanho_lut.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: crate::luz_rampa::FORMATO_DA_LUT,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &texeis,
        );
        let lut = lut.create_view(&wgpu::TextureViewDescriptor::default());
        Some((vista.clone(), lut))
    }

    /// Sem banda nenhuma neste quadro, a textura sai (G6).
    pub fn larga_se_vazia(&mut self) {
        if self.cenas.iter().all(Option::is_none) {
            self.textura = None;
        }
    }
}

/// A textura de uma banda (normal ou de rampa), do tamanho do alvo, que recebe a cópia do Vello.
pub fn textura_da_banda(
    gpu: &ph2d_gpu::GpuContext,
    tamanho: (u32, u32),
    formato: wgpu::TextureFormat,
    rotulo: &str,
) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some(rotulo),
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
    })
}

/// Copia o intermediário do Vello (a banda acabada de desenhar) para `tex`.
pub fn copia_do_intermediario(
    gpu: &ph2d_gpu::GpuContext,
    vello: &crate::VelloPass,
    tex: &wgpu::Texture,
    tamanho: (u32, u32),
) {
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ph2d-app-light2d copia da banda"),
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
}
