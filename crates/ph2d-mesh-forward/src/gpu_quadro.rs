//! O quadro: o uniforme, os passes e a leitura (o enquadramento das sombras é o `gpu_sombra.rs`).

use super::sombra_impl::Enquadra;
use super::{Forward, QUADRO, SLOT};
use crate::Cena;

/// O inverso de uma matriz `4×4` (coluna a coluna), em `f64` — a identidade se for singular.
pub(super) fn inversa(m: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let a: Vec<f64> = m.iter().flatten().map(|x| f64::from(*x)).collect();
    // Gauss-Jordan sobre [A | I] (a ordem coluna-a-coluna é a transposta; inverter a transposta
    // dá a transposta da inversa — a mesma arrumação).
    let mut t = [[0.0f64; 8]; 4];
    for (i, linha) in t.iter_mut().enumerate() {
        for j in 0..4 {
            linha[j] = a[i * 4 + j];
        }
        linha[4 + i] = 1.0;
    }
    for c in 0..4 {
        let p = (c..4)
            .max_by(|&x, &y| t[x][c].abs().total_cmp(&t[y][c].abs()))
            .unwrap_or(c);
        if t[p][c].abs() < 1.0e-30 {
            let mut id = [[0.0f32; 4]; 4];
            for (i, l) in id.iter_mut().enumerate() {
                l[i] = 1.0;
            }
            return id;
        }
        t.swap(c, p);
        let piv = t[c][c];
        for v in &mut t[c] {
            *v /= piv;
        }
        for r in 0..4 {
            if r != c {
                let f = t[r][c];
                let fonte = t[c];
                for (v, s) in t[r].iter_mut().zip(fonte) {
                    *v -= f * s;
                }
            }
        }
    }
    let mut out = [[0.0f32; 4]; 4];
    for (i, l) in out.iter_mut().enumerate() {
        for (j, v) in l.iter_mut().enumerate() {
            *v = t[i][4 + j] as f32;
        }
    }
    out
}

