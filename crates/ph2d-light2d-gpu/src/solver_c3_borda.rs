//! ⭐ W5d — a borda exata da captura que a C3 lê (o `c3_borda` do `c3_borda.wgsl`) e, com
//! `Config::borda_cantos` (W6f), o passe que espalha as cunhas dos vértices (`c3_cantos`). Módulo-filho
//! de `solver_c3.rs` (o tecto de 700 LOC).

use std::sync::atomic::{AtomicBool, Ordering};

use super::*;
use crate::passlist::NOME_LISTA_DOS_CANTOS;

/// A assinatura do `pipeline_em` da C3 (as constantes comuns por cima das próprias).
pub(super) type PipelineEm<'a> = dyn Fn(&str, &[Option<&wgpu::BindGroupLayout>], &str, &[(&str, f64)]) -> wgpu::ComputePipeline
    + 'a;

pub(super) struct Borda {
    pipeline: wgpu::ComputePipeline,
    grupo: wgpu::BindGroup,
    tamanho: [u32; 2],
    cantos: Option<Cantos>,
}

/// ⭐ W6f/W6g: o `c3_cantos` em despacho INDIRETO sobre os blocos que o `c3_borda_cantos` listou (o
/// despacho na captura inteira custava ~0,2 ms com o kernel VAZIO: o lançamento de 245 mil grupos;
/// handoff W6g §6).
struct Cantos {
    lista: wgpu::Buffer,
    zera: wgpu::ComputePipeline,
    espalha: wgpu::ComputePipeline,
    grupo: wgpu::BindGroup,
    /// O preenchimento da `V2` com a sentinela, uma vez (a textura nova é zero: θ₁ = 0 seria cunha).
    inicia: wgpu::ComputePipeline,
    iniciada: AtomicBool,
}

impl Borda {
    /// Lê a cobertura crua `cob`, escreve a da C3 em `b` (grupo 3, bindings 12–15) e, com `cantos`
    /// (`[as cunhas dos vértices, as espalhadas]`), lista os blocos de vértice e espalha-os (16–20).
    pub(super) fn nova(
        e: &Entradas<'_>,
        b: usize,
        cob: usize,
        (cantos, vazia): (Option<[usize; 2]>, Option<usize>),
        pipeline_em: &PipelineEm<'_>,
    ) -> Self {
        let device = e.placa;
        let mut entradas = vec![
            entrada_textura(12, FLOAT),
            entrada_escrita(13, wgpu::TextureFormat::Rgba16Float),
            // ⭐ W6d: a contínua lê a 3×3 por quatro `textureGather`.
            wgpu::BindGroupLayoutEntry {
                binding: 14,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
            entrada_escrita(15, wgpu::TextureFormat::Rgba16Float),
        ];
        let lista = cantos.map(|_| {
            let bytes = e
                .lista
                .buffers
                .iter()
                .find(|b| b.nome == NOME_LISTA_DOS_CANTOS)
                .expect("a lista dos cantos declarada na PassList");
            let buf = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(NOME_LISTA_DOS_CANTOS),
                size: bytes.bytes,
                usage: bytes.usos,
                mapped_at_creation: true,
            });
            // O despacho indireto: `[n, 9, 1]` — cada bloco listado espalha nos 3×3 à volta.
            buf.slice(..12)
                .get_mapped_range_mut()
                .copy_from_slice(bytemuck::cast_slice(&[0u32, 9, 1]));
            buf.unmap();
            buf
        });
        if cantos.is_some() {
            entradas.push(entrada_escrita(18, wgpu::TextureFormat::Rgba16Float));
            entradas.push(armazenamento(19, false));
        }
        let g3 = layout(device, "luz2d c3 borda", &entradas);
        let mut ligacoes = vec![
            vista(12, &e.vistas[cob]),
            vista(13, &e.vistas[b]),
            wgpu::BindGroupEntry {
                binding: 14,
                resource: wgpu::BindingResource::Sampler(e.vizinhos),
            },
            vista(
                15,
                &e.vistas[cantos.map_or_else(|| vazia.expect("a vazia"), |c| c[0])],
            ),
        ];
        if let (Some([_, v2]), Some(l)) = (cantos, &lista) {
            ligacoes.push(vista(18, &e.vistas[v2]));
            ligacoes.push(wgpu::BindGroupEntry {
                binding: 19,
                resource: l.as_entire_binding(),
            });
        }
        let grupo = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("luz2d c3 borda"),
            layout: &g3,
            entries: &ligacoes,
        });
        let cantos = cantos.zip(lista).map(|([v, v2], lista)| {
            let g = layout(
                device,
                "luz2d c3 cantos",
                &[
                    entrada_textura(16, FLOAT),
                    entrada_escrita(17, wgpu::TextureFormat::Rgba16Float),
                    armazenamento(20, true),
                ],
            );
            let grupo = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("luz2d c3 cantos"),
                layout: &g,
                entries: &[
                    vista(16, &e.vistas[v]),
                    vista(17, &e.vistas[v2]),
                    wgpu::BindGroupEntry {
                        binding: 20,
                        resource: lista.as_entire_binding(),
                    },
                ],
            });
            let com_g3 = [None, None, None, Some(&g3)];
            Cantos {
                zera: pipeline_em("luz2d c3 cantos zera", &com_g3, "c3_cantos_zera", &[]),
                inicia: pipeline_em("luz2d c3 cantos inicia", &com_g3, "c3_cantos_inicia", &[]),
                espalha: pipeline_em(
                    "luz2d c3 cantos",
                    &[None, None, None, Some(&g)],
                    "c3_cantos",
                    &[],
                ),
                grupo,
                lista,
                iniciada: AtomicBool::new(false),
            }
        });
        let entrada = if cantos.is_some() {
            "c3_borda_cantos"
        } else {
            "c3_borda"
        };
        Self {
            pipeline: pipeline_em("luz2d c3 borda", &[None, None, None, Some(&g3)], entrada, &[]),
            grupo,
            tamanho: e.lista.texturas[b].tamanho,
            cantos,
        }
    }

    /// Os despachos, antes de tudo o que lê a borda.
    pub(super) fn gravar(&self, p: &mut wgpu::ComputePass<'_>) {
        let [w, h] = self.tamanho;
        let grupos = [w.div_ceil(8), h.div_ceil(8)];
        p.set_bind_group(3, &self.grupo, &[]);
        if let Some(c) = &self.cantos {
            if !c.iniciada.swap(true, Ordering::Relaxed) {
                p.set_pipeline(&c.inicia);
                p.dispatch_workgroups(grupos[0], grupos[1], 1);
            }
            p.set_pipeline(&c.zera);
            p.dispatch_workgroups(1, 1, 1);
        }
        p.set_pipeline(&self.pipeline);
        p.dispatch_workgroups(grupos[0], grupos[1], 1);
        if let Some(c) = &self.cantos {
            p.set_pipeline(&c.espalha);
            p.set_bind_group(3, &c.grupo, &[]);
            p.dispatch_workgroups_indirect(&c.lista, 0);
        }
    }
}

/// Um buffer de armazenamento do passe (`so_leitura`: o `c3_cantos` lê a lista que o `c3_borda` escreve).
fn armazenamento(binding: u32, so_leitura: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage {
                read_only: so_leitura,
            },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}
