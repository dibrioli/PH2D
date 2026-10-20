//! ⭐ W5d — a borda exata da captura que a C3 lê (o `c3_borda` do `c3_borda.wgsl`) e, com
//! `Config::borda_cantos` (W6f), os passes das cunhas dos vértices (`c3_borda_vertices`, `c3_cantos`).
//! Módulo-filho de `solver_c3.rs` (o tecto de 700 LOC).

use std::sync::atomic::{AtomicBool, Ordering};

use super::*;
use crate::passlist::{
    NOME_CONTADORES_DOS_CANTOS, NOME_LISTA_COM_CUNHA, NOME_LISTA_DOS_CANTOS, NOME_MARCAS_DOS_BLOCOS,
};

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
/// na captura inteira custava ~0,2 ms com o kernel VAZIO), a `pela_cunha` fora do kernel da captura (r8:
/// 55–61 registos contra 32) e — r9 — a `V` e a `V2` ESPARSAS: dois atlas de blocos 8×8 e a tabela por
/// bloco que o `meio` lê (handoff W6g §25–§26).
struct Cantos {
    /// As duas listas dos candidatos, a 2.ª em `metade`; as duas dos listados com cunha (o mesmo formato).
    lista: wgpu::Buffer,
    com_cunha: wgpu::Buffer,
    metade: u64,
    /// `[slots da V2, recusados sem V, recusados sem V2, _]` do último quadro.
    contadores: wgpu::Buffer,
    zera: wgpu::ComputePipeline,
    limpa: wgpu::ComputePipeline,
    vertices: wgpu::ComputePipeline,
    espalha: wgpu::ComputePipeline,
    /// Sobre cada lista: o do `c3_borda_vertices`, o do `c3_cantos_limpa` e o do `c3_cantos`.
    grupos_vertices: [wgpu::BindGroup; 2],
    grupos_limpa: [wgpu::BindGroup; 2],
    grupos: [wgpu::BindGroup; 2],
    /// A lista deste quadro (alterna a cada gravação).
    lado: AtomicBool,
}

