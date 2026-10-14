//! **Gravar um quadro do solver** — a metade de [`super`] que corre a cada quadro: sobe a geometria,
//! grava C0 (formas e sprites) → C1 → C2 no encoder de quem chama, e move a grade no mundo.

use super::*;

/// Sobe `formas` e `vertices` para `geo` — cresce por potências de dois, nunca encolhe —, com os
/// rótulos `nomes` (o G5 conta a geometria por eles). A da C0 e a da captura fina (W5a).
pub(super) fn subir_geometria(
    gpu: &GpuContext,
    (layout, uni): (&wgpu::BindGroupLayout, &wgpu::Buffer),
    geo: &mut Option<Geometria>,
    nomes: [&'static str; 2],
    formas: &[Forma],
    vertices: &[[f32; 2]],
) {
    let device = &gpu.device;
    let precisa = |cap: usize, n: usize| cap < n.max(1);
    let refazer = geo.as_ref().is_none_or(|g| {
        precisa(g.cap_formas, formas.len()) || precisa(g.cap_vertices, vertices.len())
    });
    if refazer {
        let (cf, cv) = geo
            .as_ref()
            .map_or((1, 1), |g| (g.cap_formas, g.cap_vertices));
        let cap_formas = cf.max(formas.len().next_power_of_two());
        let cap_vertices = cv.max(vertices.len().next_power_of_two());
        let usos = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        let b_formas = criar_buffer(
            device,
            nomes[0],
            (cap_formas * size_of::<Forma>()) as u64,
            usos,
        );
        let b_vertices = criar_buffer(
            device,
            nomes[1],
            (cap_vertices * size_of::<[f32; 2]>()) as u64,
            usos,
        );
        let grupo = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("luz2d c0"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uni.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: b_formas.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: b_vertices.as_entire_binding(),
                },
            ],
        });
        *geo = Some(Geometria {
            formas: b_formas,
            vertices: b_vertices,
            cap_formas,
            cap_vertices,
            grupo,
        });
    }
    let g = geo.as_ref().expect("geometria criada acima");
    if !formas.is_empty() {
        gpu.queue
            .write_buffer(&g.formas, 0, bytemuck::cast_slice(formas));
    }
    if !vertices.is_empty() {
        gpu.queue
            .write_buffer(&g.vertices, 0, bytemuck::cast_slice(vertices));
    }
}

impl Solver {
    /// Grava um quadro inteiro em `enc` (as subidas vão pela fila, antes do submit de `enc`).
    pub fn gravar(
        &mut self,
        gpu: &GpuContext,
        enc: &mut wgpu::CommandEncoder,
        cena: &CenaDeLuz,
    ) -> RelatorioC0 {
        self.gravar_com_sprites(gpu, enc, cena, &CapturaDeSprites::default())
    }

