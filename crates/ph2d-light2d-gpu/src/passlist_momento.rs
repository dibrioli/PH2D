//! ⭐ P14 «L» — o momento `M` na lista declarada (`docs/PixelArt/P14_plano.md` §4): as quatro texturas
//! `Rgba16Float` (a c0 de `M` na grade das cascatas, a fina de `M` na da fluência) e os dois passes. Só
//! com receptor de normal à vista: a lista de [`PassList::de`] não as tem (a guarda G6), e o solver
//! acrescenta-as por [`PassList::com_momento`] quando a cena as pede. Ficheiro irmão pelo tecto de 700 LOC.

use crate::config::Config;
use crate::passlist::{
    Buffer, CASCATAS, FATIAS_DA_BASE, Opcionais, PASSO_UNIFORME, PassList, Passe, SEMENTES, Textura,
};

/// ⭐ P14c-placa 5 — o acumulado de `M` na holográfica (`Mx` e `My` de cada texel da grade, `vec4<f32>`).
pub(crate) const NOME_ACUMULADO_DO_MOMENTO: &str = "luz2d/buf/hrc_acumulado_momento";
/// ⭐ P14c-placa 5 — a grade das cascatas para a `hrc_saida_momento` (um `vec4<u32>`).
pub(crate) const NOME_GRADE_DO_MOMENTO: &str = "luz2d/buf/hrc_momento_grade";

/// ⭐ P14c-placa 6 — o conhecido fino do miolo de `M` (o índice `base + 4`) e o uniforme dele.
pub(crate) const NOME_CONHECIDO_DO_MIOLO: &str = "luz2d/tex/momento_miolo_conhecido";
pub(crate) const NOME_UNIFORME_DO_MIOLO: &str = "luz2d/buf/momento_miolo";

/// Os rótulos, por ordem: `[c0 x, c0 y, fina x, fina y]`.
pub(crate) const NOMES_DO_MOMENTO: [&str; 4] = [
    "luz2d/tex/momento_c0_x",
    "luz2d/tex/momento_c0_y",
    "luz2d/tex/momento_x",
    "luz2d/tex/momento_y",
];

/// ⭐ P14c-placa 6 — o acumulado de `M` na holográfica: `Mx`, `My` e a diferença das metades do cone 0 do nível 0
/// (`hrc_metades_momento`; um bloco por quadrante com os quadrantes juntos), `vec4<f32>` por texel da grade.
pub(crate) fn bytes_do_acumulado(cfg: &Config) -> u64 {
    let blocos = 2 + if cfg.hrc_quadrantes_juntos { 4 } else { 1 };
    blocos * 16 * u64::from(cfg.grade.largura) * u64::from(cfg.grade.altura)
}

impl PassList {
    /// Esta lista e o momento depois dela: as quatro texturas no fim (a 1.ª no índice `self.texturas.len()`)
    /// e os passes `luz2d momento c0` (lê o nível 1 das cascatas) e `luz2d momento fina` (lê a c0 de `F`,
    /// a visibilidade, o preenchimento e a c0 de `M`).
    #[must_use]
    pub fn com_momento(&self, cfg: &Config) -> Self {
        use wgpu::TextureUsages as T;
        let base = self.texturas.len();
        let grossa = [cfg.grade.largura, cfg.grade.altura];
        let gf = cfg.grade_da_fluencia();
        let fina = [gf.largura, gf.altura];
        let mut l = self.clone();
        l.texturas.extend(
            NOMES_DO_MOMENTO
                .iter()
                .enumerate()
                .map(|(k, &nome)| Textura {
                    nome,
                    formato: wgpu::TextureFormat::Rgba16Float,
                    tamanho: if k < 2 { grossa } else { fina },
                    usos: T::STORAGE_BINDING | T::TEXTURE_BINDING | T::COPY_SRC,
                    mips: 1,
                }),
        );
        let op = Opcionais::de(cfg);
        let sementes = SEMENTES[cfg.saltos_jfa().len() % 2];
        let c0 = Self::indice_da_c0(cfg);
        if cfg.holografica.is_some() {
            // ⭐ P14c-placa 5: na holográfica a c0 de `M` nasce no laço dos quadrantes (os R, buffers).
            l.buffers.push(Buffer {
                nome: NOME_ACUMULADO_DO_MOMENTO,
                bytes: bytes_do_acumulado(cfg),
                usos: wgpu::BufferUsages::STORAGE,
            });
            l.buffers.push(Buffer {
                nome: NOME_GRADE_DO_MOMENTO,
                bytes: 16,
                usos: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
            l.passes.push(Passe {
                nome: "luz2d momento hrc metades".into(),
                le: vec![],
                escreve: vec![],
            });
            l.passes.push(Passe {
                nome: "luz2d momento hrc soma".into(),
                le: vec![],
                escreve: vec![],
            });
            l.passes.push(Passe {
                nome: "luz2d momento hrc saída".into(),
                le: vec![],
                escreve: vec![base, base + 1],
            });
        } else {
            let nivel1 = CASCATAS[usize::from(c0 == CASCATAS[0])];
            l.passes.push(Passe {
                nome: "luz2d momento c0".into(),
                le: vec![nivel1],
                escreve: vec![base, base + 1],
            });
        }
        let mut le = vec![c0, op.visibilidade.unwrap_or(sementes)];
        le.push(op.preenche_final(cfg).unwrap_or(sementes));
        le.extend([base, base + 1]);
        l.passes.push(Passe {
            nome: "luz2d momento fina".into(),
            le,
            escreve: vec![base + 2, base + 3],
        });
        // ⭐ P14c-placa 6: o miolo de `M` — a pirâmide do de `F` (livre depois dele) difunde `Mx` e `My`; o
        // conhecido fino de `M` e o uniforme são dele (`solver_momento.rs`, `MioloDoMomento`).
        if let (Some(niveis), Some([k0, k, a, b, _, d])) = (cfg.niveis_do_miolo(), op.miolo) {
            l.texturas.push(Textura {
                nome: NOME_CONHECIDO_DO_MIOLO,
                formato: wgpu::TextureFormat::Rgba16Float,
                tamanho: fina,
                usos: T::STORAGE_BINDING | T::TEXTURE_BINDING,
                mips: 1,
            });
            l.buffers.push(Buffer {
                nome: NOME_UNIFORME_DO_MIOLO,
                bytes: PASSO_UNIFORME * (2 * u64::from(niveis + 1) + FATIAS_DA_BASE),
                usos: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
            l.passes.push(Passe {
                nome: "luz2d momento miolo".into(),
                le: vec![base + 2, base + 3, k0],
                escreve: vec![base + 2, base + 3, base + 4, k, a, b, d],
            });
        }
        l
    }
}
