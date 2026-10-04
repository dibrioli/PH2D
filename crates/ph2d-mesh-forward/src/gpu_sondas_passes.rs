//! A GRAVAÇÃO das capturas de reflexo ([`super::sondas_impl`]): os passes de um quadro em que a chave
//! mudou.

use super::sondas_impl::{
    FACE, FACES, MAX, NIVEIS, PASSO_QUADRO, Plano, arranjo, caixa_no_mundo, face_vp, ladrilho,
    na_face,
};
use super::{Forward, QUADRO, quadro_impl};
use crate::{Cena, Instancia};

/// Gancho dos instrumentos de custo: `1` = as faces não desenham vizinhas, `2` = sem o octaedro, a
/// cadeia e o pré-filtro.
#[cfg(test)]
pub(crate) static PULA: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

fn pula(bit: u8) -> bool {
    #[cfg(test)]
    {
        PULA.load(std::sync::atomic::Ordering::Relaxed) & bit != 0
    }
    #[cfg(not(test))]
    {
        let _ = bit;
        false
    }
}

fn resolve<'a>(
    view: &'a wgpu::TextureView,
    alvo: &'a wgpu::TextureView,
    limpo: wgpu::Color,
) -> Option<wgpu::RenderPassColorAttachment<'a>> {
    Some(wgpu::RenderPassColorAttachment {
        view,
        depth_slice: None,
        resolve_target: Some(alvo),
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(limpo),
            store: wgpu::StoreOp::Discard,
        },
    })
}

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
        let desenha = |pass: &mut wgpu::RenderPass<'_>, quem: &dyn Fn(usize) -> bool| {
            for (i, o) in objs.iter().enumerate() {
                let Some(m) = self.malhas.get(&o.malha).filter(|_| quem(i)) else {
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
                    desenha(&mut pass, &|_| true);
                }
            }
        }
        let tela = |enc: &mut wgpu::CommandEncoder,
                    pipeline: &wgpu::RenderPipeline,
                    k: u32,
                    alvos: [&wgpu::TextureView; 4]| {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ph2d-mesh-forward sondas tela"),
                color_attachments: &alvos.map(limpa),
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &sondas.binds[k as usize], &[256 * k]);
            pass.draw(0..3, 0..1);
        };
        // A caixa de cada instância no mundo: a face só desenha as vizinhas que lhe caem dentro.
        let caixas: Vec<Option<([f32; 3], [f32; 3])>> = objs
            .iter()
            .map(|o| (self.malhas.get(&o.malha)).map(|m| caixa_no_mundo(&o.modelo, m.caixa)))
            .collect();
        let g0 = self.g0_com(uq, mat_view);
        for (s, c) in plano.centros.iter().enumerate() {
            {
                let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("ph2d-mesh-forward sonda faces"),
                    color_attachments: &[
                        resolve(
                            &sondas.faces_ms[0],
                            &sondas.faces_cor,
                            wgpu::Color::TRANSPARENT,
                        ),
                        resolve(
                            &sondas.faces_ms[1],
                            &sondas.faces_dist,
                            wgpu::Color::TRANSPARENT,
                        ),
                    ],
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
                for f in 0..6 {
                    let (x, y) = ladrilho(f);
                    let l = FACE as f32;
                    pass.set_viewport(x as f32, y as f32, l, l, 0.0, 1.0);
                    pass.set_scissor_rect(x, y, FACE, FACE);
                    pass.set_bind_group(0, &g0, &[((6 * s + f) as u64 * PASSO_QUADRO) as u32]);
                    if !pula(1) {
                        desenha(&mut pass, &|i| {
                            i != s && caixas[i].is_some_and(|cx| na_face(f, *c, cx))
                        });
                    }
                }
            }
            if pula(2) {
                continue;
            }
            let (ch, al) = (&sondas.cadeia, &sondas.arranjo.alvos[s]);
            tela(
                enc,
                &sondas.octa,
                0,
                [&ch[0][0], &ch[0][1], &al[0][0], &al[0][1]],
            );
            for k in 1..NIVEIS as usize {
                tela(
                    enc,
                    &sondas.nivel,
                    k as u32,
                    [&ch[k][0], &ch[k][1], &al[k][0], &al[k][1]],
                );
            }
        }
    }
}

impl Forward {
    /// Os gates ligam e desligam as capturas (a régua de controlo).
    #[cfg(test)]
    pub(crate) fn liga_reflexos(&mut self, liga: bool) {
        self.reflexos = liga;
    }

