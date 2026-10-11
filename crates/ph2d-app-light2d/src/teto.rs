//! ⭐ **Os tetos do DISPOSITIVO** (W5e) — o degrau que a janela pede pode não caber na placa:
//!
//! - o LADO de textura (`max_texture_dimension_2d`): a captura fina dos opacos é 2× a grade fina, e a
//!   2160p (vista de fábrica) pedia 10 240 texels contra os 8 192 do dispositivo — erro de validação e
//!   a luz apagada (rodada `medida_luz2d_w5e`, A1);
//! - a MEMÓRIA: o wgpu não diz quanta há (medido, A2: só o relatório do alocador, do que JÁ se
//!   alocou, e a RECUSA — do driver, ou do `MemoryBudgetThresholds` da instância). O teto de memória
//!   é então o que o dispositivo RECUSOU nesta janela ([`ph2d_light2d_gpu::ErroSolver::SemMemoria`]).
//!
//! Os bytes de um degrau são os que a [`PassList`] declara (o G5 prova que são os alocados; A1:
//! 285 · 470 · 1 151 MB alocados contra 283 · 468 · 1 139 declarados a 1080p). A memória mora quase
//! toda na grade FINA (A0, 1080p Baixo: a `borda` 126, a cobertura, o `tau` e a fluência finos 31
//! cada, de 275 MB) — ela segue os píxeis da janela e não o nível; por isso o que cede primeiro é a
//! fina ([`Degrau::com_fina_engrossada`]) e o nível do artista fica.

use kurbo::Affine;
use ph2d_light2d_gpu::{Config, Grade, HOLOGRAFICA_PADRAO, PassList};

use crate::grade::{Degrau, captura_do_quadro, fina_do_quadro, grade_do_quadro};

/// O que o dispositivo comporta: o lado de textura e, nesta janela, os bytes que ele já recusou.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tetos {
    pub lado: u32,
    pub recusou: Option<u64>,
}

/// A configuração que a luz do quadro pede para o degrau `dg` na grade `g` da vista (a aritmética
/// fica a do padrão; o dispositivo troca-a em [`Config::padrao_para`]).
#[must_use]
pub fn config_do_quadro(g: Grade, mundo_para_tela: Affine, dg: Degrau, rebote: bool) -> Config {
    Config {
        fina: fina_do_quadro(mundo_para_tela, dg),
        captura: captura_do_quadro(mundo_para_tela, dg),
        holografica: dg.holografica.then_some(HOLOGRAFICA_PADRAO),
        rebote,
        rebotes_por_quadro: dg.rebotes_por_quadro,
        ..Config::padrao(g)
    }
}

/// O degrau cabe nos `tetos` nesta vista?
fn cabe(mundo_para_tela: Affine, alvo_px: (u32, u32), dg: Degrau, tetos: Tetos) -> bool {
    let Some(g) = grade_do_quadro(mundo_para_tela, alvo_px, dg) else {
        return false;
    };
    let cfg = config_do_quadro(g, mundo_para_tela, dg, true);
    let efetiva = Config {
        grade: cfg.grade_efetiva(),
        ..cfg
    };
    cfg.cabe_na_placa(tetos.lado).is_ok()
        && tetos
            .recusou
            .is_none_or(|b| PassList::de(&efetiva, false).bytes_alocados() < b)
}

/// ⭐ **Quantas vezes a fina tem de engrossar** para o degrau `dg` caber nos `tetos` nesta vista: o
/// menor `k`; `None` = nem a fina das cascatas cabe (a luz recusa e o aviso o diz).
#[must_use]
pub fn fina_que_cabe(
    mundo_para_tela: Affine,
    alvo_px: (u32, u32),
    dg: Degrau,
    tetos: Tetos,
) -> Option<u32> {
    (0..=dg.fina_max()).find(|&k| cabe(mundo_para_tela, alvo_px, dg.com_fina_engrossada(k), tetos))
}
