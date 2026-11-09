//! ⭐ P14 «L» — o momento `M` na lista declarada (`docs/PixelArt/P14_plano.md` §4): as quatro texturas
//! `Rgba16Float` (a c0 de `M` na grade das cascatas, a fina de `M` na da fluência) e os dois passes. Só
//! com receptor de normal à vista: a lista de [`PassList::de`] não as tem (a guarda G6), e o solver
//! acrescenta-as por [`PassList::com_momento`] quando a cena as pede. Ficheiro irmão pelo tecto de 700 LOC.

use crate::config::Config;
use crate::passlist::{CASCATAS, Opcionais, PassList, Passe, SEMENTES, Textura};

/// Os rótulos, por ordem: `[c0 x, c0 y, fina x, fina y]`.
pub(crate) const NOMES_DO_MOMENTO: [&str; 4] = [
    "luz2d/tex/momento_c0_x",
    "luz2d/tex/momento_c0_y",
    "luz2d/tex/momento_x",
    "luz2d/tex/momento_y",
];

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
        let nivel1 = CASCATAS[usize::from(c0 == CASCATAS[0])];
        l.passes.push(Passe {
            nome: "luz2d momento c0".into(),
            le: vec![nivel1],
            escreve: vec![base, base + 1],
        });
        let mut le = vec![c0, op.visibilidade.unwrap_or(sementes)];
        le.push(op.preenche_final(cfg).unwrap_or(sementes));
        le.extend([base, base + 1]);
        l.passes.push(Passe {
            nome: "luz2d momento fina".into(),
            le,
            escreve: vec![base + 2, base + 3],
        });
        l
    }
}