impl Borda {
    /// Lê a cobertura crua `cob`, escreve a da C3 em `b` (grupo 3, bindings 12–15) e, com `cantos`
    /// (`[o atlas da V, o da V2]`) e a `tabela`, lista os blocos de vértice e espalha-os (16–25).
    pub(super) fn nova(
        e: &Entradas<'_>,
        b: usize,
        cob: usize,
        (cantos, vazia): (Option<[usize; 2]>, Option<usize>),
        tabela: &wgpu::Buffer,
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
        let buffer = |nome: &str| {
            let d = e
                .lista
                .buffers
                .iter()
                .find(|b| b.nome == nome)
                .unwrap_or_else(|| panic!("{nome} declarado na PassList"));
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(d.nome),
                size: d.bytes,
                usage: d.usos,
                mapped_at_creation: false,
            })
        };
        // O cabeçalho de cada lista (o despacho indireto `[n, 9, 1]`) escreve-o o `c3_cantos_zera` (nascer mapeado
        // rebentava quando a placa recusa a memória: o buffer nasce inválido).
        let buffers = cantos.map(|_| {
            let [lista, com_cunha, marcas, contadores] = [
                NOME_LISTA_DOS_CANTOS,
                NOME_LISTA_COM_CUNHA,
                NOME_MARCAS_DOS_BLOCOS,
                NOME_CONTADORES_DOS_CANTOS,
            ]
            .map(buffer);
            let metade = lista.size() / 2;
            (lista, com_cunha, metade, marcas, contadores)
        });
        let mut entradas = base.to_vec();
        if cantos.is_some() {
            entradas.extend([19, 21, 24, 25].map(|b| armazenamento(b, false)));
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
            if let (Some(lado), Some((lista, com_cunha, metade, marcas, contadores))) =
                (lado, &buffers)
            {
                l.push(metade_em(lista, *metade, lado, 19));
                l.push(inteiro(21, marcas));
                l.push(metade_em(com_cunha, *metade, lado, 24));
                l.push(inteiro(25, contadores));
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
        let cantos = cantos.zip(buffers).map(
            |([v, v2], (lista, com_cunha, metade, marcas, contadores))| {
                let (g_vertices, grupos_vertices) = par(
                    device,
                    "luz2d c3 borda vertices",
                    &[
                        base[0],
                        base[1],
                        base[2],
                        base[3],
                        armazenamento(20, true),
                        armazenamento(24, false),
                        armazenamento(25, false),
                    ],
                    [0, 1].map(|lado| {
                        let mut l = ligacoes.to_vec();
                        l.push(metade_em(&lista, metade, lado, 20));
                        l.push(metade_em(&com_cunha, metade, lado, 24));
                        l.push(inteiro(25, &contadores));
                        l
                    }),
                );
                let (g_limpa, grupos_limpa) = par(
                    device,
                    "luz2d c3 cantos limpa",
                    &[
                        base[0],
                        armazenamento(20, true),
                        armazenamento(21, false),
                        armazenamento(23, false),
                    ],
                    [0, 1].map(|lado| {
                        vec![
                            ligacoes[0].clone(),
                            metade_em(&lista, metade, lado, 20),
                            inteiro(21, &marcas),
                            inteiro(23, tabela),
                        ]
                    }),
                );
                let (g, grupos) = par(
                    device,
                    "luz2d c3 cantos",
                    &[
                        base[0],
                        entrada_textura(16, FLOAT),
                        entrada_escrita(17, wgpu::TextureFormat::Rgba16Float),
                        armazenamento(20, true),
                        armazenamento(21, false),
                        armazenamento(23, false),
                        armazenamento(25, false),
                    ],
                    [0, 1].map(|lado| {
                        vec![
                            ligacoes[0].clone(),
                            vista(16, &e.vistas[v]),
                            vista(17, &e.vistas[v2]),
                            metade_em(&com_cunha, metade, lado, 20),
                            inteiro(21, &marcas),
                            inteiro(23, tabela),
                            inteiro(25, &contadores),
                        ]
                    }),
                );
                let com = |g: &wgpu::BindGroupLayout, rotulo: &str, entrada: &str| {
                    pipeline_em(rotulo, &[None, None, None, Some(g)], entrada, &[])
                };
                Cantos {
                    zera: com(&g3, "luz2d c3 cantos zera", "c3_cantos_zera"),
                    limpa: com(&g_limpa, "luz2d c3 cantos limpa", "c3_cantos_limpa"),
                    vertices: com(&g_vertices, "luz2d c3 borda vertices", "c3_borda_vertices"),
                    espalha: com(&g, "luz2d c3 cantos", "c3_cantos"),
                    grupos_vertices,
                    grupos_limpa,
                    grupos,
                    lista,
                    com_cunha,
                    metade,
                    contadores,
                    lado: AtomicBool::new(false),
                }
            },
        );
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

    /// ⭐ W6g r9: os contadores do último quadro (`[slots da V2, recusados sem V, recusados sem V2, _]`).
    pub(super) fn contadores(&self) -> Option<&wgpu::Buffer> {
        self.cantos.as_ref().map(|c| &c.contadores)
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
            // A tabela e as marcas dos blocos do quadro anterior voltam a 0 antes de se listarem os deste.
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
            p.dispatch_workgroups_indirect(&c.com_cunha, o);
        }
    }
}

impl super::C3 {
    /// ⭐ W6g r9: a tabela e os contadores dos cantos, para quem os lê ([`super::Solver::cantos_lidos`]).
    pub(crate) fn cantos(&self) -> Option<(&wgpu::Buffer, &wgpu::Buffer)> {
        let c = self.borda.as_ref()?.contadores()?;
        Some((&self.tabela, c))
    }
}

/// ⭐ W6g r9: a tabela da `V2` esparsa que o `meio` lê (com os cantos; senão a vazia).
pub(super) fn tabela(e: &Entradas<'_>, com_cantos: bool) -> wgpu::Buffer {
    let Some(t) = com_cantos.then(|| {
        e.lista
            .buffers
            .iter()
            .find(|b| b.nome == crate::passlist::NOME_TABELA_DOS_CANTOS)
            .expect("a tabela dos cantos declarada")
    }) else {
        return tabela_vazia(&e.placa, crate::passlist::NOME_TABELA_VAZIA_C3);
    };
    e.placa.create_buffer(&wgpu::BufferDescriptor {
        label: Some(t.nome),
        size: t.bytes,
        usage: t.usos,
        mapped_at_creation: false,
    })
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

/// O buffer `b` inteiro no `binding`.
fn inteiro(binding: u32, b: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: b.as_entire_binding(),
    }
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
