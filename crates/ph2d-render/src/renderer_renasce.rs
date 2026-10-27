//! **No fim de um episódio sem memória, o que a placa recusou ao nascer renasce** (F36 §1,
//! `docs/UI_New_and_Simple/F36_plano.md`): o átlas e as texturas individuais refazem-se com o conteúdo
//! que guardaram, e o grupo do material (que aponta para a `view` do átlas) refaz-se com eles. Corre
//! uma vez por quadro, à cabeça de todo desenho — o padrão do `FxLive` (F35).

use super::*;

impl SpriteRenderer {
    /// Refaz o que nasceu recusado quando [`ph2d_gpu::memoria::geracao`] mudou. Devolve
    /// `(átlas refeito, individuais refeitas)`.
    pub fn renasce_do_episodio(&mut self) -> (bool, usize) {
        let g = ph2d_gpu::memoria::geracao();
        if g == self.geracao_vista {
            return (false, 0);
        }
        self.geracao_vista = g;
        let atlas = self.atlas.refaz_se_recusada(&self.gpu);
        if atlas {
            self.rebuild_material_bind_group();
        }
        let individuais = self
            .individual
            .refaz_recusadas(&self.gpu, &self.pipeline.material_bgl);
        (atlas, individuais)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **G19 — o renderer refaz o átlas recusado no 1.º quadro e no fim de cada episódio, e o grupo do
    /// material passa a apontar para a textura nova**; sem mudança de geração não refaz nada.
    #[test]
    #[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
    fn the_sprite_renderer_rebuilds_a_refused_atlas_and_its_material_group() {
        let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
            return;
        };
        crate::renasce::testes::forca_recusas(1);
        let atlas = TextureAtlas::new(&gpu, 64);
        assert!(atlas.recusada());
        let mut r = SpriteRenderer::new(gpu, wgpu::TextureFormat::Rgba8UnormSrgb, atlas, 16);
        let grupo = r.material_bind_group.clone();
        // Um quadro de verdade (vazio): o renascer corre à cabeça de todo desenho.
        let alvo = r.gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("G19 alvo"),
            size: wgpu::Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let vista = alvo.create_view(&wgpu::TextureViewDescriptor::default());
        let janela = WindowSize {
            width: 4,
            height: 4,
        };
        let camara = Camera2d::new([0.0, 0.0], 4.0);
        r.render_instances_only(
            &vista,
            &camara,
            janela,
            wgpu::Color::TRANSPARENT,
            &[],
            None,
            None,
        );
        assert!(!r.atlas.recusada(), "o 1.º quadro não refez o átlas");
        assert!(
            r.material_bind_group != grupo,
            "o grupo do material ainda aponta para a textura recusada"
        );
        assert_eq!(
            r.renasce_do_episodio(),
            (false, 0),
            "refez sem mudança de geração"
        );
    }
}
