//! **A lista declarada de passes** — a MESMA lista cria as texturas e os buffers fixos do solver e
//! responde quantos bytes a luz ocupa e move por quadro (plano §2.2). O gate G5 mede-a contra o
//! que o alocador do wgpu de facto criou: um recurso fora dela reprova.

use crate::config::{Config, Precisao};

#[path = "passlist_cantos.rs"]
mod cantos;
pub(crate) use cantos::{
    NOME_CONTADORES_DOS_CANTOS, NOME_LISTA_COM_CUNHA, NOME_LISTA_DOS_CANTOS,
    NOME_MARCAS_DOS_BLOCOS, NOME_TABELA_DOS_CANTOS, NOME_TABELA_VAZIA_C2, NOME_TABELA_VAZIA_C3,
    NOME_TABELA_VAZIA_HRC, atlas_dos_cantos, buffers_dos_cantos, tabelas_vazias,
};

/// Índices das texturas em [`PassList::texturas`].
pub(crate) const EMISSAO: usize = 0;
pub(crate) const COBERTURA: usize = 1;
pub(crate) const TAU_DENTRO: usize = 2;
pub(crate) const SEMENTES: [usize; 2] = [3, 4];
/// ⭐ W5q: o buffer dos ladrilhos do anel ([`crate::ladrilhos`]).
pub(crate) const NOME_LADRILHOS: &str = "luz2d/buf/ladrilhos";
/// ⭐ W6g r5: os texels de duas faces que a `c3_fina_adia` deixa ao `c3_fina_cantos` — o cabeçalho é o
/// despacho INDIRETO dele (`[grupos de 64, 1, 1]`) e o número de texels, depois um `u32` por texel.
pub(crate) const NOME_FINA_ADIADOS: &str = "luz2d/buf/fina_adiados";
/// ⭐ W6g r5: a luz de cada texel da [`NOME_FINA_ADIADOS`] (a fluência e a transmitida, `f32`).
pub(crate) const NOME_FINA_CANTOS_LUZ: &str = "luz2d/buf/fina_cantos_luz";

pub(crate) const CASCATAS: [usize; 2] = [5, 6];
/// ⭐ C3 (W4b): a fluência FINAL, na grade fina — a que os receptores leem.
pub(crate) const FLUENCIA: usize = 7;

/// ⭐ Os índices das texturas que só existem com certas configurações — depois das fixas, nesta
/// ordem.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Opcionais {
    /// C3: a visibilidade de cada lâmpada direta nas sondas da c0 (8 × u8), com
    /// [`Config::usa_visibilidade`].
    pub visibilidade: Option<usize>,
    /// ⭐ W5a (F1, F2): a captura FINA dos opacos `[cobertura, τ dentro]`, com
    /// [`Config::usa_captura_fina`].
    pub captura: Option<[usize; 2]>,
    /// ⭐ W5a (F3): o par do JFA do complemento (o preenchimento do interior dos opacos).
    pub preenche: Option<[usize; 2]>,
    /// ⭐ W5c: o rebote — `[albedo, fonte]`: o albedo que a C0 captura e a luz que cada superfície
    /// recebeu no quadro anterior (lida no mundo pelo quadro seguinte).
    pub rebote: Option<[usize; 2]>,
    /// ⭐ W5d: a borda exata da captura que a C3 lê (`(cobertura, normal, recuo)` por texel), com
    /// [`Config::borda_exata`].
    pub borda: Option<usize>,
    /// ⭐ W6f: os cantos da borda — `[as cunhas dos parciais de vértice, as espalhadas]` (`V`, `V2`), com
    /// [`Config::usa_cantos`]; W6g r9: dois ATLAS de blocos 8×8 ([`atlas_dos_cantos`]).
    pub cantos: Option<[usize; 2]>,
    /// ⭐ W6f: sem os cantos, a textura de 1 × 1 que o `c3_borda` cita (binding 15) e não escreve.
    pub vertices_vazia: Option<usize>,
    /// ⭐ W5d: a EMISSÃO de base (antes da soma do rebote) — o 2.º rebote do quadro repõe-na, com
    /// `rebotes_por_quadro > 1`.
    pub emissao_base: Option<usize>,
    /// ⭐ W5h: o miolo por difusão — `[conhecido da fina, conhecido, a, b, transmitida]` (as três
    /// pirâmides começam no nível 1; ⭐ W5k: a luz que atravessa um corpo translúcido e a opacidade
    /// dele, que o `miolo_final` mistura; ⭐ W5n: o defeito do V-ciclo, outra pirâmide do nível 1).
    pub miolo: Option<[usize; 6]>,
}

