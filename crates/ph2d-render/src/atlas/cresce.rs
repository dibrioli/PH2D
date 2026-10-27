//! **O átlas nasce do tamanho do conteúdo e CRESCE no lugar** (F37 §1,
//! `docs/UI_New_and_Simple/F37_plano.md`). Crescer copia o nível 0 para a origem de uma textura
//! maior e alarga o empacotador: toda região fica no MESMO pixel e a free-list segue válida. Só a
//! UV normalizada muda — quem a relê por quadro acompanha, e o ladrilho branco tem a UV invariante
//! [`super::WHITE_TILE_UV`] (ele mora na origem desde o nascimento).

use super::renasce::nasce_textura;
use super::{AtlasInsertError, AtlasRegion, TextureAtlas};
use ph2d_gpu::GpuContext;

impl TextureAtlas {
    /// Cresce para `novo` px de lado (limitado a `[size_px, max_size_px]`), com as regiões no mesmo
    /// pixel. Devolve o lado novo; [`AtlasInsertError::SemMemoria`] quando a placa recusa a textura
    /// maior — o átlas fica como estava.
    ///
    /// # Errors
    /// [`AtlasInsertError::SemMemoria`].
    pub fn cresce(&mut self, gpu: &GpuContext, novo: u32) -> Result<u32, AtlasInsertError> {
        let novo = novo.clamp(self.size_px, self.max_size_px);
        if novo == self.size_px {
            return Ok(novo);
        }
        let (texture, view, recusada) = nasce_textura(gpu, novo);
        if recusada {
            return Err(AtlasInsertError::SemMemoria { size: novo });
        }
        match self.fonte.take() {
            // A antiga foi recusada ao nascer: os bytes de cada região são a única fonte.
            Some(fonte) => {
                for (key, rgba) in &fonte {
                    if let Some(&region) = self.regions.get(key) {
                        super::gpu_ops::upload_region(gpu, &texture, region, rgba);
                    }
                }
            }
            None => copia_nivel0(gpu, &self.texture, &texture, self.size_px),
        }
        self.texture = texture;
        self.view = view;
        self.size_px = novo;
        self.packer.resize(novo as i32, novo as i32);
        self.regen_mips(gpu);
        Ok(novo)
    }

    /// [`Self::insert`] que dobra o átlas (até `max_size_px`) enquanto a fonte não cabe.
    ///
    /// # Errors
    /// O erro do `insert` já no lado máximo, ou [`AtlasInsertError::SemMemoria`].
    pub fn insert_crescendo(
        &mut self,
        gpu: &GpuContext,
        key: u32,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> Result<AtlasRegion, AtlasInsertError> {
        loop {
            match self.insert(gpu, key, width, height, rgba) {
                Err(
                    AtlasInsertError::AtlasFull { .. } | AtlasInsertError::SourceTooLarge { .. },
                ) if width > 0
                    && height > 0
                    && width.max(height) <= self.max_size_px
                    && self.size_px < self.max_size_px =>
                {
                    self.cresce(gpu, self.size_px.saturating_mul(2))?;
                }
                feito => return feito,
            }
        }
    }
}

fn copia_nivel0(gpu: &GpuContext, de: &wgpu::Texture, para: &wgpu::Texture, lado: u32) {
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ph2d-render atlas cresce"),
        });
    encoder.copy_texture_to_texture(
        de.as_image_copy(),
        para.as_image_copy(),
        wgpu::Extent3d {
            width: lado,
            height: lado,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([encoder.finish()]);
}

#[cfg(test)]
#[path = "cresce_tests.rs"]
mod tests;
