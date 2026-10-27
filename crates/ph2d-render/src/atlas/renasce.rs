//! **O átlas que a placa recusou ao nascer renasce** (F36 §1, `docs/UI_New_and_Simple/F36_plano.md`).
//! Uma textura criada com a placa cheia nasce INVÁLIDA para sempre (o wgpu não a refaz): toda escrita
//! e todo desenho com ela falham. Medido no app (26/10, `ocupa_a_placa 700`): o átlas do arranque nasceu
//! recusado e seguia inválido depois do fim do episódio. A cura: a recusa vê-se NO NASCIMENTO
//! ([`ph2d_gpu::memoria::sob_escopo_de_memoria`]); só então o átlas guarda os bytes de cada região
//! (custo zero no caminho normal) e, no fim do episódio ([`ph2d_gpu::memoria::geracao`]), refaz-se com
//! as MESMAS regiões — as UVs que os chamadores guardaram continuam certas.

use super::gpu_ops::{clear_level0_transparent, create_texture, upload_region};
use super::{AtlasRegion, TextureAtlas};
use ph2d_gpu::GpuContext;

/// Cria a textura (e limpa o nível 0); `true` = a placa recusou ([`crate::renasce::nasce_textura`]).
pub(super) fn nasce_textura(
    gpu: &GpuContext,
    size_px: u32,
) -> (wgpu::Texture, wgpu::TextureView, bool) {
    let (texture, recusada) =
        crate::renasce::nasce_textura(&gpu.device, || create_texture(&gpu.device, size_px));
    // The sampled view spans ALL mip levels (default) for the trilinear sampler; the generator makes
    // its own single-level views.
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    clear_level0_transparent(gpu, &texture);
    (texture, view, recusada)
}

impl TextureAtlas {
    /// O funil de toda subida de pixels: escreve a região e, com a textura recusada, guarda os bytes.
    pub(super) fn sobe(&mut self, gpu: &GpuContext, key: u32, region: AtlasRegion, rgba: &[u8]) {
        upload_region(gpu, &self.texture, region, rgba);
        if let Some(fonte) = &mut self.fonte {
            fonte.insert(key, rgba.to_vec());
        }
    }

    /// `true` enquanto a textura é uma que a placa recusou.
    #[must_use]
    pub fn recusada(&self) -> bool {
        self.fonte.is_some()
    }

    /// Refaz a textura recusada com as mesmas regiões e os bytes guardados. `true` = refeita (quem
    /// tem bind groups sobre a `view` antiga refá-los); `false` = não estava recusada, ou a placa
    /// recusou de novo (fica para o próximo fim de episódio).
    pub fn refaz_se_recusada(&mut self, gpu: &GpuContext) -> bool {
        if self.fonte.is_none() {
            return false;
        }
        let (texture, view, recusada) = nasce_textura(gpu, self.size_px);
        if recusada {
            return false;
        }
        self.texture = texture;
        self.view = view;
        let fonte = self.fonte.take().unwrap_or_default();
        for (key, rgba) in &fonte {
            if let Some(&region) = self.regions.get(key) {
                upload_region(gpu, &self.texture, region, rgba);
            }
        }
        self.regen_mips(gpu);
        true
    }
}
