//! As LEITURAS da luz do quadro (o custo, o nível, a fina engrossada) — ficheiro irmão do `quadro.rs`
//! pelo tecto de 700 LOC (W5j).

use ph2d_editor_core::luz_edits::NivelDaLuz;
use ph2d_light2d_gpu::TemposMs;

/// ⭐ W5u — INSTRUMENTO `PH2D_LIGHT2D_CUSTO=1`: o número do Inspector partido por passe (a vista e o
/// entorno), a cada 60 leituras — o custo que o dono vê no app inteiro, não o da régua.
pub(super) fn relatar_custo(t: &TemposMs, entorno: Option<&TemposMs>, grade: (u32, u32), n: u32) {
    if !n.is_multiple_of(60) || std::env::var_os("PH2D_LIGHT2D_CUSTO").is_none() {
        return;
    }
    eprintln!(
        "[luz2d] custo {:.2} ms · grade {}×{} · c0 {:.2} c1 {:.2} c2 {:.2} vis {:.2} fina {:.2} c0_fina {:.2} \
         preenche {:.2} másc {:.2} rebote {:.2}+{:.2} miolo {:.2} · entorno {:.2}",
        t.total() + entorno.map_or(0.0, TemposMs::total),
        grade.0,
        grade.1,
        t.c0,
        t.c1,
        t.c2_total(),
        t.c3_visibilidade,
        t.c3_fina,
        t.c0_fina,
        t.c3_preenche,
        t.c3_mascaras,
        t.c0_rebote,
        t.c3_rebote,
        t.c3_miolo,
        entorno.map_or(0.0, TemposMs::total),
    );
}

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
