//! O que os gates das macros partilham: gravar quadro a quadro, tocar pelo `MacroPlayer`, comparar.

use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};
use ph2d_macro::{MacroDoc, MacroRecorder, RefRect, TrackId};
use ph2d_tool_painter::PainterTool;
use ph2d_tool_painter::macro_track::FAMILIA;
use ph2d_tool_painter::macro_track::tocar::TocadorDoPainter;

/// A gravação do lado de quem chama: o tempo é do quadro (`16,666 ms`), nunca do relógio de parede.
pub struct Gravador {
    rec: MacroRecorder,
    trilha: TrackId,
    pub relogio_us: u64,
}

impl Gravador {
    /// Arma `t` e abre a trilha com o estado inicial.
    pub fn arma(t: &mut PainterTool) -> Self {
        let (inicial, (w, h)) = t.macro_grava().expect("arma no Wet Paint");
        let mut rec = MacroRecorder::new(
            RefRect {
                w: w as f32,
                h: h as f32,
            },
            0,
        );
        let trilha = rec.open_track(FAMILIA, 0, inicial);
        Self {
            rec,
            trilha,
            relogio_us: 0,
        }
    }

    /// Fecha um quadro: os passos dele entram com o tempo do quadro.
    pub fn quadro(&mut self, t: &mut PainterTool) {
        for c in t.macro_passos() {
            self.rec
                .push(self.trilha, self.relogio_us, c)
                .expect("tempo crescente");
        }
        self.relogio_us += 16_666;
    }

    pub fn para(mut self, t: &mut PainterTool) -> MacroDoc {
        for c in t.macro_para() {
            self.rec
                .push(self.trilha, self.relogio_us, c)
                .expect("tempo crescente");
        }
        self.rec.finish()
    }
}

pub fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Toca a trilha 0 do princípio ao fim, pelo `MacroPlayer`, num tool próprio no estado inicial.
pub fn toca(doc: &MacroDoc) -> Result<TocadorDoPainter, String> {
    let tr = &doc.tracks[0];
    let toc = TocadorDoPainter::novo(&tr.initial, tr.events.iter().map(|e| e.payload.as_slice()))
        .map_err(|e| format!("{e:?}"))?;
    toca_com(doc, toc)
}

pub fn toca_com(doc: &MacroDoc, mut toc: TocadorDoPainter) -> Result<TocadorDoPainter, String> {
    let mut player = ph2d_macro::MacroPlayer::new(doc);
    player
        .advance_to(doc, doc.duration(), |_, _, ev| toc.aplica(&ev.payload))
        .map_err(|e| format!("{e:?}"))?;
    Ok(toc)
}

pub fn texels_diferentes(a: &[u8], b: &[u8]) -> usize {
    assert_eq!(a.len(), b.len(), "telas de tamanhos diferentes");
    a.chunks(4).zip(b.chunks(4)).filter(|(x, y)| x != y).count()
}
