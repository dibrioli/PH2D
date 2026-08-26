//! **COM QUE SAMPLER uma textura individual é lida** — o quarto corte por
//! responsabilidade deste ficheiro (o store POSSUI, o `individual_entry` CONSTRÓI, o
//! `individual_read` LÊ DE VOLTA, e aqui mora *com que amostragem ela é desenhada*).
//!
//! ⚠️ **Ele nasceu de um defeito de produto pré-existente, achado em 2026-08-25** (doc 89,
//! folha 17): o `material_bg` do `renderer_draw` honrava a `RenderInstance::sampling` **só
//! para o átlas partilhado**, e para toda textura individual devolvia UM grupo construído
//! contra o sampler *default do projecto*. ⇒ o filtro por-nó do Inspector (§9) estava
//! **inerte em toda textura individual do app**, e uma sprite promovida a Individual por
//! um `commit_edited_texture` perdia o filtro dela **em silêncio** — no caso para que o
//! filtro existe (*pixel-art*, que chega por importação e quase nunca está no átlas).
//!
//! O gate de PIXEL que mede a cura com um adapter real é
//! `tests/individual_texture_honours_its_sampling.rs`.

use crate::individual::IndividualTextureStore;
use ph2d_gpu::GpuContext;

impl IndividualTextureStore {
    /// Switch the sampling mode for every individually-owned texture.
    /// Recreates the store sampler AND rebuilds each entry's bind group
    /// (the bind group bakes the old sampler in, so it must be
    /// re-created against the new one). The textures and `texture_id`s
    /// are untouched — only how they're sampled — so SimWorld sprite
    /// references stay valid and no pixel data is re-uploaded.
    pub fn set_filter_mode(
        &mut self,
        gpu: &GpuContext,
        material_bgl: &wgpu::BindGroupLayout,
        filter: crate::ImageFilterMode,
    ) {
        self.sampler = crate::create_sprite_sampler(
            &gpu.device,
            filter,
            "ph2d-render individual texture sampler",
        );
        for entry in self.entries.values_mut() {
            entry.bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ph2d-render individual bg (refiltered)"),
                layout: material_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&entry.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });
        }
    }

    /// O bind group desta textura **para uma amostragem concreta** (doc 89, folha 17).
    ///
    /// `sampling == 0` (*herda o default do projecto*) devolve o [`Self::bind_group`] de
    /// sempre, que é exactamente o que ele significa. Qualquer outra chave devolve o grupo
    /// cacheado no entry — `None` se ele ainda não foi construído, o que o
    /// [`Self::ensure_sampler_bg`] resolve antes do passe.
    pub fn bind_group_for(&self, id: u32, sampling: u32) -> Option<&wgpu::BindGroup> {
        let e = self.entries.get(&id)?;
        match sampling {
            0 => Some(&e.bind_group),
            k => e.sampler_bgs.get(&k),
        }
    }

    /// Constrói (uma vez) o bind group desta textura para `sampling` — o gémeo exacto do
    /// `SpriteRenderer::ensure_atlas_sampler_bg`, e chamado do mesmo sítio: a varredura
    /// dos runs, antes do passe, porque o `material_bg` do desenho só tem `&self`.
    pub fn ensure_sampler_bg(
        &mut self,
        gpu: &GpuContext,
        material_bgl: &wgpu::BindGroupLayout,
        id: u32,
        sampling: u32,
    ) {
        if sampling == 0 {
            return;
        }
        let Some(entry) = self.entries.get_mut(&id) else {
            return;
        };
        if entry.sampler_bgs.contains_key(&sampling) {
            return;
        }
        let (filter, repeat) = crate::RenderInstance::unpack_sampling(sampling);
        let sampler = crate::image_filter::sampler_from_tags(&gpu.device, filter, repeat);
        let bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render individual per-sampling bg"),
            layout: material_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&entry.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        entry.sampler_bgs.insert(sampling, bg);
    }
}
