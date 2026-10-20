//! **A água depois do fim do clipe** (W14, `docs/Macros/12_W14_a_agua_depois_do_parar.md`; decisão do dono
//! 20/10): com a cabeça no fim do clipe, a água da gravação continua no palco, ao relógio do quadro, até à
//! imagem do artista — assentada, ou parada onde ela a guardou
//! ([`ph2d_tool_painter::macro_track::marca::MarcaDoArtista::agua_da_gravacao_parou`]). O clipe não fica
//! mais comprido.

use super::{ChaveDaMarca, PalcoDaPintura, PalcosDoPainter, marcas};
use ph2d_tool_painter::PainterTool;
use ph2d_tool_painter::macro_track::MarcaDaAgua;
use std::collections::BTreeMap;

impl PalcosDoPainter {
    /// **O tempo deste quadro** (ms do relógio de parede), antes de [`Self::sobe`]: o que a cauda anda.
    pub fn dt_do_quadro(&mut self, dt_ms: f32) {
        self.dt_da_cauda_ms = dt_ms;
    }
}

/// **Um quadro da cauda de `p`**, na subida: só no fim do clipe (todos os passos tocados) a água anda.
pub(super) fn anda(
    p: &mut PalcoDaPintura,
    dt_ms: f32,
    guardadas: &BTreeMap<ChaveDaMarca, marcas::Guardada>,
    painter: Option<&PainterTool>,
) {
    if dt_ms > 0.0 {
        let ate = ate_onde(p, guardadas, painter);
        let _ = p.em_cena.corre_a_cauda(dt_ms, ate);
    }
}

/// Onde a imagem de `p` guardou a água da gravação: a marca do Painter ligado à sprite, senão a guardada.
fn ate_onde(
    p: &PalcoDaPintura,
    guardadas: &BTreeMap<ChaveDaMarca, marcas::Guardada>,
    painter: Option<&PainterTool>,
) -> Option<MarcaDaAgua> {
    let (sprite, chave) = (p.sprite?, p.chave?);
    match painter.filter(|t| t.bound_document() == Some(sprite)) {
        Some(t) => t
            .marca_do_artista()
            .filter(|m| m.epoca() == chave.1)?
            .agua_da_gravacao_parou(),
        None => guardadas.get(&chave)?.vale().agua_da_gravacao_parou(),
    }
}