impl Opcionais {
    pub(crate) fn de(cfg: &Config) -> Self {
        let mut i = FLUENCIA + 1;
        let mut proxima = |n: usize| {
            let k = i;
            i += n;
            k
        };
        Self {
            visibilidade: cfg.usa_visibilidade().then(|| proxima(1)),
            captura: cfg.usa_captura_fina().then(|| {
                let k = proxima(2);
                [k, k + 1]
            }),
            preenche: cfg.preenchimento.map(|_| {
                let k = proxima(2);
                [k, k + 1]
            }),
            rebote: cfg.rebote.then(|| {
                let k = proxima(2);
                [k, k + 1]
            }),
            borda: cfg.borda_exata.then(|| proxima(1)),
            cantos: cfg.usa_cantos().then(|| {
                let k = proxima(2);
                [k, k + 1]
            }),
            vertices_vazia: (cfg.borda_exata && !cfg.usa_cantos()).then(|| proxima(1)),
            emissao_base: (cfg.rebote && cfg.rebotes_por_quadro > 1 && cfg.holografica.is_some())
                .then(|| proxima(1)),
            miolo: cfg.niveis_do_miolo().map(|_| {
                let k = proxima(6);
                [k, k + 1, k + 2, k + 3, k + 4, k + 5]
            }),
        }
    }

    /// A textura onde o JFA do preenchimento acaba (a semente escreve a 1.ª; o salto `j` lê `j % 2`).
    pub(crate) fn preenche_final(&self, cfg: &Config) -> Option<usize> {
        let p = self.preenche?;
        let (_, saltos) = cfg.grade_do_preenchimento()?;
        Some(p[saltos.len() % 2])
    }
}

/// O passo entre as entradas de um buffer de uniformes por passe (o alinhamento de offset que o
/// WebGPU garante, `min_uniform_buffer_offset_alignment`).
pub(crate) const PASSO_UNIFORME: u64 = 256;
/// ⭐ W5o — as fatias do uniforme do miolo que levam a `Base` do `c3_miolo_base.wgsl` (832 B).
pub(crate) const FATIAS_DA_BASE: u64 = 4;

#[derive(Clone, Debug, PartialEq)]
pub struct Textura {
    /// Também o rótulo no wgpu — o G5 acha-a por ele no relatório do alocador.
    pub nome: &'static str,
    pub formato: wgpu::TextureFormat,
    pub tamanho: [u32; 2],
    pub usos: wgpu::TextureUsages,
    /// ⭐ W5h: os níveis de mip (a pirâmide do miolo); 1 nas outras.
    pub mips: u32,
}

