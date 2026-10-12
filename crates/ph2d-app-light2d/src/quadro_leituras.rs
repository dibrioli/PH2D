//! As LEITURAS da luz do quadro (o custo, o nível, a fina engrossada) — ficheiro irmão do `quadro.rs`
//! pelo tecto de 700 LOC (W5j).

use ph2d_editor_core::luz_edits::NivelDaLuz;

impl super::LuzDoQuadro {
    /// ⭐ W5e: quantas vezes a fina está engrossada neste quadro (0 = a do nível) e se foi pelo
    /// dispositivo (o aviso do Inspector).
    #[must_use]
    pub fn fina_engrossada(&self) -> (u32, bool) {
        let pelos_tetos = self
            .fina_dos_tetos
            .and_then(|(_, f)| f)
            .is_some_and(|f| f > 0);
        let k = (self.degrau.captura_px / self.base.captura_px)
            .log2()
            .round() as u32;
        (k, pelos_tetos)
    }

    /// ⭐ W5e: quantas vezes o solver da vista foi montado (alocado) — uma grade nova, um nível novo ou
    /// uma recusa que se tenta de novo; nada cabendo abaixo da recusa, o quadro não monta.
    #[must_use]
    pub fn montagens(&self) -> u32 {
        self.montagens
    }

    /// O último custo de placa da grade atual (ms), com ou sem o «Automático»; `None` antes da 1.ª
    /// leitura dela ou sem `TIMESTAMP_QUERY` (aí o automático fica no teto).
    #[must_use]
    pub fn custo_ms(&self) -> Option<f64> {
        self.custo_ms
    }

    /// Quantas leituras de custo o «Automático» já colheu — o governador lê CONTINUAMENTE, não uma
    /// vez.
    #[must_use]
    pub fn leituras_de_custo(&self) -> u32 {
        self.leituras
    }

    /// O nível em uso e se é automático (`None` = degrau fixo).
    #[must_use]
    pub fn qualidade(&self) -> Option<(NivelDaLuz, bool)> {
        let p = self.preferencia?;
        Some((
            if p.automatica {
                self.governador.nivel
            } else {
                p.nivel
            },
            p.automatica,
        ))
    }
}
