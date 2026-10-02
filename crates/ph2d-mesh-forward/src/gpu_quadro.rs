//! O quadro: o enquadramento da sombra, o uniforme, os passes e a leitura.

use super::{Forward, QUADRO, SLOT};
use crate::Cena;

/// ⭐ **O mapa de sombra enquadra o MUNDO dos objetos**, olhando a direito para baixo (a caixa de
/// luz do estúdio é `+y`). A margem é a penumbra máxima: a sombra de um objeto à altura `h` do chão
/// espalha-se `h · tan` para cada lado.
pub(super) struct Enquadra {
    pub ha_sombra: bool,
    pub sombra_vp: [[f32; 4]; 4],
    pub chao_xz: [f32; 4],
    pub fundo: f32,
    pub texel: f32,
}

fn aplica(m: &[[f32; 4]; 4], p: [f32; 3]) -> [f32; 3] {
    [0, 1, 2].map(|i| m[0][i] * p[0] + m[1][i] * p[1] + m[2][i] * p[2] + m[3][i])
}

pub(super) fn enquadra_sombra(
    cena: &Cena<'_>,
    caixa_de: impl Fn(u64) -> Option<([f32; 3], [f32; 3])>,
) -> Enquadra {
    let (mut lo, mut hi) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
    for o in cena.objetos {
        let Some((a, b)) = caixa_de(o.malha) else {
            continue;
        };
        for k in 0..8 {
            let c = [
                if k & 1 == 0 { a[0] } else { b[0] },
                if k & 2 == 0 { a[1] } else { b[1] },
                if k & 4 == 0 { a[2] } else { b[2] },
            ];
            let w = aplica(&o.modelo, c);
            for i in 0..3 {
                lo[i] = lo[i].min(w[i]);
                hi[i] = hi[i].max(w[i]);
            }
        }
    }
    let tem = lo[0].is_finite();
    if !tem {
        lo = [-1.0; 3];
        hi = [1.0; 3];
    }
    let chao = cena.chao.unwrap_or(lo[1]);
    let tan = cena.caixa_tan.unwrap_or(0.0);
    let topo = hi[1] + 0.01 * (hi[1] - lo[1]).max(1.0e-3);
    let base = chao.min(lo[1]) - 0.01 * (hi[1] - lo[1]).max(1.0e-3);
    let fundo = (topo - base).max(1.0e-4);
    let margem = tan * (topo - chao).max(0.0);
    let (cx, cz) = ((lo[0] + hi[0]) * 0.5, (lo[2] + hi[2]) * 0.5);
    let meia = ((hi[0] - lo[0]).max(hi[2] - lo[2]) * 0.5 + margem).max(1.0e-3) * 1.02;
    // mundo → recorte: x = (X − cx)/meia · y = (Z − cz)/meia · z = (topo − Y)/fundo.
    let sombra_vp = [
        [1.0 / meia, 0.0, 0.0, 0.0],
        [0.0, 0.0, -1.0 / fundo, 0.0],
        [0.0, 1.0 / meia, 0.0, 0.0],
        [-cx / meia, -cz / meia, topo / fundo, 1.0],
    ];
    Enquadra {
        ha_sombra: tem && cena.caixa_tan.is_some(),
        sombra_vp,
        chao_xz: [cx, cz, meia, meia],
        fundo,
        texel: 2.0 * meia / crate::SOMBRA_LADO as f32,
    }
}

pub(super) fn uniforme_do_quadro(cena: &Cena<'_>, e: &Enquadra) -> Vec<f32> {
    let mut u = Vec::with_capacity(QUADRO);
    for col in cena.camera.view_proj.iter().chain(e.sombra_vp.iter()) {
        u.extend_from_slice(col);
    }
    let c = &cena.camera;
    u.extend_from_slice(&[c.olho[0], c.olho[1], c.olho[2], f32::from(u8::from(c.perspectiva))]);
    u.extend_from_slice(&[c.dir_vista[0], c.dir_vista[1], c.dir_vista[2], 0.0]);
    u.extend_from_slice(&[
        cena.chao.unwrap_or(0.0),
        f32::from(u8::from(cena.chao.is_some())),
        cena.caixa_tan.unwrap_or(0.0),
        e.texel,
    ]);
    u.extend_from_slice(&e.chao_xz);
    u.extend_from_slice(&[e.fundo, 1.5 * e.texel / e.fundo, f32::from(u8::from(e.ha_sombra)), 0.0]);
    let n = cena.luzes.len().min(crate::MAX_LUZES);
    u.extend_from_slice(&[cena.exposicao, cena.vista as f32, n as f32, 0.0]);
    for l in &cena.luzes[..n] {
        u.extend_from_slice(&[l.posicao[0], l.posicao[1], l.posicao[2], 0.0]);
        u.extend_from_slice(&[l.radiancia_a_um[0], l.radiancia_a_um[1], l.radiancia_a_um[2], 0.0]);
    }
    u.resize(QUADRO, 0.0);
    u
}

