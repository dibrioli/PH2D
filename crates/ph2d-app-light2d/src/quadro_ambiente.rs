//! As chaves de ambiente da luz (`PH2D_LIGHT2D_*=0` compara com o antes). Módulo-filho de `quadro.rs`
//! (o tecto de 700 LOC).

fn ligado(k: &str) -> bool {
    std::env::var(k).map_or(true, |v| v.trim() != "0")
}

/// ⭐ W5c: o rebote em todo nível (sem nenhum corpo que reflita, os passes dele saltam);
/// ⭐ W5h — o miolo dos opacos: `PH2D_LIGHT2D_MIOLO=0` volta ao exterior mais próximo (comparar);
/// `PH2D_LIGHT2D_SUAVIDADE` (0–1) e `PH2D_LIGHT2D_PENETRACAO` (metros) até o controlo por objeto.
pub(super) fn miolo_do_ambiente() -> Option<ph2d_light2d_gpu::Miolo> {
    if std::env::var("PH2D_LIGHT2D_MIOLO").is_ok_and(|v| v.trim() == "0") {
        return None;
    }
    let num = |k: &str| {
        std::env::var(k)
            .ok()
            .and_then(|v| v.trim().parse::<f32>().ok())
    };
    let padrao = ph2d_light2d_gpu::MIOLO_PADRAO;
    Some(ph2d_light2d_gpu::Miolo {
        suavidade: num("PH2D_LIGHT2D_SUAVIDADE").map_or(padrao.suavidade, |s| s.clamp(0.0, 1.0)),
        penetracao_m: num("PH2D_LIGHT2D_PENETRACAO")
            .filter(|p| *p > 0.0)
            .unwrap_or(padrao.penetracao_m),
        base_no_grupo: cortes_ligados(),
        ..padrao
    })
}

/// `PH2D_LIGHT2D_REBOTE=0` desliga-o (a comparação da cena 3).
pub(super) fn rebote_ligado() -> bool {
    ligado("PH2D_LIGHT2D_REBOTE")
}

/// ⭐ W5o — `PH2D_LIGHT2D_CORTES=0` volta ao custo da W5n (a mesma imagem): a base do miolo um
/// despacho por relaxação e o entorno com a grade fina e o miolo que ninguém lê.
pub(super) fn cortes_ligados() -> bool {
    ligado("PH2D_LIGHT2D_CORTES")
}

/// ⭐ W5p — `PH2D_LIGHT2D_HRC_DUAS=0`: a holográfica nasce uma vez (a `c2` do High mais lenta, comparar).
pub(super) fn hrc_duas_vezes() -> bool {
    ligado("PH2D_LIGHT2D_HRC_DUAS")
}

/// ⭐ W5r — `PH2D_LIGHT2D_HRC_COLUNAS=0`: a holográfica por linhas (o mesmo bit, mais lento; comparar).
pub(super) fn hrc_por_colunas() -> bool {
    ligado("PH2D_LIGHT2D_HRC_COLUNAS")
}

/// ⭐ W5s — `PH2D_LIGHT2D_HRC_TRACA=0`: a traça de antes (um fio por intervalo, marcha sempre; o mesmo
/// bit, mais lento; comparar).
pub(super) fn hrc_traca_nova() -> bool {
    ligado("PH2D_LIGHT2D_HRC_TRACA")
}

/// ⭐ W5t — `PH2D_LIGHT2D_HRC_3D=0`: a holográfica despachada em 1D (o mesmo bit, mais lento; comparar).
pub(super) fn hrc_em_3d() -> bool {
    ligado("PH2D_LIGHT2D_HRC_3D")
}