    /// [`Self::gravar`] com os SPRITES por texel (W3): o quad de cada um sai da C0 das formas e a
    /// textura dele entra pela pipeline irmã, no MESMO passe e nos mesmos alvos.
    pub fn gravar_com_sprites(
        &mut self,
        gpu: &GpuContext,
        enc: &mut wgpu::CommandEncoder,
        cena: &CenaDeLuz,
        sprites: &CapturaDeSprites<'_>,
    ) -> RelatorioC0 {
        let diretas = crate::direta::lampadas_diretas(cena);
        let cap = capturar_sem(cena, &self.cfg.grade, &sprites.substituidos, &diretas);
        // ⭐ W5b (F4): o mundo fora da grade no caminho até cada lâmpada direta.
        let distante = if self.cfg.distantes {
            crate::distantes::montar(
                cena,
                &self.cfg.grade,
                &diretas,
                sprites,
                self.cfg.mascara_px,
            )
        } else {
            crate::distantes::MundoDistante::default()
        };
        gpu.queue.write_buffer(
            &self.lampadas,
            0,
            &crate::direta::bytes_na_grade(cena, &diretas, &self.cfg.grade, &distante.listas),
        );
        if self.distantes.subir(
            gpu,
            enc,
            &distante,
            (&sprites.texturas, self.tempos.as_ref()),
        ) {
            self.lampadas_grupo = distantes::grupo_das_lampadas(
                &gpu.device,
                &self.lampadas_layout,
                &self.lampadas,
                &self.distantes,
            );
        }
        subir_geometria(
            gpu,
            (&self.c0_layout, &self.uni_c0),
            &mut self.geometria,
            ["luz2d/buf/formas", "luz2d/buf/vertices"],
            &cap.formas,
            &cap.vertices,
        );
        let (formas_sprite, lotes) = captura_sprite::na_grade(sprites, &self.cfg.grade);
        let lotes = if formas_sprite.is_empty() {
            Vec::new()
        } else {
            let formatos = self.formatos_c0;
            self.c0_sprites
                .get_or_insert_with(|| {
                    solver_sprite::C0Sprites::novo(
                        crate::Placa {
                            device: &gpu.device,
                            fabrica: &self.fabrica,
                            conta: None,
                        },
                        &solver_sprite::alvos_c0(formatos),
                        "luz2d/buf/sprites",
                        // A C3 lê esta captura (sem a fina) com a borda exata: a cobertura pelo meio.
                        self.cfg.borda_exata && !self.cfg.usa_captura_fina(),
                    )
                })
                .preparar(gpu, &self.uni_c0, &formas_sprite, &lotes, &sprites.texturas)
        };

        let mut uni = vec![0u8; (PASSO_UNIFORME * u64::from(self.n)) as usize];
        for i in 0..self.n {
            let (t_inicio, t_fim) = self.cfg.intervalo(i);
            let recuo = if i == 0 {
                0.0
            } else {
                let (a, b) = self.cfg.intervalo(i - 1);
                self.cfg.passo_perto.min(b - a)
            };
            let [r, g, b] = cena.ceu;
            let u = UniformeCascata {
                tamanho: [self.cfg.grade.largura, self.cfg.grade.altura],
                nivel: i,
                topo: u32::from(i + 1 == self.n),
                t_inicio,
                t_fim,
                texel_m: self.cfg.grade.texel_m as f32,
                fator_jfa: self.cfg.fator_jfa,
                recuo,
                _pad: [0.0; 3],
                ceu: [r, g, b, 0.0],
            };
            let o = i as usize * PASSO_UNIFORME as usize;
            uni[o..o + size_of::<UniformeCascata>()].copy_from_slice(bytemuck::bytes_of(&u));
        }
        gpu.queue.write_buffer(&self.uni_cascatas, 0, &uni);

        let geo = self
            .geometria
            .as_ref()
            .expect("subir_geometria cria a geometria");
        {
            let mut p = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("luz2d C0 captura"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.alvos_c0[0],
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.alvos_c0[1],
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.alvos_c0[2],
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    // ⭐ W5c: o albedo do rebote.
                    self.rebote
                        .as_ref()
                        .map(|r| wgpu::RenderPassColorAttachment {
                            view: &r.alvo_albedo,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                store: wgpu::StoreOp::Store,
                            },
                        }),
                ],
                depth_stencil_attachment: None,
                timestamp_writes: self.tempos.as_ref().map(|t| t.escrita_render(tempos::C0)),
                occlusion_query_set: None,
                multiview_mask: None,
            });
            let total = u32::try_from(cap.formas.len()).expect("formas cabem em u32");
            if total > 0 {
                p.set_pipeline(&self.c0);
                p.set_bind_group(0, &geo.grupo, &[]);
                p.draw(0..4, 0..total);
            }
            if let Some(s) = &self.c0_sprites {
                s.desenhar(&mut p, &lotes);
            }
        }
        // ⭐ W5a: a captura FINA dos opacos (a luz direta marcha nela).
        // ⭐ W5q: os ladrilhos do anel — as caixas que a captura fina desenha (sem ela, todos).
        let mut bytes_finos = 0;
        let por_caixas = self.cfg.anel_por_ladrilhos && self.fina.is_some();
        if let Some(l) = &mut self.ladrilhos {
            l.comecar(!por_caixas);
        }
        if let Some(f) = &mut self.fina {
            bytes_finos = f.gravar(
                gpu,
                enc,
                &self.c0_layout,
                cena,
                sprites,
                self.cfg.grade_da_captura(),
                self.tempos.as_ref(),
                self.ladrilhos.as_mut().filter(|_| por_caixas),
            );
        } else if let Some(t) = &self.tempos {
            // Sem ela, um passe vazio escreve os carimbos dela (valem 0; nenhum fica por escrever).
            enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("luz2d C0 captura fina (vazia)"),
                timestamp_writes: Some(t.escrita_compute(tempos::C0_FINA)),
            });
        }
        if let Some(l) = &mut self.ladrilhos {
            l.enviar(&gpu.queue);
        }
        // ⭐ W5c: a emissão ganha a luz que cada superfície refletora recebeu no quadro anterior.
        // Sem nenhum corpo que reflita, os dois passes do rebote saltam (e a fonte esquece-se).
        let reflete = self.rebote.is_some() && rebote::ha_refletor(cena, sprites);
        // ⭐ W5d: o 2.º rebote do quadro repõe a emissão de BASE (a soma do rebote é aditiva).
        let base = crate::passlist::Opcionais::de(&self.cfg)
            .emissao_base
            .filter(|_| reflete && self.hrc.is_some());
        if let Some(b) = base {
            enc.copy_texture_to_texture(
                self.texturas[EMISSAO].as_image_copy(),
                self.texturas[b].as_image_copy(),
                self.texturas[EMISSAO].size(),
            );
        }
        match self.rebote.as_mut() {
            Some(r) if reflete => {
                r.gravar(
                    gpu,
                    enc,
                    &self.cfg.grade,
                    &self.alvos_c0[0],
                    self.tempos.as_ref(),
                );
            }
            r => {
                if let Some(r) = r {
                    r.anterior = None;
                }
                rebote::carimbos_vazios(enc, self.tempos.as_ref(), tempos::C0_REBOTE);
            }
        }
        {
            let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("luz2d C1 campo"),
                timestamp_writes: self.tempos.as_ref().map(|t| t.escrita_compute(tempos::C1)),
            });
            let [w, h] = self.cfg.grade_jfa();
            let grupos = [w.div_ceil(8), h.div_ceil(8)];
            p.set_pipeline(&self.jfa_semente);
            p.set_bind_group(0, &self.jfa_grupos[0], &[]);
            p.dispatch_workgroups(grupos[0], grupos[1], 1);
            p.set_pipeline(&self.jfa_salto);
            for g in &self.jfa_grupos[1..] {
                p.set_bind_group(0, g, &[]);
                p.dispatch_workgroups(grupos[0], grupos[1], 1);
            }
        }
        let grupos = [
            self.cfg.grade.largura.div_ceil(8),
            self.cfg.grade.altura.div_ceil(8),
        ];
        // ⭐ W5c: a holográfica escreve a c0 num passe só (os carimbos da cascata 0); os das outras
        // cascatas ficam em passes vazios.
        if let Some(h) = &self.hrc {
            if h.gravar_por_fase(enc, 0) {
                rebote::carimbos_vazios(enc, self.tempos.as_ref(), tempos::da_cascata(0));
            } else {
                let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("luz2d C2 holográfica"),
                    timestamp_writes: self
                        .tempos
                        .as_ref()
                        .map(|t| t.escrita_compute(tempos::da_cascata(0))),
                });
                h.gravar(&mut p);
            }
        }
        // ⭐ W5d: os rebotes a mais do quadro — a fonte com esta cascata, a emissão de base de novo com a
        // soma da fonte nova, e a cascata outra vez (o campo de distância não muda: é da cobertura).
        // Os carimbos de cada cascata a mais vão para os da cascata `k` (vazios com a holográfica).
        let mut a_mais = 0;
        if let (Some(b), Some(h), Some(r)) = (base, &self.hrc, &self.rebote) {
            for k in 1..self.cfg.rebotes_por_quadro.min(self.n) {
                a_mais = k;
                self.c3.gravar_so_a_fonte(
                    gpu,
                    enc,
                    (&self.lampadas_grupo, !distante.indices.is_empty()),
                );
                enc.copy_texture_to_texture(
                    self.texturas[b].as_image_copy(),
                    self.texturas[EMISSAO].as_image_copy(),
                    self.texturas[EMISSAO].size(),
                );
                r.gravar_na_mesma(enc, &self.alvos_c0[0]);
                if h.gravar_por_fase(enc, k) {
                    rebote::carimbos_vazios(enc, self.tempos.as_ref(), tempos::da_cascata(k));
                    continue;
                }
                let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("luz2d C2 holográfica (rebote a mais)"),
                    timestamp_writes: self
                        .tempos
                        .as_ref()
                        .map(|t| t.escrita_compute(tempos::da_cascata(k))),
                });
                h.gravar(&mut p);
            }
        }
        if let Some(h) = &self.hrc {
            h.resolver_fases(enc);
        }
        for i in (0..self.n).rev() {
            if self.hrc.is_some() {
                if i > a_mais {
                    rebote::carimbos_vazios(enc, self.tempos.as_ref(), tempos::da_cascata(i));
                }
                continue;
            }
            let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("luz2d C2 cascata"),
                timestamp_writes: self
                    .tempos
                    .as_ref()
                    .map(|t| t.escrita_compute(tempos::da_cascata(i))),
            });
            p.set_pipeline(&self.rc);
            p.set_bind_group(0, &self.rc_grupos[i as usize], &[]);
            p.set_bind_group(1, &self.lampadas_grupo, &[]);
            p.dispatch_workgroups(grupos[0], grupos[1], 1);
        }
        self.c3.gravar(
            gpu,
            enc,
            (&self.lampadas_grupo, !distante.indices.is_empty(), reflete),
            self.tempos.as_ref(),
        );
        match &self.miolo {
            Some(m) => m.gravar(
                gpu,
                enc,
                self.cfg.grade_da_fluencia().texel_m,
                self.tempos.as_ref(),
            ),
            None => rebote::carimbos_vazios(enc, self.tempos.as_ref(), tempos::C3_MIOLO),
        }
        // A fonte que a C3 acabou de escrever está nesta grade: o quadro seguinte lê-a no mundo.
        if let Some(r) = self.rebote.as_mut().filter(|_| reflete) {
            r.anterior = Some(self.cfg.grade);
        }
        if let Some(t) = &self.tempos {
            t.resolver(enc);
        }
        let mut r = cap.relatorio;
        r.bytes_da_geometria += bytes_finos;
        r.distantes = distante.objetos.len();
        r.mascaras = distante.mascaras.len();
        r.lado_da_mascara = distante.lado;
        r
    }

    /// Move a grade no MUNDO (pan, oitava de zoom) SEM realocar: as dimensões ficam, e a origem e o
    /// texel só entram pela captura e pelos uniformes de cada quadro (`gravar`). Quem ancora a
    /// origem (G11) é o chamador.
    ///
    /// # Errors
    /// [`ErroSolver::ConfigInvalida`] com origem ou texel não finitos (a grade fica a anterior).
    pub fn reposicionar(&mut self, origem_m: [f64; 2], texel_m: f64) -> Result<(), ErroSolver> {
        let mut cfg = self.cfg;
        cfg.grade.origem_m = origem_m;
        cfg.grade.texel_m = texel_m;
        cfg.validar()?;
        self.cfg = cfg;
        Ok(())
    }

    /// ⭐ Só a REGIÃO `[inicio, fim)` da grade FINA (em texels dela) se calcula a partir do
    /// próximo quadro — a da janela (W4b: na vista de fábrica 82 % da grade fica fora dela); o
    /// resto guarda o último valor. Por omissão, a grade inteira.
    pub fn regiao_fina(&mut self, inicio: [u32; 2], fim: [u32; 2]) {
        self.c3.regiao(&self.cfg, inicio, fim);
        if let Some(m) = &mut self.miolo {
            m.regiao(inicio, fim);
        }
        if let Some(l) = &mut self.ladrilhos {
            l.janela(inicio, fim);
        }
    }

    /// ⭐ W5q — INSTRUMENTO (o controlo de `medida_luz2d_w5q_ladrilhos`): os ladrilhos do anel só na
    /// janela, sem os corpos — a fluência TEM de mudar onde um corpo atravessa a janela.
    #[doc(hidden)]
    pub fn ladrilhos_so_da_janela(&mut self) {
        if let Some(l) = &mut self.ladrilhos {
            l.so_a_janela = true;
        }
    }

    /// Um quadro: grava e submete.
    pub fn quadro(&mut self, gpu: &GpuContext, cena: &CenaDeLuz) -> RelatorioC0 {
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("luz2d quadro"),
            });
        let r = self.gravar(gpu, &mut enc, cena);
        gpu.queue.submit([enc.finish()]);
        r
    }
}