pub(super) fn uniforme_do_quadro(
    cena: &Cena<'_>,
    e: &Enquadra,
    tem_ceu: bool,
    sol: Option<&super::texturas::SolGpu>,
) -> Vec<f32> {
    let mut u = Vec::with_capacity(QUADRO);
    for col in cena.camera.view_proj.iter().chain(e.sombra_vp.iter()) {
        u.extend_from_slice(col);
    }
    let c = &cena.camera;
    u.extend_from_slice(&[
        c.olho[0],
        c.olho[1],
        c.olho[2],
        f32::from(u8::from(c.perspectiva)),
    ]);
    u.extend_from_slice(&[c.dir_vista[0], c.dir_vista[1], c.dir_vista[2], 0.0]);
    u.extend_from_slice(&[
        cena.chao.unwrap_or(0.0),
        f32::from(u8::from(cena.chao.is_some())),
        e.tan,
        e.texel,
    ]);
    u.extend_from_slice(&e.chao_xz);
    u.extend_from_slice(&[
        e.fundo,
        1.5 * e.texel / e.fundo,
        f32::from(u8::from(e.ha_sombra)),
        f32::from(u8::from(e.ha_chao)),
    ]);
    let n = cena.luzes.len().min(crate::MAX_LUZES);
    u.extend_from_slice(&[cena.exposicao, cena.vista as f32, n as f32, 0.0]);
    u.extend_from_slice(&ph2d_style::wgsl::pack(&cena.estilo));
    u.extend_from_slice(&[cena.raio_da_peca, 0.0, 0.0, 0.0]);
    let foto = cena.foto.filter(|_| tem_ceu);
    match foto {
        Some(f) => {
            u.extend_from_slice(&[f.giro[0], f.giro[1], f.forca, 1.0]);
            u.extend_from_slice(&[
                0.0,
                f.fundo.unwrap_or(0.0),
                f32::from(u8::from(f.fundo.is_some())),
                0.0,
            ]);
        }
        None => u.extend_from_slice(&[1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
    }
    // ⭐ O sol: a radiância da calote com a força do céu e o peso da luz-chave.
    match (foto, sol) {
        (Some(f), Some(s)) => {
            u.extend_from_slice(&[s.dir[0], s.dir[1], s.dir[2], 1.0]);
            let k = f.forca * f.caixa;
            u.extend_from_slice(&[
                s.radiancia[0] * k,
                s.radiancia[1] * k,
                s.radiancia[2] * k,
                0.0,
            ]);
        }
        _ => u.extend_from_slice(&[0.0; 8]),
    }
    for col in e.ceu_vp {
        u.extend_from_slice(&col);
    }
    u.extend_from_slice(&e.ceu);
    for col in inversa(&cena.camera.view_proj) {
        u.extend_from_slice(&col);
    }
    for l in &cena.luzes[..n] {
        u.extend_from_slice(&[l.posicao[0], l.posicao[1], l.posicao[2], 0.0]);
        u.extend_from_slice(&[
            l.radiancia_a_um[0],
            l.radiancia_a_um[1],
            l.radiancia_a_um[2],
            0.0,
        ]);
    }
    u.resize(QUADRO, 0.0);
    u
}

/// As colunas de um material: o `pack` e a textura.
const COLUNAS: u32 = crate::MATERIAL_V4 + crate::TEXTURA_V4;

impl Forward {
    pub(super) fn sobe_materiais(
        &mut self,
        mats: &[[f32; ph2d_material::wgsl::PACKED]],
        texs: &[Option<crate::TexturaMaterial>],
    ) {
        let n = (mats.len() as u32).max(1);
        if self.materiais.as_ref().is_none_or(|(k, _, _)| *k != n) {
            let t = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("ph2d-mesh-forward materiais"),
                size: wgpu::Extent3d {
                    width: COLUNAS,
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
        let lado = self.triplanar.lado();
        let mut dados: Vec<f32> = mats
            .iter()
            .enumerate()
            .flat_map(|(i, m)| {
                let t = texs.get(i).and_then(Option::as_ref);
                m.iter()
                    .copied()
                    .chain(crate::gpu_triplanar::colunas(t, lado))
            })
            .collect();
        dados.resize((n * COLUNAS * 4) as usize, 0.0);
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
                    bytes_per_row: Some(COLUNAS * 16),
                    rows_per_image: Some(n),
                },
                wgpu::Extent3d {
                    width: COLUNAS,
                    height: n,
                    depth_or_array_layers: 1,
                },
            );
        }
    }

    pub(super) fn sobe_objetos(&mut self, objs: &[&crate::Instancia]) {
        let precisa = (objs.len() as u64).max(1);
        if self
            .objetos
            .as_ref()
            .is_none_or(|(cap, _, _)| *cap < precisa)
        {
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
                        size: std::num::NonZeroU64::new(80),
                    }),
                }],
            });
            self.objetos = Some((cap, b, bg));
        }
        let mut dados = vec![0u8; objs.len() * SLOT as usize];
        for (i, o) in objs.iter().enumerate() {
            let at = i * SLOT as usize;
            dados[at..at + 64].copy_from_slice(bytemuck::cast_slice(&o.modelo));
            // O índice da instância na lista do quadro: o contacto não a deixa tapar-se a si.
            dados[at + 64..at + 68].copy_from_slice(&(i as f32).to_le_bytes());
        }
        if let Some((_, b, _)) = &self.objetos
            && !dados.is_empty()
        {
            self.queue.write_buffer(b, 0, &dados);
        }
    }

    /// ⭐ **Prepara o brilho do quadro**: a cadeia deste tamanho (só da 1.ª vez) e os uniformes —
    /// nada compila aqui. Devolve se o brilho corre.
    pub(super) fn prepara_brilho(
        &mut self,
        b: &ph2d_bloom::Bloom,
        tamanho: (u32, u32),
        exposicao: f32,
        vista: u32,
    ) -> bool {
        let corre = self.brilho.is_some()
            && b.contributes()
            && ph2d_bloom::levels_that_fit(tamanho.0 as usize, tamanho.1 as usize) > 0;
        let ecra = crate::gpu_brilho::ecra(b, tamanho, exposicao, vista, corre);
        self.queue.write_buffer(&self.ecra_ub, 0, &ecra);
        let (true, Some(br), Some(alvos)) = (corre, &self.brilho, &mut self.alvos) else {
            return false;
        };
        let cadeia = alvos.cadeia.get_or_insert_with(|| {
            crate::gpu_brilho::Cadeia::nova(
                &self.device,
                br,
                tamanho,
                &alvos.resolvida,
                &self.ecra_bgl,
                &self.ecra_ub,
            )
        });
        self.queue
            .write_buffer(&cadeia.uniforme, 0, &crate::gpu_brilho::passes(b, tamanho));
        true
    }

    pub(super) fn desenha(
        &self,
        cena: &Cena<'_>,
        objs: &[&crate::Instancia],
        (ha_sombra, ha_chao): (bool, bool),
        brilho: bool,
    ) {
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
                    resource: wgpu::BindingResource::TextureView(&self.mapas_sombra[0]),
                },
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: wgpu::BindingResource::TextureView(&self.mapas_sombra[1]),
                },
                wgpu::BindGroupEntry {
                    binding: 11,
                    resource: wgpu::BindingResource::TextureView(&self.mapas_sombra[2]),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&self.compara),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(&self.cobertura.vista),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::Sampler(&self.liso),
                },
                wgpu::BindGroupEntry {
                    binding: 15,
                    resource: wgpu::BindingResource::TextureView(&self.cobertura.ceu.vista),
                },
                wgpu::BindGroupEntry {
                    binding: 16,
                    resource: self.contacto.tabela.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 17,
                    resource: wgpu::BindingResource::TextureView(self.contacto.vista(0)),
                },
                wgpu::BindGroupEntry {
                    binding: 18,
                    resource: wgpu::BindingResource::TextureView(self.contacto.vista(1)),
                },
                wgpu::BindGroupEntry {
                    binding: 19,
                    resource: wgpu::BindingResource::TextureView(self.contacto.vista(2)),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::TextureView(
                        self.foto.as_ref().map_or(&self.foto_vazia, |(_, v)| v),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 12,
                    resource: wgpu::BindingResource::TextureView(&self.triplanar.vista_cor),
                },
                wgpu::BindGroupEntry {
                    binding: 13,
                    resource: wgpu::BindingResource::TextureView(&self.triplanar.vista_nrh),
                },
                wgpu::BindGroupEntry {
                    binding: 14,
                    resource: wgpu::BindingResource::Sampler(&self.triplanar.amostrador),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: wgpu::BindingResource::TextureView(
                        self.sol.as_ref().map_or(&self.sol_vazia, |s| &s.vista),
                    ),
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
                pass.set_vertex_buffer(1, m.curvatura.slice(..));
                pass.set_index_buffer(m.indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..m.n, 0, 0..1);
            }
        };
        if ha_chao {
            let g0s = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ph2d-mesh-forward g0 sombra"),
                layout: &self.g0_sombra_bgl,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.quadro.as_entire_binding(),
                }],
            });
            // ⭐ O mesmo enquadramento em cada nível (ver `gpu_sombra::NIVEIS`).
            for mapa in &self.mapas_sombra {
                let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("ph2d-mesh-forward sombra"),
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
                if ha_sombra {
                    pass.set_pipeline(&self.sombra);
                    pass.set_bind_group(0, &g0s, &[]);
                    desenha_objetos(&mut pass);
                }
            }
            if cena.chao.is_some() {
                self.cobertura.grava(&mut enc, &g0s, desenha_objetos);
            }
        }
        // ⭐ Com o brilho, o MESMO passe escreve a cena-linear num 2.º alvo (o que a cadeia lê).
        let com_brilho = self
            .brilho
            .as_ref()
            .zip(alvos.cadeia.as_ref())
            .filter(|_| brilho);
        {
            let alvo = |view, resolve| {
                Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: Some(resolve),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Discard,
                    },
                })
            };
            let alvos_cor = match com_brilho {
                Some((_, c)) => vec![
                    alvo(&alvos.cor_msaa, &alvos.resolvida),
                    alvo(&c.linear_msaa, &c.linear_vista),
                ],
                None => vec![alvo(&alvos.cor_msaa, &alvos.resolvida)],
            };
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ph2d-mesh-forward cena"),
                color_attachments: &alvos_cor,
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
            let (chao, objeto, fundo) = match com_brilho {
                Some((b, _)) => (&b.chao, &b.objeto, &b.fundo),
                None => (&self.chao, &self.objeto, &self.fundo),
            };
            if self.foto.is_some() && cena.foto.is_some_and(|f| f.fundo.is_some()) {
                pass.set_pipeline(fundo);
                pass.draw(0..3, 0..1);
            }
            if ha_chao && cena.chao.is_some() {
                pass.set_pipeline(chao);
                pass.draw(0..6, 0..1);
            }
            pass.set_pipeline(objeto);
            desenha_objetos(&mut pass);
        }
        self.grava_ecra(&mut enc, alvos, com_brilho);
        self.queue.submit([enc.finish()]);
    }

    /// A cadeia do brilho (se houver) e a codificação para o ecrã.
    pub(super) fn grava_ecra(
        &self,
        enc: &mut wgpu::CommandEncoder,
        alvos: &crate::gpu_alvo::Alvos,
        com_brilho: Option<(&crate::gpu_brilho::Brilho, &crate::gpu_brilho::Cadeia)>,
    ) {
        if let Some((b, c)) = com_brilho {
            c.grava(enc, b);
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
            let grupo = com_brilho.map_or(&alvos.ecra_bind, |(_, c)| &c.ecra_bind);
            pass.set_bind_group(0, grupo, &[]);
            pass.draw(0..3, 0..1);
        }
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
