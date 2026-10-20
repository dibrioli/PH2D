//! ⭐ **A captura FINA dos opacos** (W5a, F1 + F2 da fila do dono) — uma 2.ª C0, só de cobertura e τ
//! dentro, na grade da captura ([`Config::grade_da_captura`]): o que tapa a luz (oclusores e corpos
//! que brilham — as lâmpadas são luz, não corpo) e os sprites por texel (a camada do vetor incluída),
//! com a MESMA lei do dono e os MESMOS shaders da C0. A luz direta (C3) marcha nela: a captura das
//! cascatas arredondava o canto do oclusor em ½–¾ do texel grosso e perdia o oclusor mais fino que
//! ~1 texel. Módulo-filho do [`super`].

use super::*;
use crate::captura::capturar_opacos;
use crate::config::Grade;
use gravar::subir_geometria;

pub(super) struct CapturaFina {
    uni: wgpu::Buffer,
    /// `[cobertura, τ dentro]` da captura fina.
    alvos: [wgpu::TextureView; 2],
    pipeline: wgpu::RenderPipeline,
    formatos: solver_sprite::FormatosC0,
    geometria: Option<Geometria>,
    sprites: Option<solver_sprite::C0Sprites>,
    fabrica: crate::Fabrica,
    /// ⭐ W5d: a cobertura dos sprites pelo meio (a borda exata da C3 lê esta captura).
    cobertura_pelo_meio: bool,
}

impl Solver {
    /// ⭐ W6e: a borda exata que a C3 lê — por texel da captura que ela marcha, `(cobertura, normal,
    /// distância com sinal no centro)` na contínua (`Config::borda_continua`) — para quem a mede.
    #[must_use]
    pub fn borda_da_captura(&self) -> Option<&wgpu::Texture> {
        crate::passlist::Opcionais::de(&self.cfg)
            .borda
            .map(|b| &self.texturas[b])
    }
}

impl Solver {
    /// ⭐ W6f: as cunhas espalhadas que o `meio` lê junto aos vértices (`Config::borda_cantos`): por texel
    /// da captura, `(θ₁, s₁, θ₂, s₂)` no referencial dele (`θ₁ ≥ 50` = nenhuma; `θ₂ > 5` = côncavo), em
    /// `rgba16float` (8 bytes por texel, linha a linha) e o tamanho da captura. W6g r9: a `V2` é esparsa (um
    /// atlas e a tabela por bloco); esta é a vista DENSA dela, reconstruída para quem a mede.
    #[must_use]
    pub fn cantos_lidos(&self, gpu: &GpuContext) -> Option<(Vec<u8>, [u32; 2])> {
        let op = crate::passlist::Opcionais::de(&self.cfg);
        let (tabela, _) = self.c3.cantos()?;
        let captura = self.texturas[op.borda?].size();
        let atlas = &self.texturas[op.cantos?[1]];
        let t = ler_buffer(gpu, tabela);
        let a = ler_textura(gpu, atlas);
        let (w, h) = (captura.width, captura.height);
        let largura_do_atlas = atlas.width() as usize * 8;
        // `SEM_CUNHA` = 100 em f16.
        let sentinela = 0x5640u16.to_le_bytes();
        let mut v = vec![0u8; w as usize * h as usize * 8];
        for j in 0..h as usize {
            for i in 0..w as usize {
                let k = (j * w as usize + i) * 8;
                let b = (j / 8) * w.div_ceil(8) as usize + i / 8;
                let e = u32::from_le_bytes(t[4 * b..4 * b + 4].try_into().unwrap_or_default());
                if e == 0 {
                    v[k..k + 2].copy_from_slice(&sentinela);
                    continue;
                }
                let s = (e - 1) as usize;
                let (x, y) = ((s % 128) * 8 + i % 8, (s / 128) * 8 + j % 8);
                let o = y * largura_do_atlas + x * 8;
                v[k..k + 8].copy_from_slice(&a[o..o + 8]);
            }
        }
        Some((v, [w, h]))
    }

    /// ⭐ W6g r9 — a recusa DITA do teto dos cantos (`Config::cantos_por_blocos`) no último quadro: os
    /// blocos listados que ficaram sem slot da `V` e os que ficaram sem slot da `V2` (ficam na lei W6d).
    #[must_use]
    pub fn cantos_recusados(&self, gpu: &GpuContext) -> Option<[u32; 2]> {
        let (_, c) = self.c3.cantos()?;
        let b = ler_buffer(gpu, c);
        let n = |k: usize| u32::from_le_bytes(b[4 * k..4 * k + 4].try_into().unwrap_or_default());
        Some([n(1), n(2)])
    }
}

