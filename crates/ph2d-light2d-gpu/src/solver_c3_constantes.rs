//! ⭐ W5p — as constantes `override` da C3: uma fonte para a montagem e para a régua do compilador
//! (`medida_luz2d_w5p`, que compila a MESMA pipeline a frio). Módulo-filho de `solver_c3.rs` (o tecto
//! de 700 LOC).

use crate::config::{Config, Refino};

/// O passo da marcha da luz direta: `passo_direto` texels FINOS (o das cascatas sem o passo fino).
pub(super) fn passo(cfg: &Config, passo_fino: bool) -> f64 {
    f64::from(cfg.passo_perto)
        * if passo_fino {
            f64::from(cfg.passo_direto) / f64::from(cfg.fina)
        } else {
            1.0
        }
}

/// As constantes de TODO passe da C3: a captura ligada e o preenchimento.
#[must_use]
pub fn comuns(cfg: &Config) -> [(&'static str, f64); 22] {
    let vertice = cfg.borda_vertice.map_or((0.0, 0.0), |v| (v.pre, v.area));
    [
        ("SONDA", f64::from(cfg.sonda)),
        ("CANTO_NORMAIS", f64::from(u8::from(cfg.canto.normais))),
        ("CANTO_LUZ", f64::from(u8::from(cfg.canto.luz))),
        ("CANTO_FRACAO", f64::from(u8::from(cfg.canto.fracao))),
        ("ESCALA_DA_CAPTURA", f64::from(cfg.escala_da_captura())),
        ("COM_LUZ", 0.0),
        ("PASSO_NA_BORDA", f64::from(cfg.passo_na_borda)),
        (
            "PREENCHE_A_GRADE_INTEIRA",
            f64::from(u8::from(cfg.preenche_a_grade_inteira)),
        ),
        ("PREENCHE", f64::from(u8::from(cfg.preenchimento.is_some()))),
        ("CELULA_PREENCHE", f64::from(cfg.preenchimento.unwrap_or(1))),
        ("MEDIA_NO_EIXO", f64::from(u8::from(cfg.media_no_eixo))),
        ("C0_AO_LONGO_DA_FACE", f64::from(cfg.c0_ao_longo_da_face)),
        (
            "C0_AO_LONGO_MEDIANA",
            f64::from(u8::from(cfg.c0_ao_longo_mediana)),
        ),
        ("BORDA_EXATA", f64::from(u8::from(cfg.borda_exata))),
        ("BORDA_CONTINUA", f64::from(u8::from(cfg.borda_continua))),
        ("BORDA_FORA", f64::from(u8::from(cfg.borda_fora))),
        ("BORDA_VERTICE_PRE", f64::from(vertice.0)),
        ("BORDA_VERTICE_AREA", f64::from(vertice.1)),
        ("PE_NA_BORDA", f64::from(u8::from(cfg.pe_na_borda))),
        ("VIS_NA_FACE", f64::from(u8::from(cfg.vis_na_face))),
        ("FACE_PELO_PONTO", f64::from(u8::from(cfg.face_pelo_ponto))),
        (
            "MIOLO",
            f64::from(u8::from(cfg.niveis_do_miolo().is_some())),
        ),
    ]
}

/// A visibilidade das sondas corre (o refino por bordas, fora do entorno `so_a_c0`).
pub(super) fn com_visibilidade(cfg: &Config) -> bool {
    matches!(cfg.refino, Refino::Bordas(_)) && cfg.usa_visibilidade() && !cfg.so_a_c0
}

/// A entrada da grade fina e as constantes PRÓPRIAS dela (sem as [`comuns`] nem `DISTANTES`): a
/// `c3_fina` (refino pela visibilidade das sondas) ou a `c3_fina_exata`. O anel é `<entrada>_anel`.
#[must_use]
pub fn da_fina(cfg: &Config) -> (&'static str, Vec<(&'static str, f64)>) {
    let fundo = cfg.miolo_salta_o_fundo
        && cfg.niveis_do_miolo().is_some()
        && cfg.miolo.is_some_and(|m| m.suavidade >= 1.0);
    let fundo = ("FUNDO_SEM_LUZ", f64::from(u8::from(fundo)));
    match cfg.refino {
        Refino::Bordas(b) if com_visibilidade(cfg) => (
            "c3_fina",
            vec![
                ("PASSO", passo(cfg, true)),
                ("RAIOS_DIRETOS", f64::from(b.raios)),
                ("BISSECOES", f64::from(b.bissecoes)),
                ("JOELHO", f64::from(u8::from(b.joelho))),
                ("CURVATURA", f64::from(u8::from(b.curvatura))),
                fundo,
            ],
        ),
        _ => ("c3_fina_exata", vec![("PASSO", passo(cfg, true)), fundo]),
    }
}
