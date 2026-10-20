//! **O pincel no toque de cada etapa** do Painter: um tool no estado inicial que só recebe os passos de
//! configuração da trilha que toca.

use crate::PaintMedia;
use crate::macro_track::inicial;
use crate::macro_track::passo::Passo;
use crate::macro_track::tocar::{ErroDaMacro, TocadorDoPainter};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::PointerPhase;
use ph2d_macro::Track;

/// Passos que mudam só o estado do pincel — a leitura das propriedades aplica-os pelas portas do tool,
/// sem pintar (o ponteiro, a água e os drenos não entram).
fn e_de_configuracao(p: &Passo) -> bool {
    !matches!(
        p,
        Passo::Ponteiro { .. }
            | Passo::Hover(_)
            | Passo::Tique(_)
            | Passo::Agua(_)
            | Passo::Preview
            | Passo::PreviewBbox
            | Passo::PreviewSujo
            | Passo::PreviewAtual
            | Passo::Completo
            | Passo::Para
            | Passo::Desfaz
            | Passo::Refaz
            | Passo::PedeCommit
            | Passo::TiraCommit
            | Passo::Assado
            | Passo::TiraAssadoAdiado
            | Passo::SaiDaGravacao
            | Passo::Recusada(_)
            | Passo::Desconhecido(_)
    )
}

/// O pincel no toque de uma etapa.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NoToque {
    pub cor: [u8; 3],
    pub tamanho_px: f32,
    pub meio: PaintMedia,
    /// O limiar do Fill (o de um ColorDrop que não o afina).
    pub limiar: f32,
}

/// O pincel no toque de cada pincelada da trilha QUE TOCA, lido por um tool no estado inicial que só
/// recebe os passos de configuração.
pub(crate) fn pinceis(tocavel: &Track) -> Result<Vec<NoToque>, ErroDaMacro> {
    let mut t: PainterTool =
        inicial::tool_no_inicial(&tocavel.initial).map_err(ErroDaMacro::Inicial)?;
    let mut out = Vec::new();
    for e in &tocavel.events {
        let p = Passo::descodifica(&e.payload).map_err(ErroDaMacro::Passo)?;
        if let Passo::Ponteiro {
            fase: PointerPhase::Down,
            ..
        } = p
        {
            out.push(NoToque {
                cor: t.brush_color_srgb8(),
                tamanho_px: t.brush_size_px(),
                meio: t.paint_media(),
                limiar: t.fill_threshold(),
            });
        } else if e_de_configuracao(&p) {
            TocadorDoPainter::aplica_ao_tool(&mut t, p)?;
        }
    }
    Ok(out)
}
