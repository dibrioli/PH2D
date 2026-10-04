//! A GRAVAÇÃO das capturas de reflexo ([`super::sondas_impl`]): os passes de um quadro em que a chave
//! mudou.

use super::Forward;
use super::QUADRO;
use super::sondas_impl::{FACE, LADO, NIVEIS, PASSO_QUADRO, Plano, ladrilho};
use crate::Instancia;

fn limpa(view: &wgpu::TextureView) -> Option<wgpu::RenderPassColorAttachment<'_>> {
    Some(wgpu::RenderPassColorAttachment {
        view,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            store: wgpu::StoreOp::Store,
        },
    })
}

impl Forward {
    /// ⭐⭐ **Refaz as capturas** (só com a chave mudada): os mapas de sombra no enquadramento da cena, e
    /// por captura as `6` faces (todas as vizinhas, nunca a própria peça), o octaedro, a cadeia e o
    /// pré-filtro. Os mapas ficam a ser redesenhados para a vista a seguir, por quem chama.
    #[allow(clippy::too_many_lines)]
    pub(super) fn grava_sondas(
        &self,
        enc: &mut wgpu::CommandEncoder,
        plano: &Plano,
        objs: &[&Instancia],
        mat_view: &wgpu::TextureView,
        g1: &wgpu::BindGroup,
    ) {
        let Some(sondas) = &self.sondas else {
            return;
        };
        let Some((_, uq)) = &sondas.quadros else {
            return;
        };
        let fatia = |k: usize| wgpu::BufferBinding {
            buffer: uq,
            offset: k as u64 * PASSO_QUADRO,
            size: std::num::NonZeroU64::new((QUADRO * 4) as u64),
        };
        let desenha = |pass: &mut wgpu::RenderPass<'_>, salta: Option<usize>| {
            for (i, o) in objs.iter().enumerate() {
                let Some(m) = self.malhas.get(&o.malha).filter(|_| Some(i) != salta) else {
                    continue;
                };
                pass.set_bind_group(1, g1, &[(i as u64 * super::SLOT) as u32]);
                pass.set_vertex_buffer(0, m.vertices.slice(..));
                pass.set_vertex_buffer(1, m.curvatura.slice(..));
                pass.set_index_buffer(m.indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..m.n, 0, 0..1);
            }
        };
        if plano.ha_chao {
            let g0s = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ph2d-mesh-forward g0 sombra das sondas"),
                layout: &self.g0_sombra_bgl,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(fatia(0)),
                }],
            });
            for mapa in &self.mapas_sombra {
                let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("ph2d-mesh-forward sombra das sondas"),
                    color_attachments: &[],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: mapa,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                if plano.ha_sombra {
                    pass.set_pipeline(&self.sombra);
                    pass.set_bind_group(0, &g0s, &[]);
                    desenha(&mut pass, None);
                }
            }
        }
        let tela = |enc: &mut wgpu::CommandEncoder,
                    pipeline: &wgpu::RenderPipeline,
                    k: u32,
                    alvos: [&wgpu::TextureView; 2]| {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ph2d-mesh-forward sondas tela"),
                color_attachments: &[limpa(alvos[0]), limpa(alvos[1])],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &sondas.bind, &[256 * k]);
            pass.draw(0..3, 0..1);
        };
        let copia = |enc: &mut wgpu::CommandEncoder, k: u32| {
            for (c, (t, _)) in sondas.cadeia.iter().zip(&sondas.tmp[k as usize]) {
                enc.copy_texture_to_texture(
                    t.as_image_copy(),
                    wgpu::TexelCopyTextureInfo {
                        texture: c,
                        mip_level: k,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::Extent3d {
                        width: LADO >> k,
                        height: LADO >> k,
                        depth_or_array_layers: 1,
                    },
                );
            }
        };
        for s in 0..plano.n {
            let grupos: Vec<wgpu::BindGroup> = (0..6)
                .map(|f| self.g0_com(wgpu::BindingResource::Buffer(fatia(6 * s + f)), mat_view))
                .collect();
            {
                let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("ph2d-mesh-forward sonda faces"),
                    color_attachments: &[limpa(&sondas.faces_cor), limpa(&sondas.faces_dist)],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &sondas.faces_prof,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Discard,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                pass.set_pipeline(&sondas.faces);
                for (f, g0) in grupos.iter().enumerate() {
                    let (x, y) = ladrilho(f);
                    let l = FACE as f32;
                    pass.set_viewport(x as f32, y as f32, l, l, 0.0, 1.0);
                    pass.set_scissor_rect(x, y, FACE, FACE);
                    pass.set_bind_group(0, g0, &[]);
                    desenha(&mut pass, Some(s));
                }
            }
            let t = &sondas.tmp;
            tela(enc, &sondas.octa, 0, [&t[0][0].1, &t[0][1].1]);
            copia(enc, 0);
            for k in 1..NIVEIS {
                tela(
                    enc,
                    &sondas.desce,
                    k,
                    [&t[k as usize][0].1, &t[k as usize][1].1],
                );
                copia(enc, k);
            }
            for k in 0..NIVEIS {
                let camada = |c: u32| {
                    sondas.arranjo.1.create_view(&wgpu::TextureViewDescriptor {
                        dimension: Some(wgpu::TextureViewDimension::D2),
                        base_mip_level: k,
                        mip_level_count: Some(1),
                        base_array_layer: c,
                        array_layer_count: Some(1),
                        ..Default::default()
                    })
                };
                let (cor, dist) = (camada(2 * s as u32), camada(2 * s as u32 + 1));
                tela(enc, &sondas.prefiltro, k, [&cor, &dist]);
            }
        }
    }
}
