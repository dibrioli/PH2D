//! ⭐ W5d — a borda exata da captura que a C3 lê (o `c3_borda` do `c3_borda.wgsl`) e, com
//! `Config::borda_cantos` (W6f), os passes das cunhas dos vértices (`c3_borda_vertices`, `c3_cantos`).
//! Módulo-filho de `solver_c3.rs` (o tecto de 700 LOC).

use std::sync::atomic::{AtomicBool, Ordering};

use super::*;
use crate::passlist::{NOME_LISTA_DOS_CANTOS, NOME_MARCAS_DOS_BLOCOS};

/// A assinatura do `pipeline_em` da C3 (as constantes comuns por cima das próprias).
pub(super) type PipelineEm<'a> = dyn Fn(&str, &[Option<&wgpu::BindGroupLayout>], &str, &[(&str, f64)]) -> wgpu::ComputePipeline
    + 'a;

pub(super) struct Borda {
    pipeline: wgpu::ComputePipeline,
    /// O grupo 3 da borda; com os cantos, um por lista (o `c3_borda_adia` acrescenta à deste quadro).
    grupos: Vec<wgpu::BindGroup>,
    tamanho: [u32; 2],
    cantos: Option<Cantos>,
}

/// ⭐ W6f/W6g: as cunhas só nos blocos que o `c3_borda_adia` listou, em despachos INDIRETOS (o despacho
/// na captura inteira custava ~0,2 ms com o kernel VAZIO), a `V2` válida em toda a captura pela lista do
/// quadro anterior (`c3_cantos_limpa`) e — r8 — a `pela_cunha` fora do kernel da captura (55–61 registos
/// contra 32; handoff W6g §21).
struct Cantos {
    /// As duas listas, a 2.ª em `metade`.
    lista: wgpu::Buffer,
    metade: u64,
    zera: wgpu::ComputePipeline,
    limpa: wgpu::ComputePipeline,
    vertices: wgpu::ComputePipeline,
    espalha: wgpu::ComputePipeline,
    /// Sobre cada lista: o do `c3_borda_vertices`, o do `c3_cantos_limpa` (e `inicia`) e o do `c3_cantos`.
    grupos_vertices: [wgpu::BindGroup; 2],
    grupos_limpa: [wgpu::BindGroup; 2],
    grupos: [wgpu::BindGroup; 2],
    /// O preenchimento da `V` e da `V2` com a sentinela, uma vez (a textura nova é zero: θ₁ = 0 seria cunha).
    inicia: wgpu::ComputePipeline,
    iniciada: AtomicBool,
    /// A lista deste quadro (alterna a cada gravação).
    lado: AtomicBool,
}

