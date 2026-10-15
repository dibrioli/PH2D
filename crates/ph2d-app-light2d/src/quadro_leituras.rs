//! As LEITURAS da luz do quadro (o custo, o nível, a fina engrossada) — ficheiro irmão do `quadro.rs`
//! pelo tecto de 700 LOC (W5j).

use ph2d_editor_core::luz_edits::NivelDaLuz;
use ph2d_light2d_gpu::{Config, Grade, TemposMs};

/// A configuração da vista, a grade da fluência, a região da janela nela e a janela.
type Onde<'a> = (&'a Config, Grade, ([u32; 2], [u32; 2]), (u32, u32));

/// ⭐ W5u — INSTRUMENTO `PH2D_LIGHT2D_CUSTO=1`: o número do Inspector partido por passe (a vista e o
/// entorno), a cada 60 leituras — o custo que o dono vê no app inteiro, não o da régua.
/// ⭐ W5v: e ONDE — a janela, a grade das cascatas, a fina (texels finos por texel, a captura, a região da
/// janela na grade da fluência) e a aritmética: o que uma régua tem de reproduzir.
/// ⭐ W5w: e o MÍNIMO e a MEDIANA das 60 leituras (a amostra solta é ruído); `PH2D_LIGHT2D_RENASCE=<k>` faz o
/// solver da vista renascer a cada `k` janelas no MESMO processo (a `c2` do High varia entre processos e
/// não dentro: a régua do regime de alocação). Devolve se o solver renasce agora.
pub(super) fn relatar_custo(
    janela: &mut JanelaDeCusto,
    t: &TemposMs,
    entorno: Option<&TemposMs>,
    onde: Onde<'_>,
    n: u32,
) -> bool {
    if std::env::var_os("PH2D_LIGHT2D_CUSTO").is_none() {
        return false;
    }
    let total = t.total() + entorno.map_or(0.0, TemposMs::total);
    janela.total.push(total);
    janela.c2.push(t.c2_total());
    janela.fina.push(t.c3_fina);
    if !n.is_multiple_of(60) {
        return false;
    }
    let (total_min, total_med) = min_e_mediana(&mut janela.total);
    let (c2_min, c2_med) = min_e_mediana(&mut janela.c2);
    let (fina_min, fina_med) = min_e_mediana(&mut janela.fina);
    eprintln!(
        "[luz2d] janela g{} w{} · {} leituras · total min {total_min:.2} med {total_med:.2} · c2 min \
         {c2_min:.2} med {c2_med:.2} · fina min {fina_min:.2} med {fina_med:.2}",
        janela.geracao,
        janela.n,
        janela.c2.len(),
    );
    janela.total.clear();
    janela.c2.clear();
    janela.fina.clear();
    janela.n += 1;
    let renasce = std::env::var("PH2D_LIGHT2D_RENASCE")
        .ok()
        .and_then(|v| v.trim().parse::<u32>().ok())
        .is_some_and(|k| k > 0 && janela.n.is_multiple_of(k));
    if renasce {
        janela.geracao += 1;
        janela.n = 0;
    }
    let (cfg, fluencia, (i, f), alvo) = onde;
    eprintln!(
        "[luz2d] onde · janela {}×{} · grade {}×{} · fina ×{} captura ×{} · fluência {}×{} região {}×{} · {:?}",
        alvo.0,
        alvo.1,
        cfg.grade.largura,
        cfg.grade.altura,
        cfg.fina,
        cfg.captura,
        fluencia.largura,
        fluencia.altura,
        f[0] - i[0],
        f[1] - i[1],
        cfg.aritmetica,
    );
    let grade = (cfg.grade.largura, cfg.grade.altura);
    eprintln!(
        "[luz2d] custo {:.2} ms · grade {}×{} · c0 {:.2} c1 {:.2} c2 {:.2} vis {:.2} fina {:.2} c0_fina {:.2} \
         preenche {:.2} másc {:.2} rebote {:.2}+{:.2} miolo {:.2} · entorno {:.2}",
        total,
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
    renasce
}

/// As leituras da janela corrente do instrumento ([`relatar_custo`]): a geração do solver e a janela nela.
#[derive(Default)]
pub(super) struct JanelaDeCusto {
    total: Vec<f64>,
    c2: Vec<f64>,
    fina: Vec<f64>,
    geracao: u32,
    n: u32,
}

fn min_e_mediana(v: &mut [f64]) -> (f64, f64) {
    if v.is_empty() {
        return (f64::NAN, f64::NAN);
    }
    v.sort_by(f64::total_cmp);
    (v[0], v[v.len() / 2])
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