impl Forward {
    pub(super) fn sobe_materiais(&mut self, mats: &[[f32; ph2d_material::wgsl::PACKED]]) {
        let n = (mats.len() as u32).max(1);
        if self.materiais.as_ref().is_none_or(|(k, _, _)| *k != n) {
            let t = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("ph2d-mesh-forward materiais"),
                size: wgpu::Extent3d {
                    width: crate::MATERIAL_V4,
                    height: n,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let v = t.create_view(&wgpu::TextureViewDescriptor::default());
            self.materiais = Some((n, t, v));
        }
        let mut dados: Vec<f32> = mats.iter().flatten().copied().collect();
        dados.resize((n * crate::MATERIAL_V4 * 4) as usize, 0.0);
        if let Some((_, t, _)) = &self.materiais {
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: t,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                bytemuck::cast_slice(&dados),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(crate::MATERIAL_V4 * 16),
                    rows_per_image: Some(n),
                },
                wgpu::Extent3d {
                    width: crate::MATERIAL_V4,
                    height: n,
                    depth_or_array_layers: 1,
                },
            );
        }
    }

    pub(super) fn sobe_objetos(&mut self, objs: &[&crate::Instancia]) {
        let precisa = (objs.len() as u64).max(1);
        if self.objetos.as_ref().is_none_or(|(cap, _, _)| *cap < precisa) {
            let cap = precisa.next_power_of_two();
            let b = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ph2d-mesh-forward objetos"),
                size: cap * SLOT,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ph2d-mesh-forward g1"),
                layout: &self.g1_bgl,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &b,
                        offset: 0,
                        size: std::num::NonZeroU64::new(64),
                    }),
                }],
            });
            self.objetos = Some((cap, b, bg));
        }
        let mut dados = vec![0u8; objs.len() * SLOT as usize];
        for (i, o) in objs.iter().enumerate() {
            let at = i * SLOT as usize;
            dados[at..at + 64].copy_from_slice(bytemuck::cast_slice(&o.modelo));
        }
        if let Some((_, b, _)) = &self.objetos
            && !dados.is_empty()
        {
            self.queue.write_buffer(b, 0, &dados);
        }
    }

    pub(super) fn desenha(&self, cena: &Cena<'_>, objs: &[&crate::Instancia], ha_sombra: bool) {
        let (Some(alvos), Some((_, _, mat_view)), Some((_, _, g1))) =
            (&self.alvos, &self.materiais, &self.objetos)
        else {
            return;
        };
        let g0 = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-mesh-forward g0"),
            layout: &self.g0_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.quadro.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.ceu.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.tabela),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(mat_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&self.mapa_sombra),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&self.compara),
                },
            ],
        });
        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-mesh-forward quadro"),
            });
        let desenha_objetos = |pass: &mut wgpu::RenderPass<'_>| {
            for (i, o) in objs.iter().enumerate() {
                let Some(m) = self.malhas.get(&o.malha) else {
                    continue;
                };
                pass.set_bind_group(1, g1, &[(i as u64 * SLOT) as u32]);
                pass.set_vertex_buffer(0, m.vertices.slice(..));
                pass.set_index_buffer(m.indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..m.n, 0, 0..1);
            }
        };
        if ha_sombra {
            let g0s = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ph2d-mesh-forward g0 sombra"),
                layout: &self.g0_sombra_bgl,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.quadro.as_entire_binding(),
                }],
            });
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ph2d-mesh-forward sombra"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.mapa_sombra,
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
            pass.set_pipeline(&self.sombra);
            pass.set_bind_group(0, &g0s, &[]);
            desenha_objetos(&mut pass);
        }
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ph2d-mesh-forward cena"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &alvos.cor_msaa,
                    depth_slice: None,
                    resolve_target: Some(&alvos.resolvida),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Discard,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &alvos.profundidade,
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
            pass.set_bind_group(0, &g0, &[]);
            if ha_sombra && cena.chao.is_some() {
                pass.set_pipeline(&self.chao);
                pass.draw(0..6, 0..1);
            }
            pass.set_pipeline(&self.objeto);
            desenha_objetos(&mut pass);
        }
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ph2d-mesh-forward ecra"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &alvos.saida_vista,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.ecra);
            pass.set_bind_group(0, &alvos.ecra_bind, &[]);
            pass.draw(0..3, 0..1);
        }
        self.queue.submit([enc.finish()]);
    }

    pub(super) fn le(&self, (w, h): (u32, u32)) -> Option<Vec<u8>> {
        let alvos = self.alvos.as_ref()?;
        let bpr = (w * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-mesh-forward leitura"),
            size: u64::from(bpr) * u64::from(h),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-mesh-forward leitura"),
            });
        enc.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &alvos.saida,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bpr),
                    rows_per_image: Some(h),
                },
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([enc.finish()]);
        let fatia = buffer.slice(..);
        fatia.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
        let dados = fatia.get_mapped_range();
        let mut out = Vec::with_capacity((w * h * 4) as usize);
        for linha in 0..h as usize {
            let a = linha * bpr as usize;
            out.extend_from_slice(&dados[a..a + (w * 4) as usize]);
        }
        Some(out)
    }
}