impl Textura {
    #[must_use]
    pub fn bytes(&self) -> u64 {
        let texel = self
            .formato
            .block_copy_size(None)
            .expect("formato de cor sem aspeto");
        (0..self.mips)
            .map(|m| {
                u64::from((self.tamanho[0] >> m).max(1))
                    * u64::from((self.tamanho[1] >> m).max(1))
                    * u64::from(texel)
            })
            .sum()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Buffer {
    pub nome: &'static str,
    pub bytes: u64,
    pub usos: wgpu::BufferUsages,
}

/// Um passe e as texturas que ele lê e escreve (índices em [`PassList::texturas`]).
#[derive(Clone, Debug, PartialEq)]
pub struct Passe {
    pub nome: String,
    pub le: Vec<usize>,
    pub escreve: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PassList {
    pub texturas: Vec<Textura>,
    /// Os buffers de tamanho fixo. A geometria da cena (formas e vértices) cresce com ela e é
    /// contada à parte, no [`crate::RelatorioC0`].
    pub buffers: Vec<Buffer>,
    pub passes: Vec<Passe>,
}

fn formato_cascata(p: Precisao) -> wgpu::TextureFormat {
    match p {
        Precisao::F16 => wgpu::TextureFormat::Rgba16Float,
        Precisao::F32 => wgpu::TextureFormat::Rgba32Float,
    }
}

impl PassList {
    /// A lista para `cfg`. `com_tempos`: o dispositivo tem `TIMESTAMP_QUERY` (os dois buffers dos
    /// carimbos só existem então).
    #[must_use]
    pub fn de(cfg: &Config, com_tempos: bool) -> Self {
        use wgpu::TextureUsages as T;
        let g = cfg.grade_efetiva();
        let fino = [g.largura, g.altura];
        let gf = cfg.grade_da_fluencia();
        let jfa = cfg.grade_jfa();
        let n = cfg.n_cascatas();
        let saltos = cfg.saltos_jfa();
        let amostrada = T::TEXTURE_BINDING | T::COPY_SRC;
        let op = Opcionais::de(cfg);
        // ⭐ W5d: o 2.º rebote repõe a emissão de base por cópia.
        let reposta = if op.emissao_base.is_some() {
            T::COPY_DST
        } else {
            T::empty()
        };
        let texturas = vec![
            Textura {
                nome: "luz2d/tex/emissao",
                formato: wgpu::TextureFormat::Rgba16Float,
                tamanho: fino,
                usos: T::RENDER_ATTACHMENT | amostrada | reposta,
                mips: 1,
            },
            Textura {
                nome: "luz2d/tex/cobertura",
                formato: wgpu::TextureFormat::R16Float,
                tamanho: fino,
                usos: T::RENDER_ATTACHMENT | amostrada,
                mips: 1,
            },
            Textura {
                nome: "luz2d/tex/tau_dentro",
                formato: wgpu::TextureFormat::R16Float,
                tamanho: fino,
                usos: T::RENDER_ATTACHMENT | amostrada,
                mips: 1,
            },
            Textura {
                nome: "luz2d/tex/sementes_a",
                formato: wgpu::TextureFormat::R32Uint,
                tamanho: jfa,
                usos: T::STORAGE_BINDING | amostrada,
                mips: 1,
            },
            Textura {
                nome: "luz2d/tex/sementes_b",
                formato: wgpu::TextureFormat::R32Uint,
                tamanho: jfa,
                usos: T::STORAGE_BINDING | amostrada,
                mips: 1,
            },
            Textura {
                nome: "luz2d/tex/cascata_a",
                formato: formato_cascata(cfg.armazenamento),
                tamanho: fino,
                usos: T::STORAGE_BINDING | amostrada,
                mips: 1,
            },
            Textura {
                nome: "luz2d/tex/cascata_b",
                formato: formato_cascata(cfg.armazenamento),
                tamanho: fino,
                usos: T::STORAGE_BINDING | amostrada,
                mips: 1,
            },
            Textura {
                nome: "luz2d/tex/fluencia",
                formato: formato_cascata(cfg.armazenamento),
                tamanho: [gf.largura, gf.altura],
                usos: T::STORAGE_BINDING | amostrada,
                mips: 1,
            },
        ];
        let mut texturas = texturas;
        if op.visibilidade.is_some() {
            texturas.push(Textura {
                nome: "luz2d/tex/visibilidade",
                formato: wgpu::TextureFormat::Rg32Uint,
                tamanho: fino,
                usos: T::STORAGE_BINDING | amostrada,
                mips: 1,
            });
        }
        if op.captura.is_some() {
            let gc = cfg.grade_da_captura();
            for nome in ["luz2d/tex/cobertura_fina", "luz2d/tex/tau_dentro_fina"] {
                texturas.push(Textura {
                    nome,
                    formato: wgpu::TextureFormat::R16Float,
                    tamanho: [gc.largura, gc.altura],
                    usos: T::RENDER_ATTACHMENT | amostrada,
                    mips: 1,
                });
            }
        }
        let preenche = cfg.grade_do_preenchimento();
        if let Some((celulas, _)) = &preenche {
            for nome in ["luz2d/tex/preenche_a", "luz2d/tex/preenche_b"] {
                texturas.push(Textura {
                    nome,
                    formato: wgpu::TextureFormat::R32Uint,
                    tamanho: *celulas,
                    usos: T::STORAGE_BINDING | amostrada,
                    mips: 1,
                });
            }
        }

        if op.rebote.is_some() {
            texturas.push(Textura {
                nome: "luz2d/tex/albedo",
                formato: wgpu::TextureFormat::Rgba8Unorm,
                tamanho: fino,
                usos: T::RENDER_ATTACHMENT | amostrada,
                mips: 1,
            });
            texturas.push(Textura {
                nome: "luz2d/tex/rebote",
                formato: wgpu::TextureFormat::Rgba16Float,
                tamanho: fino,
                usos: T::STORAGE_BINDING | amostrada,
                mips: 1,
            });
        }

        if op.borda.is_some() {
            let gc = if op.captura.is_some() {
                cfg.grade_da_captura()
            } else {
                g
            };
            texturas.push(Textura {
                nome: "luz2d/tex/borda",
                formato: wgpu::TextureFormat::Rgba16Float,
                tamanho: [gc.largura, gc.altura],
                usos: T::STORAGE_BINDING | amostrada,
                mips: 1,
            });
            if op.vertices_vazia.is_some() {
                texturas.push(Textura {
                    nome: "luz2d/tex/vertices_vazia",
                    formato: wgpu::TextureFormat::Rgba16Float,
                    tamanho: [1, 1],
                    usos: T::STORAGE_BINDING,
                    mips: 1,
                });
            }
            if op.cantos.is_some() {
                for nome in ["luz2d/tex/vertices", "luz2d/tex/cantos"] {
                    texturas.push(Textura {
                        nome,
                        formato: wgpu::TextureFormat::Rgba16Float,
                        tamanho: atlas_dos_cantos([gc.largura, gc.altura], cfg.cantos_por_blocos),
                        usos: T::STORAGE_BINDING | amostrada,
                        mips: 1,
                    });
                }
            }
        }

        if op.emissao_base.is_some() {
            texturas.push(Textura {
                nome: "luz2d/tex/emissao_base",
                formato: wgpu::TextureFormat::Rgba16Float,
                tamanho: fino,
                usos: T::COPY_SRC | T::COPY_DST,
                mips: 1,
            });
        }

        // ⭐ W5h: o conhecido da fina no formato da fluência; a pirâmide (do nível 1) com o conhecido
        // em f16 e a solução em f32 (o Jacobi em f16 estagna — medido).
        if let Some(l) = cfg.niveis_do_miolo() {
            use wgpu::TextureFormat as F;
            let base = [gf.largura, gf.altura];
            let meia = base.map(|x| (x >> 1).max(1));
            for (nome, formato, tamanho, mips) in [
                (
                    "luz2d/tex/miolo_fino",
                    formato_cascata(cfg.armazenamento),
                    base,
                    1,
                ),
                ("luz2d/tex/miolo_conhecido", F::Rgba16Float, meia, l),
                ("luz2d/tex/miolo_a", F::Rgba32Float, meia, l),
                ("luz2d/tex/miolo_b", F::Rgba32Float, meia, l),
                (
                    "luz2d/tex/miolo_transmitida",
                    formato_cascata(cfg.armazenamento),
                    base,
                    1,
                ),
                ("luz2d/tex/miolo_defeito", F::Rgba32Float, meia, l),
            ] {
                texturas.push(Textura {
                    nome,
                    formato,
                    tamanho,
                    usos: T::STORAGE_BINDING | T::TEXTURE_BINDING,
                    mips,
                });
            }
        }

        // ⭐ W5b (F4): a atlas das máscaras dos sprites distantes — a de 1 × 1 (cresce com elas,
        // contada na geometria residente).
        texturas.push(Textura {
            nome: crate::solver::NOME_DA_ATLAS,
            formato: crate::solver::FORMATO_DA_ATLAS,
            tamanho: [1, 1],
            usos: crate::solver::USOS_DA_ATLAS,
            mips: 1,
        });

        // ⭐ W5z: os T e os R da holográfica em textura (`Config::hrc_niveis_em_textura`).
        let niveis_em_textura = cfg.hrc_niveis_em_textura();
        if niveis_em_textura {
            texturas.extend(crate::hrc::texturas(fino));
        }

        use wgpu::BufferUsages as B;
        let uniforme = B::UNIFORM | B::COPY_DST;
        let mut buffers = vec![
            Buffer {
                nome: "luz2d/buf/c0",
                bytes: 16,
                usos: uniforme,
            },
            Buffer {
                nome: "luz2d/buf/jfa",
                bytes: PASSO_UNIFORME * (saltos.len() as u64 + 1),
                usos: uniforme,
            },
            Buffer {
                nome: "luz2d/buf/cascatas",
                bytes: PASSO_UNIFORME * u64::from(n),
                usos: uniforme,
            },
            // ⭐ C3 — as lâmpadas diretas (o teto, `direta::LAMPADAS_DIRETAS_MAX`).
            Buffer {
                nome: "luz2d/buf/lampadas",
                bytes: crate::direta::BYTES_DO_BUFFER,
                usos: B::STORAGE | B::COPY_DST,
            },
            // ⭐ C3 — a grade fina e a região do quadro (`Fina` do `c3_direta.wgsl`).
            Buffer {
                nome: "luz2d/buf/c3",
                bytes: 48,
                usos: uniforme,
            },
        ];
        // ⭐ W5b (F4): os objetos distantes, as arestas e os índices — um elemento cada (crescem
        // com o mundo fora da grade, contados na geometria residente).
        for (nome, bytes) in crate::solver::NOMES_DISTANTES
            .into_iter()
            .zip(crate::solver::MINIMOS_DISTANTES)
        {
            buffers.push(Buffer {
                nome,
                bytes,
                usos: B::STORAGE | B::COPY_DST,
            });
        }
        if op.captura.is_some() {
            buffers.push(Buffer {
                nome: "luz2d/buf/c0_fina",
                bytes: 16,
                usos: uniforme,
            });
        }
        if let Some((_, saltos)) = &preenche {
            buffers.push(Buffer {
                nome: "luz2d/buf/preenche",
                bytes: PASSO_UNIFORME * (saltos.len() as u64 + 1),
                usos: uniforme,
            });
        }
        if let Some(l) = cfg.niveis_do_miolo() {
            buffers.push(Buffer {
                nome: "luz2d/buf/miolo",
                bytes: PASSO_UNIFORME * (2 * u64::from(l + 1) + FATIAS_DA_BASE),
                usos: uniforme,
            });
            // ⭐ W5q: os ladrilhos do anel (um bit cada; o miolo e o anel leem-nos).
            let gf = cfg.grade_da_fluencia();
            buffers.push(Buffer {
                nome: NOME_LADRILHOS,
                bytes: crate::ladrilhos::bytes([gf.largura, gf.altura]),
                usos: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
        }
        if cfg.adia_o_canto() {
            let gf = cfg.grade_da_fluencia();
            buffers.push(Buffer {
                nome: NOME_FINA_ADIADOS,
                bytes: 4 * (4 + u64::from(gf.largura) * u64::from(gf.altura)),
                usos: B::STORAGE | B::INDIRECT | B::COPY_DST,
            });
            buffers.push(Buffer {
                nome: NOME_FINA_CANTOS_LUZ,
                bytes: 32 * u64::from(gf.largura) * u64::from(gf.altura),
                usos: B::STORAGE,
            });
        }
        if let (Some(_), Some(b)) = (op.cantos, op.borda) {
            buffers.extend(buffers_dos_cantos(texturas[b].tamanho));
        }
        buffers.extend(tabelas_vazias(
            op.cantos.is_some(),
            cfg.holografica.is_some(),
        ));
        // ⭐ W5c: a cascata holográfica (os T, o par R, o acumulado, o uniforme e os vazios).
        if cfg.holografica.is_some() {
            buffers.extend(crate::hrc::buffers(
                fino,
                crate::hrc::n_passes(fino),
                niveis_em_textura,
            ));
        }
        if op.rebote.is_some() {
            buffers.push(Buffer {
                nome: "luz2d/buf/rebote",
                bytes: 32,
                usos: uniforme,
            });
        }
        if op.emissao_base.is_some() {
            buffers.push(Buffer {
                nome: "luz2d/buf/rebote_mesma",
                bytes: 32,
                usos: uniforme,
            });
        }
        if com_tempos {
            let bytes = 8 * u64::from(crate::tempos::n_carimbos(n));
            buffers.push(Buffer {
                nome: "luz2d/buf/tempos_resolve",
                bytes,
                usos: B::QUERY_RESOLVE | B::COPY_SRC,
            });
            // W6g r9: + os 16 bytes dos contadores dos cantos ([`crate::tempos::TemposMs::cantos_recusados`]).
            buffers.push(Buffer {
                nome: "luz2d/buf/tempos_leitura",
                bytes: bytes + 16,
                usos: B::MAP_READ | B::COPY_DST,
            });
        }

        let mut passes = vec![Passe {
            nome: "C0 captura".into(),
            le: vec![],
            escreve: [EMISSAO, COBERTURA, TAU_DENTRO]
                .into_iter()
                .chain(op.rebote.map(|r| r[0]))
                .collect(),
        }];
        // ⭐ W5c: a emissão ganha albedo × a luz que a superfície recebeu no quadro anterior.
        if let Some([albedo, fonte]) = op.rebote {
            passes.push(Passe {
                nome: "C0 rebote".into(),
                le: vec![albedo, fonte],
                escreve: vec![EMISSAO],
            });
        }
        if let Some(c) = op.captura {
            passes.push(Passe {
                nome: "C0 captura fina".into(),
                le: vec![],
                escreve: c.to_vec(),
            });
        }
        passes.push(Passe {
            nome: "C1 sementes".into(),
            le: vec![COBERTURA],
            escreve: vec![SEMENTES[0]],
        });
        for (j, k) in saltos.iter().enumerate() {
            passes.push(Passe {
                nome: format!("C1 salto {k}"),
                le: vec![SEMENTES[j % 2]],
                escreve: vec![SEMENTES[(j + 1) % 2]],
            });
        }
        let sementes_finais = SEMENTES[saltos.len() % 2];
        // ⭐ W5c: a holográfica escreve a c0 direto (os T e os R vivem em buffers).
        if cfg.holografica.is_some() {
            passes.push(Passe {
                nome: "C2 holográfica".into(),
                le: vec![EMISSAO, COBERTURA, TAU_DENTRO],
                escreve: vec![Self::indice_da_c0(cfg)],
            });
        }
        for i in (0..n).rev().filter(|_| cfg.holografica.is_none()) {
            let saida = CASCATAS[((n - 1 - i) % 2) as usize];
            let mut le = vec![EMISSAO, COBERTURA, TAU_DENTRO, sementes_finais];
            if i + 1 < n {
                le.push(CASCATAS[((n - i) % 2) as usize]);
            }
            passes.push(Passe {
                nome: format!("C2 cascata {i}"),
                le,
                escreve: vec![saida],
            });
        }
        // ⭐ C3: a visibilidade nas sondas da c0 e a grade fina (c0 interpolada + a luz direta). A
        // luz direta marcha na captura fina quando há (W5a) e não lê a emissão (só a transmissão).
        let [cob, tau] = op.captura.unwrap_or([COBERTURA, TAU_DENTRO]);
        // ⭐ W5d: a borda exata dessa captura, que a C3 lê no lugar da cobertura.
        if let Some(b) = op.borda {
            passes.push(Passe {
                nome: "C3 borda".into(),
                le: vec![cob],
                // W6g: com os cantos, também as cunhas dos vértices (`V`).
                escreve: [b].into_iter().chain(op.cantos.map(|c| c[0])).collect(),
            });
        }
        if let Some([v, v2]) = op.cantos {
            passes.push(Passe {
                nome: "C3 cantos".into(),
                le: vec![v],
                escreve: vec![v2],
            });
        }
        let cob = op.borda.unwrap_or(cob);
        let cena: Vec<usize> = [cob, tau, sementes_finais]
            .into_iter()
            .chain(op.cantos.map(|c| c[1]))
            .collect();
        let c0 = Self::indice_da_c0(cfg);
        let mut le_fina = vec![c0];
        if let Some(v) = op.visibilidade {
            passes.push(Passe {
                nome: "C3 visibilidade".into(),
                le: cena.to_vec(),
                escreve: vec![v],
            });
            le_fina.push(v);
        }
        // ⭐ W5a (F3): o JFA do complemento — cada célula do interior de um opaco aponta o texel
        // fino exterior mais próximo.
        if let (Some(p), Some((_, saltos))) = (op.preenche, &preenche) {
            passes.push(Passe {
                nome: "C3 preenche sementes".into(),
                le: vec![cob, tau],
                escreve: vec![p[0]],
            });
            for (j, k) in saltos.iter().enumerate() {
                passes.push(Passe {
                    nome: format!("C3 preenche salto {k}"),
                    le: vec![p[j % 2]],
                    escreve: vec![p[(j + 1) % 2]],
                });
            }
            le_fina.push(p[saltos.len() % 2]);
        }
        le_fina.extend(cena);
        let mut escreve_fina = vec![FLUENCIA];
        escreve_fina.extend(op.miolo.map(|m| m[4]));
        passes.push(Passe {
            nome: "C3 fina".into(),
            le: le_fina,
            escreve: escreve_fina,
        });
        // ⭐ W5h: o miolo — um piso (cada textura uma vez; as relaxações relêem a pirâmide).
        if let Some(m) = op.miolo {
            let mut todas = vec![FLUENCIA];
            todas.extend(m);
            passes.push(Passe {
                nome: "C3 miolo".into(),
                le: todas.clone(),
                escreve: todas,
            });
        }
        // ⭐ W5c: a luz que cada superfície recebe (as cascatas + a luz direta), para o quadro seguinte.
        if let Some([_, fonte]) = op.rebote {
            passes.push(Passe {
                nome: "C3 rebote".into(),
                le: vec![c0, cob, tau, sementes_finais],
                escreve: vec![fonte],
            });
        }
        Self {
            texturas,
            buffers,
            passes,
        }
    }

    /// Bytes residentes: texturas + buffers fixos.
    #[must_use]
    pub fn bytes_alocados(&self) -> u64 {
        self.texturas.iter().map(Textura::bytes).sum::<u64>()
            + self.buffers.iter().map(|b| b.bytes).sum::<u64>()
    }

    /// Bytes movidos por quadro: cada passe lê e escreve UMA vez inteira cada textura que declara
    /// (cache perfeita — um piso; a marcha relê a cena e as sementes, e a cache decide quanto).
    #[must_use]
    pub fn bytes_movidos(&self) -> u64 {
        self.passes
            .iter()
            .flat_map(|p| p.le.iter().chain(&p.escreve))
            .map(|&t| self.texturas[t].bytes())
            .sum()
    }

    /// Os bytes que a luz move por quadro com a configuração `cfg` — derivados da mesma lista que
    /// cria as texturas.
    #[must_use]
    pub fn bytes_por_quadro(cfg: &Config) -> u64 {
        Self::de(cfg, false).bytes_movidos()
    }

    /// A textura onde a c0 escreve a fluência das CASCATAS (o par alterna desde o topo).
    #[must_use]
    pub fn indice_da_c0(cfg: &Config) -> usize {
        CASCATAS[((cfg.n_cascatas() - 1) % 2) as usize]
    }
}
