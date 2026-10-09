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
        ];

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
        ];
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

        let mut passes = vec![
            Passe {
                nome: "C0 captura".into(),
                le: vec![],
                escreve: vec![EMISSAO, COBERTURA, TAU_DENTRO],
            },
            Passe {
                nome: "C1 sementes".into(),
                le: vec![COBERTURA],
                escreve: vec![SEMENTES[0]],
            },
        ];
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

    /// A textura onde a c0 escreve a fluência (o par de cascatas alterna desde o topo).
    #[must_use]
    pub fn indice_da_fluencia(cfg: &Config) -> usize {
        CASCATAS[((cfg.n_cascatas() - 1) % 2) as usize]
    }
}
