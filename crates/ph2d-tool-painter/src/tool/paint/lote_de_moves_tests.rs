//! B27.2: o mesmo traço, entregue em lotes de 1, 3, 16 ou todos os Moves, dá a mesma tela ao bit — com as
//! leis por lote: a força da marca do artista (W10), Solid, a aguada (`wet_mix_depletion`), Color mixing.

use crate::macro_track::marca::MarcaDoArtista;
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool,
};
use ph2d_painter_brush::mistura_opaca::{MisturaOpaca, ModoMisturaOpaca};

const LADO: u32 = 128;
const N: u32 = 40;

fn ev(i: u32, phase: PointerPhase) -> CanvasPointer {
    #[allow(clippy::cast_precision_loss)]
    let t = i as f32;
    CanvasPointer {
        pos: [20.0 + 2.2 * t, 64.0 + 30.0 * (t * 0.21).sin()],
        pressure: 0.3 + 0.7 * ((t * 0.37).sin() * 0.5 + 0.5),
        tilt: [(t * 0.11).sin() * 0.5, (t * 0.13).cos() * 0.4],
        phase,
    }
}

#[derive(Clone, Copy, Debug)]
enum Pincel {
    Digital,
    ForcaW10,
    Solid,
    Aguada,
    ColorMixing,
    WetPaint,
    Impasto,
}

fn tool(media: PaintMedia) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (LADO * LADO * 4) as usize], LADO, LADO);
    t.set_paint_media(media);
    crate::tool::paint::tests::aquarela_neutra(&mut t);
    t.set_brush_size_px(8.0);
    t
}

fn arma(p: Pincel) -> PainterTool {
    let media = match p {
        Pincel::Aguada => PaintMedia::Watercolor,
        Pincel::WetPaint => PaintMedia::WetPaint,
        Pincel::Impasto => PaintMedia::Impasto,
        _ => PaintMedia::Digital,
    };
    let mut t = tool(media);
    match p {
        Pincel::ForcaW10 => t.poe_a_marca(Some(MarcaDoArtista::nova(LADO, LADO, 1))),
        Pincel::Solid => {
            t.handle_panel_event(PanelEvent::Click(crate::ids::PAINTER_LINE_SOLID));
        }
        Pincel::ColorMixing => {
            t.paint.brush.mistura_opaca = MisturaOpaca {
                modo: ModoMisturaOpaca::WetBlend,
                paint_amount: 0.3,
                color_stretch: 0.95,
                ..MisturaOpaca::default()
            };
        }
        _ => {}
    }
    t
}

/// `lote = None`: um Move por entrega (o oráculo); `Some(k)`: `k` Moves por lote.
/// A tela, o relevo e as gotas do traço.
type Traco = (Vec<u8>, Vec<u8>, Vec<(u16, [u8; 4])>);

fn traco(p: Pincel, lote: Option<usize>) -> Traco {
    let mut t = arma(p);
    t.on_canvas_pointer(ev(0, PointerPhase::Down));
    let mut no_lote = 0;
    for i in 1..N {
        if let Some(k) = lote {
            if no_lote == 0 {
                t.abre_lote();
            }
            t.on_canvas_pointer(ev(i, PointerPhase::Move));
            no_lote += 1;
            if no_lote >= k {
                t.fecha_lote();
                no_lote = 0;
            }
        } else {
            t.on_canvas_pointer(ev(i, PointerPhase::Move));
        }
        t.on_tick(1000.0 / 60.0);
    }
    t.fecha_lote();
    t.on_canvas_pointer(ev(N - 1, PointerPhase::Up));
    for _ in 0..3 {
        t.on_tick(1000.0 / 60.0);
    }
    let preview = t
        .take_preview_arc()
        .map(|(d, _, _)| d.to_vec())
        .unwrap_or_default();
    // A marca por texel (cobertura + o de baixo): o `Debug` dela leva a versão, que não se repete.
    let marca = t.marca_do_artista().map_or_else(Vec::new, |m| {
        (0..(LADO * LADO) as usize)
            .map(|i| (m.cobertura(i), m.por_baixo(i)))
            .collect()
    });
    (t.canvas_rgba.to_vec(), preview, marca)
}

#[test]
fn a_particao_em_lotes_nao_muda_o_traco() {
    let mut falhas = Vec::new();
    for p in [
        Pincel::Digital,
        Pincel::ForcaW10,
        Pincel::Solid,
        Pincel::Aguada,
        Pincel::ColorMixing,
        Pincel::WetPaint,
        Pincel::Impasto,
    ] {
        let oraculo = traco(p, None);
        assert!(
            traco(p, None) == oraculo,
            "{p:?}: o oráculo não se repete — a régua não é determinística"
        );
        assert!(
            oraculo.0.iter().chain(&oraculo.1).any(|&b| b != 255),
            "{p:?}: o traço não pintou — a régua não mede nada"
        );
        for k in [1, 3, 16, usize::MAX] {
            let l = traco(p, Some(k));
            let dif = |a: &[u8], b: &[u8]| {
                a.iter().zip(b).filter(|(x, y)| x != y).count() + a.len().abs_diff(b.len())
            };
            let (dt, dp, dm) = (
                dif(&l.0, &oraculo.0),
                dif(&l.1, &oraculo.1),
                l.2 != oraculo.2,
            );
            if dt + dp > 0 || dm {
                falhas.push(format!(
                    "{p:?} lote {k}: tela {dt} B · prévia {dp} B · marca difere {dm}"
                ));
            }
        }
    }
    assert!(
        falhas.is_empty(),
        "a partição mudou o traço:\n{}",
        falhas.join("\n")
    );
}
