//! **A textura individual que a placa recusou ao nascer renasce, com o MESMO id** (F36 §1,
//! `docs/UI_New_and_Simple/F36_plano.md`). A recusa vê-se no nascimento ([`crate::renasce`]); só então
//! a entrada guarda o seu conteúdo, e [`IndividualTextureStore::refaz_recusadas`] refá-la no fim do
//! episódio ([`ph2d_gpu::memoria::geracao`]). As sprites que apontam para o id não mudam.

use crate::individual::IndividualTextureStore;
use ph2d_gpu::GpuContext;

/// O conteúdo de uma entrada que nasceu recusada.
pub(crate) enum FonteRecusada {
    Rgba8(Vec<u8>),
    Rgba16(Vec<u16>),
    /// Slot vazio (`acquire_empty`): enchido por cópia de outra textura da placa — renasce
    /// transparente e o dono volta a copiar (o preview da pintura recompõe no fim do episódio).
    Vazia,
}

impl IndividualTextureStore {
    /// Quantas entradas estão recusadas agora.
    #[must_use]
    pub fn recusadas(&self) -> usize {
        self.entries.values().filter(|e| e.fonte.is_some()).count()
    }

    /// Refaz cada entrada recusada (mesmo id, mesmo refcount) com o conteúdo guardado. Devolve quantas
    /// renasceram; a que a placa recusar de novo fica para o próximo fim de episódio.
    pub fn refaz_recusadas(
        &mut self,
        gpu: &GpuContext,
        material_bgl: &wgpu::BindGroupLayout,
    ) -> usize {
        let ids: Vec<u32> = self
            .entries
            .iter()
            .filter(|(_, e)| e.fonte.is_some())
            .map(|(&id, _)| id)
            .collect();
        let mut refeitas = 0;
        for id in ids {
            let Some(velha) = self.entries.get(&id) else {
                continue;
            };
            let (w, h, formato, refcount) =
                (velha.width, velha.height, velha.format, velha.refcount);
            let mut nova = crate::individual_entry::create_entry_empty(
                gpu,
                material_bgl,
                &self.sampler,
                w,
                h,
                formato,
            );
            if nova.fonte.is_some() {
                continue;
            }
            let Some(velha) = self.entries.get_mut(&id) else {
                continue;
            };
            match velha.fonte.take() {
                Some(FonteRecusada::Rgba8(px)) => {
                    crate::individual_entry::write_pixels(gpu, &nova.texture, w, h, &px);
                }
                Some(FonteRecusada::Rgba16(halves)) => {
                    crate::individual_entry::write_pixels_16(gpu, &nova.texture, w, h, &halves);
                }
                Some(FonteRecusada::Vazia) | None => {}
            }
            nova.refcount = refcount;
            *velha = nova;
            self.regen_mips(gpu, id);
            refeitas += 1;
        }
        refeitas
    }

    /// Uma escrita de pixels inteira numa entrada recusada passa a ser a sua fonte.
    pub(crate) fn anota_pixels_recusados(&mut self, id: u32, rgba: &[u8]) {
        if let Some(e) = self.entries.get_mut(&id)
            && e.fonte.is_some()
        {
            e.fonte = Some(FonteRecusada::Rgba8(rgba.to_vec()));
        }
    }

    /// Uma escrita de região numa entrada recusada com fonte de 8 bits remenda a fonte.
    pub(crate) fn anota_regiao_recusada(
        &mut self,
        id: u32,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        px: &[u8],
    ) {
        let Some(e) = self.entries.get_mut(&id) else {
            return;
        };
        let largura = e.width as usize;
        if let Some(FonteRecusada::Rgba8(fonte)) = &mut e.fonte {
            let linha = w as usize * 4;
            for r in 0..h as usize {
                let destino = ((y as usize + r) * largura + x as usize) * 4;
                fonte[destino..destino + linha].copy_from_slice(&px[r * linha..(r + 1) * linha]);
            }
        }
    }
}
