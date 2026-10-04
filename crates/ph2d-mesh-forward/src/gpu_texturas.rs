//! As TEXTURAS do desenhista: o atlas do céu fotográfico, a tabela do céu de quem chama, e a porta de
//! paridade do brilho (que sobe imagens dadas). Irmão do `gpu.rs` por responsabilidade e pelo tecto
//! de LOC.

use super::Forward;
use crate::gpu_alvo::Alvos;

/// ⭐ **O sol do céu subido** — a tabela dele (`R32Float`) e o que o uniforme do quadro leva.
pub(super) struct SolGpu {
    pub vista: wgpu::TextureView,
    /// A direcção PARA o sol, no referencial do céu.
    pub dir: [f32; 3],
    pub radiancia: [f32; 3],
    /// O raio angular (radianos) — a penumbra da sombra.
    pub raio: f32,
}

impl Forward {
    /// ⭐⭐ **Sobe (ou troca) o céu fotográfico** — o atlas `f16` do [`ph2d_sky::Ceu`] numa textura
    /// `Rgba16Float` (só da 1.ª vez se cria; trocar de céu só escreve). Nada compila.
    pub fn sobe_ceu(&mut self, ceu: &ph2d_sky::Ceu) {
        self.geracao += 1;
        let (w, h) = (ph2d_sky::ATLAS_W, ph2d_sky::ATLAS_H);
        let (t, _) = self.foto.get_or_insert_with(|| {
            let t = self
                .device
                .create_texture(&textura_meia("ph2d-mesh-forward ceu foto", w, h));
            let v = t.create_view(&wgpu::TextureViewDescriptor::default());
            (t, v)
        });
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: t,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(ceu.atlas()),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * 8),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        // O sol: a tabela numa textura nova (`100 KB`, só ao trocar de céu); sem sol, nada.
        self.sol = ceu.sol().map(|s| SolGpu {
            vista: textura_de_floats(&self.device, &self.queue, s.tabela()),
            dir: s.dir,
            radiancia: s.radiancia,
            raio: s.raio,
        });
    }
}

impl Forward {
    /// ⭐⭐ **Sobe (ou troca) a textura da `camada`** ([`ph2d_triplanar::Mapas`]). Nada compila; crescer
    /// a matriz copia as camadas que já lá estavam.
    pub fn sobe_textura(&mut self, camada: u32, mapas: &ph2d_triplanar::Mapas) {
        self.geracao += 1;
        self.triplanar
            .sobe(&self.device, &self.queue, camada, mapas);
    }
}

/// O texel vazio que a ligação do céu fotográfico lê enquanto nenhum céu subiu.
pub(super) fn vazia(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::TextureView {
    let t = device.create_texture(&textura_meia("ph2d-mesh-forward ceu vazio", 1, 1));
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &t,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &[0u8; 8],
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(8),
            rows_per_image: Some(1),
        },
        wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    t.create_view(&wgpu::TextureViewDescriptor::default())
}

impl Forward {
    /// ⭐⭐ **A PORTA DE PARIDADE do brilho** — a cadeia e a codificação sobre imagens dadas por
    /// quem chama: `olhar` é o que o passe da cena resolveria (pré-multiplicado, já no olhar) e
    /// `cena` é a cena-linear. A malha fica de fora para a pergunta ser SÓ a do brilho; o gate do
    /// modelador compara isto com `ph2d_bloom::halo` + `ph2d_field_render::soma_halo`.
    ///
    /// `None` onde a placa não tem o brilho, ou com tamanhos que não batem.
    #[must_use]
    pub fn brilho_sobre(
        &mut self,
        olhar: &[[f32; 4]],
        cena: &[[f32; 4]],
        (w, h): (u32, u32),
        brilho: &ph2d_bloom::Bloom,
        exposicao: f32,
        vista: u32,
    ) -> Option<Vec<u8>> {
        let n = (w as usize) * (h as usize);
        if self.brilho.is_none() || n == 0 || olhar.len() != n || cena.len() != n {
            return None;
        }
        if self.alvos.as_ref().is_none_or(|a| a.tamanho != (w, h)) {
            self.alvos = Some(Alvos::novos(
                &self.device,
                (w, h),
                self.cor,
                &self.ecra_bgl,
                &self.ecra_ub,
            ));
        }
        let corre = self.prepara_brilho(&brilho.sanitized(), (w, h), exposicao, vista);
        let alvos = self.alvos.as_ref()?;
        sobe_meia(&self.queue, &alvos.resolvida_tex, olhar, (w, h));
        let com_brilho = self
            .brilho
            .as_ref()
            .zip(alvos.cadeia.as_ref())
            .filter(|_| corre);
        if let Some((_, c)) = com_brilho {
            sobe_meia(&self.queue, &c.linear, cena, (w, h));
        }
        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-mesh-forward brilho sobre"),
            });
        self.grava_ecra(&mut enc, alvos, com_brilho);
        self.queue.submit([enc.finish()]);
        self.le((w, h))
    }
}

/// Uma imagem RGBA `f32` numa textura `Rgba16Float`.
pub(super) fn sobe_meia(
    queue: &wgpu::Queue,
    t: &wgpu::Texture,
    v: &[[f32; 4]],
    (w, h): (u32, u32),
) {
    let dados: Vec<half::f16> = v
        .iter()
        .flatten()
        .map(|x| half::f16::from_f32(*x))
        .collect();
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: t,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        bytemuck::cast_slice(&dados),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(w * 8),
            rows_per_image: Some(h),
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
}

/// Uma textura `Rgba16Float` `w × h` para ler por `textureLoad`.
pub(super) fn textura_meia(
    label: &'static str,
    w: u32,
    h: u32,
) -> wgpu::TextureDescriptor<'static> {
    wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    }
}

/// Uma lista de floats numa textura `R32Float` de [`crate::TAB_W`] colunas.
pub(super) fn textura_de_floats(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    v: &[f32],
) -> wgpu::TextureView {
    let w = crate::TAB_W;
    let h = (v.len() as u32).div_ceil(w).max(1);
    let mut dados = v.to_vec();
    dados.resize((w * h) as usize, 0.0);
    let t = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-mesh-forward tabela"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &t,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        bytemuck::cast_slice(&dados),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(w * 4),
            rows_per_image: Some(h),
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    t.create_view(&wgpu::TextureViewDescriptor::default())
}
