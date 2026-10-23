//! **O pincel no toque de cada etapa** do Painter: um tool no estado inicial que só recebe os passos de
//! configuração da trilha que toca.

use crate::PaintMedia;
use crate::macro_track::inicial;
use crate::macro_track::passo::Passo;
use crate::macro_track::passo::faixa_de;
use crate::macro_track::tocar::{ErroDaMacro, TocadorDoPainter};
use crate::tool::PainterTool;
use crate::tool::paint::tracos::{PincelDoTraco, TracoGuardado};
use ph2d_editor_core::tool::PointerPhase;
use ph2d_macro::Track;
use std::collections::BTreeMap;

/// Passos que mudam só o estado do pincel — a leitura das propriedades aplica-os pelas portas do tool,
/// sem pintar (o ponteiro, a água e os drenos não entram).
pub(super) fn e_de_configuracao(p: &Passo) -> bool {
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
            | Passo::PincelDaFaixa(_)
    )
}

/// **O pincel que cada faixa leva** (W16): um tool no estado inicial que só recebe os passos de
/// configuração da faixa 0, pela ordem da trilha que toca, lido em cada [`Passo::PincelDaFaixa`]. Lido
/// ANTES de tocar: a etapa levada para mais cedo toca antes do instante dela.
pub(crate) fn pinceis_das_faixas(
    inicial: &[u8],
    cargas: &[&[u8]],
) -> Result<BTreeMap<u32, PincelDoTraco>, ErroDaMacro> {
    let mut out = BTreeMap::new();
    if !cargas
        .iter()
        .any(|c| matches!(Passo::descodifica(c), Ok(Passo::PincelDaFaixa(_))))
    {
        return Ok(out);
    }
    let mut t: PainterTool = inicial::tool_no_inicial(inicial).map_err(ErroDaMacro::Inicial)?;
    for c in cargas {
        if faixa_de(c) != 0 {
            continue;
        }
        match Passo::descodifica(c).map_err(ErroDaMacro::Passo)? {
            Passo::PincelDaFaixa(n) => {
                out.insert(n, t.pincel_do_traco());
            }
            p if e_de_configuracao(&p) => {
                TocadorDoPainter::aplica_ao_tool(&mut t, p)?;
            }
            _ => {}
        }
    }
    Ok(out)
}

/// O pincel no toque de uma etapa.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NoToque {
    pub cor: [u8; 3],
    pub tamanho_px: f32,
    pub meio: PaintMedia,
    /// O limiar do Fill (o de um ColorDrop que não o afina).
    pub limiar: f32,
    /// Uma pincelada simples, que se pode sobrepor a outra (W16, [`PainterTool::toque_simples`]).
    pub simples: bool,
}

/// O pincel no toque de cada pincelada da trilha QUE TOCA, pela ordem dos toques, com a faixa dele: um
/// tool no estado inicial que só recebe os passos de configuração; os de uma faixa (W16) mudam o pincel
/// do traço dela, que nasce com o da [`pinceis_das_faixas`].
pub(crate) fn pinceis(tocavel: &Track) -> Result<Vec<(u32, NoToque)>, ErroDaMacro> {
    let cargas: Vec<&[u8]> = tocavel
        .events
        .iter()
        .map(|e| e.payload.as_slice())
        .collect();
    let das_faixas = pinceis_das_faixas(&tocavel.initial, &cargas)?;
    let mut t: PainterTool =
        inicial::tool_no_inicial(&tocavel.initial).map_err(ErroDaMacro::Inicial)?;
    let mut faixas: BTreeMap<u32, TracoGuardado> = BTreeMap::new();
    let mut out = Vec::new();
    for c in cargas {
        let p = Passo::descodifica(c).map_err(ErroDaMacro::Passo)?;
        let n = faixa_de(c);
        let toque = matches!(
            p,
            Passo::Ponteiro {
                fase: PointerPhase::Down,
                ..
            }
        );
        if !toque && !e_de_configuracao(&p) {
            continue;
        }
        let mut g = (n != 0).then(|| {
            faixas.remove(&n).unwrap_or_else(|| {
                let pincel = das_faixas
                    .get(&n)
                    .copied()
                    .unwrap_or_else(|| t.pincel_do_traco());
                t.traco_novo(&pincel)
            })
        });
        if let Some(g) = g.as_mut() {
            t.troca_o_traco(g);
        }
        if toque {
            out.push((
                n,
                NoToque {
                    cor: t.brush_color_srgb8(),
                    tamanho_px: t.brush_size_px(),
                    meio: t.paint_media(),
                    limiar: t.fill_threshold(),
                    simples: t.toque_simples(),
                },
            ));
        } else {
            TocadorDoPainter::aplica_ao_tool(&mut t, p)?;
        }
        if let Some(mut g) = g {
            t.troca_o_traco(&mut g);
            faixas.insert(n, g);
        }
    }
    Ok(out)
}