    /// A camada `camada` das capturas no nível `k`, lida de volta (`lado × lado` texels RGBA, linha a linha).
    #[cfg(test)]
    pub(crate) fn le_sonda(&self, camada: u32, k: u32) -> Option<Vec<[f32; 4]>> {
        let s = self.sondas.as_ref()?;
        let w = super::sondas_impl::LADO >> k;
        let bpr = (w * 8).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-mesh-forward sonda lida"),
            size: u64::from(bpr * w),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        enc.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &s.arranjo.textura,
                mip_level: k,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: camada,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bpr),
                    rows_per_image: Some(w),
                },
            },
            wgpu::Extent3d {
                width: w,
                height: w,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([enc.finish()]);
        let fatia = buffer.slice(..);
        fatia.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
        let dados = fatia.get_mapped_range();
        let mut out = Vec::with_capacity((w * w) as usize);
        for y in 0..w as usize {
            let linha: &[half::f16] =
                bytemuck::cast_slice(&dados[y * bpr as usize..y * bpr as usize + w as usize * 8]);
            out.extend(linha.chunks(4).map(|c| [0, 1, 2, 3].map(|e| c[e].to_f32())));
        }
        Some(out)
    }

    /// Quantas vezes as capturas foram refeitas.
    #[cfg(test)]
    pub(crate) fn sondas_refeitas(&self) -> u64 {
        self.sondas.as_ref().map_or(0, |s| s.refeitas)
    }

    /// ⭐ **Quem tem captura**: o campo `sonda` do `Objeto` de cada instância do quadro — `(camada, centro)`
    /// ou `−1`. Só com vizinhas (duas peças ou mais), com a placa que as desenha, até [`MAX`].
    pub(super) fn atribui_sondas(&self, objs: &[&Instancia]) -> Vec<[f32; 4]> {
        let liga = self.reflexos && self.sondas.is_some() && objs.len() >= 2;
        objs.iter()
            .enumerate()
            .map(|(i, o)| match self.malhas.get(&o.malha) {
                Some(m) if liga && i < MAX => {
                    let (a, b) = caixa_no_mundo(&o.modelo, m.caixa);
                    [
                        2.0 * i as f32,
                        0.5 * (a[0] + b[0]),
                        0.5 * (a[1] + b[1]),
                        0.5 * (a[2] + b[2]),
                    ]
                }
                _ => [-1.0, 0.0, 0.0, 0.0],
            })
            .collect()
    }

    /// ⭐⭐ **O plano das capturas**: os uniformes das faces (o enquadramento das sombras da cena INTEIRA
    /// — a da vista mudaria com a câmara) e a CHAVE: tudo o que as faces leem. Igual à do quadro anterior
    /// ⇒ as capturas ficam como estão.
    pub(super) fn planeia_sondas(
        &mut self,
        cena: &Cena<'_>,
        objs: &[&Instancia],
        atrib: &[[f32; 4]],
        extra: (&[u8], &[f32]),
    ) -> Option<Plano> {
        let n = atrib.iter().filter(|a| a[0] >= 0.0).count();
        if n == 0 {
            return None;
        }
        let chave_luz = super::sombra_impl::chave(cena, self.foto.is_some(), self.sol.as_ref());
        let e = super::sombra_impl::enquadra(
            cena,
            chave_luz,
            |id| self.malhas.get(&id).map(|m| m.caixa),
            false,
        );
        let (mut lo, mut hi) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
        for o in objs {
            if let Some(m) = self.malhas.get(&o.malha) {
                let (a, b) = caixa_no_mundo(&o.modelo, m.caixa);
                for k in 0..3 {
                    lo[k] = lo[k].min(a[k]);
                    hi[k] = hi[k].max(b[k]);
                }
            }
        }
        let diag = (0..3).map(|k| (hi[k] - lo[k]).powi(2)).sum::<f32>().sqrt();
        let longe = 2.0 * diag + 1.0e-3;
        let perto = longe * 1.0e-4;
        let passo = PASSO_QUADRO as usize;
        let mut dados = vec![0u8; 6 * n * passo];
        for (s, a) in atrib.iter().take(n).enumerate() {
            let c = [a[1], a[2], a[3]];
            for (f, (olha, _)) in FACES.iter().enumerate() {
                let cam = crate::Camera {
                    view_proj: face_vp(f, c, perto, longe),
                    olho: c,
                    perspectiva: true,
                    dir_vista: *olha,
                };
                let u = quadro_impl::uniforme_do_quadro(
                    &Cena {
                        camera: cam,
                        ..*cena
                    },
                    &e,
                    self.foto.is_some(),
                    self.sol.as_ref(),
                    true,
                );
                let at = (6 * s + f) * passo;
                dados[at..at + QUADRO * 4].copy_from_slice(bytemuck::cast_slice(&u));
            }
        }
        let mut chave = dados.clone();
        chave.extend_from_slice(extra.0);
        chave.extend_from_slice(bytemuck::cast_slice(extra.1));
        chave.extend_from_slice(&self.geracao.to_le_bytes());
        let sondas = self.sondas.as_mut()?;
        let cap = n.next_power_of_two();
        if sondas.arranjo.cap < cap {
            sondas.arranjo = arranjo(&self.device, cap);
            sondas.chave.clear();
        }
        if sondas.quadros.as_ref().is_none_or(|(c, _)| *c < 6 * cap) {
            let b = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ph2d-mesh-forward sondas quadros"),
                size: (6 * cap) as u64 * PASSO_QUADRO,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            sondas.quadros = Some((6 * cap, b));
        }
        let refaz = sondas.chave != chave;
        if refaz {
            if let Some((_, b)) = &sondas.quadros {
                self.queue.write_buffer(b, 0, &dados);
            }
            sondas.chave = chave;
            sondas.refeitas += 1;
        }
        Some(Plano {
            centros: atrib.iter().take(n).map(|a| [a[1], a[2], a[3]]).collect(),
            refaz,
            ha_sombra: e.ha_sombra,
            ha_chao: e.ha_chao,
        })
    }
}
