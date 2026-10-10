//! **A lista declarada de passes** — a MESMA lista cria as texturas e os buffers fixos do solver e
//! responde quantos bytes a luz ocupa e move por quadro (plano §2.2). O gate G5 mede-a contra o
//! que o alocador do wgpu de facto criou: um recurso fora dela reprova.

use crate::config::{Config, Precisao};

/// Índices das texturas em [`PassList::texturas`].
pub(crate) const EMISSAO: usize = 0;
pub(crate) const COBERTURA: usize = 1;
pub(crate) const TAU_DENTRO: usize = 2;
pub(crate) const SEMENTES: [usize; 2] = [3, 4];
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

#[derive(Clone, Debug, PartialEq)]
pub struct Textura {
    /// Também o rótulo no wgpu — o G5 acha-a por ele no relatório do alocador.
    pub nome: &'static str,
    pub formato: wgpu::TextureFormat,
    pub tamanho: [u32; 2],
    pub usos: wgpu::TextureUsages,
}

impl Textura {
    #[must_use]
    pub fn bytes(&self) -> u64 {
        let texel = self
            .formato
            .block_copy_size(None)
            .expect("formato de cor sem aspeto");
        u64::from(self.tamanho[0]) * u64::from(self.tamanho[1]) * u64::from(texel)
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
        let texturas = vec![
            Textura {
                nome: "luz2d/tex/emissao",
                formato: wgpu::TextureFormat::Rgba16Float,
                tamanho: fino,
                usos: T::RENDER_ATTACHMENT | amostrada,
            },
            Textura {
                nome: "luz2d/tex/cobertura",
                formato: wgpu::TextureFormat::R16Float,
                tamanho: fino,
                usos: T::RENDER_ATTACHMENT | amostrada,
            },
            Textura {
                nome: "luz2d/tex/tau_dentro",
                formato: wgpu::TextureFormat::R16Float,
                tamanho: fino,
                usos: T::RENDER_ATTACHMENT | amostrada,
            },
            Textura {
                nome: "luz2d/tex/sementes_a",
                formato: wgpu::TextureFormat::R32Uint,
                tamanho: jfa,
                usos: T::STORAGE_BINDING | amostrada,
            },
            Textura {
                nome: "luz2d/tex/sementes_b",
                formato: wgpu::TextureFormat::R32Uint,
                tamanho: jfa,
                usos: T::STORAGE_BINDING | amostrada,
            },
            Textura {
                nome: "luz2d/tex/cascata_a",
                formato: formato_cascata(cfg.armazenamento),
                tamanho: fino,
                usos: T::STORAGE_BINDING | amostrada,
            },
            Textura {
                nome: "luz2d/tex/cascata_b",
                formato: formato_cascata(cfg.armazenamento),
                tamanho: fino,
                usos: T::STORAGE_BINDING | amostrada,
            },
            Textura {
                nome: "luz2d/tex/fluencia",
                formato: formato_cascata(cfg.armazenamento),
                tamanho: [gf.largura, gf.altura],
                usos: T::STORAGE_BINDING | amostrada,
            },
        ];
        let mut texturas = texturas;
        let op = Opcionais::de(cfg);
        if op.visibilidade.is_some() {
            texturas.push(Textura {
                nome: "luz2d/tex/visibilidade",
                formato: wgpu::TextureFormat::Rg32Uint,
                tamanho: fino,
                usos: T::STORAGE_BINDING | amostrada,
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
                });
            }
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
        if com_tempos {
            let bytes = 8 * u64::from(crate::tempos::n_carimbos(n));
            buffers.push(Buffer {
                nome: "luz2d/buf/tempos_resolve",
                bytes,
                usos: B::QUERY_RESOLVE | B::COPY_SRC,
            });
            buffers.push(Buffer {
                nome: "luz2d/buf/tempos_leitura",
                bytes,
                usos: B::MAP_READ | B::COPY_DST,
            });
        }

        let mut passes = vec![Passe {
            nome: "C0 captura".into(),
            le: vec![],
            escreve: vec![EMISSAO, COBERTURA, TAU_DENTRO],
        }];
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
        for i in (0..n).rev() {
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
        let cena = [cob, tau, sementes_finais];
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
        passes.push(Passe {
            nome: "C3 fina".into(),
            le: le_fina,
            escreve: vec![FLUENCIA],
        });
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
