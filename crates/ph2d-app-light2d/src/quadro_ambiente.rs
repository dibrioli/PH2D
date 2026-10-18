//! As chaves de ambiente da luz (`PH2D_LIGHT2D_*=0` compara com o antes). Módulo-filho de `quadro.rs`
//! (o tecto de 700 LOC).

use kurbo::Affine;
use ph2d_light2d_gpu::{Config, Grade};

use crate::grade::{Degrau, MARGEM_DO_ENTORNO, grade_do_quadro};

fn ligado(k: &str) -> bool {
    std::env::var(k).map_or(true, |v| v.trim() != "0")
}

/// ⭐ W5c: o rebote em todo nível (sem nenhum corpo que reflita, os passes dele saltam);
/// ⭐ W5h — o miolo dos opacos: `PH2D_LIGHT2D_MIOLO=0` volta ao exterior mais próximo (comparar);
/// `PH2D_LIGHT2D_SUAVIDADE` (0–1) e `PH2D_LIGHT2D_PENETRACAO` (metros) até o controlo por objeto.
fn miolo_do_ambiente() -> Option<ph2d_light2d_gpu::Miolo> {
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
fn rebote_ligado() -> bool {
    ligado("PH2D_LIGHT2D_REBOTE")
}

/// ⭐ W5o — `PH2D_LIGHT2D_CORTES=0` volta ao custo da W5n (a mesma imagem): a base do miolo um
/// despacho por relaxação e o entorno com a grade fina e o miolo que ninguém lê.
fn cortes_ligados() -> bool {
    ligado("PH2D_LIGHT2D_CORTES")
}

/// ⭐ W5p — `PH2D_LIGHT2D_HRC_DUAS=0`: a holográfica nasce uma vez (a `c2` do High mais lenta, comparar).
fn hrc_duas_vezes() -> bool {
    ligado("PH2D_LIGHT2D_HRC_DUAS")
}

/// ⭐ W5x — `PH2D_LIGHT2D_HRC_BERCO=0`: o 1.º nascimento da holográfica em buffers (o da W5p; comparar).
fn hrc_berco() -> bool {
    ligado("PH2D_LIGHT2D_HRC_BERCO")
}

/// ⭐ W5z — `PH2D_LIGHT2D_HRC_TEXTURAS=0`: os T e os R da holográfica em buffers (o berço; comparar).
fn hrc_texturas() -> bool {
    ligado("PH2D_LIGHT2D_HRC_TEXTURAS")
}

/// ⭐ W5r — `PH2D_LIGHT2D_HRC_COLUNAS=0`: a holográfica por linhas (o mesmo bit, mais lento; comparar).
fn hrc_por_colunas() -> bool {
    ligado("PH2D_LIGHT2D_HRC_COLUNAS")
}

/// ⭐ W5s — `PH2D_LIGHT2D_HRC_TRACA=0`: a traça de antes (um fio por intervalo, marcha sempre; o mesmo
/// bit, mais lento; comparar).
fn hrc_traca_nova() -> bool {
    ligado("PH2D_LIGHT2D_HRC_TRACA")
}

/// ⭐ W5t — `PH2D_LIGHT2D_HRC_3D=0`: a holográfica despachada em 1D (o mesmo bit, mais lento; comparar).
fn hrc_em_3d() -> bool {
    ligado("PH2D_LIGHT2D_HRC_3D")
}

/// ⭐ W5u — `PH2D_LIGHT2D_HRC_POR_FIO=0`: uma entrada por fio na subida e na descida (o mesmo bit, mais
/// lento; comparar), na vista e no entorno.
fn hrc_por_fio(c: ph2d_light2d_gpu::Config) -> ph2d_light2d_gpu::Config {
    if ligado("PH2D_LIGHT2D_HRC_POR_FIO") {
        return c;
    }
    ph2d_light2d_gpu::Config {
        hrc_cones_por_fio: 1,
        hrc_pares_na_subida: 0,
        ..c
    }
}

/// ⭐ W5v — a configuração do solver da VISTA do produto para o degrau `dg` na grade `g`: a ÚNICA porta
/// (o quadro, a estufa e as réguas que medem o app — a aritmética é a do dispositivo, F16 na RTX).
#[must_use]
pub fn config_da_vista(
    device: &wgpu::Device,
    g: Grade,
    mundo_para_tela: Affine,
    dg: Degrau,
) -> Config {
    hrc_por_fio(Config {
        aritmetica: Config::padrao_para(device, g).aritmetica,
        pe_na_borda: ligado("PH2D_LIGHT2D_PE"),
        vis_na_face: ligado("PH2D_LIGHT2D_FACE"),
        // ⭐ W5o: `=0` volta à marcha a partir do ponto afastado (a penumbra deslocada, comparar).
        face_pelo_ponto: ligado("PH2D_LIGHT2D_FACE_PONTO"),
        // ⭐ W6d: `PH2D_LIGHT2D_BORDA=0` volta à reta por texel e à marcha de fora pelo iso-½ (os pontos
        // pretos rentes à borda, comparar).
        borda_continua: ligado("PH2D_LIGHT2D_BORDA"),
        borda_fora: ligado("PH2D_LIGHT2D_BORDA"),
        // `PH2D_LIGHT2D_BORDA_UNIAO=0`: só o campo (o canto convexo chanfrado, comparar).
        borda_uniao: ligado("PH2D_LIGHT2D_BORDA") && ligado("PH2D_LIGHT2D_BORDA_UNIAO"),
        hrc_nasce_duas_vezes: hrc_duas_vezes(),
        hrc_berco: hrc_berco(),
        hrc_em_texturas: hrc_texturas(),
        // ⭐ W5x: a régua da memória (`PH2D_LIGHT2D_MEMORIA=1`) mede os buffers da holográfica da vista;
        // ⭐ W5y: `=lista` mede só o que o solver já guarda (sem clones: a alocação do produto).
        hrc_guarda_os_grandes: std::env::var("PH2D_LIGHT2D_MEMORIA").is_ok_and(|v| v != "lista"),
        // ⭐ W5q: `=0` = o anel na grade inteira (o mesmo bit, mais lento; comparar).
        anel_por_ladrilhos: ligado("PH2D_LIGHT2D_LADRILHOS"),
        hrc_por_colunas: hrc_por_colunas(),
        hrc_salta_o_vazio: hrc_traca_nova(),
        hrc_fio_por_sonda: hrc_traca_nova(),
        hrc_grade_3d: hrc_em_3d(),
        miolo: miolo_do_ambiente(),
        // ⭐ W5v: `=0` = a fina calcula o fundo dos opacos (o mesmo bit, mais lento; comparar).
        miolo_salta_o_fundo: ligado("PH2D_LIGHT2D_FUNDO"),
        ..crate::teto::config_do_quadro(g, mundo_para_tela, dg, rebote_ligado())
    })
}

/// ⭐ W5v — a configuração do ENTORNO do produto na vista (`None` = sem ele: o degrau, a holográfica,
/// `PH2D_LIGHT2D_ENTORNO=0` para comparar).
#[must_use]
pub fn config_do_entorno(
    device: &wgpu::Device,
    mundo_para_tela: Affine,
    alvo_px: (u32, u32),
    dg: Degrau,
) -> Option<Config> {
    let holografica = dg
        .holografica
        .then_some(ph2d_light2d_gpu::HOLOGRAFICA_PADRAO);
    if dg.entorno <= 0.0 || holografica.is_none() || !ligado("PH2D_LIGHT2D_ENTORNO") {
        return None;
    }
    let de = Degrau {
        c0_px: dg.c0_px * dg.entorno,
        margem: MARGEM_DO_ENTORNO,
        ..dg
    };
    let ge = grade_do_quadro(mundo_para_tela, alvo_px, de)?;
    Some(hrc_por_fio(Config {
        holografica,
        rebote: rebote_ligado(),
        // ⭐ W5o: da vista só se lê a c0 do entorno (`Solver::somar_entorno`).
        so_a_c0: cortes_ligados(),
        hrc_nasce_duas_vezes: hrc_duas_vezes(),
        hrc_berco: hrc_berco(),
        hrc_em_texturas: hrc_texturas(),
        hrc_por_colunas: hrc_por_colunas(),
        ..Config::padrao_para(device, ge)
    }))
}