impl Borda {
    /// Lê a cobertura crua `cob`, escreve a da C3 em `b` (grupo 3, bindings 12–15) e, com `cantos`
    /// (`[as cunhas dos vértices, as espalhadas]`), lista os blocos de vértice e espalha-os (16–21).
    pub(super) fn nova(
        e: &Entradas<'_>,
        b: usize,
        cob: usize,
        (cantos, vazia): (Option<[usize; 2]>, Option<usize>),
        pipeline_em: &PipelineEm<'_>,
    ) -> Self {
        let device = e.placa;
        let base = [
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
        let declarado = |nome: &str| {
            e.lista
                .buffers
                .iter()
                .find(|b| b.nome == nome)
                .unwrap_or_else(|| panic!("{nome} declarado na PassList"))
        };
        let lista = cantos.map(|_| {
            let bytes = declarado(NOME_LISTA_DOS_CANTOS);
            let buf = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(NOME_LISTA_DOS_CANTOS),
                size: bytes.bytes,
                usage: bytes.usos,
                mapped_at_creation: true,
            });
            let metade = bytes.bytes / 2;
            // O despacho indireto de cada lista: `[n, 9, 1]` — cada bloco listado cobre os 3×3 à volta.
            for o in [0, metade] {
                buf.slice(o..o + 12)
                    .get_mapped_range_mut()
                    .copy_from_slice(bytemuck::cast_slice(&[0u32, 9, 1]));
            }
            buf.unmap();
            let m = declarado(NOME_MARCAS_DOS_BLOCOS);
            let marcas = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(NOME_MARCAS_DOS_BLOCOS),
                size: m.bytes,
                usage: m.usos,
                mapped_at_creation: false,
            });
            (buf, metade, marcas)
        });
        let mut entradas = base.to_vec();
        if cantos.is_some() {
            entradas.extend([armazenamento(19, false), armazenamento(21, false)]);
        }
        let g3 = layout(device, "luz2d c3 borda", &entradas);
        let v = cantos.map_or_else(|| vazia.expect("a vazia"), |c| c[0]);
        let ligacoes = [
            vista(12, &e.vistas[cob]),
            vista(13, &e.vistas[b]),
            wgpu::BindGroupEntry {
                binding: 14,
                resource: wgpu::BindingResource::Sampler(e.vizinhos),
            },
            vista(15, &e.vistas[v]),
        ];
        let grupo_da_borda = |lado: Option<u64>| {
            let mut l = ligacoes.to_vec();
            if let (Some(lado), Some((buf, metade, marcas))) = (lado, &lista) {
                l.push(metade_em(buf, *metade, lado, 19));
                l.push(wgpu::BindGroupEntry {
                    binding: 21,
                    resource: marcas.as_entire_binding(),
                });
            }
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("luz2d c3 borda"),
                layout: &g3,
                entries: &l,
            })
        };
        let grupos = if cantos.is_some() {
            vec![grupo_da_borda(Some(0)), grupo_da_borda(Some(1))]
        } else {
            vec![grupo_da_borda(None)]
        };
        let cantos = cantos.zip(lista).map(|([v, v2], (lista, metade, marcas))| {
            let marcas = wgpu::BindGroupEntry {
                binding: 21,
                resource: marcas.as_entire_binding(),
            };
            let (g_vertices, grupos_vertices) = par(
                device,
                "luz2d c3 borda vertices",
                &[base[0], base[1], base[2], base[3], armazenamento(20, true)],
                [0, 1].map(|lado| {
                    let mut l = ligacoes.to_vec();
                    l.push(metade_em(&lista, metade, lado, 20));
                    l
                }),
            );
            let (g_limpa, grupos_limpa) = par(
                device,
                "luz2d c3 cantos limpa",
                &[
                    entrada_escrita(15, wgpu::TextureFormat::Rgba16Float),
                    entrada_escrita(17, wgpu::TextureFormat::Rgba16Float),
                    armazenamento(20, true),
                    armazenamento(21, false),
                ],
                [0, 1].map(|lado| {
                    vec![
                        vista(15, &e.vistas[v]),
                        vista(17, &e.vistas[v2]),
                        metade_em(&lista, metade, lado, 20),
                        marcas.clone(),
                    ]
                }),
            );
            let (g, grupos) = par(
                device,
                "luz2d c3 cantos",
                &[
                    entrada_textura(16, FLOAT),
                    entrada_escrita(17, wgpu::TextureFormat::Rgba16Float),
                    armazenamento(20, true),
                ],
                [0, 1].map(|lado| {
                    vec![
                        vista(16, &e.vistas[v]),
                        vista(17, &e.vistas[v2]),
                        metade_em(&lista, metade, lado, 20),
                    ]
                }),
            );
            let com = |g: &wgpu::BindGroupLayout, rotulo: &str, entrada: &str| {
                pipeline_em(rotulo, &[None, None, None, Some(g)], entrada, &[])
            };
            Cantos {
                zera: com(&g3, "luz2d c3 cantos zera", "c3_cantos_zera"),
                inicia: com(&g_limpa, "luz2d c3 cantos inicia", "c3_cantos_inicia"),
                limpa: com(&g_limpa, "luz2d c3 cantos limpa", "c3_cantos_limpa"),
                vertices: com(&g_vertices, "luz2d c3 borda vertices", "c3_borda_vertices"),
                espalha: com(&g, "luz2d c3 cantos", "c3_cantos"),
                grupos_vertices,
                grupos_limpa,
                grupos,
                lista,
                metade,
                iniciada: AtomicBool::new(false),
                lado: AtomicBool::new(false),
            }
        });
        let entrada = if cantos.is_some() {
            "c3_borda_adia"
        } else {
            "c3_borda"
        };
        Self {
            pipeline: pipeline_em(
                "luz2d c3 borda",
                &[None, None, None, Some(&g3)],
                entrada,
                &[],
            ),
            grupos,
            tamanho: e.lista.texturas[b].tamanho,
            cantos,
        }
    }

    /// Os despachos, antes de tudo o que lê a borda.
    pub(super) fn gravar(&self, p: &mut wgpu::ComputePass<'_>) {
        let [w, h] = self.tamanho;
        let grupos = [w.div_ceil(8), h.div_ceil(8)];
        let lado = self.cantos.as_ref().map_or(0, |c| {
            usize::from(c.lado.fetch_xor(true, Ordering::Relaxed))
        });
        if let Some(c) = &self.cantos {
            p.set_pipeline(&c.zera);
            p.set_bind_group(3, &self.grupos[lado], &[]);
            p.dispatch_workgroups(1, 1, 1);
            if !c.iniciada.swap(true, Ordering::Relaxed) {
                p.set_pipeline(&c.inicia);
                p.set_bind_group(3, &c.grupos_limpa[lado], &[]);
                p.dispatch_workgroups(grupos[0], grupos[1], 1);
            }
            // Os blocos do quadro anterior voltam à sentinela (`V`, `V2`) e à marca 0 antes de se
            // listarem os deste.
            let anterior = 1 - lado;
            p.set_pipeline(&c.limpa);
            p.set_bind_group(3, &c.grupos_limpa[anterior], &[]);
            p.dispatch_workgroups_indirect(&c.lista, anterior as u64 * c.metade);
        }
        p.set_pipeline(&self.pipeline);
        p.set_bind_group(3, &self.grupos[lado], &[]);
        p.dispatch_workgroups(grupos[0], grupos[1], 1);
        if let Some(c) = &self.cantos {
            let o = lado as u64 * c.metade;
            p.set_pipeline(&c.vertices);
            p.set_bind_group(3, &c.grupos_vertices[lado], &[]);
            p.dispatch_workgroups_indirect(&c.lista, o);
            p.set_pipeline(&c.espalha);
            p.set_bind_group(3, &c.grupos[lado], &[]);
            p.dispatch_workgroups_indirect(&c.lista, o);
        }
    }
}

/// O layout `rotulo` e um grupo sobre cada lista (`ligacoes[lado]`).
fn par(
    device: crate::Placa<'_>,
    rotulo: &str,
    entradas: &[wgpu::BindGroupLayoutEntry],
    ligacoes: [Vec<wgpu::BindGroupEntry<'_>>; 2],
) -> (wgpu::BindGroupLayout, [wgpu::BindGroup; 2]) {
    let g = layout(device, rotulo, entradas);
    let grupos = ligacoes.map(|l| {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(rotulo),
            layout: &g,
            entries: &l,
        })
    });
    (g, grupos)
}

/// A lista `lado` (0 ou 1) das duas do buffer `l`, cada uma com `metade` bytes.
fn metade_em(l: &wgpu::Buffer, metade: u64, lado: u64, binding: u32) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: l,
            offset: lado * metade,
            size: wgpu::BufferSize::new(metade),
        }),
    }
}

/// Um buffer de armazenamento do passe (`so_leitura`: os indiretos leem a lista que o `c3_borda_adia` escreve).
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