impl CapturaFina {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn novo(
        device: crate::Placa<'_>,
        modulo: &wgpu::ShaderModule,
        layout: &wgpu::PipelineLayout,
        uni: wgpu::Buffer,
        alvos: [wgpu::TextureView; 2],
        formatos: solver_sprite::FormatosC0,
        g: Grade,
        fila: &wgpu::Queue,
        cobertura_pelo_meio: bool,
    ) -> Self {
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("luz2d c0 captura fina"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: modulo,
                entry_point: Some("vs"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: modulo,
                entry_point: Some("fs"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &solver_sprite::alvos_c0_opacos(formatos),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        fila.write_buffer(
            &uni,
            0,
            bytemuck::bytes_of(&UniformeC0 {
                tamanho: [g.largura as f32, g.altura as f32],
                _pad: [0.0; 2],
            }),
        );
        Self {
            uni,
            alvos,
            pipeline,
            formatos,
            geometria: None,
            sprites: None,
            fabrica: device.fabrica.clone(),
            cobertura_pelo_meio,
        }
    }

    /// Captura `cena` e `sprites` na grade `g` (a da captura) e acende os `ladrilhos` das caixas que
    /// desenha (W5q). Devolve os bytes de geometria que subiram.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn gravar(
        &mut self,
        gpu: &GpuContext,
        enc: &mut wgpu::CommandEncoder,
        c0_layout: &wgpu::BindGroupLayout,
        cena: &CenaDeLuz,
        sprites: &CapturaDeSprites<'_>,
        g: Grade,
        tempos: Option<&Tempos>,
        ladrilhos: Option<&mut crate::ladrilhos::Ladrilhos>,
    ) -> u64 {
        let cap = capturar_opacos(cena, &g, &sprites.substituidos);
        subir_geometria(
            gpu,
            (c0_layout, &self.uni),
            &mut self.geometria,
            ["luz2d/buf/formas_fina", "luz2d/buf/vertices_fina"],
            &cap.formas,
            &cap.vertices,
        );
        let (formas_sprite, lotes) = captura_sprite::na_grade(sprites, &g);
        if let Some(l) = ladrilhos {
            let caixas = cap.formas.iter().map(|f| f.caixa);
            for c in caixas.chain(formas_sprite.iter().map(|f| f.caixa)) {
                l.caixa_da_captura(c);
            }
        }
        let lotes = if formas_sprite.is_empty() {
            Vec::new()
        } else {
            let formatos = self.formatos;
            self.sprites
                .get_or_insert_with(|| {
                    solver_sprite::C0Sprites::novo(
                        crate::Placa {
                            device: &gpu.device,
                            fabrica: &self.fabrica,
                            conta: None,
                        },
                        &solver_sprite::alvos_c0_opacos(formatos),
                        "luz2d/buf/sprites_fina",
                        self.cobertura_pelo_meio,
                    )
                })
                .preparar(gpu, &self.uni, &formas_sprite, &lotes, &sprites.texturas)
        };
        let alvo = |v, cor| {
            Some(wgpu::RenderPassColorAttachment {
                view: v,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(cor),
                    store: wgpu::StoreOp::Store,
                },
            })
        };
        let mut p = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("luz2d C0 captura fina"),
            color_attachments: &[
                None,
                alvo(&self.alvos[0], wgpu::Color::TRANSPARENT),
                alvo(&self.alvos[1], wgpu::Color::WHITE),
                None,
            ],
            depth_stencil_attachment: None,
            timestamp_writes: tempos.map(|t| t.escrita_render(tempos::C0_FINA)),
            occlusion_query_set: None,
            multiview_mask: None,
        });
        let total = u32::try_from(cap.formas.len()).expect("formas cabem em u32");
        if total > 0 {
            let geo = self.geometria.as_ref().expect("subida acima");
            p.set_pipeline(&self.pipeline);
            p.set_bind_group(0, &geo.grupo, &[]);
            p.draw(0..4, 0..total);
        }
        if let Some(s) = &self.sprites {
            s.desenhar(&mut p, &lotes);
        }
        cap.relatorio.bytes_da_geometria
    }

    pub(super) fn bytes_residentes(&self) -> u64 {
        self.geometria.as_ref().map_or(0, |g| {
            (g.cap_formas * size_of::<Forma>() + g.cap_vertices * size_of::<[f32; 2]>()) as u64
        }) + self
            .sprites
            .as_ref()
            .map_or(0, solver_sprite::C0Sprites::bytes_residentes)
    }
}

/// ⭐ W6g r9: os bytes de `b` (cópia e mapa; para quem mede, nunca no quadro).
fn ler_buffer(gpu: &GpuContext, b: &wgpu::Buffer) -> Vec<u8> {
    let d = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("luz2d leitura"),
        size: b.size(),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    enc.copy_buffer_to_buffer(b, 0, &d, 0, b.size());
    gpu.queue.submit([enc.finish()]);
    d.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let v = d.slice(..).get_mapped_range().to_vec();
    d.unmap();
    v
}

/// Os texels de `t` (`rgba16float`), linha a linha sem o alinhamento da cópia.
fn ler_textura(gpu: &GpuContext, t: &wgpu::Texture) -> Vec<u8> {
    let (w, h) = (t.width(), t.height());
    let linha = (8 * w).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    let d = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("luz2d leitura"),
        size: u64::from(linha * h),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    enc.copy_texture_to_buffer(
        t.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &d,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(linha),
                rows_per_image: None,
            },
        },
        t.size(),
    );
    gpu.queue.submit([enc.finish()]);
    d.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let m = d.slice(..).get_mapped_range();
    let mut v = Vec::with_capacity((8 * w * h) as usize);
    for j in 0..h {
        let o = (j * linha) as usize;
        v.extend_from_slice(&m[o..o + 8 * w as usize]);
    }
    v
}
